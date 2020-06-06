use chrono::{Duration, Local};
use mysql_async::Value;

use nature_common::{NatureError, Result};

use crate::MySql;
use crate::raw_models::{RawTask, RawTaskError};

pub struct TaskDaoImpl;

impl TaskDaoImpl {
    pub async fn insert(raw: &RawTask) -> Result<usize> {
        let sql = r"INSERT INTO task
            (task_id, task_key, task_type, task_for, task_state, `data`, create_time, execute_time, retried_times)
            VALUES(:task_id, :task_key, :task_type, :task_for, :task_state, :data, :create_time, :execute_time, :retried_times)";

        let p: Vec<(String, Value)> = raw.clone().into();
        let num: usize = MySql::idu(sql, p).await?;
        if num == 1 {
            debug!("---- saved task KEY: {} FOR: {} TYPE: {}", &raw.task_key, &raw.task_for, raw.task_type)
        } else {
            warn!("==== saved 0 task KEY: {} FOR: {} TYPE: {}", &raw.task_key, &raw.task_for, raw.task_type)
        }
        Ok(num)
    }

    async fn delete(_record_id: &str) -> Result<usize> {
        let sql = r"DELETE FROM nature.task
            WHERE task_id=:task_id";

        let p = params! {
            "task_id" => _record_id,
        };

        let rtn: usize = MySql::idu(sql, p).await?;
        Ok(rtn)
    }

    /// delete finished task after `delay` seconds
    pub async fn delete_finished(_delay: i64) -> Result<usize> {
        let sql = r"DELETE FROM task
            WHERE execute_time < date_sub(now(), interval :delay second) AND task_state = 1";

        let p = params! {
            "delay" => _delay,
        };

        let rtn: usize = MySql::idu(sql, p).await?;
        Ok(rtn)
    }

    pub async fn raw_to_error(err: &NatureError, raw: &RawTask) -> Result<usize> {
        let sql = r"INSERT INTO task_error
            (task_id, task_key, task_type, task_for, `data`, create_time, msg)
            VALUES(:task_id, :task_key, :task_type, :task_for, :data, :create_time, :msg)";

        let rd = RawTaskError::from_raw(err, raw);
        let p: Vec<(String, Value)> = rd.into();
        let num: usize = MySql::idu(sql, p).await?;
        Ok(num)
    }

    pub async fn get_overdue(delay: i64, _limit: i64) -> Result<Vec<RawTask>> {
        let sql = r"SELECT task_id, task_key, task_type, task_for, task_state, `data`, create_time, execute_time, retried_times
            FROM task
            WHERE execute_time < :execute_time and task_state = 0
            LIMIT :limit";

        let _execute_time = Local::now().checked_add_signed(Duration::seconds(delay)).unwrap().naive_local();
        let p = params! {
            "execute_time" => _execute_time,
            "limit" => _limit,
        };

        MySql::fetch(sql, p, RawTask::from).await
    }

    pub async fn update_execute_time(_record_id: &str, delay: i64) -> Result<usize> {
        let sql = r"UPDATE nature.task
            SET execute_time=:execute_time
            WHERE task_id=:task_id";

        let _time = Local::now().checked_add_signed(Duration::seconds(delay)).unwrap().naive_local();
        let p = params! {
            "execute_time" => _time,
            "task_id" => _record_id,
        };
        let rtn = MySql::idu(sql, p).await?;
        Ok(rtn)
    }

    pub async fn finish_task(_record_id: &str) -> Result<usize> {
        let sql = r"UPDATE nature.task
            SET task_state=1
            WHERE task_id=:task_id and task_state=0";

        let p = params! {
            "task_id" => _record_id,
        };
        let rtn = MySql::idu(sql, p).await?;
        Ok(rtn)
    }

    /// increase one times and delay `delay` seconds
    pub async fn increase_times_and_delay(_record_id: &str, delay: i32) -> Result<usize> {
        let sql = r"UPDATE nature.task
            SET execute_time=:execute_time, retried_times = retried_times+1
            WHERE task_id=:task_id";

        let _time = Local::now().checked_add_signed(Duration::seconds(delay as i64)).unwrap().naive_local();
        let p = params! {
            "execute_time" => _time,
            "task_id" => _record_id,
        };
        let rtn = MySql::idu(sql, p).await?;
        Ok(rtn)
    }

    pub async fn get(_record_id: &str) -> Result<Option<RawTask>> {
        let sql = r"SELECT task_id, task_key, task_type, task_for, task_state, `data`, create_time, execute_time, retried_times
            FROM task
            WHERE task_id=:task_id";

        let p = params! {
            "task_id" => _record_id,
        };

        let rtn = MySql::fetch(sql, p, RawTask::from).await?;
        match rtn.len() {
            0 => Ok(None),
            1 => Ok(Some(rtn[0].clone())),
            _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
        }
    }
}
