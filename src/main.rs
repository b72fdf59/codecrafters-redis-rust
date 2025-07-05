use crate::cmd::Command;
use crate::resp::RespListener;
use anyhow::Result;
use bytes::Bytes;
use clap::Parser;
use resp::DataType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

mod cmd;
mod resp;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 6379)]
    port: u32,
}

#[derive(Debug)]
struct ExpirableValue {
    value: Bytes,
    expiry: Option<Instant>,
}

type Db = Arc<Mutex<HashMap<String, ExpirableValue>>>;

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
            Command::Set(key, value, expiry_ms) => {
                let mut db = db.lock().unwrap();
                let expiry = expiry_ms.map(|ms| Instant::now() + Duration::from_millis(ms));
                db.insert(
                    key,
                    ExpirableValue {
                        value: Bytes::from(value),
                        expiry,
                    },
                );
                DataType::SimpleString("OK".to_string())
            }
            Command::Get(key) => {
                let mut db = db.lock().unwrap();
                match db.get(&key) {
                    Some(expirable_value) => {
                        if let Some(expiry) = expirable_value.expiry {
                            if expiry <= Instant::now() {
                                db.remove(&key);
                                DataType::Null
                            } else {
                                DataType::BulkString(
                                    String::from_utf8_lossy(&expirable_value.value).to_string(),
                                )
                            }
                        } else {
                            DataType::BulkString(
                                String::from_utf8_lossy(&expirable_value.value).to_string(),
                            )
                        }
                    }
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
    let args = Args::parse();

    let port = args.port;
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    println!("Listening on port {port}");

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
