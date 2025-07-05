use resp::{parse_resp, DataType, RespListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

mod resp;

async fn process(mut socket: tokio::net::TcpStream) -> std::io::Result<()> {
    loop {
        let resp_listener = RespListener::new();
        let response = match resp_listener.read(&mut socket) {
            DataType::SimpleString(s) => {

            }

        }


            // Send response back to client
            socket.write_all(response.serialise()).await?;
            socket.flush().await?;
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
