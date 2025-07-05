use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Null,
    SimpleString(String),
    BulkString(String),
    Error(String),
    Array(Vec<DataType>),
}

impl DataType {
    pub fn serialize(self) -> String {
        match self {
            DataType::Null => "$-1\r\n".to_string(),
            DataType::SimpleString(s) => format!("+{s}\r\n"),
            DataType::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            DataType::Array(arr) => {
                let mut result = format!("* {}\r\n", arr.len());
                for item in arr {
                    result.push_str(&item.serialize());
                }
                result
            }
            DataType::Error(s) => format!("-{s}\r\n"),
        }
    }
}

pub struct RespListener {
    data: BytesMut,
}

impl RespListener {
    pub fn new() -> Self {
        Self {
            data: BytesMut::with_capacity(1024),
        }
    }

    pub async fn read(&mut self, stream: &mut TcpStream) -> Result<Option<DataType>> {
        let n = stream.read_buf(&mut self.data).await?;

        if n == 0 {
            // Connection closed by peer
            return Ok(None);
        }

        if let Some((data_type, consumed)) = parse_resp(&self.data)? {
            self.data.advance(consumed);
            Ok(Some(data_type))
        } else {
            // Not enough data to parse a full message
            Ok(None)
        }
    }
}

pub fn parse_resp(data: &[u8]) -> Result<Option<(DataType, usize)>> {
    if data.is_empty() {
        return Ok(None);
    }

    match data[0] as char {
        '+' => parse_simple_string(data),
        '$' => parse_bulk_string(data),
        '*' => parse_array(data),
        _ => Err(anyhow!("Unknown mesage: {data:?}")),
    }
}

fn parse_simple_string(buffer: &[u8]) -> Result<Option<(DataType, usize)>> {
    if let Some((line, len)) = read_line(buffer) {
        let s = String::from_utf8(line[1..].to_vec())
            .map_err(|_| anyhow!("Invalid UTF-8 in simple string"))?;
        Ok(Some((DataType::SimpleString(s), len)))
    } else {
        Ok(None)
    }
}

fn parse_bulk_string(buffer: &[u8]) -> Result<Option<(DataType, usize)>> {
    let (line, line_len) = match read_line(buffer) {
        Some(it) => it,
        None => return Ok(None),
    };

    let len_str = std::str::from_utf8(&line[1..])
        .map_err(|_| anyhow!("Invalid UTF-8 in bulk string length"))?;
    let len: isize = len_str
        .parse()
        .map_err(|_| anyhow!("Invalid bulk string length"))?;

    if len == -1 {
        return Ok(Some((DataType::BulkString("".to_string()), line_len)));
    }

    let len = len as usize;
    let total_len = line_len + len + 2;

    if buffer.len() < total_len {
        // TODO: send error message back: mismatch string length?
        return Ok(None);
    }

    if &buffer[total_len - 2..total_len] != b"\r\n" {
        // TODO: send error message back
        return Err(anyhow!("Bulk string missing trailing CRLF"));
    }

    let data = &buffer[line_len..line_len + len];
    let s = String::from_utf8(data.to_vec())
        .map_err(|_| anyhow!("Invalid UTF-8 in bulk string data"))?;

    Ok(Some((DataType::BulkString(s), total_len)))
}

fn parse_array(buffer: &[u8]) -> Result<Option<(DataType, usize)>> {
    let (line, line_len) = match read_line(buffer) {
        Some(it) => it,
        None => return Ok(None),
    };

    let count_str =
        std::str::from_utf8(&line[1..]).map_err(|_| anyhow!("Invalid UTF-8 in array length"))?;
    let count: isize = count_str
        .parse()
        .map_err(|_| anyhow!("Invalid array length"))?;

    if count == -1 {
        return Ok(Some((DataType::Array(vec![]), line_len)));
    }

    let count = count as usize;
    let mut elements = Vec::with_capacity(count);
    let mut total_consumed = line_len;

    for _ in 0..count {
        match parse_resp(&buffer[total_consumed..])? {
            Some((element, consumed)) => {
                elements.push(element);
                total_consumed += consumed;
            }
            None => return Ok(None),
        }
    }

    Ok(Some((DataType::Array(elements), total_consumed)))
}

fn find_crlf(buffer: &[u8]) -> Option<usize> {
    buffer.windows(2).position(|window| window == b"\r\n")
}

fn read_line(buffer: &[u8]) -> Option<(&[u8], usize)> {
    find_crlf(buffer).map(|pos| (&buffer[..pos], pos + 2))
}
