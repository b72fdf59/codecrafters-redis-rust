use std::net::TcpListener;

use std::io::{Read, Write};

fn handle_client(mut stream: std::net::TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024]; // buffer for reading

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                // Process the received data (optional: print for debugging)
                println!("Received: {}", String::from_utf8_lossy(&buffer));

                // Write response
                let response = "+PONG\r\n";
                stream.write_all(response.as_bytes())?;
                stream.flush()?;
            }
            Err(e) => return Err(e),
        }
    }

    println!("Connection closed, final buffer",);
    Ok(())
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                if let Err(e) = handle_client(stream) {
                    eprintln!("Error handling client: {e}")
                }
            }
            Err(e) => {
                println!("error: {e}");
            }
        }
    }
}
