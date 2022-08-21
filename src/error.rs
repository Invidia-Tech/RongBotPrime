use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    num::ParseIntError,
};

#[derive(Debug)]
pub enum RongError {
    Database(sqlx::Error),
    Parsing(ParseIntError),
    Serenity(serenity::Error),
    Custom(String),
}

impl Display for RongError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let error = match self {
            RongError::Database(e) => e.to_string(),
            RongError::Parsing(e) => e.to_string(),
            RongError::Serenity(e) => e.to_string(),
            RongError::Custom(e) => e.to_string(),
        };
        f.write_str(&error)?;
        Ok(())
    }
}

impl Error for RongError {}

impl From<sqlx::Error> for RongError {
    fn from(err: sqlx::Error) -> RongError { RongError::Database(err) }
}

impl From<ParseIntError> for RongError {
    fn from(err: ParseIntError) -> RongError { RongError::Parsing(err) }
}

impl From<String> for RongError {
    fn from(err: String) -> RongError { RongError::Custom(err) }
}

impl From<serenity::Error> for RongError {
    fn from(err: serenity::Error) -> RongError { RongError::Serenity(err) }
}
