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

    fn get_overdue(&self) -> Result<Option<Vec<RawDelivery>>> {
        unimplemented!()
    }
}
