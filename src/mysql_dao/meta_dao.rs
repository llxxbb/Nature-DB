use diesel::prelude::*;
use tokio::macros::support::Future;

use nature_common::{Meta, NatureError, Result};

use crate::{DbError, get_conn, MySql};
use crate::raw_models::RawMeta;
use crate::schema;
use mysql_async::Value;

// pub type MetaGetter = fn(&str) -> dyn Future<Output=Result<Option<RawMeta>>>;
//
// pub static MG: &MetaGetter = &(MetaDaoImpl::get as MetaGetter);

pub struct MetaDaoImpl;

impl MetaDaoImpl {
    pub async fn get(meta_str: &str) -> Result<Option<RawMeta>> {
        let sql = r"SELECT meta_type, meta_key, description, version, states, fields, config, flag, create_time
            FROM nature.meta
            WHERE meta_type = :meta_type and meta_key = :meta_key and version = :version and flag = 1
            ";

        let m = Meta::from_string(meta_str)?;
        let p = params! {
            "meta_type" => m.get_meta_type().get_prefix(),
            "meta_key" => m.get_key(),
            "version" => m.version,
        };

        let rtn = MySql::fetch(sql, p, RawMeta::from).await?;
        match rtn.len() {
            1 => {
                let meta = rtn[0].clone();
                debug!("load meta : {:?}", &rtn);
                Ok(Some(meta))
            }
            0 => Ok(None),
            _ => Err(NatureError::LogicalError("should not return more than one rows".to_string()))
        }
    }

    pub async fn insert(define: &RawMeta) -> Result<usize> {
        let sql = r"INSERT INTO nature.meta
            (meta_type, meta_key, description, version, states, fields, config, flag, create_time)
            VALUES(:meta_type, :meta_key, :description, :version, :states, :fields, :config, :flag, :create_time)";
        let vec: Vec<(String, Value)> = define.into();
        let rtn: usize = MySql::insert_or_delete(sql, vec).await?;
        debug!("Saved meta : {}:{}:{}", define.meta_type, define.meta_key, define.version);
        Ok(rtn)
    }

    pub fn update_flag(meta_str: &str, flag_f: i32) -> Result<usize> {
        use self::schema::meta::dsl::*;
        let m = Meta::from_string(meta_str)?;
        let rtn = diesel::update(
            meta.filter(meta_type.eq(m.get_meta_type().get_prefix()))
                .filter(meta_key.eq(m.get_key()))
                .filter(version.eq(m.version as i32)))
            .set(flag.eq(flag_f))
            .execute(&get_conn()?);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn delete(m: &Meta) -> Result<usize> {
        use self::schema::meta::dsl::*;
        let rtn = diesel::delete(
            meta.filter(meta_type.eq(m.get_meta_type().get_prefix()))
                .filter(meta_key.eq(m.get_key()))
                .filter(version.eq(m.version as i32)))
            .execute(&get_conn()?);
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