use nature_common::{Executor, TargetState};

use crate::FlowSelector;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RelationSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub selector: Option<FlowSelector>,
    pub executor: Vec<Executor>,
    /// if the downstream is state meta, when `is_main` is set to true, the upstream's id will be used as downstream's id
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub use_upstream_id: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub target_states: Option<TargetState>,
}

fn is_false(val: &bool) -> bool {
    !val.clone()
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
        set.insert("two".to_string());

        let setting = RelationSettings {
            selector: Some(FlowSelector {
                source_state_include: set,
                source_state_exclude: Default::default(),
                target_state_include: Default::default(),
                target_state_exclude: Default::default(),
                context_include: Default::default(),
                context_exclude: Default::default(),
            }),
            executor: vec![],
            use_upstream_id: false,
            target_states: None,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"selector":{"source_status_include":["one","two"]},"executor":[]}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn empty_executor_test() {
        let setting = RelationSettings {
            selector: None,
            executor: vec![],
            use_upstream_id: false,
            target_states: None,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[]}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn executor_test() {
        let setting = RelationSettings {
            selector: None,
            executor: vec![Executor {
                protocol: Protocol::LocalRust,
                url: "nature_demo.dll:order_new".to_string(),
                group: "".to_string(),
                proportion: 1,
            }],
            use_upstream_id: false,
            target_states: None,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[{"protocol":"LocalRust","url":"nature_demo.dll:order_new","proportion":1}]}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn use_upstream_id() {
        let setting = RelationSettings {
            selector: None,
            executor: vec![],
            use_upstream_id: true,
            target_states: None,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[],"use_upstream_id":true}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn target_state() {
        let setting = RelationSettings {
            selector: None,
            executor: vec![],
            use_upstream_id: false,
            target_states: Some(TargetState { add: Some(vec!["new".to_string()]), remove: None }),
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[],"target_states":{"add":["new"]}}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }
}