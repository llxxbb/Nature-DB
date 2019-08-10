use serde_json;

use nature_common::*;

use crate::schema::one_step_flow;
use crate::models::converter_cfg::{OneStepFlowSettings, OneStepFlow};

#[derive(Debug)]
#[derive(Insertable, Queryable)]
#[derive(Clone)]
#[table_name = "one_step_flow"]
pub struct RawOneStepFlow {
    pub from_meta: String,
    pub from_version: i32,
    pub to_meta: String,
    pub to_version: i32,
    pub settings: String,
}

impl RawOneStepFlow {
    pub fn new(from: &Meta, to: &Meta, settings: &OneStepFlowSettings) -> Result<Self> {
        let st = serde_json::to_string(settings)?;
        let rtn = RawOneStepFlow {
            from_meta: from.get_full_key(),
            from_version: from.version,
            to_meta: to.get_full_key(),
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
                from: Meta::from_full_key(&val.from_meta, version).unwrap(),
                to: Meta::from_full_key(&val.to_meta, val.to_version).unwrap(),
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
    use crate::models::converter_cfg::OneStepFlowSettings;

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
            from_meta: "from".to_string(),
            from_version: 0,
            to_meta: "to".to_string(),
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
            from_meta: "from".to_string(),
            from_version: 0,
            to_meta: "to".to_string(),
            to_version: 0,
            settings: serde_json::to_string(&settings).unwrap(),
        };
        let rtn = OneStepFlow::from_raw(raw);
        assert_eq!(rtn, Err(NatureError::VerifyError("in one setting all executor's grpup must be same.".to_string())));
    }
}