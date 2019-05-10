use diesel::prelude::*;
use diesel::result::*;

use nature_common::util::id_tool::vec_to_hex_string;

use crate::{CONN, CONNNECTION};
use crate::raw_models::{RawTask, RawTaskError};

use super::*;

use self::schema::task::dsl::*;

pub struct TaskDaoImpl;

impl TaskDaoImpl {
    pub fn insert(raw: &RawTask) -> Result<usize> {
        use self::schema::task;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(task::table).values(raw).execute(conn);
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
                error!("insert task error: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    pub fn delete(record_id: &[u8]) -> Result<usize> {
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(task.filter(task_id.eq(record_id))).execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    pub fn raw_to_error(err: &NatureError, raw: &RawTask) -> Result<usize> {
        use self::schema::task_error;
        let rtn = {
            let conn: &CONNNECTION = &CONN.lock().unwrap();
            let rd = RawTaskError::from_raw(err, raw);
            diesel::insert_into(task_error::table).values(rd).execute(conn)
        };
        match rtn {
            Ok(num) => Ok(num),
            Err(Error::DatabaseError(kind, info)) => match kind {
                DatabaseErrorKind::UniqueViolation => {
                    debug!("delete task already in task_error table");
                    let _ = Self::delete(&raw.task_id);
                    debug!("delete succeed!");
                    Ok(1)
                }
                DatabaseErrorKind::__Unknown => Err(NatureError::DaoEnvironmentError(format!("{:?}", info))),
                _ => Err(NatureError::DaoLogicalError(format!("{:?}", info))),
            },
            Err(e) => {
                error!("insert task_error to db occurred error");
                Err(DbError::from(e))
            }
        }
    }

    pub fn get_overdue(seconds: &str) -> Result<Vec<RawTask>> {
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::sql_query(format!("select * from task where execute_time < datetime('now','localtime','-{} seconds') limit 100", seconds))
            .load::<RawTask>(conn);
        match rtn {
            Ok(rtn) => Ok(rtn),
            Err(e) => Err(DbError::from(e))
        }
    }


    pub fn update_execute_time(_id: &[u8], _delay: i64) -> Result<()> {
        // TODO
        unimplemented!()
    }

    /// increase one times and delay `delay` seconds
    pub fn increase_times_and_delay(record_id: &[u8], delay: i32) -> Result<usize> {
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let sql = format!("update task set retried_times = retried_times + 1, execute_time = datetime('now', '+{} seconds') where task_id = x'{}'", delay, vec_to_hex_string(&record_id));
        println!("{}", &sql);
        match diesel::sql_query(sql)
            .execute(conn) {
            Err(e) => Err(DbError::from(e)),
            Ok(num) => Ok(num)
        }
    }

    pub fn get(record_id: &[u8]) -> Result<Option<RawTask>> {
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = task.filter(task_id.eq(record_id))
            .limit(1)
            .load::<RawTask>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].clone())),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

}

#[cfg(test)]
mod test {
    // TODO
//    use std::env;
//
//    use chrono::Duration;
//    use chrono::prelude::*;
//
//    use crate::CONN_STR;
//
//    use super::*;
//
//    #[test]
//    fn task_dao_test() {
//        env::set_var("DATABASE_URL", CONN_STR);
//        let dao = TaskDaoImpl {};
//        let record_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
//
//        // delete it after being used
//        let _ = dao.delete(&record_id);
//
//        // no data
//        let x = dao.get_overdue("0");
//        assert_eq!(x.unwrap().len(), 0);
//
//        // insert one and test
//        let time = Local::now();
//        let time2 = Local::now() - Duration::seconds(10);
//        let raw = RawTask {
//            task_id: record_id.clone(),
//            thing: "lxb".to_string(),
//            data_type: 1,
//            data: "hello".to_string(),
//            create_time: time.naive_local(),
//            execute_time: time2.naive_local(),
//            retried_times: 1,
//        };
//        println!(" insert for {:?}", &raw);
//        let rtn = dao.insert(&raw);
//        assert_eq!(rtn, Ok(1));
//
//        // verify overdue
//        let x = dao.get_overdue("5");
//        let vec = x.unwrap();
//        assert_eq!(vec.len(), 1);
//        println!(" the overdue date: {:?}", vec[0]);
//
//        // increase retry times and execute_time
//        let rtn = dao.increase_times_and_delay(&record_id, 20);
//        assert_eq!(rtn.unwrap(), 1);
//
//        // very get function
//        let rtn = dao.get(&record_id);
//        let rtn = rtn.unwrap().unwrap();
//        assert_eq!(rtn.retried_times, 2);
//        let diff = rtn.execute_time.timestamp() - Local::now().timestamp();
//        assert_eq!(diff <= 20, true);
//
//        // delete it after being used
//        let _ = dao.delete(&record_id);
//    }
}