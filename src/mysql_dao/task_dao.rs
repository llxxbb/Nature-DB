use chrono::{Duration, Local};
use mysql_async::Value;

use nature_common::{NatureError, Result};

use crate::MySql;
use crate::raw_models::{RawTask, RawTaskError};

lazy_static! {
    pub static ref D_T: TaskDaoImpl = TaskDaoImpl {};
}

#[async_trait]
pub trait TaskDao {
    async fn insert(&self, raw: &RawTask) -> Result<usize>;
    async fn delete(&self, _record_id: &str) -> Result<usize>;
    async fn delete_finished(&self, _delay: i64) -> Result<usize>;
    async fn raw_to_error(&self, err: &NatureError, raw: &RawTask) -> Result<usize>;
    async fn get_overdue(&self, delay: i64, _limit: i64) -> Result<Vec<RawTask>>;
    async fn update_execute_time(&self, _record_id: &str, delay: i64) -> Result<usize>;
    async fn finish_task(&self, _record_id: &str) -> Result<usize>;
    async fn increase_times_and_delay(&self, _record_id: &str, delay: i32) -> Result<usize>;
    async fn get(&self, _record_id: &str) -> Result<Option<RawTask>>;
}

pub struct TaskDaoImpl;

#[async_trait]
impl TaskDao for TaskDaoImpl {
    async fn insert(&self, raw: &RawTask) -> Result<usize> {
        let sql = r"INSERT INTO task
            (task_id, task_key, task_type, task_for, task_state, `data`, create_time, execute_time, retried_times)
            VALUES(:task_id, :task_key, :task_type, :task_for, :task_state, :data, :create_time, :execute_time, :retried_times)";

        let p: Vec<(String, Value)> = raw.clone().into();
        let num: usize = match MySql::idu(sql, p).await {
            Ok(n) => {
                debug!("---- saved task KEY: {} FOR: {} TYPE: {}", &raw.task_key, &raw.task_for, raw.task_type);
                n
            }
            Err(e) => match e {
                NatureError::DaoDuplicated(_) => {
                    warn!("==== task repeated. KEY: {} FOR: {} TYPE: {}", &raw.task_key, &raw.task_for, raw.task_type);
                    0
                }
                _ => return Err(e)
            }
        };
        Ok(num)
    }

    #[allow(dead_code)]
    async fn delete(&self, _record_id: &str) -> Result<usize> {
        let sql = r"DELETE FROM nature.task
            WHERE task_id=:task_id";

        let p = params! {
            "task_id" => _record_id,
        };

        let rtn: usize = MySql::idu(sql, p).await?;
        Ok(rtn)
    }

    /// delete finished task after `delay` seconds
    async fn delete_finished(&self, _delay: i64) -> Result<usize> {
        let sql = r"DELETE FROM task
            WHERE execute_time < date_sub(now(), interval :delay second) AND task_state = 1";

        let p = params! {
            "delay" => _delay,
        };

        let rtn: usize = MySql::idu(sql, p).await?;
        Ok(rtn)
    }

    async fn raw_to_error(&self, err: &NatureError, raw: &RawTask) -> Result<usize> {
        let sql = r"INSERT INTO task_error
            (task_id, task_key, task_type, task_for, `data`, create_time, msg)
            VALUES(:task_id, :task_key, :task_type, :task_for, :data, :create_time, :msg)";

        let rd = RawTaskError::from_raw(err, raw);
        let p: Vec<(String, Value)> = rd.into();
        let num: usize = MySql::idu(sql, p).await?;
        Ok(num)
    }

    async fn get_overdue(&self, delay: i64, _limit: i64) -> Result<Vec<RawTask>> {
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

    async fn update_execute_time(&self, _record_id: &str, delay: i64) -> Result<usize> {
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

    async fn finish_task(&self, _record_id: &str) -> Result<usize> {
        let sql = r"UPDATE nature.task
            SET task_state=1
            WHERE task_id=:task_id and task_state=0";

        let p = params! {
            "task_id" => _record_id,
        };
        let rtn = match MySql::idu(sql, p).await {
            Ok(n) => n,
            Err(e) => {
                warn!("**** save task error : {}", _record_id);
                return Err(e);
            }
        };
        Ok(rtn)
    }

    /// increase one times and delay `delay` seconds
    async fn increase_times_and_delay(&self, _record_id: &str, delay: i32) -> Result<usize> {
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

    async fn get(&self, _record_id: &str) -> Result<Option<RawTask>> {
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

#[cfg(test)]
mod test {
    use std::env;

    use tokio::runtime::Runtime;

    use crate::CONN_STR;

    use super::*;

    #[test]
    #[ignore]
    fn insert_repeat_test() {
        env::set_var("DATABASE_URL", CONN_STR);
        let mut runtime = Runtime::new().unwrap();
        let mut task = RawTask::default();
        let _num = runtime.block_on(D_T.delete("lxb")).unwrap();
        task.task_id = "lxb".to_string();
        let num = runtime.block_on(D_T.insert(&task)).unwrap();
        assert_eq!(1, num);
        let num = runtime.block_on(D_T.insert(&task)).unwrap();
        assert_eq!(0, num);
    }
}