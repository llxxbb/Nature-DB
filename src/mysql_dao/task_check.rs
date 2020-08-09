use chrono::NaiveDateTime;

use nature_common::Result;

use crate::MySql;

pub struct TaskChecker;

impl TaskChecker {
    pub async fn check(cfg: &Condition) -> Result<usize> {
        let sql = r"SELECT count(1) as num
                FROM nature.task
                WHERE task_key > :task_gt and task_key < :task_lt
                    and create_time >= :time_ge and create_time < :time_lt
                    and task_state = :state
                    and task_state = :state
            ";
        let p = params! {
            "task_gt" => cfg.key_gt.to_string(),
            "task_lt" => cfg.key_lt.to_string(),
            "time_ge" => cfg.time_ge,
            "time_lt" => cfg.time_lt,
            "state" => cfg.state,
        };
        let vec = MySql::fetch(sql, p, mysql_async::from_row).await?;
        Ok(vec[0])
    }
}


pub struct Condition {
    pub key_gt: String,
    pub key_lt: String,
    pub time_ge: NaiveDateTime,
    pub time_lt: NaiveDateTime,
    pub state: i8,
}

#[cfg(test)]
mod test {
    use std::env;

    use chrono::{Local, TimeZone};

    use nature_common::setup_logger;

    use crate::CONN_STR;

    use super::*;

    #[tokio::test]
    async fn get_test() {
        env::set_var("DATABASE_URL", CONN_STR);
        let _ = setup_logger();

        let condition = Condition {
            key_gt: "".to_string(),
            key_lt: "".to_string(),
            time_ge: Local::now().naive_local(),
            time_lt: Local::now().naive_local(),
            state: 1,
        };
        let num = TaskChecker::check(&condition).await.unwrap();
        assert_eq!(0, num)
    }

    #[tokio::test]
    #[ignore]
    async fn get_ignore_test() {
        env::set_var("DATABASE_URL", CONN_STR);
        let _ = setup_logger();

        let condition = Condition {
            key_gt: "B:sale/item/count/tag_second:1".to_string(),
            key_lt: "B:sale/item/count/tag_second:2".to_string(),
            time_ge: Local.ymd(2020, 8, 7).and_hms(0, 0, 0).naive_local(),
            time_lt: Local::now().naive_local(),
            state: 1,
        };
        let num = TaskChecker::check(&condition).await.unwrap();
        assert_eq!(5, num)
    }
}