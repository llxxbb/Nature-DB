pub use self::conn::*;

mod conn {
    use std::env;
    use std::sync::Mutex;

    use diesel::Connection;
    use diesel::mysql::MysqlConnection;

    lazy_static! {
        pub static ref CONN :Mutex<MysqlConnection>  = Mutex::new(establish_connection());
    }

    fn establish_connection() -> MysqlConnection {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        info!("connect to db : {}", &database_url);
        MysqlConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connect to db: {}", database_url))
    }

//lazy_static! {
//    pub static ref POOL : Pool<ConnectionManager<SqliteConnection>> = make_db_connection_pool();
//}

//impl From<Error> for NatureError {
//    fn from(err: Error) -> Self {
//        NatureError::R2D2Error(err.to_string())
//    }
//}

//fn make_db_connection_pool() -> Pool<ConnectionManager<SqliteConnection>> {
//    dotenv().ok();
//
//    let database_url = env::var("DATABASE_URL")
//        .expect("DATABASE_URL must be set");
//
//    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
//    Pool::builder().build(manager).expect("Failed to create pool.")
//}
//
//pub struct DBPool;
//
//impl DBPool {

// /// 使用说明：
// ///
// /// ```rust
// /// use std::ops::Deref;
// /// let conn = DBPool::get_connection()?;
// /// conn.deref()
// /// ```
//    pub fn get_connection() -> Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
//        match POOL.clone().get() {
//            Err(err) => Err(NatureError::from(err)),
//            Ok(conn) => Ok(conn),
//        }
//    }
//}
}
