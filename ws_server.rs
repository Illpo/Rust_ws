use tokio::net::TcpListener;

use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:1234").await.expect("Faild to creat a socket!");
    println!("┌──Server running...\n└─» ws:...:1234\n ┌───┐  ┌───┐\n | ▀▄▀  ▀▄▀ |\n └───┘  └───┘");
    let (tx, rx) = broadcast::channel(10);
    loop {
        if let Ok((mut socket, addr)) = listener.accept().await {
            let tx = tx.clone();
            let mut rx = tx.subscribe();

            let ws_stream = accept_async(socket).await.expect("Error during the websocket handshake occurred");
            println!("WebSocket connection established: {}", addr);
            tokio::spawn(async move {
                let (mut write, mut read) = ws_stream.split();  
                loop {
                    
                    tokio::select! {
                        result = read.next() => {
                            match result {
                                Some(Ok(msg)) => {
                                    let msg_c = msg.clone();
                                    tx.send((msg, addr)).unwrap();
                                    println!("Received message: {}", msg_c);
                                },
                                None => break,
                                Some(Err(e)) => println!("{:?}", e),
                            }
                        }
                        result = rx.recv() => {
                            let(msg, other_addr) = result.unwrap();
                            if addr != other_addr {
                                if msg.is_text() {
                                    if let Err(err) = write.send(Message::Text(msg.to_string())).await {
                                        eprintln!("Error sending message: {}", err);
                                        break;
                                    }
                                } else if msg.is_binary() {
                                    if let Err(err) = write.send(Message::Binary(msg.into())).await {
                                        eprintln!("Error sending message: {}", err);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
             });
        }
    }
}
