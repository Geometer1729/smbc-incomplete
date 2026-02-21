use crate::ai::pref;
use crate::api::API;
use crate::bayesian::{self, StrategyMap};
use crate::moves::other;
use crate::types::{InMsg::*, OutMsg::*, Pref, Square::*, *};

use std::sync::Arc;

use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tokio::sync::mpsc::*;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Message;
use tungstenite::protocol::Message::*;

#[cfg(not(debug_assertions))]
const INDEX_HTML: &str = include_str!("../web/game.html");
#[cfg(not(debug_assertions))]
const GAME_JS: &str = include_str!("../web/game.js");
#[cfg(not(debug_assertions))]
const GAME_CSS: &str = include_str!("../web/game.css");

fn load_web_file(path: &str) -> String {
    #[cfg(debug_assertions)]
    {
        std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Failed to read {}: {}", path, e);
            format!("<!-- Error loading {} -->", path)
        })
    }
    #[cfg(not(debug_assertions))]
    {
        match path {
            "web/game.html" => INDEX_HTML.to_string(),
            "web/game.js" => GAME_JS.to_string(),
            "web/game.css" => GAME_CSS.to_string(),
            _ => String::from("Not Found"),
        }
    }
}

#[cfg(debug_assertions)]
mod livereload {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, SystemTime};
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;

    static GENERATION: AtomicU64 = AtomicU64::new(0);

    pub fn current_generation() -> u64 {
        GENERATION.load(Ordering::Relaxed)
    }

    pub async fn watch_files() {
        let mut last_mtime = get_max_mtime();
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            let mtime = get_max_mtime();
            if mtime > last_mtime {
                last_mtime = mtime;
                let gen = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
                eprintln!("[livereload] web/ changed, generation {}", gen);
            }
        }
    }

    fn get_max_mtime() -> SystemTime {
        let mut max = SystemTime::UNIX_EPOCH;
        if let Ok(entries) = std::fs::read_dir("web") {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        if mtime > max {
                            max = mtime;
                        }
                    }
                }
            }
        }
        max
    }

    pub fn inject_livereload(html: String) -> String {
        let gen = current_generation();
        let script = format!(
            r#"<script>
(function() {{
    let gen = {};
    function poll() {{
        fetch('/__livereload?gen=' + gen)
            .then(r => r.text())
            .then(t => {{
                let newGen = parseInt(t) || gen + 1;
                if(newGen !== gen) {{
                    gen = newGen;
                    location.reload();
                }} else {{
                    poll();
                }}
            }})
            .catch(() => setTimeout(poll, 1000));
    }}
    poll();
}})();
</script>"#,
            gen
        );
        html.replace("</body>", &format!("{}\n</body>", script))
    }

    pub async fn handle(mut stream: TcpStream, query: &str) {
        let client_gen: u64 = query
            .split('&')
            .find_map(|param| {
                let (key, val) = param.split_once('=')?;
                if key == "gen" {
                    val.parse().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Wait until generation actually changes
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            let current = current_generation();
            if current != client_gen {
                let body = current.to_string();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(body.as_bytes()).await;
                return;
            }
        }
    }
}

pub struct CheatState {
    pub table: Table,
    pub strategy: Arc<StrategyMap>,
    pub human_type: bayesian::TypeId,
    pub human_obj: Objective,
}

pub struct WebPlayer {
    in_handle: Receiver<InMsg>,
    out_handle: Sender<OutMsg>,
    pub cheat: Option<CheatState>,
}

/// Bind the WebSocket listener. Returns the TcpListener for use in an accept loop.
pub async fn bind_listener(addr: &str) -> TcpListener {
    #[cfg(debug_assertions)]
    {
        eprintln!("[livereload] watching web/ for changes");
        spawn(livereload::watch_files());
    }
    TcpListener::bind(addr).await.expect("Failed to bind")
}

/// Handle an incoming TCP connection: serve static files for HTTP requests,
/// or create a WebPlayer for WebSocket upgrades.
pub async fn handle_connection(stream: TcpStream) -> Option<WebPlayer> {
    let mut buf = [0u8; 4096];
    let n = match stream.peek(&mut buf).await {
        Ok(n) => n,
        Err(_) => return None,
    };
    let request = match std::str::from_utf8(&buf[..n]) {
        Ok(s) => s,
        Err(_) => return None,
    };

    if request.to_ascii_lowercase().contains("upgrade: websocket") {
        mk_web_player_from_stream(stream).await
    } else {
        serve_http(stream, request).await;
        None
    }
}

async fn serve_http(mut stream: TcpStream, peeked: &str) {
    // Consume the request data from the stream
    let mut buf = vec![0u8; 4096];
    let _ = stream.read(&mut buf).await;

    let raw_path = peeked
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let path = raw_path.split('?').next().unwrap_or(raw_path);

    #[cfg(debug_assertions)]
    if path == "/__livereload" {
        let query = raw_path.split_once('?').map(|(_, q)| q).unwrap_or("");
        livereload::handle(stream, query).await;
        return;
    }

    let (content_type, body) = match path {
        "/" | "/index.html" | "/game.html" => {
            let html = load_web_file("web/game.html");
            #[cfg(debug_assertions)]
            let html = livereload::inject_livereload(html);
            ("text/html; charset=utf-8", html)
        }
        "/game.js" => (
            "application/javascript; charset=utf-8",
            load_web_file("web/game.js"),
        ),
        "/game.css" => ("text/css; charset=utf-8", load_web_file("web/game.css")),
        _ => {
            let resp =
                "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
            let _ = stream.write_all(resp.as_bytes()).await;
            return;
        }
    };

    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        content_type,
        body.len()
    );
    let _ = stream.write_all(header.as_bytes()).await;
    let _ = stream.write_all(body.as_bytes()).await;
}

