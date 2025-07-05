use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn process(mut socket: tokio::net::TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024]; // buffer for reading

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(_) => {
                // Process the received data (optional: print for debugging)
                println!("Received: {}", String::from_utf8_lossy(&buffer));

                // Write response
                let response = "+PONG\r\n";
                socket.write_all(response.as_bytes()).await?;
                socket.flush().await?;
            }
            Err(e) => return Err(e),
        }
    }

    println!("Connection closed, final buffer",);
    Ok(())
}

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        println!("accepted new connection");
        tokio::spawn(async move {
            if let Err(e) = process(socket).await {
                eprintln!("err prococessing socket {e}");
            }
        });
    }
}
