pub use self::conn::*;

mod conn {
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

        info!("connect to db : {}", &database_url);
        SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connect to db: {}", database_url))
    }
}
