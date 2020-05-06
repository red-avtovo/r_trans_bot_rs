use crate::logic::repository::{RError, PError};
use std::{fmt,error};

/// ***************
/// Bot Errors
/// ***************
#[derive(Debug)]
pub struct BotError(BotErrorKind);

#[derive(Debug)]
pub(crate) enum BotErrorKind{
    TelegramError(telegram_bot::Error),
    DbError(DbError),
}

impl From<telegram_bot::Error> for BotError {
    fn from(error: telegram_bot::Error) -> Self {
        BotError(BotErrorKind::TelegramError(error))
    }
}

impl From<DbError> for BotError {
    fn from(error: DbError) -> Self {
        BotError(BotErrorKind::DbError(error))
    }
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            BotErrorKind::TelegramError(error) => write!(f, "{}", error),
            BotErrorKind::DbError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for BotError{}

/// ***************
/// Magnet Generation Error
/// ***************
#[derive(Debug, Clone)]
pub struct MagnetMappingError(&'static str);

impl MagnetMappingError {
    pub(crate) fn new(str: &'static str) -> Self {
        MagnetMappingError(str)
    }
}
    
impl fmt::Display for MagnetMappingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Magnet Mapping error: {}", self.0)
    }
}

impl error::Error for MagnetMappingError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}


/// ***************
/// DB ERRORS
/// ***************
#[derive(Debug)]
enum DbErrorKind {
    PostgresError(PError),
    RuntimePostgresError(RError),
    NotFoundError(&'static str),
}

#[derive(Debug)]
pub struct DbError(DbErrorKind);

impl From<PError> for DbError {
    fn from(error: PError) -> Self {
        DbError(DbErrorKind::PostgresError(error))
    }
}

impl From<RError> for DbError {
    fn from(error: RError) -> Self {
        DbError(DbErrorKind::RuntimePostgresError(error))
    }
}

impl From<&'static str> for DbError {
    fn from(error: &'static str) -> Self {
        DbError(DbErrorKind::NotFoundError(error))
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            DbErrorKind::PostgresError(error) => write!(f, "{}", error),
            DbErrorKind::RuntimePostgresError(error) => write!(f, "{}", error),
            DbErrorKind::NotFoundError(error) => write!(f, "{}", error),
        }
    }
}