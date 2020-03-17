use nature_common::{Executor, is_false, is_zero, TargetState};

use crate::FlowSelector;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RelationSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub selector: Option<FlowSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub executor: Option<Vec<Executor>>,
    /// if the downstream is state meta, when `is_main` is set to true, the upstream's id will be used as downstream's id
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub use_upstream_id: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub target_states: Option<TargetState>,
    // delay seconds to execute the converter
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub delay: i32,
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use nature_common::Protocol;

    use super::*;

    #[test]
    fn selector_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());

        let setting = RelationSettings {
            selector: Some(FlowSelector {
                source_state_include: set,
                source_state_exclude: Default::default(),
                target_state_include: Default::default(),
                target_state_exclude: Default::default(),
                context_include: Default::default(),
                context_exclude: Default::default(),
            }),
            executor: None,
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"selector":{"source_state_include":["one"]}}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn empty_executor_test() {
        let setting = RelationSettings {
            selector: None,
            executor: None,
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn executor_test() {
        let setting = RelationSettings {
            selector: None,
            executor: Some(vec![Executor {
                protocol: Protocol::LocalRust,
                url: "nature_demo.dll:order_new".to_string(),
                group: "".to_string(),
                weight: 1,
            }]),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[{"protocol":"localRust","url":"nature_demo.dll:order_new"}]}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn use_upstream_id() {
        let setting = RelationSettings {
            selector: None,
            executor: None,
            use_upstream_id: true,
            target_states: None,
            delay: 0,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"use_upstream_id":true}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn target_state() {
        let setting = RelationSettings {
            selector: None,
            executor: None,
            use_upstream_id: false,
            target_states: Some(TargetState { add: Some(vec!["new".to_string()]), remove: None }),
            delay: 0,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"target_states":{"add":["new"]}}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

//    #[test]
//    fn other(){
//        let setting = r#"{“delay”:1,"selector":{"source_state_include":["dispatching"]}, "executor":[{"protocol":"localRust","url":"nature_demo_executor.dll:auto_sign"}]}"#;
//        let obj : RelationSettings = serde_json::from_str(setting).unwrap();
//        dbg!(obj);
//    }
}