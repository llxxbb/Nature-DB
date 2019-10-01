use diesel::result::*;

use nature_common::*;

pub struct DbError;

impl DbError {
    // put it aside because can't find diesel's Timeout Error
    pub fn from(err: Error) -> NatureError {
        Self::from_with_msg(err, "")
    }

    pub fn from_with_msg(err: Error, msg: &str) -> NatureError {
        match err {
            Error::DatabaseError(kind, info) => match kind {
                DatabaseErrorKind::UniqueViolation => NatureError::DaoDuplicated(msg.to_string()),
                DatabaseErrorKind::__Unknown => NatureError::EnvironmentError(format!("{:?}", info)),
                _ => NatureError::SystemError(format!("{:?}", info)),
            }
            _ => NatureError::SystemError(err.to_string()),
        }
    }
}
