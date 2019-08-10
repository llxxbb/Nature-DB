use chrono::prelude::*;

use crate::*;

use super::super::schema::meta;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable)]
#[derive(Insertable)]
#[table_name = "meta"]
pub struct RawThingDefine {
    pub full_key: String,

    /// For human readable what the `Thing` is.
    pub description: Option<String>,

    /// version of the `Thing`
    pub version: i32,

    pub states: Option<String>,

    /// Define whats the `Thing` should include
    pub fields: Option<String>,

    pub create_time: NaiveDateTime,
}

impl Default for RawThingDefine {
    fn default() -> Self {
        RawThingDefine {
            full_key: String::new(),
            description: None,
            version: 1,
            states: None,
            fields: None,
            create_time: Local::now().naive_local(),
        }
    }
}

impl RawThingDefine {
    pub fn is_status(&self) -> bool {
        self.states.is_some()
    }
}

