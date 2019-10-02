use std::collections::{HashMap, HashSet};

use nature_common::{DynamicConverter, Executor, Instance, Meta, MetaType, NatureError, Result, TargetState};

use crate::{FlowSelector, Relation};

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Mission {
    pub to: Meta,
    pub executor: Executor,
    pub last_states_demand: Option<LastStatusDemand>,
    pub use_upstream_id: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LastStatusDemand {
    pub target_states_include: HashSet<String>,
    pub target_states_exclude: HashSet<String>,
    pub target_states: Option<TargetState>,
}

impl LastStatusDemand {
    pub fn check(&self, last: &HashSet<String>) -> Result<()> {
        for s in &self.target_states_include {
            if !last.contains(s) {
                return Err(NatureError::VerifyError(format!("must include status: {}", &s)));
            }
        }
        for s in &self.target_states_exclude {
            if last.contains(s) {
                return Err(NatureError::VerifyError(format!("can not include status: {}", &s)));
            }
        }
        Ok(())
    }
}

pub type MissionFilter = fn(&Instance, &Vec<Relation>) -> Option<Vec<Mission>>;

impl Mission {
    pub fn for_dynamic(dynamic: Vec<DynamicConverter>) -> Result<Vec<Mission>> {
        debug!("------------------get_dynamic_route------------------------");
        let mut missions: Vec<Mission> = Vec::new();
        for d in dynamic {
            let t = match d.to {
                None => Meta::new("", 1, MetaType::Null)?,
                Some(s) => Meta::new(&s, 1, MetaType::Dynamic)?,
            };
            let mission = Mission {
                to: t,
                executor: d.fun.clone(),
                last_states_demand: None,
                use_upstream_id: d.use_upstream_id,
            };
            missions.push(mission)
        }
        debug!("missions : {:?}", missions);
        Ok(missions)
    }

    pub fn filter_relations(instance: &Instance, maps: &Vec<Relation>) -> Option<Vec<Mission>> {
        let mut rtn: Vec<Mission> = Vec::new();
        for m in maps {
            if m.selector.is_some() {
                let selector = &m.selector.clone().unwrap();
                if !Self::context_check(&instance.data.context, selector) {
                    continue;
                }
                // only verify source status, target status will be checked later.
                if !Self::source_status_check(&instance.data.states, selector) {
                    continue;
                }
            }
            let t = Mission {
                to: m.to.clone(),
                executor: m.executor.clone(),
                last_states_demand: {
                    match &m.selector {
                        None => None,
                        Some(demand) => {
                            let last_demand = LastStatusDemand {
                                target_states_include: demand.target_status_include.clone(),
                                target_states_exclude: demand.target_status_exclude.clone(),
                                target_states: m.target_states.clone(),
                            };
                            Some(last_demand)
                        }
                    }
                },
                use_upstream_id: m.use_upstream_id,
            };
            rtn.push(t);
        }
        match rtn.len() {
            x  if x > 0 => {
                Some(rtn)
            }
            _ => None
        }
    }

    fn context_check(contexts: &HashMap<String, String>, selector: &FlowSelector) -> bool {
        for exclude in &selector.context_exclude {
            if contexts.contains_key(exclude) {
                return false;
            }
        }
        for include in &selector.context_include {
            if !contexts.contains_key(include) {
                return false;
            }
        }
        true
    }

    fn source_status_check(status: &HashSet<String>, selector: &FlowSelector) -> bool {
        for exclude in &selector.source_status_exclude {
            if status.contains(exclude) {
                return false;
            }
        }
        for include in &selector.source_status_include {
            if !status.contains(include) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod selector_test {
    use super::*;

    #[test]
    fn source_status_needed() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        // set status required.
        let osf = vec![new_for_source_status_needed("/B/from:1", "/B/to:1", &set).unwrap()];

        // condition does not satisfy.
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states = HashSet::new();
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.insert("three".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.insert("one".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);

        // condition satisfy
        instance.data.states.insert("two".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.states.insert("four".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
    }

    #[test]
    fn source_status_exclude() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        // set state required.
        let osf = vec![new_for_source_status_excluded("/B/from:1", "/B/to:1", &set).unwrap()];

        // condition satisfy
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.states = HashSet::new();
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.states.insert("three".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);

        // condition does not satisfy
        instance.data.states.insert("one".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.insert("two".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.remove("one");
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
    }

    #[test]
    fn context_needed() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        // set state required.
        let osf = vec![new_for_context_include("/B/from:1", "/B/to:1", &set).unwrap()];

        // condition does not satisfy.
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context = HashMap::new();
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.insert("three".to_string(), "three".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.insert("one".to_string(), "one".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);

        // condition satisfy
        instance.data.context.insert("two".to_string(), "two".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.context.insert("four".to_string(), "four".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
    }

    #[test]
    fn context_exclude_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        // set state required.
        let osf = vec![new_for_context_excluded("/B/from:1", "/B/to:1", &set).unwrap()];

        // condition satisfy
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.context = HashMap::new();
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.context.insert("three".to_string(), "three".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_some(), true);

        // condition does not satisfy
        instance.data.context.insert("one".to_string(), "one".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.insert("two".to_string(), "two".to_string());
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.remove("one");
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true);
    }

    pub fn new_for_source_status_needed(from: &str, to: &str, set: &HashSet<String>) -> Result<Relation> {
        Ok(Relation {
            from: from.to_string(),
            to: Meta::from_string(to)?,
            selector: Some(FlowSelector {
                source_status_include: set.clone(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }

    pub fn new_for_context_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Relation> {
        Ok(Relation {
            from: from.to_string(),
            to: Meta::from_string(to)?,
            selector: Some(FlowSelector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: set.clone(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }

    pub fn new_for_source_status_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Relation> {
        Ok(Relation {
            from: from.to_string(),
            to: Meta::from_string(to)?,
            selector: Some(FlowSelector {
                source_status_include: HashSet::new(),
                source_status_exclude: set.clone(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }

    pub fn new_for_context_include(from: &str, to: &str, set: &HashSet<String>) -> Result<Relation> {
        Ok(Relation {
            from: from.to_string(),
            to: Meta::from_string(to)?,
            selector: Some(FlowSelector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: set.clone(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }
}

#[cfg(test)]
mod other_test {
    use nature_common::Protocol;

    use super::*;

    #[test]
    fn input_cfg_is_empty() {
        let instance = Instance::default();
        let osf: Vec<Relation> = Vec::new();
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.is_none(), true)
    }

    #[test]
    fn no_selector_but_only_executor() {
        let instance = Instance::default();
        let osf = vec![new_for_local_executor("/B/from:1", "/B/to:1", "local").unwrap()];
        let option = Mission::filter_relations(&instance, &osf);
        assert_eq!(option.unwrap().len(), 1)
    }

    pub fn new_for_local_executor(from: &str, to: &str, local_executor: &str) -> Result<Relation> {
        Ok(Relation {
            from: from.to_string(),
            to: Meta::from_string(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: "".to_string(),
                proportion: 1,
            },
            use_upstream_id: false,
            target_states: None,
        })
    }
}

