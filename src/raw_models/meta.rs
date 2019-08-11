use chrono::prelude::*;

use crate::*;

use super::super::schema::meta;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable)]
#[derive(Insertable)]
#[table_name = "meta"]
pub struct RawMeta {
    pub full_key: String,

    /// For human readable what the `Meta` is.
    pub description: Option<String>,

    /// version of the `Meta`
    pub version: i32,

    pub states: Option<String>,

    /// Define whats the `Meta` should include
    pub fields: Option<String>,

    pub config: String,

    pub create_time: NaiveDateTime,
}

impl Default for RawMeta {
    fn default() -> Self {
        RawMeta {
            full_key: String::new(),
            description: None,
            version: 1,
            states: None,
            fields: None,
            config: "{}".to_string(),
            create_time: Local::now().naive_local(),
        }
    }
}

impl RawMeta {
    pub fn is_status(&self) -> bool {
        self.states.is_some()
    }
}

