use crate::*;
use chrono::prelude::*;
use super::super::schema::thing_defines;
use crate::models::thing_define::ThingDefine;

#[derive(Debug)]
#[derive(Insertable)]
#[table_name = "thing_defines"]
pub struct RawThingDefine<'a> {
    pub key: &'a str,

    /// For human readable what the `Thing` is.
    pub description: Option<String>,

    /// version of the `Thing`
    pub version: i32,

    pub states: Option<String>,

    /// Define whats the `Thing` should include
    pub fields: Option<String>,

    pub create_time: &'a NaiveDateTime,
}

impl<'a> RawThingDefine<'a> {
    pub fn new(define: &'a ThingDefine) -> RawThingDefine {
        RawThingDefine {
            key: &define.key,
            description: define.description.clone(),
            version: define.version,
            states: define.states.clone(),
            fields: define.fields.clone(),
            create_time: &define.create_time,
        }
    }
}
