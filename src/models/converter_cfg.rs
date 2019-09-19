use std::collections::HashSet;

use nature_common::*;

use crate::OneStepFlow;

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub instance: Instance,
    pub maps: Vec<OneStepFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Selector {
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_status_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_status_exclude: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_status_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_status_exclude: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_exclude: HashSet<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OneStepFlowSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub selector: Option<Selector>,
    pub executor: Vec<Executor>,
    /// if the downstream is state meta, when `is_main` is set to true, the upstream's id will be used as downstream's id
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub use_upstream_id: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub target_state: Option<TargetState>,
}

fn is_false(val: &bool) -> bool {
    !val.clone()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn selector_serder_test() {
        let mut se = Selector {
            source_status_include: HashSet::new(),
            source_status_exclude: HashSet::new(),
            target_status_include: HashSet::new(),
            target_status_exclude: HashSet::new(),
            context_include: HashSet::new(),
            context_exclude: HashSet::new(),
        };
        // test for null
        let rtn = serde_json::to_string(&se);
        assert_eq!(rtn.unwrap(), "{}");

        // test for some value
        se.source_status_include.insert("aaa".to_string());
        let rtn = serde_json::to_string(&se).unwrap();
        assert_eq!(rtn, "{\"source_status_include\":[\"aaa\"]}");

        // deserialize

        let de: Selector = serde_json::from_str(&rtn).unwrap();
        assert_eq!(de.context_exclude.is_empty(), true);
        assert_eq!(de.source_status_include.len(), 1);
    }

    #[test]
    fn none_for_selector_one_step_flow_settings() {
        let setting = OneStepFlowSettings {
            selector: None,
            executor: vec![],
            use_upstream_id: false,
            target_state: None,
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
            target_state: None,
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[{"protocol":"LocalRust","url":"nature_demo.dll:order_new","proportion":1}],"use_upstream_id":true}"#;
        assert_eq!(result, res_str);
        let res_obj: OneStepFlowSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }
}
