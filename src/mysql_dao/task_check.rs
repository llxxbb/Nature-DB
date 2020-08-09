use chrono::NaiveDateTime;

use nature_common::Result;

pub struct TaskChecker;

impl TaskChecker {
    pub async fn check(cfg: &Condition) -> Result<usize> {
        let sql = r"SELECT count(1)
            FROM task
            WHERE execute_time < :execute_time and task_state = 0
            LIMIT :limit";

        // let _execute_time = Local::now().checked_add_signed(Duration::seconds(delay)).unwrap().naive_local();
        // let p = params! {
        //     "execute_time" => _execute_time,
        //     "limit" => _limit,
        // };
        //
        // MySql::fetch(sql, p, RawTask::from).await
        Ok(0)
    }
}

pub struct Condition {
    pub key_gt: String,
    pub key_lt: String,
    pub time_ge: NaiveDateTime,
    pub time_lt: NaiveDateTime,
}