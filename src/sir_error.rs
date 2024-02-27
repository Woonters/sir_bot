use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};
use thiserror::Error;

#[derive(Debug)]
pub enum SirError {
    JoinLeaveMessageDatabaseError,
    GenerateVoiceError,
    VoiceStateUpdateError,
    NoVoiceIdError,
}

impl Display for self::SirError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            SirError::JoinLeaveMessageDatabaseError => write!(f, "Unable to create a list of join and leave messages, please check prerecordedtable.toml"),
            SirError::VoiceStateUpdateError => write!(f, "Unexpected format of Voice state update event"),
            SirError::GenerateVoiceError => write!(f, "Error in generating the voice clip"),
            SirError::NoVoiceIdError => write!(f, "The bot can't join a vc if the user calling the command isn't in one"),
        }
    }
}
impl Error for SirError {}
