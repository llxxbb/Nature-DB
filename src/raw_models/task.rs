use std::fmt::Debug;

use chrono::prelude::*;
use lazy_static::__Deref;
use serde::Serialize;

use nature_common::*;
use nature_common::util::id_tool::generate_id;
use nature_common::util::u128_to_vec_u8;

use crate::models::define::*;

use super::super::schema::task;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Insertable, Queryable, QueryableByName)]
#[table_name = "task"]
pub struct RawTask {
    pub task_id: Vec<u8>,
    pub meta: String,
    pub data_type: i16,
    pub data: String,
    pub create_time: NaiveDateTime,
    pub execute_time: NaiveDateTime,
    pub retried_times: i16,
}

impl RawTask {
    pub fn new<T: Serialize + Debug>(task: &T, meta: &str, data_type: i16) -> Result<RawTask> {
        let json = serde_json::to_string(task)?;
        if json.len() > *TASK_CONTENT_MAX_LENGTH.deref() {
            return Err(NatureError::DaoLogicalError("data's length can' be over : ".to_owned() + &TASK_CONTENT_MAX_LENGTH.to_string()));
        }
        let time = Local::now().naive_local();
        Ok(RawTask {
            task_id: u128_to_vec_u8(generate_id(&task)?),
            meta: meta.to_string(),
            data_type,
            data: json,
            create_time: time,
            execute_time: time,
            retried_times: 0,
        })
    }

    pub fn save<T: Serialize + Debug, F>(task: &T, meta: &str, data_type: i16, saver: F) -> Result<RawTask>
        where F: Fn(&RawTask) -> Result<usize>
    {
        let result = Self::new(task, meta, data_type)?;
        saver(&result)?;
        Ok(result)
    }

    /// by performance reason, for one-to-one carry we can reuse the beginning carry to finish all flows.
    /// That way we need not to communicate with DB for create new and delete old carrier.
    /// But for failure we must redo from beginning. but I think it has small chance.
    /// Another disadvantage is the failure information will be attached to the beginning.
    pub fn finish_old<FI, FD>(&mut self, old: &RawTask, _dao_insert: FI, _dao_delete: FD) -> Result<usize>
        where FI: Fn(&RawTask) -> Result<usize>,
              FD: Fn(&[u8]) -> Result<usize>
    {
        // TODO  当遇到错误时如果要结束的 delivery ID 和新的delivery 不一样 需要结束之前的 delivery 并创建新的 delivery
        self.task_id = old.task_id.clone(); // the id is used for final finished
        Ok(1)
    }


    pub fn save_batch<FI, FD>(news: &[RawTask], old_id: &[u8], dao_insert: FI, dao_delete: FD) -> Result<()>
        where FI: Fn(&RawTask) -> Result<usize>,
              FD: Fn(&[u8]) -> Result<usize>
    {
        for v in news {
            dao_insert(v)?;
        }
        dao_delete(old_id)?;
        Ok(())
    }
}


