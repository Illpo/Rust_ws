mod files;

use std::env;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use files::{serve, serve_static, login, register};

#[tokio::main]
async fn main() {
    
    //let args: Vec<String> = env.args().collect();
    
    let listener = TcpListener::bind("localhost:80").await.expect("Faild creat a socket!");
    println!("┌──Server running...\n└─» Go to localhost\n ┌───┐  ┌───┐\n | ▀▄▀  ▀▄▀ |\n └───┘  └───┘");

    loop {

        let (mut socket, _addr) = listener.accept().await.unwrap();
        
        tokio::spawn(async move {
           
            println!("New connection: {}", _addr);
                      
            let mut buff = [0; 1024];

            if let Ok(n) = socket.read(&mut buff).await {
                
                if n > 0 {

                    let req = String::from_utf8_lossy(&buff[..n]);
                    let v: Vec<&str> = req.split_terminator('\n').collect();
                    let mut f = v[0].split_whitespace();
                    let method = f.next().unwrap();
                    let path = f.next().unwrap();
                    let last_index = &v.len() - 1;
                    println!("{}", req);

                    if method == "GET" {
                        match path {
                            "/" => {
                                _ = serve("index.html", socket).await;
                            }
                            "/chat" => {
                                _ = serve("chat.html", socket).await;
                            }
                            _ => {
                                _ = serve_static(path, socket).await;
                            }
                        }
                        
                    } else if method == "POST" {
                        match path {
                          "/login" => {
                            }
                            "/register" => {
                            }
                            _ => {
                                println!("Ooops");
                            }
                        }

                    } else {
                        println!("No");
                    }
                }
            }
        });
    }
}
