use std::str::FromStr;

use diesel::prelude::*;

use crate::{CONN, CONNNECTION};
use crate::models::converter_cfg::{OneStepFlow, OneStepFlowSettings};
use crate::raw_models::RawOneStepFlow;

use super::*;

pub struct OneStepFlowDaoImpl;

impl OneStepFlowDaoImpl {
    pub fn get_relations(from: &Meta) -> Result<Option<Vec<OneStepFlow>>> {
        use self::schema::one_step_flow::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = match one_step_flow
            .filter(from_meta.eq(&from.get_string()))
            .filter(flag.eq(1))
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
                .filter(from_meta.eq(one.from_meta))
                .filter(to_meta.eq(one.to_meta)),
        )
            .execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    /// `from` and `to`'s form are full_key:version
    pub fn update_flag(from: &str, to: &str, flag_f: i32) -> Result<usize> {
        use self::schema::one_step_flow::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::update(
            one_step_flow.filter(from_meta.eq(Meta::new(from).unwrap().get_string()))
                .filter(to_meta.eq(Meta::new(to).unwrap().get_string())))
            .set(flag.eq(flag_f))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
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
            &Meta::new(from)?,
            &Meta::new(to)?,
            &OneStepFlowSettings {
                selector: None,
                executor: vec![Executor {
                    protocol: Protocol::from_str(protocol)?,
                    url: url.to_string(),
                    group: "".to_string(),
                    proportion: 1,
                }],
                use_upstream_id: false,
            },
        )?;
        let _ = OneStepFlowDaoImpl::insert(one.clone());
        Ok(one)
    }

    pub fn delete_by_biz(from: &str, to: &str) -> Result<usize> {
        let from = &Meta::new(from)?;
        let to = &Meta::new(to)?;
        let row = RawOneStepFlow {
            from_meta: from.get_string(),
            to_meta: to.get_string(),
            settings: String::new(),
            flag: 1,
        };
        OneStepFlowDaoImpl::delete(row)
    }
}

#[cfg(test)]
mod test {
    // TODO
    extern crate log;

    use std::env;

    use crate::CONN_STR;

    use super::*;

    #[test]
    fn one_step_test() {
        env::set_var("DATABASE_URL", CONN_STR);
        let _ = setup_logger();

        // clear before test
        let _ = OneStepFlowDaoImpl::delete_by_biz("from", "to");

        // get null
        let meta = Meta::new("from").unwrap();
        let rtn = OneStepFlowDaoImpl::get_relations(&meta).unwrap();
        assert_eq!(rtn, None);

        // insert
        let _ = OneStepFlowDaoImpl::insert_by_biz("from", "to", "url", "http");
        let rtn = OneStepFlowDaoImpl::get_relations(&meta).unwrap();
        assert_eq!(rtn.unwrap().len(), 1);

        // update flag
        let _ = OneStepFlowDaoImpl::update_flag("from", "to", 0);
        let rtn = OneStepFlowDaoImpl::get_relations(&meta).unwrap();
        assert_eq!(rtn, None);

        // delete after test
        let _ = OneStepFlowDaoImpl::delete_by_biz("from", "to");
    }
}
