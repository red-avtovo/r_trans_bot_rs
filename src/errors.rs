use crate::logic::repository::{RError, PError};
use std::{fmt,error};
use crate::fromError;

/// ***************
/// Bot Errors
/// ***************
#[derive(Debug)]
pub struct BotError(BotErrorKind);

impl BotError {
    pub(crate) fn new(kind: BotErrorKind) -> BotError {
        BotError(kind)
    }
}

#[derive(Debug)]
pub(crate) enum BotErrorKind{
    TelegramError(telegram_bot::Error),
    DbError(DbError),
}

fromError!(telegram_bot::Error, BotError, BotErrorKind::TelegramError);
fromError!(DbError, BotError, BotErrorKind::DbError);

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
pub(crate) enum DbErrorKind {
    PostgresError(PError),
    RuntimePostgresError(RError),
    NotFoundError(&'static str),
}

#[derive(Debug)]
pub struct DbError(DbErrorKind);

impl DbError {
    pub(crate) fn new(kind: DbErrorKind) -> DbError {
        DbError(kind)
    }
}

fromError!(PError, DbError, DbErrorKind::PostgresError);
fromError!(RError, DbError, DbErrorKind::RuntimePostgresError);
fromError!(&'static str, DbError, DbErrorKind::NotFoundError);

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            DbErrorKind::PostgresError(error) => write!(f, "{}", error),
            DbErrorKind::RuntimePostgresError(error) => write!(f, "{}", error),
            DbErrorKind::NotFoundError(error) => write!(f, "{}", error),
        }
    }
}

#[macro_export]
macro_rules! fromError {
    (
        $from: ty,
        $error: ty,
        $kind: path
    ) => {
        impl From<$from> for $error {
            fn from(error: $from) -> Self {
                <$error>::new($kind(error))
            }
        }
    };
}