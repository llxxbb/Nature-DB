use diesel::prelude::*;

use nature_common::*;

use crate::{CONN, CONNNECTION, DbError};
use crate::raw_models::RawInstance;

pub struct InstanceDaoImpl;

impl InstanceDaoImpl {
    pub fn insert(instance: &Instance) -> Result<usize> {
        use super::schema::instances;
        let new = RawInstance::new(instance)?;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        match diesel::insert_into(instances::table)
            .values(new)
            .execute(conn) {
            Ok(rtn) => {
                debug!("Saved instance for `Meta` {:?}, id : {:?}", instance.meta.get_full_key(), instance.id);
                Ok(rtn)
            }
            Err(err) => Err(DbError::from_with_msg(err, &instance.id.to_string()))
        }
    }

    /// check whether source stored earlier
    pub fn is_exists(ins: &Instance) -> Result<bool> {
        use super::schema::instances::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = instances
            .filter(instance_id.eq(ins.id.to_ne_bytes().to_vec()))
            .filter(meta.eq(ins.meta.get_string()))
            .filter(status_version.eq(ins.status_version))
            .order(status_version.desc())
            .limit(1)
            .load::<RawInstance>(conn);
        match def {
            Ok(rs) => match rs.len() {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string()))
            },
            Err(e) => Err(DbError::from(e))
        }
    }
    pub fn get_by_id(record_id: u128) -> Result<Option<Instance>> {
        use super::schema::instances::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = instances
            .filter(instance_id.eq(u128_to_vec_u8(record_id)))
            .order(status_version.desc())
            .limit(1)
            .load::<RawInstance>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].to()?)),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn delete(ins: &Instance) -> Result<usize> {
        debug!("delete instance, id is : {:?}", ins.id);
        use super::schema::instances::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rows = instances
            .filter(instance_id.eq(ins.id.to_ne_bytes().to_vec()))
            .filter(meta.eq(ins.meta.get_string()))
            .filter(status_version.eq(ins.status_version));
        //        debug!("rows filter : {:?}", rows);
        match diesel::delete(rows).execute(conn) {
            Ok(rtn) => Ok(rtn),
            Err(e) => Err(DbError::from(e))
        }
    }
}


#[cfg(test)]
mod test {
    // TODO
//    use std::collections::HashMap;
//    use std::collections::HashSet;
//    use std::env;
//
//    use crate::CONN_STR;
//    use crate::dao::instance::InstanceDaoImpl;
//
//    use super::*;
//
//    #[test]
//    fn instance_insert_exists_delete_test() {
//        let tester = InstanceDaoImpl {};
//        env::set_var("DATABASE_URL", CONN_STR);
//        // prepare data to insert
//        let instance = Instance {
//            id: 0,
//            data: InstanceNoID {
//                meta: Meta::new_with_version_and_type("/instance/common", 100, MetaType::Business).unwrap(),
//                event_time: 0,
//                execute_time: 0,
//                create_time: 0,
//                content: String::new(),
//                context: HashMap::new(),
//                status: HashSet::new(),
//                status_version: 123,
//                from: None,
//            },
//        };
//        // delete if it exists
//        if let Ok(true) = tester.is_exists(&instance) {
//            let _ = InstanceDaoImpl::delete(&instance);
//        }
//        // insert one
//        assert_eq!(Ok(1), tester.insert(&instance));
//        // insert twice
//        assert_eq!(tester.insert(&instance), Err(NatureError::DaoDuplicated("".to_string())));
//        // exists
//        assert_eq!(true, tester.is_exists(&instance).unwrap());
//        // delete it
//        assert_eq!(1, InstanceDaoImpl::delete(&instance).unwrap());
//    }
//
//    #[test]
//    fn get_last_status() {
//        let tester = InstanceDaoImpl {};
//        env::set_var("DATABASE_URL", CONN_STR);
//        // prepare data to insert
//        let mut instance = Instance {
//            id: 0,
//            data: InstanceNoID {
//                meta: Meta::new_with_version_and_type("/instance/getLast", 100, MetaType::Business).unwrap(),
//                event_time: 0,
//                execute_time: 0,
//                create_time: 0,
//                content: String::new(),
//                context: HashMap::new(),
//                status: HashSet::new(),
//                status_version: 123,
//                from: None,
//            },
//        };
//        // delete old if exists
//        if let Ok(true) = tester.is_exists(&instance) {
//            let _ = InstanceDaoImpl::delete(&instance);
//        }
//        instance.data.status_version = 111;
//        if let Ok(true) = tester.is_exists(&instance) {
//            let _ = InstanceDaoImpl::delete(&instance);
//        }
//        // insert one
//        instance.data.status_version = 123;
//        assert_eq!(Ok(1), tester.insert(&instance));
//        // insert two
//        instance.data.status_version = 111;
//        assert_eq!(Ok(1), tester.insert(&instance));
//        // get last
//        if let Ok(Some(x)) = tester.get_by_id(instance.id) {
//            assert_eq!(123, x.status_version);
//        } else {
//            panic!("shouldn't get error");
//        }
//        // delete after test
//        let _ = InstanceDaoImpl::delete(&instance);
//        instance.data.status_version = 123;
//        let _ = InstanceDaoImpl::delete(&instance);
//    }
}