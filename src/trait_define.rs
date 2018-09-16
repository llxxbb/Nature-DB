use serde::Serialize;
use std::fmt::Debug;
use super::*;

pub trait ThingDefineDaoTrait {
    fn get(thing: &Thing) -> Result<Option<ThingDefine>, NatureError>;
    fn insert(define: &ThingDefine) -> Result<usize, NatureError>;
    fn delete(thing: &Thing) -> Result<usize, NatureError>;
}

pub trait ThingDefineCacheTrait {
    fn get(thing: &Thing) -> Result<ThingDefine, NatureError>;
}


pub trait OneStepFlowDaoTrait {
    fn get_relations(from: &Thing) -> Result<Option<Vec<OneStepFlow>>, NatureError>;
}

pub trait DeliveryDaoTrait {
    fn insert<T: Sized + Serialize + Send + Debug>(carrier: &Carrier<T>) -> Result<u128, NatureError>;
    fn delete(id: u128) -> Result<(), NatureError>;
    fn move_to_error<T: Sized + Serialize + Debug>(err: CarryError<T>) -> Result<(), NatureError>;
    fn update_execute_time(_id: u128, new_time: i64) -> Result<(), NatureError>;
    fn get<T: Sized + Serialize + Debug>(id: u128) -> Result<Carrier<T>, NatureError>;
}

pub trait InstanceDaoTrait {
    fn insert(instance: &Instance) -> Result<usize, NatureError>;
    /// check whether source stored earlier
    fn is_exists(instance: &Instance) -> Result<bool, NatureError>;
    fn get_by_id(id: u128) -> Result<Option<Instance>, NatureError>;
    fn get_by_key(key: &str, id: u128) -> Result<Option<Instance>, NatureError>;
}

pub trait StorePlanDaoTrait {
    /// replace the plan if plan exists.
    fn save(plan: &PlanInfo) -> Result<(), NatureError>;
    fn get(key: &str) -> Result<Option<PlanInfo>, NatureError>;
}
