use crate::api::API;
use crate::moves::other;
use crate::types::{InMsg::*, OutMsg::*, Square::*, *};

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

const INDEX_HTML: &str = include_str!("../web/game.html");
const GAME_JS: &str = include_str!("../web/game.js");
const GAME_CSS: &str = include_str!("../web/game.css");

pub struct WebPlayer {
    in_handle: Receiver<InMsg>,
    out_handle: Sender<OutMsg>,
}

/// Bind the WebSocket listener. Returns the TcpListener for use in an accept loop.
pub async fn bind_listener(addr: &str) -> TcpListener {
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

    let path = peeked
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let (content_type, body) = match path {
        "/" | "/index.html" | "/game.html" => ("text/html; charset=utf-8", INDEX_HTML),
        "/game.js" => ("application/javascript; charset=utf-8", GAME_JS),
        "/game.css" => ("text/css; charset=utf-8", GAME_CSS),
        _ => {
            let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
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
