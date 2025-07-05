use crate::resp::DataType;

#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(String),
    Get(String),
    Set(String, String, Option<u64>),
    Info(String),
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

                        let mut expiry = None;
                        if let Some(arg) = args.next() {
                            let next_arg = match arg {
                                DataType::SimpleString(s) => s,
                                DataType::BulkString(bs) => bs,
                                _ => {
                                    return Self::Unknown(
                                        "Invalid argument type for SET option".to_string(),
                                    )
                                }
                            };

                            if next_arg.to_lowercase() == "px" {
                                if let Some(expiry_val_arg) = args.next() {
                                    let expiry_val_str = match expiry_val_arg {
                                        DataType::SimpleString(s) => s,
                                        DataType::BulkString(bs) => bs,
                                        _ => {
                                            return Self::Unknown(
                                                "Invalid value for PX option".to_string(),
                                            )
                                        }
                                    };
                                    if let Ok(ms) = expiry_val_str.parse::<u64>() {
                                        expiry = Some(ms);
                                    } else {
                                        return Self::Unknown(
                                            "PX value must be a positive integer".to_string(),
                                        );
                                    }
                                } else {
                                    return Self::Unknown("PX option requires a value".to_string());
                                }
                            }
                        }
                        Self::Set(key, value, expiry)
                    }
                    "info" => {
                        let section = match args.next() {
                            Some(DataType::SimpleString(s)) => s,
                            other => {
                                return Self::Unknown(format!(
                                    "expected a bulk string, got {other:?}"
                                ))
                            }
                        };
                        Self::Info(section)
                    }
                    other => Self::Unknown(other.to_string()),
                }
            }
            other => Self::Unknown(format!("expected an array, got {other:?}")),
        }
    }
}
