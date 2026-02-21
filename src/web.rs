use crate::api::API;
use crate::moves::other;
use crate::types::{InMsg::*, OutMsg::*, Square::*, *};

use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tokio::sync::mpsc::*;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Message;
use tungstenite::protocol::Message::*;

pub struct WebPlayer {
    in_handle: Receiver<InMsg>,
    out_handle: Sender<OutMsg>,
}

/// Bind the WebSocket listener. Returns the TcpListener for use in an accept loop.
pub async fn bind_listener(addr: &str) -> TcpListener {
    TcpListener::bind(addr).await.expect("Failed to bind")
}

/// Create a WebPlayer from an already-accepted TCP stream.
/// Returns None if the stream is not a valid WebSocket upgrade.
pub async fn mk_web_player_from_stream(stream: TcpStream) -> Option<WebPlayer> {
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
