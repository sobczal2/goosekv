use thiserror::Error;

use crate::{
    data_type::GString,
    frame::GFrame,
};

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

#[derive(Debug)]
pub enum GCommand {
    Ping(PingGCommand),
    Get(GetGCommand),
    Set(SetGCommand),
    Del(DelGCommand),
    Exists(ExistsGCommand),
    ConfigGet(ConfigGetGCommand),
}

#[derive(Debug)]
pub struct PingGCommand {
    pub message: Option<GString>,
}

#[derive(Debug)]
pub struct GetGCommand {
    pub key: GString,
}

#[derive(Debug)]
pub struct SetGCommand {
    pub key: GString,
    pub value: GString,
}

#[derive(Debug)]
pub struct DelGCommand {
    pub keys: Box<[GString]>,
}

#[derive(Debug)]
pub struct ExistsGCommand {
    pub keys: Box<[GString]>,
}

#[derive(Debug)]
pub struct ConfigGetGCommand {
    pub parameter: GString,
}

impl GCommand {
    pub fn from_frame(frame: &GFrame) -> Result<Self> {
        let frames = frame.as_array().map_err(|_| Error::InvalidFrame)?;

        if frames.is_empty() {
            return Err(Error::InvalidFrame);
        }

        match frames[0].as_bulk_string().map_err(|_| Error::InvalidFrame)?.bytes().as_ref() {
            b"PING" => Self::parse_ping(&frames[1..]),
            b"GET" => Self::parse_get(&frames[1..]),
            b"SET" => Self::parse_set(&frames[1..]),
            b"DEL" => Self::parse_del(&frames[1..]),
            b"EXISTS" => Self::parse_exists(&frames[1..]),
            b"CONFIG" => {
                if frames.len() >= 2 {
                    match frames[1].as_bulk_string().map_err(|_| Error::InvalidFrame)?.bytes().as_ref() {
                        b"GET" => Self::parse_config_get(&frames[2..]),
                        _ => Err(Error::InvalidCommand),
                    }
                } else {
                    Err(Error::InvalidCommand)
                }
            }
            _ => Err(Error::InvalidCommand),
        }
    }

    fn parse_ping(frames: &[GFrame]) -> Result<Self> {
        if frames.is_empty() {
            return Ok(GCommand::Ping(PingGCommand { message: None }));
        }

        if frames.len() != 1 {
            return Err(Error::TooManyArgs);
        }

        match frames[0] {
            GFrame::BulkString(ref value) => {
                Ok(GCommand::Ping(PingGCommand { message: Some(value.clone()) }))
            }
            _ => Err(Error::InvalidArg("invalid message frame".to_string())),
        }
    }

    fn parse_get(frames: &[GFrame]) -> Result<Self> {
        if frames.is_empty() {
            return Err(Error::NotEnoughArgs);
        }

        if frames.len() > 1 {
            return Err(Error::TooManyArgs);
        }

        let key =
            frames[0].as_bulk_string().map_err(|_| Error::InvalidArg("invalid key".to_string()))?;

        Ok(GCommand::Get(GetGCommand { key }))
    }

    fn parse_set(frames: &[GFrame]) -> Result<Self> {
        if frames.len() < 2 {
            return Err(Error::NotEnoughArgs);
        }

        if frames.len() > 2 {
            return Err(Error::TooManyArgs);
        }

        let key =
            frames[0].as_bulk_string().map_err(|_| Error::InvalidArg("invalid key".to_string()))?;

        let value = frames[1]
            .as_bulk_string()
            .map_err(|_| Error::InvalidArg("invalid value".to_string()))?;

        Ok(GCommand::Set(SetGCommand { key, value }))
    }

    fn parse_del(frames: &[GFrame]) -> Result<Self> {
        if frames.is_empty() {
            return Err(Error::NotEnoughArgs);
        }

        let keys = frames
            .iter()
            .map(|frame| {
                frame
                    .as_bulk_string()
                    .map_err(|_| Error::InvalidArg("invalid value".to_string()))
            })
            .collect::<Result<_>>()?;

        Ok(GCommand::Del(DelGCommand { keys }))
    }
    
    fn parse_exists(frames: &[GFrame]) -> Result<Self> {
        if frames.is_empty() {
            return Err(Error::NotEnoughArgs);
        }

        let keys = frames
            .iter()
            .map(|frame| {
                frame
                    .as_bulk_string()
                    .map_err(|_| Error::InvalidArg("invalid value".to_string()))
            })
            .collect::<Result<_>>()?;

        Ok(GCommand::Exists(ExistsGCommand { keys }))
    }

    fn parse_config_get(frames: &[GFrame]) -> Result<Self> {
        if frames.is_empty() {
            return Err(Error::NotEnoughArgs);
        }

        if frames.len() > 1 {
            return Err(Error::TooManyArgs);
        }

        let parameter =
            frames[0].as_bulk_string().map_err(|_| Error::InvalidArg("invalid key".to_string()))?;

        const ALLOWED_PARAMETER_VALUES: [&[u8]; 1] = [b"save"];

        if ALLOWED_PARAMETER_VALUES.contains(&parameter.bytes().as_ref()) {
            Ok(GCommand::ConfigGet(ConfigGetGCommand { parameter }))
        } else {
            Err(Error::InvalidArg("not supported parameter".to_string()))
        }
    }
}
