use nature_common::{DynamicConverter, Executor, Instance, Meta, MetaType, Result, TargetState};

use crate::flow_tool::ContextChecker;
use crate::flow_tool::StateChecker;
use crate::Relation;

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Mission {
    pub to: Meta,
    pub executor: Executor,
    pub states_demand: Option<TargetState>,
    pub use_upstream_id: bool,
    pub delay: i32,
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

    /// Check the instance's context, sys_context and states whether satisfy the Selector request
    pub fn get_by_instance(instance: &Instance, relations: &Option<Vec<Relation>>, ctx_chk: ContextChecker, sta_chk: StateChecker) -> Option<Vec<Mission>> {
        if relations.is_none() { return None; }
        let relations = relations.as_ref().unwrap();
        let mut rtn: Vec<Mission> = Vec::new();
        for r in relations {
            if r.selector.is_some() {
                let selector = &r.selector.clone().unwrap();
                if !ctx_chk(&instance.data.context, &selector.context_none, &selector.context_all, &selector.context_any) {
                    continue;
                }
                if !ctx_chk(&instance.data.sys_context, &selector.sys_context_none, &selector.sys_context_all, &selector.sys_context_any) {
                    continue;
                }
                // only verify source status, target status will be checked later.
                if !sta_chk(&instance.data.states, &selector.state_none, &selector.state_all, &selector.state_any) {
                    continue;
                }
            }
            let m = Mission {
                to: r.to.clone(),
                executor: r.executor.clone(),
                states_demand: r.target_states.clone(),
                use_upstream_id: r.use_upstream_id,
                delay: r.delay,
            };
            // debug!("instance meta: {}, selected relation is {}", instance.meta, r.relation_string());
            rtn.push(m);
        }
        match rtn.len() {
            x  if x > 0 => {
                Some(rtn)
            }
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::FlowSelector;
    use crate::models::flow_tool::{context_check, state_check};

    use super::*;

    #[test]
    fn state_verify() {
        let mut relation = Relation::default();
        let mut selector = FlowSelector::default();
        selector.state_any.insert("a".to_string());
        relation.selector = Some(selector);
        let relations = Some(vec![relation]);
        let mut instance = Instance::default();
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_some(), false);
        instance.states.insert("a".to_string());
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_some(), true);
    }

    #[test]
    fn sys_context_verify() {
        let mut relation = Relation::default();
        let mut selector = FlowSelector::default();
        selector.sys_context_any.insert("a".to_string());
        relation.selector = Some(selector);
        let relations = Some(vec![relation]);
        let mut instance = Instance::default();
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_some(), false);
        instance.sys_context.insert("a".to_string(), "x".to_string());
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_some(), true);
    }

    #[test]
    fn context_verify() {
        let mut relation = Relation::default();
        let mut selector = FlowSelector::default();
        selector.context_any.insert("a".to_string());
        relation.selector = Some(selector);
        let relations = Some(vec![relation]);
        let mut instance = Instance::default();
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_some(), false);
        instance.context.insert("a".to_string(), "x".to_string());
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_some(), true);
    }

    #[test]
    fn mission_copy_from_relation() {
        let meta = Meta::from_string("B:hello:1").unwrap();
        let executor = Executor::for_local("abc");
        let mut state = TargetState::default();
        state.add = Some(vec!["a".to_string()]);
        let state = Some(state);
        let relation = Relation {
            from: "a".to_string(),
            to: meta.clone(),
            selector: None,
            executor: executor.clone(),
            use_upstream_id: true,
            target_states: state.clone(),
            delay: 2,
        };
        let relations = Some(vec![relation]);
        let rtn = Mission::get_by_instance(&Instance::default(), &relations, context_check, state_check);
        let rtn = &rtn.unwrap()[0];
        assert_eq!(rtn.delay, 2);
        assert_eq!(rtn.executor, executor);
        assert_eq!(rtn.to, meta);
        assert_eq!(rtn.use_upstream_id, true);
        assert_eq!(rtn.states_demand, state);
    }

    #[test]
    fn many_relations() {
        let relations = Some(vec![Relation::default(), Relation::default(), Relation::default()]);
        let rtn = Mission::get_by_instance(&Instance::default(), &relations, context_check, state_check);
        assert_eq!(rtn.unwrap().len(), 3);
    }

    #[test]
    fn one_relation_but_no_selector() {
        let relations = Some(vec![Relation::default()]);
        let rtn = Mission::get_by_instance(&Instance::default(), &relations, context_check, state_check);
        assert_eq!(rtn.is_some(), true);
        assert_eq!(rtn.unwrap().len(), 1);
    }

    #[test]
    fn no_relation() {
        let rtn = Mission::get_by_instance(&Instance::default(), &None, context_check, state_check);
        assert_eq!(rtn.is_none(), true);
    }
}


// #[cfg(test)]
// mod other_test {
//     use nature_common::Protocol;
//
//     use super::*;
//
//     #[test]
//     fn input_cfg_is_empty() {
//         let instance = Instance::default();
//         let osf: Vec<Relation> = Vec::new();
//         let osf = Some(osf);
//         let option = Mission::get_by_instance(&instance, &osf);
//         assert_eq!(option.is_none(), true)
//     }
//
//     #[test]
//     fn no_selector_but_only_executor() {
//         let instance = Instance::default();
//         let osf = vec![new_for_local_executor("B:from:1", "B:to:1", "local").unwrap()];
//         let osf = Some(osf);
//         let option = Mission::get_by_instance(&instance, &osf);
//         assert_eq!(option.unwrap().len(), 1)
//     }
//
//     pub fn new_for_local_executor(from: &str, to: &str, local_executor: &str) -> Result<Relation> {
//         Ok(Relation {
//             from: from.to_string(),
//             to: Meta::from_string(to)?,
//             selector: None,
//             executor: Executor {
//                 protocol: Protocol::LocalRust,
//                 url: local_executor.to_string(),
//                 group: "".to_string(),
//                 weight: 1,
//             },
//             use_upstream_id: false,
//             target_states: None,
//             delay: 0,
//         })
//     }
// }

