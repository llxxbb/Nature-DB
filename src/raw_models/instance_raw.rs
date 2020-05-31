use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;

use chrono::prelude::*;
use lazy_static::__Deref;
use mysql_async::{Row, Value};
use serde_json;

use nature_common::*;

use crate::models::define::*;

use super::super::schema::instances;

#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "instances"]
pub struct RawInstance {
    ins_key: String,
    content: String,
    context: Option<String>,
    states: Option<String>,
    state_version: i32,
    create_time: NaiveDateTime,
    sys_context: Option<String>,
    from_key: String,
}

impl RawInstance {
    pub fn to(&self) -> Result<Instance> {
        let from = if self.from_key.eq("") { None } else {
            Some(FromInstance::from_str(&self.from_key)?)
        };

        let key = FromInstance::from_key_no_state(&self.ins_key)?;
        let context = match self.context {
            None => HashMap::new(),
            Some(ref s) => serde_json::from_str::<HashMap<String, String>>(s)?
        };
        let sys_context = match self.sys_context {
            None => HashMap::new(),
            Some(ref s) => serde_json::from_str::<HashMap<String, String>>(s)?
        };
        let states = match self.states {
            None => HashSet::new(),
            Some(ref s) => serde_json::from_str::<HashSet<String>>(s)?
        };
        Ok(Instance {
            id: key.id,
            data: BizObject {
                meta: key.meta.clone(),
                content: self.content.clone(),
                context,
                sys_context,
                states,
                state_version: self.state_version,
                from,
                para: key.para.clone(),
            },
            create_time: self.create_time.timestamp_millis(),
        })
    }

    pub fn new(instance: &Instance) -> Result<RawInstance> {
        Ok(RawInstance {
            ins_key: instance.key_no_state(),
            content: {
                if instance.content.len() > *INSTANCE_CONTENT_MAX_LENGTH.deref() {
                    return Err(NatureError::SystemError("content's length can' be over : ".to_owned() + &INSTANCE_CONTENT_MAX_LENGTH.to_string()));
                }
                instance.content.clone()
            },
            context: Self::context_to_raw(&instance.context, "context")?,
            states: match instance.states.len() {
                0 => None,
                _ => Some(serde_json::to_string(&instance.states)?)
            },
            state_version: instance.state_version,
            create_time: Local.timestamp_millis(instance.create_time).naive_local(),
            sys_context: Self::context_to_raw(&instance.sys_context, "sys_context")?,
            from_key: match &instance.from {
                None => "".to_string(),
                Some(from) => from.to_string()
            },
        })
    }

    fn context_to_raw(context: &HashMap<String, String>, which: &str) -> Result<Option<String>> {
        let ctx_len = context.len();
        if ctx_len > *INSTANCE_CONTEXT_MAX_LENGTH.deref() {
            let msg = format!("{}'s length can' be over : {}", which, INSTANCE_CONTEXT_MAX_LENGTH.to_string());
            return Err(NatureError::SystemError(msg));
        }
        match ctx_len {
            0 => Ok(None),
            _ => Ok(Some(serde_json::to_string(context)?))
        }
    }
}

impl From<Row> for RawInstance {
    fn from(row: Row) -> Self {
        let (ins_key, content, context, states, state_version, create_time, sys_context, from_key) = mysql_async::from_row(row);
        RawInstance {
            ins_key,
            content,
            context,
            states,
            state_version,
            create_time,
            sys_context,
            from_key,
        }
    }
}

impl Into<Vec<(String, Value)>> for RawInstance {
    fn into(self) -> Vec<(String, Value)> {
        params! {
            "ins_key" => self.ins_key,
            "content" => self.content,
            "context" => self.context,
            "states" => self.states,
            "state_version" => self.state_version,
            "create_time" => self.create_time,
            "sys_context" => self.sys_context,
            "from_key" => self.from_key,
        }
    }
}
