#[cfg(feature = "mysql")]
pub use self::mysql::*;
#[cfg(feature = "sqlite")]
pub use self::sqlite::*;

#[cfg(feature = "mysql")]
mod mysql {
    use diesel::mysql::MysqlConnection;

    pub use crate::mysql::CONN;

    pub type CONNNECTION = MysqlConnection;
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use diesel::sqlite::SqliteConnection;

    pub use crate::sqlite::CONN;

    pub type CONNNECTION = SqliteConnection;
}
