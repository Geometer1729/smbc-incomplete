use crate::types::{*,Square::*};
use crate::moves::other;
use crate::api::API;

use std::{env, io::Error};
use std::iter::*;

use futures_util::{future, StreamExt, TryStreamExt};
use tungstenite::protocol::Message;
use tungstenite::protocol::Message::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Handle;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::accept_async;
use futures_util::SinkExt;
use async_trait::async_trait;

pub struct WebPlayer
    {pub stream : WebSocketStream<TcpStream>
    }

pub async fn mkWebPlayer() -> WebPlayer {
    let addr : String = "127.0.0.1:8080".to_string();
    let listener : TcpListener = TcpListener::bind(&addr).await.expect("Failed to bind");
    WebPlayer{stream:
        accept_async(listener.accept().await.expect("arg").0).await.expect("failed here")
    }
}

#[async_trait]
impl API for WebPlayer {
    async fn rend(&mut self,p:Pos){
        self.stream.send(marshal_pos(p)).await.expect("hurr");
    }

    async fn ask(&mut self,p:Pos) -> Pos {
        match self.stream.next().await.expect("read failed").expect("read failed2") {
            Ping(ping_vec) => { self.stream.send(Pong(ping_vec)).await ; self.ask(p).await },
            Binary(bytes)
                if bytes.len() == 1
                    && bytes[0] < 9
                    && p.board[bytes[0] as usize] == Open
                    =>
                    { let mut new_board = p.board.clone();
                      new_board[bytes[0] as usize]= Taken{by:p.turn};
                      Pos{turn:other(p.turn),board:new_board}
                    },
            _ => self.ask(p).await
        }
    }
}

fn marshal_pos(p:Pos) -> Message {
    Text(format!("{}{}{}{}{}{}{}{}{}{}"
                ,p.turn
                ,p.board[0]
                ,p.board[1]
                ,p.board[2]
                ,p.board[3]
                ,p.board[4]
                ,p.board[5]
                ,p.board[6]
                ,p.board[7]
                ,p.board[8]
    ))
}