/// Create a WebPlayer from an already-accepted TCP stream.
/// Returns None if the stream is not a valid WebSocket upgrade.
async fn mk_web_player_from_stream(stream: TcpStream) -> Option<WebPlayer> {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Non-WebSocket connection: {}", e);
            return None;
        }
    };
    let (api_send, co_rec) = channel(100);
    let (co_send, api_rec) = channel(100);
    let (ws_send, ws_rec) = ws.split();
    spawn(handle_web_in(co_send, api_send.clone(), ws_rec));
    spawn(handle_web_out(co_rec, ws_send));
    Some(WebPlayer {
        in_handle: api_rec,
        out_handle: api_send,
        cheat: None,
    })
}

type WS = WebSocketStream<TcpStream>;

async fn handle_web_in(
    in_sender: Sender<InMsg>,
    out_sender: Sender<OutMsg>,
    mut ws: SplitStream<WS>,
) {
    loop {
        let msg = match ws.next().await {
            Some(Ok(msg)) => msg,
            _ => return, // Connection closed
        };
        match msg {
            Ping(ping_vec) => {
                let _ = out_sender.send(ReqPong(ping_vec)).await;
            }
            Binary(bytes) if bytes.len() == 1 && bytes[0] < 9 => {
                let _ = in_sender.send(MoveAt(bytes[0] as usize)).await;
            }
            Binary(bytes) if bytes.len() == 1 && bytes[0] == 9 => {
                let _ = in_sender.send(Reset).await;
            }
            _ => {}
        }
    }
}

async fn handle_web_out(mut rec: Receiver<OutMsg>, mut ws: SplitSink<WS, Message>) {
    loop {
        match rec.recv().await {
            Some(Upd(pos)) => {
                let _ = ws.send(marshal_pos(pos)).await;
            }
            Some(Info(msg)) => {
                let _ = ws.send(Text(msg)).await;
            }
            Some(ReqPong(ping_vec)) => {
                let _ = ws.send(Pong(ping_vec)).await;
            }
            None => return, // Channel closed
        }
    }
}

impl WebPlayer {
    pub async fn send_info(&self, msg: String) {
        let _ = self.out_handle.send(Info(msg)).await;
    }

    pub async fn wait_for_reset(&mut self) {
        loop {
            match self.in_handle.recv().await {
                Some(Reset) => return,
                None => return, // Connection closed
                _ => continue,
            }
        }
    }
}

impl API for WebPlayer {
    async fn rend(&mut self, p: Pos) {
        let _ = self.out_handle.send(Upd(p)).await;
        if let Some(cheat) = &self.cheat {
            // Skip cheat data for terminal positions (game already decided)
            if crate::eval::eval_pos(p).is_some() {
                return;
            }
            // Show Nash probabilities on human's turn (X)
            let nash_probs: [f64; 9] = if p.turn == crate::types::Player::X {
                cheat
                    .strategy
                    .get(&p)
                    .map(|s| s[cheat.human_type])
                    .unwrap_or([0.0; 9])
            } else {
                [0.0; 9]
            };

            let mut parts: Vec<String> = Vec::with_capacity(9);
            for i in 0..9 {
                if p.board[i] != Open {
                    parts.push("-,-".to_string());
                } else {
                    // Make the move and evaluate the child position
                    let mut board2 = p.board;
                    board2[i] = Taken { by: p.turn };
                    let child = Pos {
                        turn: other(p.turn),
                        board: board2,
                    };
                    // Use eval_pos first for terminal children, otherwise table lookup
                    let maybe_eval = match crate::eval::eval_pos(child) {
                        Some(crate::types::Outcome::Win {
                            with: crate::types::Player::X,
                        }) => Some(crate::eval::X_WON),
                        Some(crate::types::Outcome::Draw) => Some(crate::eval::DRAW),
                        Some(crate::types::Outcome::Win {
                            with: crate::types::Player::O,
                        }) => Some(crate::eval::O_WON),
                        None => {
                            use crate::table::cannon_lookup_safe;
                            cannon_lookup_safe(&cheat.table, child)
                        }
                    };
                    match maybe_eval {
                        Some(eval) => {
                            let p_char = match pref(cheat.human_obj, eval) {
                                Pref::Winable => 'W',
                                Pref::Drawable => 'D',
                                Pref::Lost => 'L',
                            };
                            parts.push(format!("{},{:.4}", p_char, nash_probs[i]));
                        }
                        None => {
                            eprintln!("Cheat: missing table entry for child at square {}", i);
                            parts.push("-,-".to_string());
                        }
                    }
                }
            }
            let msg = format!("CHEAT:{}", parts.join("|"));
            let _ = self.out_handle.send(Info(msg)).await;
        }
    }

    async fn ask(&mut self, p: Pos) -> Pos {
        loop {
            match self.in_handle.recv().await.expect("connection closed") {
                MoveAt(ind) if p.board[ind] == Open => {
                    let mut new_board = p.board;
                    new_board[ind] = Taken { by: p.turn };
                    return Pos {
                        turn: other(p.turn),
                        board: new_board,
                    };
                }
                MoveAt(_) | Reset => continue,
            }
        }
    }

    async fn close(&mut self, _o: Outcome) {}
}

fn marshal_pos(p: Pos) -> Message {
    Text(format!(
        "{}{}{}{}{}{}{}{}{}{}",
        p.turn,
        p.board[0],
        p.board[1],
        p.board[2],
        p.board[3],
        p.board[4],
        p.board[5],
        p.board[6],
        p.board[7],
        p.board[8]
    ))
}
