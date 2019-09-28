use serde_json;

use nature_common::*;

use crate::OneStepFlowSettings;
use crate::schema::one_step_flow;

#[derive(Debug)]
#[derive(Insertable, Queryable)]
#[derive(Clone)]
#[table_name = "one_step_flow"]
pub struct RawOneStepFlow {
    pub from_meta: String,
    pub to_meta: String,
    pub settings: String,
    pub flag: i32,
}

impl RawOneStepFlow {
    pub fn new(from: &str, to: &str, settings: &OneStepFlowSettings) -> Result<Self> {
        let st = serde_json::to_string(settings)?;
        let rtn = RawOneStepFlow {
            from_meta: from.to_string(),
            to_meta: to.to_string(),
            settings: st,
            flag: 1,
        };
        Ok(rtn)
    }
}
