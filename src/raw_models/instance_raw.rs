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
    from_meta: Option<String>,
    from_id: Option<Vec<u8>>,
    from_state_version: Option<i32>,
    execute_time: NaiveDateTime,
    create_time: NaiveDateTime,
}

impl RawInstance {
    pub fn to(&self) -> Result<Instance> {
        let from = match &self.from_meta {
            None => None,
            Some(meta) => {
                Some(FromInstance {
                    id: vec_to_u128(&self.from_id.as_ref().unwrap()),
                    meta: meta.to_string(),
                    state_version: self.from_state_version.unwrap(),
                })
            }
        };
        let id = vec_to_u128(&self.instance_id);
        let context = match self.context {
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
                states,
                state_version: self.state_version,
                from,
            },
            execute_time: self.execute_time.timestamp_millis(),
            create_time: self.create_time.timestamp_millis(),
        })
    }

    pub fn new(instance: &Instance) -> Result<RawInstance> {
        let (from_id, from_meta, from_state_version) = match instance.from {
            None => (None, None, None),
            Some(ref from) => (Some(u128_to_vec_u8(from.id)), Some(from.meta.to_string()), Some(from.state_version))
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
            context: {
                let ctx_len = instance.context.len();
                if ctx_len > *INSTANCE_CONTEXT_MAX_LENGTH.deref() {
                    return Err(NatureError::SystemError("context's length can' be over : ".to_owned() + &INSTANCE_CONTEXT_MAX_LENGTH.to_string()));
                }
                match ctx_len {
                    0 => None,
                    _ => Some(serde_json::to_string(&instance.context)?)
                }
            },
            states: match instance.states.len() {
                0 => None,
                _ => Some(serde_json::to_string(&instance.states)?)
            },
            state_version: instance.state_version,
            from_meta,
            para: "".to_string(),
            from_state_version,
            from_id,
            execute_time: Local.timestamp_millis(instance.execute_time).naive_local(),
            create_time: Local.timestamp_millis(instance.create_time).naive_local(),
        })
    }
}