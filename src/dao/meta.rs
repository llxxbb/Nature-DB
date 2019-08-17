use diesel::prelude::*;

use crate::{CONN, CONNNECTION};
use crate::raw_models::RawMeta;

use super::*;

pub struct MetaDaoImpl;

impl MetaDaoImpl {
    pub fn get(meta_def: &Meta) -> Result<Option<RawMeta>> {
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = meta.filter(full_key.eq(&meta_def.get_full_key()))
            .filter(version.eq(meta_def.version))
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
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(meta.filter(full_key.eq(&meta_def.get_full_key())).filter(version.eq(meta_def.version)))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(err) => Err(DbError::from(err))
        }
    }
}

impl MetaDaoImpl {
    pub fn new_by_key(key: &str) -> Result<usize> {
        let meta = Meta::new(key)?;
        let mut define = RawMeta::default();
        define.version = meta.version;
        define.full_key = meta.get_full_key();
        MetaDaoImpl::insert(&define)
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use chrono::prelude::*;

    use crate::*;
    use crate::dao::MetaDaoImpl;

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
        let meta = Meta::new_with_version_and_type("/test", 100, MetaType::Business).unwrap();

        // delete if it exists
        if let Ok(Some(_)) = MetaDaoImpl::get(&meta) {
            let _ = MetaDaoImpl::delete(&meta);
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
        MetaDaoImpl::delete(&meta).unwrap();
    }
}