use chrono::prelude::*;

use crate::schema::plan;

#[derive(Debug)]
#[derive(Insertable, Queryable, Clone)]
#[table_name = "plan"]
pub struct RawPlanInfo {
    pub upstream: String,
    pub to_biz: String,
    pub to_version: i32,
    pub content: String,
    pub create_time: NaiveDateTime,
}

