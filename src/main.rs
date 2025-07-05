use crate::cmd::Command;
use crate::resp::RespListener;
use anyhow::Result;
use bytes::Bytes;
use resp::DataType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

mod cmd;
mod resp;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

async fn process(mut socket: TcpStream, db: Db) -> Result<()> {
    loop {
        let Some(data_type) = RespListener::new().read(&mut socket).await? else {
            return Ok(());
        };

        let command = Command::parse(data_type);
        println!("command: {command:?}");

        let response = match command {
            Command::Ping => DataType::SimpleString("PONG".to_string()),
            Command::Echo(s) => DataType::BulkString(s),
            Command::Set(key, value) => {
                let mut db = db.lock().unwrap();
                db.insert(key, Bytes::from(value));
                DataType::BulkString("OK".to_string())
            }
            Command::Get(key) => {
                let db = db.lock().unwrap();
                match db.get(&key) {
                    Some(value) => DataType::BulkString(String::from_utf8_lossy(value).to_string()),
                    None => DataType::Null,
                }
            }
            Command::Unknown(s) => DataType::Error(s),
        };

        socket.write_all(response.serialize().as_bytes()).await?;
        socket.flush().await?;
    }
}

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Listening!");
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        println!("accepted new connection");
        tokio::spawn(async move {
            if let Err(e) = process(socket, db).await {
                eprintln!("err prococessing socket {e}");
            }
        });
    }
}
