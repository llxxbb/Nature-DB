use std::clone::Clone;
use std::collections::HashSet;
use std::string::ToString;

use nature_common::{Executor, Meta, NatureError, Protocol, Result};

use crate::{OneStepFlowSettings, RawOneStepFlow, Selector};

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OneStepFlow {
    pub from: Meta,
    pub to: Meta,
    pub selector: Option<Selector>,
    pub executor: Executor,
    pub use_upstream_id: bool,
}

impl Iterator for OneStepFlow {
    type Item = OneStepFlow;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.clone())
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
            return Err(NatureError::VerifyError("in one setting all executor's group must be same.".to_string()));
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

    pub fn new_for_local_executor(from: &str, to: &str, local_executor: &str) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: "".to_string(),
                proportion: 1,
            },
            use_upstream_id: false,
        })
    }
    pub fn new_for_local_executor_with_group_and_proportion(from: &str, to: &str, local_executor: &str, group: &str, proportion: u32) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: group.to_string(),
                proportion,
            },
            use_upstream_id: false,
        })
    }
    pub fn new_for_source_status_needed(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(Selector {
                source_status_include: set.clone(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
        })
    }
    pub fn new_for_source_status_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(Selector {
                source_status_include: HashSet::new(),
                source_status_exclude: set.clone(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
        })
    }
    pub fn new_for_context_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(Selector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: set.clone(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
        })
    }
    pub fn new_for_context_include(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(Selector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: set.clone(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
        })
    }
}

#[cfg(test)]
mod test_from_raw {

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
            target_state: None,
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
            target_state: None,
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