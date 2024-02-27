use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug)]
pub enum SirError {
    JoinLeaveMessageDatabase,
    GenerateVoice,
    VoiceStateUpdate,
    NoVoiceId,
}

impl Display for self::SirError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            SirError::JoinLeaveMessageDatabase => write!(f, "Unable to create a list of join and leave messages, please check prerecordedtable.toml"),
            SirError::VoiceStateUpdate => write!(f, "Unexpected format of Voice state update event"),
            SirError::GenerateVoice => write!(f, "Error in generating the voice clip"),
            SirError::NoVoiceId => write!(f, "The bot can't join a vc if the user calling the command isn't in one"),
        }
    }
}
impl Error for SirError {}
