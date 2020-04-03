use diesel::prelude::*;

use nature_common::{Meta, NatureError, Result};

use crate::{CONN, CONNNECTION, DbError};
use crate::raw_models::RawMeta;
use crate::schema;

pub type MetaGetter = fn(&str) -> Result<Option<RawMeta>>;

pub static MG: &MetaGetter = &(MetaDaoImpl::get as MetaGetter);

pub struct MetaDaoImpl;

impl MetaDaoImpl {
    pub fn get(meta_str: &str) -> Result<Option<RawMeta>> {
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let m = Meta::from_string(meta_str)?;
        let def = meta.filter(meta_type.eq(&m.get_meta_type().get_prefix()))
            .filter(meta_key.eq(m.get_key()))
            .filter(version.eq(m.version as i32))
            .filter(flag.eq(1))
            .load::<RawMeta>(conn);
        match def {
            Ok(rtn) => {
                debug!("load meta : {:?}", &rtn);
                match rtn.len() {
                    0 => Ok(None),
                    1 => Ok(Some(rtn[0].clone())),
                    _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
                }
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

    pub fn update_flag(meta_str: &str, flag_f: i32) -> Result<usize> {
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let m = Meta::from_string(meta_str)?;
        let rtn = diesel::update(
            meta.filter(meta_type.eq(m.get_meta_type().get_prefix()))
                .filter(meta_key.eq(m.get_key()))
                .filter(version.eq(m.version as i32)))
            .set(flag.eq(flag_f))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn delete(m: &Meta) -> Result<usize> {
        use self::schema::meta::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(
            meta.filter(meta_type.eq(m.get_meta_type().get_prefix()))
                .filter(meta_key.eq(m.get_key()))
                .filter(version.eq(m.version as i32)))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(err) => Err(DbError::from(err))
        }
    }
}

// #[cfg(test)]
// mod test {
//     use std::env;
//
//     use chrono::prelude::*;
//
//     use crate::CONN_STR;
//
//     use super::*;
//
//     #[test]
//     fn define_test() {
//         // prepare data to insert
//         env::set_var("DATABASE_URL", CONN_STR);
//         let define = RawMeta {
//             meta_type: "B".to_string(),
//             description: Some("description".to_string()),
//             version: 100,
//             states: Some("status".to_string()),
//             fields: Some("fields".to_string()),
//             config: "{}".to_string(),
//             flag: 1,
//             create_time: Local::now().naive_local(),
//             meta_key: "test".to_string(),
//         };
//         let meta = "B:test:100";
//         let m = Meta::from_string(meta).unwrap();
//         // delete if it exists
//         if let Ok(Some(_)) = MetaDaoImpl::get("B:test:100") {
//             let _ = MetaDaoImpl::delete(&m);
//         }
//
//         // insert
//         let rtn = MetaDaoImpl::insert(&define);
//         assert_eq!(rtn.unwrap(), 1);
//         // repeat insert
//         let rtn = MetaDaoImpl::insert(&define);
//         let _ = match rtn {
//             Err(err) => match err {
//                 NatureError::DaoDuplicated(_) => (),
//                 _ => panic!("match error"),
//             }
//             _ => panic!("match error")
//         };
//         // find inserted
//         let row = MetaDaoImpl::get(&meta).unwrap().unwrap();
//         assert_eq!(row, define);
//
//         // change flag
//         let _ = MetaDaoImpl::update_flag("B:test:100", 0);
//         let row = MetaDaoImpl::get(&meta).unwrap();
//         assert_eq!(row, None);
//
//         // delete it
//         let _ = MetaDaoImpl::delete(&m);
//     }
// }