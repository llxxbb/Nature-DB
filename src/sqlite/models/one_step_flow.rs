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

impl RawOneStepFlow {
    pub fn new(from: &Thing, to: &Thing, settings: &OneStepFlowSettings) -> Result<Self> {
        let st = serde_json::to_string(settings)?;
        let rtn = RawOneStepFlow {
            from_thing: from.key.clone(),
            from_version: from.version,
            to_thing: to.key.clone(),
            to_version: to.version,
            settings: st,
        };
        Ok(rtn)
    }
}

impl OneStepFlow {
    pub fn from_raw(val: RawOneStepFlow) -> Result<Vec<OneStepFlow>> {
        let version = val.from_version;
        let settings = serde_json::from_str::<OneStepFlowSettings>(&val.settings)?;
        let selector = settings.selector;
        let mut group = String::new();
        let id = uuid::Uuid::new_v4().to_string();
        // check group: only one group is allowed.
        let mut group_check = true;
        settings.executor.iter().for_each(|e| {
            if group.is_empty() {
                group = e.group.clone();
                if group.is_empty() {
                    group = id.clone()
                }
            } else if group != e.group {
                group_check = false;
            }
        });
        if !group_check {
            return Err(NatureError::VerifyError("in one setting all executor's grpup must be same.".to_string()));
        }
        let rtn = settings.executor.iter().map(|e| {
            let mut e2 = e.clone();
            e2.group = group.clone();
            OneStepFlow {
                from: Thing {
                    key: val.from_thing.clone(),
                    version,
                    thing_type: ThingType::Business,
                },
                to: Thing {
                    key: val.to_thing.clone(),
                    version: val.to_version,
                    thing_type: ThingType::Business,
                },
                selector: selector.clone(),
                executor: e2,
            }
        }).collect();
        Ok(rtn)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn can_not_have_multiple_group() {
        // TODO
    }
}