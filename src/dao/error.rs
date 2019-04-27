use diesel::result::*;
use nature_common::*;

pub struct DbError;

impl DbError {
    // put it aside because can't find diesel's Timeout Error
    pub fn from(err: Error) -> NatureError {
        match err {
            Error::DatabaseError(kind, info) => match kind {
                DatabaseErrorKind::UniqueViolation => NatureError::DaoDuplicated("".to_string()),
                DatabaseErrorKind::__Unknown => NatureError::DaoEnvironmentError(format!("{:?}", info)),
                _ => NatureError::DaoLogicalError(format!("{:?}", info)),
            }
            _ => NatureError::DaoLogicalError(err.to_string()),
        }
    }
}
