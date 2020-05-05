use crate::logic::repository::{RError, PError};
use std::fmt;

#[derive(Debug)]
pub struct BotError(BotErrorKind);

#[derive(Debug)]
pub(crate) enum BotErrorKind{
    TelegramError(telegram_bot::Error),
    PostgresError(PError),
    RuntimePostgresError(RError),
}

impl From<telegram_bot::Error> for BotError {
    fn from(error: telegram_bot::Error) -> Self {
        BotError(BotErrorKind::TelegramError(error))
    }
}

impl From<PError> for BotError {
    fn from(error: PError) -> Self {
        BotError(BotErrorKind::PostgresError(error))
    }
}

impl From<RError> for BotError {
    fn from(error: RError) -> Self {
        BotError(BotErrorKind::RuntimePostgresError(error))
    }
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            BotErrorKind::TelegramError(error) => write!(f, "{}", error),
            BotErrorKind::PostgresError(error) => write!(f, "{}", error),
            BotErrorKind::RuntimePostgresError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for BotError{}