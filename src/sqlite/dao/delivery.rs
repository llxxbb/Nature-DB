use *;
use diesel::result::*;
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

    fn delete(&self, carrier_id: &Vec<u8>) -> Result<usize> {
        use self::schema::delivery::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::delete(delivery.filter(id.eq(carrier_id))).execute(conn);
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
        use self::schema::delivery::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::update(delivery.filter(id.eq(record_id)))
            .set(retried_times.eq(retried_times + 1)).
            execute(conn);
        match rtn {
            Ok(_) => Ok(()),
            Err(err) => Err(DbError::from(err)),
        }
    }

    fn get(&self, _id: &Vec<u8>) -> Result<RawDelivery> {
        unimplemented!()
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
    fn test_get_overdue() {
        env::set_var("DATABASE_URL", "nature.sqlite");
        let dao = DeliveryDaoImpl {};
        let id = vec![1, 2, 3, 4, 5, ];

        // delete it after being used
        let _ = dao.delete(&id);

        // no data
        let x = dao.get_overdue("0");
        assert_eq!(x.unwrap().len(), 0);

        // insert one and test
        let time = Local::now();
        let time2 = Local::now() - Duration::seconds(10);
        let raw = RawDelivery {
            id,
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

        // verify overdue
        let x = dao.get_overdue("5");
        println!("{:?}", x);
        assert_eq!(x.unwrap().len(), 1);

        // delete it after being used
        let _ = dao.delete(&raw.id);
    }
}