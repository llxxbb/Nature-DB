use *;
use diesel::result::*;
use nature_common::util::*;
use serde::Serialize;
use std::fmt::Debug;
use super::*;

pub struct DeliveryDaoImpl;

impl DeliveryDaoTrait for DeliveryDaoImpl {
    fn insert<T: Sized + Serialize + Send + Debug>(carrier: &Carrier<T>) -> Result<u128> {
        use self::schema::delivery;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let d = RawDelivery::new(carrier)?;
        let id = d.id.clone();
        let rtn = diesel::insert_into(delivery::table).values(d).execute(conn);
        match rtn {
            Ok(_) => {
                //                debug!("insert carrier to db for id: {:?} successful", carrier.id);
                Ok(vec_to_u128(&id))
            }
            Err(Error::DatabaseError(kind, info)) => match kind {
                DatabaseErrorKind::UniqueViolation => Ok(vec_to_u128(&id)),
                DatabaseErrorKind::__Unknown => {
                    Err(NatureError::DaoEnvironmentError(format!("{:?}", info)))
                }
                _ => Err(NatureError::DaoLogicalError(format!("{:?}", info))),
            },
            Err(e) => {
                error!(
                    "insert carrier to db for id: {:?} occurred error",
                    carrier.id
                );
                Err(DbError::from(e))
            }
        }
    }

    fn delete(carrier_id: u128) -> Result<usize> {
        use self::schema::delivery::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::delete(delivery.filter(id.eq(u128_to_vec_u8(carrier_id)))).execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    fn carrier_to_error<T: Serialize + Debug>(err: &NatureError, carrier: &Carrier<T>) -> Result<usize> {
        let raw = RawDelivery::new(carrier)?;
        Self::raw_to_error(err,&raw)
    }

    fn raw_to_error(err: &NatureError, raw: &RawDelivery) -> Result<usize> {
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

    fn update_execute_time(_id: u128, _new_time: i64) -> Result<()> {
        unimplemented!()
    }

    fn increase_times(record_id: Vec<u8>) -> Result<()> {
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

    fn get<T: Sized + Serialize + Debug>(_id: u128) -> Result<Carrier<T>> {
        unimplemented!()
    }

    fn get_overdue() -> Result<Option<Vec<RawDelivery>>> {
        unimplemented!()
    }
}
