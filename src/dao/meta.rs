use diesel::prelude::*;

use nature_common::{Meta, MetaString, NatureError, Result};

use crate::{CONN, CONNNECTION, DbError};
use crate::raw_models::RawMeta;
use crate::schema;

pub type MetaGetter = fn(&str) -> Result<Option<RawMeta>>;

pub struct MetaDaoImpl;

impl MetaDaoImpl {
    pub fn get(meta_str: &str) -> Result<Option<RawMeta>> {
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let (fk, ver) = MetaString::make_tuple_from_str(meta_str)?;
        let def = meta.filter(full_key.eq(&fk))
            .filter(version.eq(ver))
            .filter(flag.eq(1))
            .load::<RawMeta>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].clone())),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn insert(define: &RawMeta) -> Result<usize> {
        use self::schema::meta;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(meta::table)
            .values(define)
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn update_flag(full_key_f: &str, version_f: i32, flag_f: i32) -> Result<usize> {
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::update(
            meta.filter(full_key.eq(full_key_f))
                .filter(version.eq(version_f)))
            .set(flag.eq(flag_f))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn delete(meta_def: &Meta) -> Result<usize> {
        Self::delete_by_full_key(&meta_def.get_full_key(), meta_def.version)
    }

    pub fn delete_by_full_key(full_key_f: &str, version_f: i32) -> Result<usize> {
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(meta.filter(full_key.eq(full_key_f)).filter(version.eq(version_f)))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(err) => Err(DbError::from(err))
        }
    }
}

impl MetaDaoImpl {
    pub fn new_by_key(key: &str) -> Result<usize> {
        let mut define = RawMeta::default();
        define.version = 1;
        define.full_key = MetaString::full_key(key)?;
        MetaDaoImpl::insert(&define)
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use chrono::prelude::*;

    use crate::CONN_STR;

    use super::*;

    #[test]
    fn define_test() {
        // prepare data to insert
        env::set_var("DATABASE_URL", CONN_STR);
        let define = RawMeta {
            full_key: "/B/test".to_string(),
            description: Some("description".to_string()),
            version: 100,
            states: Some("status".to_string()),
            fields: Some("fields".to_string()),
            config: "{}".to_string(),
            flag: 1,
            create_time: Local::now().naive_local(),
        };
        let meta = "/B/test:100";

        // delete if it exists
        if let Ok(Some(_)) = MetaDaoImpl::get("/B/test:100") {
            let _ = MetaDaoImpl::delete_by_full_key("/B/test", 100);
        }

        // insert
        let rtn = MetaDaoImpl::insert(&define);
        assert_eq!(rtn.unwrap(), 1);
        // repeat insert
        let rtn = MetaDaoImpl::insert(&define);
        let _ = match rtn {
            Err(err) => match err {
                NatureError::DaoDuplicated(_) => (),
                _ => panic!("match error"),
            }
            _ => panic!("match error")
        };
        // find inserted
        let row = MetaDaoImpl::get(&meta).unwrap().unwrap();
        assert_eq!(row, define);

        // change flag
        let _ = MetaDaoImpl::update_flag("/B/test", 100, 0);
        let row = MetaDaoImpl::get(&meta).unwrap();
        assert_eq!(row, None);

        // delete it
        let _ = MetaDaoImpl::delete_by_full_key("/B/test", 100);
    }
}