use std::collections::HashMap;
use std::collections::HashSet;

use chrono::prelude::*;
use lazy_static::__Deref;
use serde_json;

use nature_common::*;

use crate::models::define::*;

use super::super::schema::instances;

#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "instances"]
pub struct RawInstance {
    instance_id: Vec<u8>,
    meta: String,
    para: String,
    content: String,
    context: Option<String>,
    states: Option<String>,
    state_version: i32,
    from_meta: String,
    from_para: String,
    from_id: Vec<u8>,
    from_state_version: i32,
    execute_time: NaiveDateTime,
    create_time: NaiveDateTime,
    sys_context: Option<String>,
}

impl RawInstance {
    pub fn to(&self) -> Result<Instance> {
        let from = if self.from_meta.eq("") { None } else {
            Some(FromInstance {
                id: vec_to_u128(&self.from_id),
                meta: self.from_meta.to_string(),
                para: self.from_para.clone(),
                state_version: self.from_state_version,
            })
        };
        let id = vec_to_u128(&self.instance_id);
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
            id,
            data: BizObject {
                meta: self.meta.clone(),
                content: self.content.clone(),
                context,
                sys_context,
                states,
                state_version: self.state_version,
                from,
                para: self.para.clone(),
            },
            execute_time: self.execute_time.timestamp_millis(),
            create_time: self.create_time.timestamp_millis(),
        })
    }

    pub fn new(instance: &Instance) -> Result<RawInstance> {
        let (from_id, from_meta, from_para, from_state_version) = match instance.from {
            None => (vec![0], "".to_string(), "".to_string(), 0),
            Some(ref from) => (u128_to_vec_u8(from.id), from.meta.to_string(), from.para.to_string(), from.state_version)
        };
        Ok(RawInstance {
            instance_id: instance.id.to_ne_bytes().to_vec(),
            meta: instance.meta.clone(),
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
            from_meta,
            from_para,
            para: instance.para.clone(),
            from_state_version,
            from_id,
            execute_time: Local.timestamp_millis(instance.execute_time).naive_local(),
            create_time: Local.timestamp_millis(instance.create_time).naive_local(),
            sys_context: Self::context_to_raw(&instance.sys_context, "sys_context")?,
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
