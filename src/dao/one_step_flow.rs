use std::str::FromStr;

use diesel::prelude::*;

use crate::{CONN, CONNNECTION};
use crate::models::converter_cfg::{OneStepFlow, OneStepFlowSettings};
use crate::raw_models::RawOneStepFlow;

use super::*;

pub struct OneStepFlowDaoImpl;

impl OneStepFlowDaoImpl {
    pub fn get_relations(from: &Thing) -> Result<Option<Vec<OneStepFlow>>> {
        use self::schema::one_step_flow::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = match one_step_flow
            .filter(from_thing.eq(&from.get_full_key()))
            .filter(from_version.eq(from.version))
            .load::<RawOneStepFlow>(conn)
            {
                Ok(rows) => rows,
                Err(e) => return Err(DbError::from(e)),
            };
        match def.len() {
            0 => Ok(None),
            x if x > 0 => {
                let mut rtn: Vec<OneStepFlow> = Vec::new();
                for d in def {
                    match OneStepFlow::from_raw(d) {
                        Ok(multi) => multi.into_iter().for_each(|e| rtn.push(e)),
                        Err(e) => {
                            warn!("raw to `one_step_flow` occur error : {:?}", e);
                        }
                    }
                }
                if rtn.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(rtn))
                }
            }
            _ => Err(NatureError::SystemError(
                "unknown error occurred".to_string(),
            )),
        }
    }
    pub fn insert(one: RawOneStepFlow) -> Result<usize> {
        use self::schema::one_step_flow;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(one_step_flow::table)
            .values(one)
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e)),
        }
    }
    pub fn delete(one: RawOneStepFlow) -> Result<usize> {
        use self::schema::one_step_flow::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(
            one_step_flow
                .filter(from_thing.eq(one.from_thing))
                .filter(from_version.eq(one.from_version))
                .filter(to_thing.eq(one.to_thing))
                .filter(to_version.eq(one.to_version)),
        )
            .execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    /// `version` will be set to 0
    pub fn insert_by_biz(
        from: &str,
        to: &str,
        url: &str,
        protocol: &str,
    ) -> Result<RawOneStepFlow> {
        let one = RawOneStepFlow::new(
            &Thing::new(from)?,
            &Thing::new(to)?,
            &OneStepFlowSettings {
                selector: None,
                executor: vec![Executor {
                    protocol: Protocol::from_str(protocol)?,
                    url: url.to_string(),
                    group: "".to_string(),
                    proportion: 1,
                }],
            },
        )?;
        let _ = OneStepFlowDaoImpl::insert(one.clone());
        Ok(one)
    }

    pub fn delete_by_biz(from: &str, to: &str) -> Result<usize> {
        let from = &Thing::new(from)?;
        let to = &Thing::new(to)?;
        let row = RawOneStepFlow {
            from_thing: from.get_full_key(),
            from_version: from.version,
            to_thing: to.get_full_key(),
            to_version: to.version,
            settings: String::new(),
        };
        OneStepFlowDaoImpl::delete(row)
    }
}

#[cfg(test)]
mod test {
    // TODO
//    extern crate log;
//
//    use std::env;
//
//    use nature_common::*;
//
//    use crate::CONN_STR;
//
//    use super::*;
//
//    #[test]
//    fn one_step_test_for_http() {
//        let one = before_test("from_good", "http");
//        let thing = Thing::new("from_good").unwrap();
//        let svc = OneStepFlowDaoImpl {};
//        let rtn = svc.get_relations(&thing);
//        assert_eq!(rtn.unwrap().unwrap().len(), 1);
//        let _ = OneStepFlowDaoImpl::delete(one.unwrap());
//    }
//
//    #[test]
//    fn one_step_test_for_error_protocol() {
//        let rtn = before_test("from_bad", "bad");
//        assert_eq!(
//            rtn.err().unwrap(),
//            NatureError::VerifyError("unknown protocol : bad".to_string())
//        );
//    }
//
//    fn before_test(biz: &str, protocol: &str) -> Result<RawOneStepFlow> {
//        env::set_var("DATABASE_URL", CONN_STR);
//        let _ = setup_logger();
//        // clear before insert
//        let _ = OneStepFlowDaoImpl::delete_by_biz(biz, "to");
//        // insert
//        OneStepFlowDaoImpl::insert_by_biz(biz, "to", "url", protocol)
//    }
}
