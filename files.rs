use std::path::Path;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::fs;


fn handle_type(ex: &str) -> String {
    let mut content_type = String::new();
    if let Some(extension) = Path::new(&ex).extension() {
        if let Some(extension_str) = extension.to_str() {
            match extension_str {
                "html" => content_type = String::from("text/html"),
                "css" => content_type = String::from("text/css"),
                "jpg" | "jpeg" | "jfif"=> content_type = String::from("image/jpeg"),
                "ico" => content_type = String::from("image/x-icon"),
                "mp4" => content_type = String::from("video/mp4"),
                "png" => content_type = String::from("image/png"),
                _ => content_type = String::from("application/octet-stream"),
            };
        }
    }

    return content_type;
}

pub async fn serve(file: &str, mut socket: TcpStream) -> Result<(), std::io::Error> {
    let content_type = handle_type(file);
    match fs::read(file) {
        Ok(file) => {
            let form = format!("HTTP/1.1 200 OK\r\nServer: CIA/9.0.1\r\nConnection: close\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n", content_type, file.len());
            println!("{}", form);
            _ = socket.write(&form.as_bytes()).await;
            _ = socket.write(&file).await;

            Ok(())
        }
        Err(e) => {
            let error_response = format!("HTTP/1.1 500 Server Error\r\n\r\n{}", e);
            println!("{}", error_response);
            _ = socket.write(error_response.as_bytes()).await;
            Err(e)
        }
    }
}

pub async fn register(mut socket: TcpStream) {
    
    let form = format!("HTTP/1.1 301 Move and Load\r\nConnection: close\r\nLocation: /login\r\n\r\n");
    println!("{}", form);
    _ = socket.write(&form.as_bytes()).await;

}

pub async fn login(cookie: String, mut socket: TcpStream) {
    
    let form = format!("HTTP/1.1 301 Move and Load\r\nConnection: close\r\nSet-Cookie: session_id={}; max-age=40\r\nLocation: /\r\n\r\n", cookie);
    println!("{}", form);
    _ = socket.write(&form.as_bytes()).await;

}

pub async fn serve_static(url: &str, mut socket: TcpStream)  {
    let mut stut = false;

        if url.starts_with("/static") || url.starts_with("/favicon") {
            let file = &mut url.to_string();
            file.remove(0);
            let content_type = handle_type(file);
            match fs::read(file) {
                Ok(file) => {
                    let form = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nConnection: close\r\nCache-Control: max-age=2\r\nContent-Length: {}\r\n\r\n", content_type, file.len());
                    println!("{}", form);
                    _ = socket.write(&form.as_bytes()).await;
                    _ = socket.write(&file).await;
                }
                Err(e) => {
                    let error_response = format!("HTTP/1.1 500 Server Error No No\r\nConnection: close\r\n\r\n{}", e);
                    println!("{}", error_response);
                    _ = socket.write(error_response.as_bytes()).await;
                }
            }
            stut = true; 
        }
        
    if !stut {
        let error_response = format!("HTTP/1.1 500 Server Error 0_0\r\nConnection: close\r\n\r\n");
        let msg = String::from("<h1>Nooo, bad</h1> <a href=\"https://www.youtube.com/watch?v=dQw4w9WgXcQ\"><h1>Here</h1></a>");
        println!("{}", error_response);
        _ = socket.write(error_response.as_bytes()).await;
        _ = socket.write(msg.as_bytes()).await;
    }
}
