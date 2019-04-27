pub use self::conn::*;
pub use self::models::*;

mod conn{
    use std::env;
    use std::sync::Mutex;

    use diesel::Connection;
    use diesel::sqlite::SqliteConnection;

    lazy_static! {
    pub static ref CONN :Mutex<SqliteConnection>  = Mutex::new(establish_connection());
}

    fn establish_connection() -> SqliteConnection {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connectinstance_key_undefineding to {}", database_url))
    }
}