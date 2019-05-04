use std::env;

use nature_common::{Thing, NatureError, Result, Instance};
use crate::raw_models::{RawTask, RawPlanInfo};
use crate::models::plan::PlanInfo;
use crate::models::converter_cfg::OneStepFlow;
use crate::RawThingDefine;

pub trait ThingDefineDaoTrait {
    fn get(thing: &Thing) -> Result<Option<RawThingDefine>>;
    fn insert(define: &RawThingDefine) -> Result<usize>;
    fn delete(thing: &Thing) -> Result<usize>;
}

pub trait ThingDefineCacheTrait {
    fn get(&self, thing: &Thing) -> Result<RawThingDefine>;
}

pub trait OneStepFlowDaoTrait {
    fn get_relations(&self, from: &Thing) -> Result<Option<Vec<OneStepFlow>>>;
}

pub trait TaskDaoTrait {
    fn insert(&self, raw: &RawTask) -> Result<usize>;
    fn delete(&self, record_id: &[u8]) -> Result<usize>;
    fn raw_to_error(&self, err: &NatureError, raw: &RawTask) -> Result<usize>;
    fn update_execute_time(&self, record_id: &[u8], delay: i64) -> Result<()>;
    fn increase_times_and_delay(&self, record_id: &[u8], delay: i32) -> Result<usize>;
    fn get(&self, record_id: &[u8]) -> Result<Option<RawTask>>;
    fn get_overdue(&self, seconds: &str) -> Result<Vec<RawTask>>;
}

pub trait InstanceDaoTrait {
    fn insert(&self, instance: &Instance) -> Result<usize>;
    /// check whether source stored earlier
    fn is_exists(&self, instance: &Instance) -> Result<bool>;
    fn get_by_id(&self, id: u128) -> Result<Option<Instance>>;
    fn get_by_key(&self, key: &str, limit: i64) -> Result<Option<Vec<Instance>>>;
    fn get_by_full_key(&self, key: &str, limit: i64) -> Result<Option<Vec<Instance>>>;
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
