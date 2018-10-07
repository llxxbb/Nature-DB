use diesel::prelude::*;
use super::*;
use ThingDefine;

pub struct ThingDefineDaoImpl;

impl ThingDefineDaoTrait for ThingDefineDaoImpl {
    fn get(thing: &Thing) -> Result<Option<ThingDefine>> {
        use super::schema::thing_defines::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let def = thing_defines.filter(key.eq(&thing.key))
            .filter(version.eq(thing.version))
            .load::<ThingDefine>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].clone())),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    fn insert(define: &ThingDefine) -> Result<usize> {
        use self::schema::thing_defines;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(thing_defines::table)
            .values(RawThingDefine::new(define))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }

    fn delete(thing: &Thing) -> Result<usize> {
        use self::schema::thing_defines::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::delete(thing_defines.filter(key.eq(&thing.key)).filter(version.eq(thing.version)))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(err) => Err(DbError::from(err))
        }
    }
}

#[cfg(test)]
mod test{
    use ::*;
    use chrono::prelude::*;
    use std::env;


    #[test]
    fn define_test() {
        // prepare data to insert
        env::set_var("DATABASE_URL","nature.sqlite");
        let define = ThingDefine {
            key: "/test".to_string(),
            description: Some("description".to_string()),
            version: 100,
            states: Some("status".to_string()),
            fields: Some("fields".to_string()),
            create_time: Local::now().naive_local(),
        };
        let thing = Thing {
            key: "/test".to_string(),
            version: 100,
            thing_type: ThingType::Business,
        };
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