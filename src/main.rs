use crate::cmd::Command;
use crate::resp::RespListener;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

mod cmd;
mod resp;

async fn process(mut socket: TcpStream) -> Result<()> {
    loop {
        let Some(data_type) = RespListener::new().read(&mut socket).await? else {
            return Ok(());
        };

        let command = Command::parse(data_type);
        println!("command: {command:?}");

        let response = command.respond();

        socket.write_all(response.serialize().as_bytes()).await?;
        socket.flush().await?;
    }
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
