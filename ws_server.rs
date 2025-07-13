use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use sha1::{Sha1, Digest};
use base64::{engine::general_purpose, Engine};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("192.168.33.6:81").await.expect("Failed to create a socket!");
    println!("┌──Server running...\n└─» ws:...:81\n ┌───┐  ┌───┐\n | ▀▄▀  ▀▄▀ |\n └───┘  └───┘");
    
    let (tx, _) = broadcast::channel(10);
    
    loop {
        if let Ok((socket, addr)) = listener.accept().await {
            let tx = tx.clone();
            let mut rx = tx.subscribe();
            
            println!("New_con: {}", addr);
            
            tokio::spawn(async move {
                
                if let Err(e) = handle_connection(socket, addr, tx, rx).await {
                    eprintln!("Error with connection {}: {}", addr, e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    addr: std::net::SocketAddr,
    tx: broadcast::Sender<(Vec<u8>, std::net::SocketAddr)>,
    mut rx: broadcast::Receiver<(Vec<u8>, std::net::SocketAddr)>,
) -> Result<(), Box<dyn std::error::Error>> {

    let mut handshake = [0u8; 1024];
    let n = socket.read(&mut handshake).await?;
    let request = String::from_utf8_lossy(&handshake[..n]);

    if let Some(key_line) = request.lines().find(|line| line.starts_with("Sec-WebSocket-Key:")) {
        let key = key_line.trim().split(":").nth(1).unwrap().trim();
        let mut hasher = Sha1::new();
        hasher.update(format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key));
        let result = hasher.finalize();
        let accept_key = general_purpose::STANDARD.encode(result);

        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );
        socket.write_all(response.as_bytes()).await?;
    } else {
        return Err("Invalid WebSocket request".into());
    }

    let (mut read_half, mut write_half) = socket.split();
    let mut buffer = [0u8; 1024];

    loop {
        tokio::select! {
            result = read_half.read(&mut buffer) => {
                match result {
                    Ok(n) if n == 0 => break,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                };


                if buffer[0] != 0x81 {
                    eprintln!("Unsupported frame type");
                    break;
                }

                let payload_len = (buffer[1] & 0x7F) as usize;
                let masking_key = &buffer[2..6];
                let masked_payload = &buffer[6..6 + payload_len];

                let payload: Vec<u8> = masked_payload.iter()
                    .enumerate()
                    .map(|(i, b)| b ^ masking_key[i % 4])
                    .collect();

                tx.send((payload.clone(), addr))?;
                println!("R_M:.. {:?}: {}",&addr, String::from_utf8_lossy(&payload));
            }

            result = rx.recv() => {
                let (msg, other_addr) = result?;
                if addr != other_addr {
                    let mut frame = Vec::new();
                    frame.push(0x81); // FIN + text frame
                    frame.push(msg.len() as u8); // payload length
                    frame.extend_from_slice(&msg);

                    if let Err(e) = write_half.write_all(&frame).await {
                        eprintln!("Write error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}



