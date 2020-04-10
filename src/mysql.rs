pub use self::conn::*;

mod conn {
    use std::env;

    use diesel::mysql::MysqlConnection;
    use diesel::r2d2::ConnectionManager;
    use r2d2::{Pool, PooledConnection};

    use nature_common::Result;

    lazy_static! {
       static ref POOL : Pool<ConnectionManager<MysqlConnection>> = make_db_connection_pool();
    }

    fn make_db_connection_pool() -> Pool<ConnectionManager<MysqlConnection>> {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
    }

    pub fn get_conn() -> Result<PooledConnection<ConnectionManager<MysqlConnection>>> {
        let rtn = POOL.clone().get()?;
        Ok(rtn)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    #[test]
    fn conn_test() {
        env::set_var("DATABASE_URL", "mysql://root@localhost/nature");
        let rtn = get_conn();
        assert_eq!(rtn.is_ok(), true);
    }
}
