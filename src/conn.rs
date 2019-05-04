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

#[cfg(feature = "mysql")]
pub static CONN_STR : &str = "mysql://root@localhost/nature";
#[cfg(feature = "sqlite")]
pub static CONN_STR : &str = "nature.sqlite";