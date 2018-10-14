use *;
use diesel::result::*;
use self::schema::delivery::dsl::*;
use super::*;

pub struct DeliveryDaoImpl;

impl DeliveryDaoTrait for DeliveryDaoImpl {
    fn insert(&self, raw: &RawDelivery) -> Result<usize> {
        use self::schema::delivery;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(delivery::table).values(raw).execute(conn);
        match rtn {
            Ok(num) => {
                Ok(num)
            }
            Err(Error::DatabaseError(kind, info)) => match kind {
                DatabaseErrorKind::UniqueViolation => Ok(0),
                DatabaseErrorKind::__Unknown => {
                    Err(NatureError::DaoEnvironmentError(format!("{:?}", info)))
                }
                _ => Err(NatureError::DaoLogicalError(format!("{:?}", info))),
            },
            Err(e) => {
                error!("insert delivery error: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    fn delete(&self, record_id: &Vec<u8>) -> Result<usize> {
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::delete(delivery.filter(id.eq(record_id))).execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    fn raw_to_error(&self, err: &NatureError, raw: &RawDelivery) -> Result<usize> {
        use self::schema::delivery_error;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rd = RawDeliveryError::from_raw(err, raw);
        let rtn = diesel::insert_into(delivery_error::table).values(rd).execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(Error::DatabaseError(kind, info)) => match kind {
                DatabaseErrorKind::UniqueViolation => Ok(1),
                DatabaseErrorKind::__Unknown => Err(NatureError::DaoEnvironmentError(format!("{:?}", info))),
                _ => Err(NatureError::DaoLogicalError(format!("{:?}", info))),
            },
            Err(e) => {
                error!("insert delivery_error to db occurred error");
                Err(DbError::from(e))
            }
        }
    }

    fn update_execute_time(&self, _id: &Vec<u8>, _delay: i64) -> Result<()> {
        unimplemented!()
    }

    fn increase_times(&self, record_id: Vec<u8>) -> Result<()> {
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::update(delivery.filter(id.eq(record_id)))
            .set(retried_times.eq(retried_times + 1)).
            execute(conn);
        match rtn {
            Ok(_) => Ok(()),
            Err(err) => Err(DbError::from(err)),
        }
    }

    fn get(&self, record_id: &Vec<u8>) -> Result<Option<RawDelivery>> {
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let def = delivery.filter(id.eq(record_id))
            .limit(1)
            .load::<RawDelivery>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].clone())),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    fn get_overdue(&self, seconds: &str) -> Result<Vec<RawDelivery>> {
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::sql_query(format!("select * from delivery where execute_time < datetime('now','localtime','-{} seconds') limit 100", seconds))
            .load::<RawDelivery>(conn);
        match rtn {
            Ok(rtn) => Ok(rtn),
            Err(e) => Err(DbError::from(e))
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::Duration;
    use chrono::prelude::*;
    use std::env;
    use super::*;

    #[test]
    fn test_delete_insert_get_and_overdue() {
        env::set_var("DATABASE_URL", "nature.sqlite");
        let dao = DeliveryDaoImpl {};
        let record_id = vec![1, 2, 3, 4, 5, ];

        // delete it after being used
        let _ = dao.delete(&record_id);

        // no data
        let x = dao.get_overdue("0");
        assert_eq!(x.unwrap().len(), 0);

        // insert one and test
        let time = Local::now();
        let time2 = Local::now() - Duration::seconds(10);
        let raw = RawDelivery {
            id: record_id.clone(),
            thing: "lxb".to_string(),
            data_type: 1,
            data: "hello".to_string(),
            create_time: time.naive_local(),
            execute_time: time2.naive_local(),
            retried_times: 1,
        };
        println!(" insert for {:?}", &raw);
        let rtn = dao.insert(&raw);
        assert_eq!(rtn, Ok(1));

        // very get function
        let rtn = dao.get(&record_id);
        assert_eq!(rtn, Ok(Some(raw)));
        println!(" get {:?}", rtn.unwrap().unwrap());

        // verify overdue
        let x = dao.get_overdue("5");
        let vec = x.unwrap();
        assert_eq!(vec.len(), 1);
        println!(" the overdue date: {:?}", vec[0]);

        // delete it after being used
        let _ = dao.delete(&record_id);
    }
}