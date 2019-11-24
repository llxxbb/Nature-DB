use std::collections::{HashMap, HashSet};

use nature_common::{DynamicConverter, Executor, Instance, Meta, MetaType, NatureError, Result, TargetState};

use crate::{FlowSelector, Relation};

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Mission {
    pub to: Meta,
    pub executor: Executor,
    pub states_demand: Option<StateDemand>,
    pub use_upstream_id: bool,
    pub delay: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StateDemand {
    pub last_states_include: HashSet<String>,
    pub last_states_exclude: HashSet<String>,
    pub target_states: Option<TargetState>,
}

impl StateDemand {
    pub fn check_last(&self, last: &HashSet<String>) -> Result<()> {
        for s in &self.last_states_include {
            if !last.contains(s) {
                return Err(NatureError::VerifyError(format!("must include status: {}", &s)));
            }
        }
        for s in &self.last_states_exclude {
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
                states_demand: None,
                use_upstream_id: d.use_upstream_id,
                delay: d.delay,
            };
            missions.push(mission)
        }
        debug!("missions : {:?}", missions);
        Ok(missions)
    }

    pub fn get_by_instance(instance: &Instance, relations: &Option<Vec<Relation>>) -> Option<Vec<Mission>> {
        if relations.is_none() { return None; }
        let relations = relations.as_ref().unwrap();
        let mut rtn: Vec<Mission> = Vec::new();
        for r in relations {
            if r.selector.is_some() {
                let selector = &r.selector.clone().unwrap();
                if !Self::context_check(&instance.data.context, selector) {
                    continue;
                }
                // only verify source status, target status will be checked later.
                if !Self::source_status_check(&instance.data.states, selector) {
                    continue;
                }
            }
            let m = Mission {
                to: r.to.clone(),
                executor: r.executor.clone(),
                states_demand: {
                    let demand = match &r.selector {
                        None => None,
                        Some(demand) => {
                            let last_demand = StateDemand {
                                last_states_include: demand.target_state_include.clone(),
                                last_states_exclude: demand.target_state_exclude.clone(),
                                target_states: None,
                            };
                            Some(last_demand)
                        }
                    };
                    if r.target_states.is_none() {
                        demand
                    } else {
                        let mut d = demand.unwrap_or(StateDemand {
                            last_states_include: Default::default(),
                            last_states_exclude: Default::default(),
                            target_states: None,
                        });
                        d.target_states = r.target_states.clone();
                        Some(d)
                    }
                },
                use_upstream_id: r.use_upstream_id,
                delay: r.delay,
            };
            rtn.push(m);
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
        for exclude in &selector.source_state_exclude {
            if status.contains(exclude) {
                return false;
            }
        }
        for include in &selector.source_state_include {
            if !status.contains(include) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod demand_test {
    use super::*;

    #[test]
    fn source_status_needed() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: set,
                source_state_exclude: HashSet::new(),
                target_state_include: HashSet::new(),
                target_state_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };

        // set status required.
        let osf = vec![relation];
        let osf = Some(osf);

        // condition does not satisfy.
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states = HashSet::new();
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.insert("three".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.insert("one".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);

        // condition satisfy
        instance.data.states.insert("two".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.states.insert("four".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
    }

    #[test]
    fn source_status_exclude() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: HashSet::new(),
                source_state_exclude: set,
                target_state_include: HashSet::new(),
                target_state_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };

        // set state required.
        let osf = vec![relation];
        let osf = Some(osf);

        // condition satisfy
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.states = HashSet::new();
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.states.insert("three".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);

        // condition does not satisfy
        instance.data.states.insert("one".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.insert("two".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.states.remove("one");
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
    }

    #[test]
    fn context_needed() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: HashSet::new(),
                source_state_exclude: HashSet::new(),
                target_state_include: HashSet::new(),
                target_state_exclude: HashSet::new(),
                context_include: set,
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };

        // set state required.
        let osf = vec![relation];
        let osf = Some(osf);

        // condition does not satisfy.
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context = HashMap::new();
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.insert("three".to_string(), "three".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.insert("one".to_string(), "one".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);

        // condition satisfy
        instance.data.context.insert("two".to_string(), "two".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.context.insert("four".to_string(), "four".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
    }

    #[test]
    fn context_exclude_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let mut instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: HashSet::new(),
                source_state_exclude: HashSet::new(),
                target_state_include: HashSet::new(),
                target_state_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: set,
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };

        // set state required.
        let osf = vec![relation];
        let osf = Some(osf);

        // condition satisfy
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.context = HashMap::new();
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);
        instance.data.context.insert("three".to_string(), "three".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_some(), true);

        // condition does not satisfy
        instance.data.context.insert("one".to_string(), "one".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.insert("two".to_string(), "two".to_string());
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
        instance.data.context.remove("one");
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true);
    }

    #[test]
    fn target_state_include_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: HashSet::new(),
                source_state_exclude: HashSet::new(),
                target_state_include: set,
                target_state_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };
        // set state required.
        let osf = Some(vec![relation]);

        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.unwrap()[0].states_demand.as_ref().unwrap().last_states_include.len(), 2);
    }

    #[test]
    fn target_state_exclude_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: HashSet::new(),
                source_state_exclude: HashSet::new(),
                target_state_include: HashSet::new(),
                target_state_exclude: set,
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };
        // set state required.
        let osf = Some(vec![relation]);

        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.unwrap()[0].states_demand.as_ref().unwrap().last_states_exclude.len(), 2);
    }

    #[test]
    fn target_state_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: HashSet::new(),
                source_state_exclude: HashSet::new(),
                target_state_include: set.clone(),
                target_state_exclude: set,
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: Some(TargetState {
                add: Some(vec!["hello".to_string()]),
                remove: None,
            }),
            delay: 0,
        };
        // set state required.
        let osf = Some(vec![relation]);

        let option = Mission::get_by_instance(&instance, &osf);
        let demand = option.unwrap();
        let mission = &demand[0];
        let demand = mission.states_demand.as_ref().unwrap();
        assert_eq!(demand.last_states_include.len(), 2);
        assert_eq!(demand.last_states_exclude.len(), 2);
        assert_eq!(demand.target_states.is_some(), true);
    }

    #[test]
    fn target_all_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let instance = Instance::default();

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_state_include: HashSet::new(),
                source_state_exclude: HashSet::new(),
                target_state_include: HashSet::new(),
                target_state_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: Some(TargetState {
                add: Some(vec!["hello".to_string()]),
                remove: None,
            }),
            delay: 0,
        };
        // set state required.
        let osf = Some(vec![relation]);

        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.unwrap()[0].states_demand.as_ref().unwrap().target_states.is_some(), true);
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
        let osf = Some(osf);
        let option = Mission::get_by_instance(&instance, &osf);
        assert_eq!(option.is_none(), true)
    }

    #[test]
    fn no_selector_but_only_executor() {
        let instance = Instance::default();
        let osf = vec![new_for_local_executor("/B/from:1", "/B/to:1", "local").unwrap()];
        let osf = Some(osf);
        let option = Mission::get_by_instance(&instance, &osf);
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
            delay: 0,
        })
    }
}

