use serde::Serialize;
use std::env;
use std::fmt::Debug;
use super::*;

pub trait ThingDefineDaoTrait {
    fn get(thing: &Thing) -> Result<Option<ThingDefine>>;
    fn insert(define: &ThingDefine) -> Result<usize>;
    fn delete(thing: &Thing) -> Result<usize>;
}

pub trait ThingDefineCacheTrait {
    fn get(thing: &Thing) -> Result<ThingDefine>;
}


pub trait OneStepFlowDaoTrait {
    fn get_relations(from: &Thing) -> Result<Option<Vec<OneStepFlow>>>;
}

pub trait DeliveryDaoTrait {
    fn insert<T: Sized + Serialize + Send + Debug>(carrier: &Carrier<T>) -> Result<u128>;
    fn delete(id: u128) -> Result<usize>;
    fn carrier_to_error<T: Serialize + Debug>(err: &NatureError, carrier: &Carrier<T>) -> Result<usize>;
    fn raw_to_error(err: &NatureError, raw: &RawDelivery) -> Result<usize>;
    fn update_execute_time(_id: u128, new_time: i64) -> Result<()>;
    fn increase_times(_id: Vec<u8>) -> Result<()>;
    fn get<T: Sized + Serialize + Debug>(id: u128) -> Result<Carrier<T>>;
    fn get_overdue() -> Result<Option<Vec<RawDelivery>>>;
}

pub trait InstanceDaoTrait {
    fn insert(instance: &Instance) -> Result<usize>;
    /// check whether source stored earlier
    fn is_exists(instance: &Instance) -> Result<bool>;
    fn get_by_id(id: u128) -> Result<Option<Instance>>;
    fn get_by_key(key: &str, id: u128) -> Result<Option<Instance>>;
}

pub trait StorePlanDaoTrait {
    /// replace the plan if plan exists.
    fn save(plan: &RawPlanInfo) -> Result<()>;
    fn get(key: &str) -> Result<Option<PlanInfo>>;
}

lazy_static! {
    pub static ref INSTANCE_CONTENT_MAX_LENGTH : usize = {
        env::var("INSTANCE_CONTENT_MAX_LENGTH").unwrap_or("65535".to_string()).parse::<usize>().unwrap()
    };
    pub static ref INSTANCE_CONTEXT_MAX_LENGTH : usize = {
        env::var("INSTANCE_CONTEXT_MAX_LENGTH").unwrap_or("65535".to_string()).parse::<usize>().unwrap()
    };
    pub static ref DELIVERY_CONTENT_MAX_LENGTH : usize = {
        env::var("DELIVERY_CONTENT_MAX_LENGTH").unwrap_or("16777215".to_string()).parse::<usize>().unwrap()
    };
    pub static ref PLAN_CONTENT_MAX_LENGTH : usize = {
        env::var("PLAN_CONTENT_MAX_LENGTH").unwrap_or("16777215".to_string()).parse::<usize>().unwrap()
    };
}
