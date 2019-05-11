use std::env;

use nature_common::{Thing, Result};
use crate::raw_models::{RawPlanInfo};
use crate::models::plan::PlanInfo;
use crate::RawThingDefine;

pub trait ThingDefineDaoTrait {
    fn get(thing: &Thing) -> Result<Option<RawThingDefine>>;
    fn insert(define: &RawThingDefine) -> Result<usize>;
    fn delete(thing: &Thing) -> Result<usize>;
}

pub trait StorePlanDaoTrait {
    /// replace the plan if plan exists.
    fn save(&self, plan: &RawPlanInfo) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<PlanInfo>>;
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
    pub static ref PLAN_CONTENT_MAX_LENGTH : usize = {
        env::var("PLAN_CONTENT_MAX_LENGTH").unwrap_or_else(|_| "16777215".to_string()).parse::<usize>().unwrap()
    };
}
