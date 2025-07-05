use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn process(mut socket: tokio::net::TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024]; // buffer for reading
    let mut data = Vec::new();

    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        data.extend_from_slice(&buffer[..n]);

        while let Some((command, argument, consumed)) = parse_resp(&data) {
            let response = match (command.to_uppercase().as_str(), argument.as_str()) {
                ("PING", _) => "+PONG\r\n".to_string(),
                ("ECHO", "") => "-ERR ECHO requires an argument\r\n".to_string(),
                ("ECHO", arg) => format!("+{}\r\n", arg),
                _ => "-ERR unknown command\r\n".to_string(),
            };

            // Send response back to client
            socket.write_all(response.as_bytes()).await?;
            socket.flush().await?;

            println!("drained {:?}", ..consumed);
            data.drain(..consumed);
        }
    }

    println!("Connection closed, final buffer",);
    Ok(())
}

fn parse_resp(data: &[u8]) -> Option<(String, String, usize)> {
    if data.is_empty() {
        return None;
    }

    match data[0] as char {
        '+' => parse_simple_string_resp(&data),
        '*' -> parse_bulk_string_resp(&data),
        _ => None,
    }

    // Process the received data (optional: print for debugging)
    // if let Some(prefix_stripped_input) = input.strip_prefix("+") {
    //     println!("Received prefix stripped input: {prefix_stripped_input}");
    //     if let Some(command) = prefix_stripped_input[1..].strip_suffix("\r\n") {
    //         println!("Received command: {command}");
    //         return (command.to_string(), String::new());
    //     }
    // }
    //
}

fn parse_simple_string_resp(data: &[u8]) -> Option<(String, String, usize)> {
    if data.is_empty() || data[0] as char != '+' {
        return None;
    }

    if let Some(pos) = data.windows(2).position(|w| w == b"\r\n") {
        let end = pos + 2;
        let s = String::from_utf8_lossy(&data[1..pos]).to_string();
        return Some((s, String::new(), end));
    }

    None
}


fn parse_bulk_string_resp(data: &[u8]) -> Option<(String, String, usize)> {
    if data.is_empty() || data[0] as char != '$' {
        return None;
    }

    if let Some(pos) = data.windows(2).position(|w| w == b"") {
        let end = pos + 2;
        let s = String::from_utf8_lossy(&data[1..pos]).to_string();
        return Some((s, String::new(), end));
    }

    None
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
