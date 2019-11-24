use diesel::prelude::*;
use diesel::result::*;

use nature_common::{Instance, NatureError, Result, vec_to_hex_string, vec_to_u128};

use crate::{CONN, CONNNECTION, DbError};
use crate::dao::schema::task::dsl::*;
use crate::raw_models::{RawTask, RawTaskError};

pub struct TaskDaoImpl;

impl TaskDaoImpl {
    pub fn insert(raw: &RawTask) -> Result<usize> {
        use crate::dao::schema::task;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(task::table).values(raw).execute(conn);
        match rtn {
            Ok(num) => {
                Ok(num)
            }
            Err(Error::DatabaseError(kind, info)) => match kind {
                DatabaseErrorKind::UniqueViolation => Ok(0),
                DatabaseErrorKind::__Unknown => {
                    Err(NatureError::EnvironmentError(format!("{:?}", info)))
                }
                _ => Err(NatureError::SystemError(format!("{:?}", info))),
            },
            Err(e) => {
                error!("insert task error: {:?}", e);
                Err(DbError::from_with_msg(e, &format!("mata : {}, id : {}", &raw.meta, vec_to_u128(&raw.task_id))))
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
        use crate::dao::schema::task_error;
        let rtn = {
            let conn: &CONNNECTION = &CONN.lock().unwrap();
            let rd = RawTaskError::from_raw(err, raw);
            diesel::insert_into(task_error::table).values(rd).execute(conn)
        };
        let rtn = match rtn {
            Ok(num) => {
                let _ = Self::delete(&raw.task_id)?;
                Ok(num)
            }
            Err(Error::DatabaseError(kind, info)) => match kind {
                DatabaseErrorKind::UniqueViolation => {
                    debug!("delete task already in task_error table");
                    let _ = Self::delete(&raw.task_id);
                    debug!("delete succeed!");
                    Ok(1)
                }
                DatabaseErrorKind::__Unknown => Err(NatureError::EnvironmentError(format!("{:?}", info))),
                _ => Err(NatureError::SystemError(format!("{:?}", info))),
            },
            Err(e) => {
                error!("insert task_error to db occurred error");
                Err(DbError::from(e))
            }
        };
        rtn
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


    pub fn update_execute_time(record_id: &[u8], delay: i64, last_state: &Option<Instance>) -> Result<()> {
        let version = match last_state {
            Some(ins) => ins.state_version,
            None => 0
        };
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let sql = format!("update task set execute_time = datetime('now', '+{} seconds', 'localtime'), last_state_version = {} where task_id = x'{}'", delay, version, vec_to_hex_string(&record_id));
        match diesel::sql_query(sql)
            .execute(conn) {
            Err(e) => {
                warn!("Update delay error: {}", &e);
                Err(DbError::from(e))
            }
            Ok(_) => {
                Ok(())
            }
        }
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
