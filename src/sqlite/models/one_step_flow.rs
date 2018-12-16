use serde_json;

use converter_cfg::*;
use nature_common::*;

use super::super::schema::one_step_flow;

#[derive(Debug)]
#[derive(Insertable, Queryable)]
#[derive(Clone)]
#[table_name = "one_step_flow"]
pub struct RawOneStepFlow {
    pub from_thing: String,
    pub from_version: i32,
    pub to_thing: String,
    pub to_version: i32,
    pub settings: String,
}

impl OneStepFlow {
    pub fn from_raw(val: RawOneStepFlow) -> Result<Vec<OneStepFlow>> {
        let settings = serde_json::from_str::<OneStepFlowSettings>(&val.settings)?;
        let rtn = settings.executor.into_iter().map(|e| {
            OneStepFlow {
                from: Thing {
                    key: val.from_thing,
                    version: val.from_version,
                    thing_type: ThingType::Business,
                },
                to: Thing {
                    key: val.to_thing,
                    version: val.to_version,
                    thing_type: ThingType::Business,
                },
                selector: settings.selector.clone(),
                executor: e,
            }
        }).collect();
        Ok(rtn)
    }
}