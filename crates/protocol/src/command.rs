use thiserror::Error;

use crate::frame::Frame;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid frame")]
    InvalidFrame,
    #[error("invalid arg: {0}")]
    InvalidArg(String),
    #[error("too many args")]
    TooManyArgs,
    #[error("not enough args")]
    NotEnoughArgs,
    #[error("invalid command")]
    InvalidCommand,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(pub Box<[u8]>);

#[derive(Debug, Clone)]
pub struct Value(pub Box<[u8]>);

#[derive(Debug)]
pub enum Command<'a> {
    Ping(PingCommand<'a>),
    Get(GetCommand),
    Set(SetCommand),
    ConfigGet(ConfigGetCommand),
}

#[derive(Debug)]
pub struct PingCommand<'a> {
    pub message: Option<&'a str>,
}

#[derive(Debug)]
pub struct GetCommand {
    pub key: Key,
}

#[derive(Debug)]
pub struct SetCommand {
    pub key: Key,
    pub value: Value,
}

#[derive(Debug)]
pub struct ConfigGetCommand {
    pub parameter: String,
}

impl<'a> Command<'a> {
    pub fn from_frame(frame: &'a Frame) -> Result<Self> {
        let frames = frame.as_array().map_err(|_| Error::InvalidFrame)?;

        if frames.is_empty() {
            return Err(Error::InvalidFrame);
        }

        match frames[0].as_bulk_string().map_err(|_| Error::InvalidFrame)? {
                "PING" => Self::parse_ping(&frames[1..]),
                "GET" => Self::parse_get(&frames[1..]),
                "SET" => Self::parse_set(&frames[1..]),
                "CONFIG" => {
                    if frames.len() >= 2 {
                        match frames[1].as_bulk_string().map_err(|_| Error::InvalidFrame)? {
                            "GET" => Self::parse_config_get(&frames[2..]),
                            _ => Err(Error::InvalidCommand),
                        }
                    }
                    else {
                        Err(Error::InvalidCommand)
                    }
                }
                _ => Err(Error::InvalidCommand),
        }
    }

    fn parse_ping(frames: &'a [Frame]) -> Result<Self> {
        if frames.is_empty() {
            return Ok(Command::Ping(PingCommand { message: None }));
        }

        if frames.len() != 1 {
            return Err(Error::TooManyArgs);
        }

        match &frames[0] {
            Frame::BulkString(value) => {
                Ok(Command::Ping(PingCommand { message: Some(value.as_str()) }))
            }
            _ => Err(Error::InvalidArg("invalid message frame".to_string())),
        }
    }

    fn parse_get(frames: &'a [Frame]) -> Result<Self> {
        if frames.is_empty() {
            return Err(Error::NotEnoughArgs);
        }

        if frames.len() > 1 {
            return Err(Error::TooManyArgs);
        }

        let key = Key(frames[0]
            .as_bulk_string()
            .map_err(|_| Error::InvalidArg("invalid key".to_string()))?
            .as_bytes()
            .to_vec()
            .into_boxed_slice());

        Ok(Command::Get(GetCommand { key }))
    }

    fn parse_set(frames: &'a [Frame]) -> Result<Self> {
        if frames.len() < 2 {
            return Err(Error::NotEnoughArgs);
        }

        if frames.len() > 2 {
            return Err(Error::TooManyArgs);
        }

        let key = Key(frames[0]
            .as_bulk_string()
            .map_err(|_| Error::InvalidArg("invalid key".to_string()))?
            .as_bytes()
            .to_vec()
            .into_boxed_slice());
        let value = Value(
            frames[1]
                .as_bulk_string()
                .map_err(|_| Error::InvalidArg("invalid value".to_string()))?
                .as_bytes()
                .to_vec()
                .into_boxed_slice(),
        );

        Ok(Command::Set(SetCommand { key, value }))
    }

    fn parse_config_get(frames: &'a [Frame]) -> Result<Self> {
        if frames.is_empty() {
            return Err(Error::NotEnoughArgs);
        }

        if frames.len() > 1 {
            return Err(Error::TooManyArgs);
        }

        let parameter = frames[0]
            .as_bulk_string()
            .map_err(|_| Error::InvalidArg("invalid key".to_string()))?;

        const ALLOWED_PARAMETER_VALUES: [&str; 1] = ["save"];

        if ALLOWED_PARAMETER_VALUES.contains(&parameter) {
            Ok(Command::ConfigGet(ConfigGetCommand { parameter: parameter.to_string() }))
        }
        else {
            Err(Error::InvalidArg("not supported parameter".to_string()))
        }
    }

    pub fn key(&self) -> Option<Key> {
        match self {
            Command::Ping(..) => None,
            Command::Get(get_command) => Some(get_command.key.clone()),
            Command::Set(set_command) => Some(set_command.key.clone()),
            Command::ConfigGet(..) => None,
        }
    }
}
