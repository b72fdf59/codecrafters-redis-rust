use crate::resp::DataType;

#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(String),
    Get(String),
    Set(String, String),
    Unknown(String),
}

impl Command {
    pub fn parse(v: DataType) -> Self {
        match v {
            DataType::Array(a) => {
                if a.is_empty() {
                    return Self::Unknown("empty command".to_string());
                }

                let mut args = a.into_iter();
                let command = match args.next().unwrap() {
                    DataType::SimpleString(s) => s,
                    DataType::BulkString(bs) => bs,
                    other => {
                        return Self::Unknown(format!("expected a bulk string, got {other:?}"))
                    }
                };

                match command.to_lowercase().as_str() {
                    "ping" => Self::Ping,
                    "echo" => {
                        let arg1 = match args.next() {
                            Some(DataType::SimpleString(s)) => s,
                            Some(DataType::BulkString(bs)) => bs,
                            other => {
                                return Self::Unknown(format!(
                                    "expected a bulk string, got {other:?}"
                                ))
                            }
                        };
                        Self::Echo(arg1)
                    }
                    "get" => {
                        let key = match args.next() {
                            Some(DataType::SimpleString(s)) => s,
                            Some(DataType::BulkString(bs)) => bs,
                            other => {
                                return Self::Unknown(format!(
                                    "expected a bulk string, got {other:?}"
                                ))
                            }
                        };
                        Self::Get(key)
                    }
                    "set" => {
                        let key = match args.next() {
                            Some(DataType::SimpleString(s)) => s,
                            Some(DataType::BulkString(bs)) => bs,
                            other => {
                                return Self::Unknown(format!(
                                    "expected a bulk string, got {other:?}"
                                ))
                            }
                        };
                        let value = match args.next() {
                            Some(DataType::SimpleString(s)) => s,
                            Some(DataType::BulkString(bs)) => bs,
                            other => {
                                return Self::Unknown(format!(
                                    "expected a bulk string, got {other:?}"
                                ))
                            }
                        };
                        Self::Set(key, value)
                    }
                    other => Self::Unknown(other.to_string()),
                }
            }
            other => Self::Unknown(format!("expected an array, got {other:?}")),
        }
    }

    pub fn respond(self) -> DataType {
        match self {
            Self::Ping => DataType::SimpleString("PONG".to_string()),
            Self::Echo(s) => DataType::BulkString(s),
            Self::Set(_, _) => DataType::SimpleString("OK".to_string()),
            Self::Get(_) => DataType::SimpleString("".to_string()),
            Self::Unknown(s) => DataType::Error(s),
        }
    }
}

