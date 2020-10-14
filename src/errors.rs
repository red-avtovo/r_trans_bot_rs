use std::{fmt,error};
use crate::{
    fromError,
    fromErrorString
};

/// ***************
/// Bot Errors
/// ***************
#[derive(Debug)]
pub struct BotError(BotErrorKind);

impl BotError {
    pub(crate) fn new(kind: BotErrorKind) -> BotError {
        BotError(kind)
    }

    #[allow(dead_code)]
    pub(crate) fn logic(message: String) -> BotError {
        BotError(BotErrorKind::BotLogic(message))
    }

}

#[derive(Debug)]
pub(crate) enum BotErrorKind{
    TelegramError(telegram_bot::Error),
    DbError(DbError),
    BotLogic(String)
}

fromError!(telegram_bot::Error, BotError, BotErrorKind::TelegramError);
fromError!(DbError, BotError, BotErrorKind::DbError);

fromError!(r2d2::Error, BotError, BotErrorKind::DbError);
fromErrorString!(r2d2::Error, DbError, DbErrorKind::Connection);
fromErrorString!(diesel::result::Error, DbError, DbErrorKind::Execution);
fromErrorString!(String, DbError, DbErrorKind::Execution);

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            BotErrorKind::TelegramError(error) => write!(f, "{}", error),
            BotErrorKind::DbError(error) => write!(f, "{}", error),
            BotErrorKind::BotLogic(error) => write!(f, "{}", error),
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
    #[allow(dead_code)]
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
    Connection(String),
    Execution(String)
}

#[derive(Debug)]
pub struct DbError(DbErrorKind);

impl DbError {
    pub(crate) fn new(kind: DbErrorKind) -> DbError {
        DbError(kind)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            DbErrorKind::Connection(error) => write!(f, "{}", error),
            DbErrorKind::Execution(error) => write!(f, "{}", error),
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
                <$error>::new($kind(error.into()))
            }
        }
    };
}

#[macro_export]
macro_rules! fromErrorString {
    (
        $from: ty,
        $error: ty,
        $kind: path
    ) => {
        impl From<$from> for $error {
            fn from(error: $from) -> Self {
                <$error>::new($kind(format!("{}", error)))
            }
        }
    };
}

#[cfg(test)]
#[allow(dead_code, non_snake_case)]
mod test {

    use super::*;

    //compilation test
    fn r2d2_BotError(e: r2d2::Error) -> BotError { e.into() }
    fn r2d2_DbError(e: r2d2::Error) -> DbError { e.into() }
    fn diesel_result_ErrorDbError(e: diesel::result::Error) -> DbError { e.into() }
    fn string_DbError(e: String) -> DbError { e.into() }
}