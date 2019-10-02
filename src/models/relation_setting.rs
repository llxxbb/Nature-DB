use nature_common::{Executor, TargetState};

use crate::FlowSelector;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OneStepFlowSettings {
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
    use nature_common::Protocol;

    use super::*;

    #[test]
    fn none_for_selector_one_step_flow_settings() {
        let setting = OneStepFlowSettings {
            selector: None,
            executor: vec![],
            use_upstream_id: false,
            target_states: None,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[]}"#;
        assert_eq!(result, res_str);
        let res_obj: OneStepFlowSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }

    #[test]
    fn string_to_setting_no_selector() {
        let setting = OneStepFlowSettings {
            selector: None,
            executor: vec![Executor {
                protocol: Protocol::LocalRust,
                url: "nature_demo.dll:order_new".to_string(),
                group: "".to_string(),
                proportion: 1,
            }],
            use_upstream_id: true,
            target_states: Some(TargetState { add: Some(vec!["new".to_string()]), remove: None }),
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[{"protocol":"LocalRust","url":"nature_demo.dll:order_new","proportion":1}],"use_upstream_id":true,"target_states":{"add":["new"]}}"#;
        assert_eq!(result, res_str);
        let res_obj: OneStepFlowSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }
}