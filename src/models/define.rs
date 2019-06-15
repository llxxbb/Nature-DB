use std::env;

use nature_common::{BizMeta, Result};
use crate::RawThingDefine;

pub trait ThingDefineDaoTrait {
    fn get(thing: &BizMeta) -> Result<Option<RawThingDefine>>;
    fn insert(define: &RawThingDefine) -> Result<usize>;
    fn delete(thing: &BizMeta) -> Result<usize>;
}

lazy_static! {
    pub static ref INSTANCE_CONTENT_MAX_LENGTH : usize = {
        env::var("INSTANCE_CONTENT_MAX_LENGTH").unwrap_or_else(|_| "65535".to_string()).parse::<usize>().unwrap()
    };
    pub static ref INSTANCE_CONTEXT_MAX_LENGTH : usize = {
        env::var("INSTANCE_CONTEXT_MAX_LENGTH").unwrap_or_else(|_| "65535".to_string()).parse::<usize>().unwrap()
    };
    pub static ref TASK_CONTENT_MAX_LENGTH : usize = {
        env::var("TASKY_CONTENT_MAX_LENGTH").unwrap_or_else(|_| "16777215".to_string()).parse::<usize>().unwrap()
    };
}
