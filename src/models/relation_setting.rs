use nature_common::{Executor, is_default};

use crate::FlowSelector;
use crate::relation_target::RelationTarget;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RelationSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub selector: Option<FlowSelector>,
    /// array executor will share the convert task
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub executor: Option<Executor>,
    /// filter will execute after executor,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub filter_after: Vec<Executor>,
    /// if the downstream is state meta, when `is_main` is set to true, the upstream's id will be used as downstream's id
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub use_upstream_id: bool,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub target: RelationTarget,
    // delay seconds to execute the converter
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub delay: i32,
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use nature_common::{Protocol, TargetState};

    use super::*;

    #[test]
    fn selector_test() {
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        let mut fs = FlowSelector::default();
        fs.state_all = set;

        let mut setting = RelationSettings::default();
        setting.selector = Some(fs);

        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"selector":{"state_all":["one"]}}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn empty_executor_test() {
        let setting = RelationSettings::default();
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn executor_test() {
        let executor = Executor {
            protocol: Protocol::LocalRust,
            url: "nature_demo:order_new".to_string(),
            settings: "".to_string(),
        };
        let mut setting = RelationSettings::default();
        setting.executor = Some(executor);
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":{"protocol":"localRust","url":"nature_demo:order_new"}}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn use_upstream_id() {
        let mut setting = RelationSettings::default();
        setting.use_upstream_id = true;
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"use_upstream_id":true}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn target_state() {
        let state = TargetState { add: Some(vec!["new".to_string()]), remove: None, need_all: Default::default(), need_any: Default::default(), need_none: Default::default() };
        let mut setting = RelationSettings::default();
        setting.target.states = Some(state);
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"target":{"states":{"add":["new"]}}}"#;
        assert_eq!(result, res_str);
        let res_obj: RelationSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }
}