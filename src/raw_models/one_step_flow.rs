use serde_json;

use nature_common::*;

use crate::models::converter_cfg::{OneStepFlow, OneStepFlowSettings};
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
    pub fn new(from: &Meta, to: &Meta, settings: &OneStepFlowSettings) -> Result<Self> {
        let st = serde_json::to_string(settings)?;
        let rtn = RawOneStepFlow {
            from_meta: from.get_string(),
            to_meta: to.get_string(),
            settings: st,
            flag: 1,
        };
        Ok(rtn)
    }
}

impl OneStepFlow {
    pub fn from_raw(val: RawOneStepFlow) -> Result<Vec<OneStepFlow>> {
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
        let use_upstream_id = settings.use_upstream_id;
        let rtn = settings.executor.iter().map(|e| {
            let mut e2 = e.clone();
            e2.group = group.clone();
            OneStepFlow {
                from: Meta::from_string(&val.from_meta).unwrap(),
                to: Meta::from_string(&val.to_meta).unwrap(),
                selector: selector.clone(),
                executor: e2,
                use_upstream_id,
            }
        }).collect();
        Ok(rtn)
    }
}

#[cfg(test)]
mod test {
    use crate::models::converter_cfg::OneStepFlowSettings;

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
            use_upstream_id: false,
        };
        let raw = RawOneStepFlow {
            from_meta: "from".to_string(),
            to_meta: "to".to_string(),
            settings: serde_json::to_string(&settings).unwrap(),
            flag: 1,
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
            use_upstream_id: false,
        };
        let raw = RawOneStepFlow {
            from_meta: "from".to_string(),
            to_meta: "to".to_string(),
            settings: serde_json::to_string(&settings).unwrap(),
            flag: 1,
        };
        let rtn = OneStepFlow::from_raw(raw);
        assert_eq!(rtn, Err(NatureError::VerifyError("in one setting all executor's grpup must be same.".to_string())));
    }
}