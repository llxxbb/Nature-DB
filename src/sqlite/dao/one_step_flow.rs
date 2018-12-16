use diesel::prelude::*;

use converter_cfg::OneStepFlow;

use super::*;

pub struct OneStepFlowDaoImpl;

impl OneStepFlowDaoTrait for OneStepFlowDaoImpl {
    fn get_relations(&self, from: &Thing) -> Result<Option<Vec<OneStepFlow>>> {
        use self::schema::one_step_flow::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let def = match one_step_flow
            .filter(from_thing.eq(&from.key))
            .filter(from_version.eq(from.version))
            .load::<RawOneStepFlow>(conn) {
            Ok(rows) => rows,
            Err(e) => return Err(DbError::from(e))
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
            _ => Err(NatureError::SystemError("unknown error occurred".to_string())),
        }
    }
}

impl OneStepFlowDaoImpl {
    pub fn insert(one: RawOneStepFlow) -> Result<usize> {
        use self::schema::one_step_flow;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(one_step_flow::table)
            .values(one)
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }
    pub fn delete(one: RawOneStepFlow) -> Result<usize> {
        use self::schema::one_step_flow::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::delete(one_step_flow
            .filter(from_thing.eq(one.from_thing))
            .filter(from_version.eq(one.from_version))
            .filter(to_thing.eq(one.to_thing))
            .filter(to_version.eq(one.to_version))
        ).execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }
}

#[cfg(test)]
mod test {
    extern crate log;

    use std::env;

    use nature_common::util::setup_logger;

    use super::*;

    #[test]
    fn one_step_test_for_http() {
        let one = before_test("from_good", "http");
        let thing = Thing {
            key: "from_good".to_string(),
            version: 123,
            thing_type: ThingType::Business,
        };
        let svc = OneStepFlowDaoImpl {};
        let rtn = svc.get_relations(&thing);
        assert_eq!(rtn.unwrap().unwrap().len(), 1);
        let _ = OneStepFlowDaoImpl::delete(one);
    }

    #[test]
    fn one_step_test_for_error_protocol() {
        let one = before_test("from_bad", "bad");
        let thing = Thing {
            key: "from_bad".to_string(),
            version: 123,
            thing_type: ThingType::Business,
        };
        let svc = OneStepFlowDaoImpl {};
        let rtn = svc.get_relations(&thing);
        assert_eq!(rtn.unwrap().is_none(), true);
        let _ = OneStepFlowDaoImpl::delete(one);
    }

    fn before_test(biz: &str, protocal: &str) -> RawOneStepFlow {
        env::set_var("DATABASE_URL", "nature.sqlite");
        let _ = setup_logger();
        let one = RawOneStepFlow {
            from_thing: biz.to_string(),
            from_version: 123,
            to_thing: "to".to_string(),
            to_version: 0,
            exe_protocol: protocal.to_string(),
            exe_url: "url".to_string(),
            selector: None,
            group: None,
            weight: None,
        };
// clear before insert
        let _ = OneStepFlowDaoImpl::delete(one.clone());
        let _ = OneStepFlowDaoImpl::insert(one.clone());
        one
    }
}