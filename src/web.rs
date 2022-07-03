use crate::types::{*,Square::*,InMsg::*,OutMsg::*};
use crate::moves::other;
use crate::api::API;

use async_trait::async_trait;
use futures_util::SinkExt;
use futures_util::{StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::*;
use tokio::spawn;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message::*;
use tungstenite::protocol::Message;
use futures_util::stream::SplitStream;
use futures_util::stream::SplitSink;

pub struct WebPlayer
    { in_handle: Receiver<InMsg>,
      out_handle: Sender<OutMsg>,
    }

pub async fn mk_web_player() -> WebPlayer {
    let addr : String = "127.0.0.1:8080".to_string();
    let listener : TcpListener = TcpListener::bind(&addr).await.expect("Failed to bind");
    let (api_send,co_rec) = channel(100);
    let (co_send,api_rec) = channel(100);
    let (ws_send,ws_rec) = accept_async(
                listener
                .accept()
                .await
                .expect("arg")
                .0
            ).await
            .expect("failed to make socket").split();
    spawn(handle_web_in(co_send,api_send.clone(),ws_rec));
    spawn(handle_web_out(co_rec,ws_send));
    WebPlayer{in_handle:api_rec,out_handle:api_send}
}

type WS = WebSocketStream<TcpStream>;

async fn handle_web_in(in_sender:Sender<InMsg>,out_sender:Sender<OutMsg>,mut ws : SplitStream<WS> ) {
    loop {
        match ws.next().await.expect("read fail 1").expect("read fail 2"){
            Ping(ping_vec) => { out_sender.send(ReqPong(ping_vec)).await.expect("ahh") }
            Binary(bytes)
                if bytes.len() == 1
                    && bytes[0] < 9
                    => in_sender.send(MoveAt(bytes[0] as usize)).await.expect("faild to send move at"),
            _ => {},
        }
    }

}

async fn handle_web_out(mut rec:Receiver<OutMsg>,mut ws: SplitSink<WS,Message>) {
    loop {
        match rec.recv().await.expect("ahhh") {
          Upd(pos) => ws.send(marshal_pos(pos)).await.expect("ahhh"),
          ReqPong(ping_vec) => ws.send(Pong(ping_vec)).await.expect("ahh"),
        }
    }

}

#[async_trait]
impl API for WebPlayer {
    async fn rend(&mut self,p:Pos){
        self.out_handle.send(Upd(p)).await.expect("ahh")
        //self.stream.send(marshal_pos(p)).await.expect("hurr");
    }

    async fn ask(&mut self,p:Pos) -> Pos {
        match self.in_handle.recv().await.expect("ahhh") {
            MoveAt(ind) if p.board[ind] == Open
              => { let mut new_board = p.board.clone();
                   new_board[ind]= Taken{by:p.turn};
                   Pos{turn:other(p.turn),board:new_board}
                 },
            MoveAt(_) => self.ask(p).await,
            Reset => todo!("implement reset"),
        }
    }

    async fn close(&mut self,_o:Outcome) {
        // self.stream.send(Text(format!("{}",o))).await.expect("failed to send result");
        // self.stream.close(None).await.expect("close didn't work?");
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
