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
    status_version: i32,
    from_meta: Option<String>,
    from_status_version: Option<i32>,
    event_time: NaiveDateTime,
    execute_time: NaiveDateTime,
    create_time: NaiveDateTime,
}

impl RawInstance {
    pub fn to(&self) -> Result<Instance> {
        let from = match &self.from_meta {
            None => None,
            Some(k) => {
                let meta = Meta::from_string(k)?;
                Some(FromInstance {
                    meta,
                    status_version: self.from_status_version.unwrap(),
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
            data: InstanceNoID {
                meta: Meta::from_string(&self.meta)?,
                event_time: self.event_time.timestamp_millis(),
                execute_time: self.execute_time.timestamp_millis(),
                create_time: self.create_time.timestamp_millis(),
                content: self.content.clone(),
                context,
                states,
                status_version: self.status_version,
                from,
            },
        })
    }

    pub fn new(instance: &Instance) -> Result<RawInstance> {
        let (from_meta, from_status_version) = match instance.from {
            None => (None, None),
            Some(ref from) => (Some(from.meta.get_string()), Some(from.status_version))
        };
        Ok(RawInstance {
            instance_id: instance.id.to_ne_bytes().to_vec(),
            meta: instance.meta.get_string(),
            content: {
                if instance.content.len() > *INSTANCE_CONTENT_MAX_LENGTH.deref() {
                    return Err(NatureError::DaoLogicalError("content's length can' be over : ".to_owned() + &INSTANCE_CONTENT_MAX_LENGTH.to_string()));
                }
                instance.content.clone()
            },
            context: {
                let ctx_len = instance.context.len();
                if ctx_len > *INSTANCE_CONTEXT_MAX_LENGTH.deref() {
                    return Err(NatureError::DaoLogicalError("context's length can' be over : ".to_owned() + &INSTANCE_CONTEXT_MAX_LENGTH.to_string()));
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
            status_version: instance.status_version,
            from_meta: from_meta,
            para: "".to_string(),
            from_status_version,
            event_time: NaiveDateTime::from_timestamp(instance.event_time, 0),
            execute_time: NaiveDateTime::from_timestamp(instance.execute_time, 0),
            create_time: NaiveDateTime::from_timestamp(instance.create_time, 0),
        })
    }
}
