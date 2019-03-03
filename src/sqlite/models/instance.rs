use chrono::prelude::*;
use define::*;
use lazy_static::__Deref;
use nature_common::*;
use nature_common::util::vec_to_u128;
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use super::super::schema::instances;

#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "instances"]
pub struct RawInstance {
    instance_id: Vec<u8>,
    thing: String,
    version: i32,
    content: String,
    context: Option<String>,
    status: Option<String>,
    status_version: i32,
    from_thing: Option<String>,
    from_version: Option<i32>,
    from_status_version: Option<i32>,
    event_time: NaiveDateTime,
    execute_time: NaiveDateTime,
    create_time: NaiveDateTime,
}

impl RawInstance {
    pub fn to(&self) -> Result<Instance> {
        let from = match &self.from_thing {
            None => None,
            Some(k) => Some(FromInstance {
                thing: Thing {
                    key: k.to_string(),
                    version: self.from_version.unwrap(),
                    thing_type: ThingType::Business,
                    is_null: false
                },
                status_version: self.from_status_version.unwrap(),
            })
        };
        let id = vec_to_u128(&self.instance_id);
        let context = match self.context {
            None => HashMap::new(),
            Some(ref s) => serde_json::from_str::<HashMap<String, String>>(s)?
        };
        let status = match self.status {
            None => HashSet::new(),
            Some(ref s) => serde_json::from_str::<HashSet<String>>(s)?
        };
        Ok(Instance {
            id,
            data: InstanceNoID {
                thing: Thing {
                    key: self.thing.clone(),
                    version: self.version,
                    thing_type: ThingType::Business,
                    is_null: false
                },
                event_time: self.event_time.timestamp_millis(),
                execute_time: self.execute_time.timestamp_millis(),
                create_time: self.create_time.timestamp_millis(),
                content: self.content.clone(),
                context,
                status,
                status_version: self.status_version,
                from,
            },
        })
    }
}

impl RawInstance {
    pub fn new(instance: &Instance) -> Result<RawInstance> {
        let (from_thing, from_version, from_status_version) = match instance.from {
            None => (None, None, None),
            Some(ref from) => (Some(from.thing.key.clone()), Some(from.thing.version), Some(from.status_version))
        };
        Ok(RawInstance {
            instance_id: instance.id.to_ne_bytes().to_vec(),
            thing: instance.thing.key.clone(),
            version: instance.thing.version,
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
            status: match instance.status.len() {
                0 => None,
                _ => Some(serde_json::to_string(&instance.status)?)
            },
            status_version: instance.status_version,
            from_thing,
            from_version,
            from_status_version,
            event_time: NaiveDateTime::from_timestamp(instance.event_time, 0),
            execute_time: NaiveDateTime::from_timestamp(instance.execute_time, 0),
            create_time: NaiveDateTime::from_timestamp(instance.create_time, 0),
        })
    }
}
