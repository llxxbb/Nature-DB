use diesel::prelude::*;
use mysql_async::Value;
use tokio::macros::support::Future;

use nature_common::{Meta, NatureError, Result};

use crate::{DbError, get_conn, MySql};
use crate::raw_models::RawMeta;
use crate::schema;

// pub type MetaGetter = fn(&str) -> dyn Future<Output=Result<Option<RawMeta>>>;
//
// pub static MG: &MetaGetter = &(MetaDaoImpl::get as MetaGetter);

pub struct MetaDaoImpl;

impl MetaDaoImpl {
    pub async fn get(meta_str: &str) -> Result<Option<RawMeta>> {
        let sql = r"SELECT meta_type, meta_key, description, version, states, fields, config, flag, create_time
            FROM meta
            WHERE meta_type = :meta_type and meta_key = :meta_key and version = :version and flag = 1";

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
        let sql = r"INSERT INTO meta
            (meta_type, meta_key, description, version, states, fields, config, flag, create_time)
            VALUES(:meta_type, :meta_key, :description, :version, :states, :fields, :config, :flag, :create_time)";
        let p: Vec<(String, Value)> = define.clone().into();
        let rtn: usize = MySql::idu(sql, p).await?;
        debug!("Saved meta : {}:{}:{}", define.meta_type, define.meta_key, define.version);
        Ok(rtn)
    }

    pub async fn update_flag(meta_str: &str, flag_f: i32) -> Result<usize> {
        let sql = r"UPDATE meta
            SET flag=:flag
            WHERE meta_type = :meta_type and meta_key = :meta_key and version = :version";

        let m = Meta::from_string(meta_str)?;
        let p = params! {
            "meta_type" => m.get_meta_type().get_prefix(),
            "meta_key" => m.get_key(),
            "version" => m.version,
            "flag" => flag_f,
        };
        let rtn = MySql::idu(sql, p).await?;
        debug!("meta flag updated: {}:{}:{}", m.get_meta_type().get_prefix(), m.get_key(), m.version);
        Ok(rtn)
    }

    pub async fn delete(m: &Meta) -> Result<usize> {
        let sql = r"DELETE FROM meta
            WHERE meta_type = :meta_type and meta_key = :meta_key and version = :version";

        let p = params! {
            "meta_type" => m.get_meta_type().get_prefix(),
            "meta_key" => m.get_key(),
            "version" => m.version,
        };

        let rtn: usize = MySql::idu(sql, p).await?;
        debug!("meta deleted: {}:{}:{}", m.get_meta_type().get_prefix(), m.get_key(), m.version);
        Ok(rtn)
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