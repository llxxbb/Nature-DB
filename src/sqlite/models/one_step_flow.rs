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
    use super::*;

    #[test]
    fn can_group_is_ok() {
        let settings = OneStepFlowSettings {
            selector: None,
            executor: vec![
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_one".to_string(),
                    group: "grp_one".to_string(),
                    proportion: 100,
                },
            ],
        };
        let raw = RawOneStepFlow {
            from_thing: "from".to_string(),
            from_version: 0,
            to_thing: "to".to_string(),
            to_version: 0,
            settings: serde_json::to_string(&settings).unwrap(),
        };
        let rtn = OneStepFlow::from_raw(raw);
        assert_eq!(rtn.is_ok(), true);
    }

    #[test]
    fn multiple_group_is_illegal() {
        let settings = OneStepFlowSettings {
            selector: None,
            executor: vec![
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_one".to_string(),
                    group: "grp_one".to_string(),
                    proportion: 100,
                },
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_two".to_string(),
                    group: "url_two".to_string(),
                    proportion: 200,
                },
            ],
        };
        let raw = RawOneStepFlow {
            from_thing: "from".to_string(),
            from_version: 0,
            to_thing: "to".to_string(),
            to_version: 0,
            settings: serde_json::to_string(&settings).unwrap(),
        };
        let rtn = OneStepFlow::from_raw(raw);
        assert_eq!(rtn, Err(NatureError::VerifyError("in one setting all executor's grpup must be same.".to_string())));
    }
}