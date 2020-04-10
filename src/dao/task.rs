use chrono::{Duration, Local};
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::result::*;

use nature_common::{NatureError, Result, vec_to_hex_string};

use crate::{DbError, get_conn};
use crate::raw_models::{RawTask, RawTaskError};
use crate::schema::task::dsl::*;

pub struct TaskDaoImpl;

impl TaskDaoImpl {
    pub fn insert(raw: &RawTask) -> Result<usize> {
        use crate::schema::task;
        let rtn = diesel::insert_into(task::table).values(raw).execute(&get_conn()?);
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
                Err(DbError::from_with_msg(e, &raw.task_string()))
            }
        }
    }

    pub fn delete(record_id: &[u8]) -> Result<usize> {
        let rtn = diesel::delete(task.filter(task_id.eq(record_id))).execute(&get_conn()?);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    /// delete finished task after `delay` seconds
    pub fn delete_finished(delay: i64) -> Result<usize> {
        let time = sql(&format!("'execute_time' < date_sub(now(), interval {} second)", delay));
        let rtn = diesel::delete(task)
            .filter(time)
            .filter(task_state.eq(1))
            .execute(&get_conn()?);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    pub fn raw_to_error(err: &NatureError, raw: &RawTask) -> Result<usize> {
        use crate::schema::task_error;
        let rtn = {
            let rd = RawTaskError::from_raw(err, raw);
            diesel::insert_into(task_error::table).values(rd).execute(&get_conn()?)
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

    pub fn get_overdue(delay: i64, lim: i64) -> Result<Vec<RawTask>> {
        use crate::schema::task::dsl::*;
        let rtn = task.filter(execute_time.lt(Local::now().checked_add_signed(Duration::seconds(delay)).unwrap().naive_local()))
            .filter(task_state.eq(0))
            .limit(lim)
            .load::<RawTask>(&get_conn()?);
        match rtn {
            Ok(rtn) => Ok(rtn),
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn update_execute_time(record_id: &[u8], delay: i64) -> Result<()> {
        let time = Local::now().checked_add_signed(Duration::seconds(delay)).unwrap().naive_local();
        match diesel::update(task)
            .set(execute_time.eq(time))
            .filter(task_id.eq(record_id))
            .execute(&get_conn()?) {
            Err(e) => {
                warn!("Update delay error: {}", &e);
                Err(DbError::from(e))
            }
            Ok(_) => {
                Ok(())
            }
        }
    }

    pub fn finish_task(record_id: &[u8]) -> Result<usize> {
        match diesel::update(task)
            .set(task_state.eq(1))
            .filter(task_id.eq(record_id))
            .filter(task_state.eq(0))
            .execute(&get_conn()?) {
            Err(e) => {
                warn!("Update task status error: {}", &e);
                Err(DbError::from(e))
            }
            Ok(i) => Ok(i)
        }
    }

    /// increase one times and delay `delay` seconds
    pub fn increase_times_and_delay(record_id: &[u8], delay: i32) -> Result<usize> {
        let time = Local::now().checked_add_signed(Duration::seconds(delay as i64)).unwrap().naive_local();
        let sql = format!("update task set retried_times = retried_times + 1, execute_time = datetime('now', '+{} seconds', 'localtime') where task_id = x'{}'", delay, vec_to_hex_string(&record_id));
        println!("{}", &sql);
        match diesel::update(task)
            .set((execute_time.eq(time), retried_times.eq(retried_times + 1)))
            .filter(task_id.eq(record_id))
            .execute(&get_conn()?) {
            Err(e) => Err(DbError::from(e)),
            Ok(num) => Ok(num)
        }
    }

    pub fn get(record_id: &[u8]) -> Result<Option<RawTask>> {
        let def = task.filter(task_id.eq(record_id))
            .limit(1)
            .load::<RawTask>(&get_conn()?);
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
