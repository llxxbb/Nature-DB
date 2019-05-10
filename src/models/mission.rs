use nature_common::{Thing, Executor, Instance};
use std::collections::{HashSet, HashMap};
use crate::{OneStepFlow, Selector};

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct  Mission {
    pub to: Thing,
    pub executor: Executor,
    pub last_status_demand: Option<LastStatusDemand>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LastStatusDemand {
    pub target_status_include: HashSet<String>,
    pub target_status_exclude: HashSet<String>,
}

impl Mission {
    pub fn filter_relations(instance: &Instance, maps: Vec<OneStepFlow>) -> Option<Vec<Mission>> {
        debug!("------------------filter_relations------------------------");
        let mut rtn: Vec<Mission> = Vec::new();
        for m in maps {
            if m.selector.is_some() {
                let selector = &m.selector.clone().unwrap();
                if !Self::context_check(&instance.data.context, selector) {
                    continue;
                }
                // only verify source status, target status will be checked later.
                if !Self::source_status_check(&instance.data.status, selector) {
                    continue;
                }
            }
            let t = Mission {
                to: m.to.clone(),
                executor: m.executor,
                last_status_demand: {
                    match m.selector {
                        None => None,
                        Some(demand) => {
                            let last_demand = LastStatusDemand {
                                target_status_include: demand.target_status_include,
                                target_status_exclude: demand.target_status_exclude,
                            };
                            Some(last_demand)
                        }
                    }
                },
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

    fn context_check(contexts: &HashMap<String, String>, selector: &Selector) -> bool {
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

    fn source_status_check(status: &HashSet<String>, selector: &Selector) -> bool {
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
