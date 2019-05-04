use diesel::prelude::*;

use crate::{CONN, CONNNECTION};
use crate::models::define::ThingDefineDaoTrait;
use crate::raw_models::RawThingDefine;

use super::*;

pub struct ThingDefineDaoImpl;

impl ThingDefineDaoTrait for ThingDefineDaoImpl {
    fn get(thing: &Thing) -> Result<Option<RawThingDefine>> {
        use super::schema::thing_defines::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = thing_defines.filter(full_key.eq(&thing.get_full_key()))
            .filter(version.eq(thing.version))
            .load::<RawThingDefine>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].clone())),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    fn insert(define: &RawThingDefine) -> Result<usize> {
        use self::schema::thing_defines;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(thing_defines::table)
            .values(define)
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }

    fn delete(thing: &Thing) -> Result<usize> {
        use self::schema::thing_defines::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(thing_defines.filter(full_key.eq(&thing.get_full_key())).filter(version.eq(thing.version)))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(err) => Err(DbError::from(err))
        }
    }
}

impl ThingDefineDaoImpl {
    pub fn new_by_key(key: &str) -> Result<usize> {
        let thing = Thing::new(key)?;
        let mut define = RawThingDefine::default();
        define.version = thing.version;
        define.full_key = thing.get_full_key();
        ThingDefineDaoImpl::insert(&define)
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use chrono::prelude::*;

    use crate::*;
    use crate::dao::ThingDefineDaoImpl;
    use crate::models::define::ThingDefineDaoTrait;

    #[test]
    fn define_test() {
        // prepare data to insert
        env::set_var("DATABASE_URL", CONN_STR);
        let define = RawThingDefine {
            full_key: "/test".to_string(),
            description: Some("description".to_string()),
            version: 100,
            states: Some("status".to_string()),
            fields: Some("fields".to_string()),
            create_time: Local::now().naive_local(),
        };
        let thing = Thing::new_with_version_and_type("/test", 100, ThingType::Business).unwrap();
        // delete if it exists
        if let Ok(Some(_)) = ThingDefineDaoImpl::get(&thing) {
            let _ = ThingDefineDaoImpl::delete(&thing);
        }
        // insert
        let rtn = ThingDefineDaoImpl::insert(&define);
        assert_eq!(rtn.unwrap(), 1);
        // repeat insert
        let rtn = ThingDefineDaoImpl::insert(&define);
        let _ = match rtn {
            Err(err) => match err {
                NatureError::DaoDuplicated(_) => (),
                _ => panic!("match error"),
            }
            _ => panic!("match error")
        };
        // find inserted
        let row = ThingDefineDaoImpl::get(&thing).unwrap().unwrap();
        assert_eq!(row, define);
        // delete it
        ThingDefineDaoImpl::delete(&thing).unwrap();
    }
}