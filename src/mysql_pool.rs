// pub use self::conn::*;
//
// mod conn {
//     use std::env;
//
//     use diesel::mysql::MysqlConnection;
//     use r2d2::Pool;
//     use r2d2_diesel::ConnectionManager;
//
//     use nature_common::Result;
//
//     lazy_static! {
//        pub static ref POOL : Pool<ConnectionManager<MysqlConnection>> = make_db_connection_pool();
//     }
//
//     fn make_db_connection_pool() -> Pool<ConnectionManager<MysqlConnection>> {
//         let database_url = env::var("DATABASE_URL")
//             .expect("DATABASE_URL must be set");
//         let manager = ConnectionManager::<MysqlConnection>::new(database_url);
//         r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
//     }
//
//     pub fn get_conn() -> Result<MysqlConnection> {
//         let rtn = POOL.clone();
//         let rtn = rtn.get()?;
//         Ok(rtn.into())
//     }
// }
