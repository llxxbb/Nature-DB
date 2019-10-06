use serde_json;

use nature_common::*;

use crate::RelationSettings;
use crate::schema::relation;

#[derive(Debug)]
#[derive(Insertable, Queryable)]
#[derive(Clone)]
#[table_name = "relation"]
pub struct RawRelation {
    pub from_meta: String,
    pub to_meta: String,
    pub settings: String,
    pub flag: i32,
}

impl RawRelation {
    pub fn new(from: &str, to: &str, settings: &RelationSettings) -> Result<Self> {
        let st = serde_json::to_string(settings)?;
        let rtn = RawRelation {
            from_meta: from.to_string(),
            to_meta: to.to_string(),
            settings: st,
            flag: 1,
        };
        Ok(rtn)
    }
}

#[cfg(test)]
mod test{
    use crate::{Relation, FlowSelector};
    use nature_common::{Meta, Executor};
    use std::collections::HashSet;

    #[test]
    fn setting_test(){
        let mut set = HashSet::<String>::new();
        set.insert("one".to_string());
        set.insert("two".to_string());

        let relation = Relation {
            from: "/B/from:1".to_string(),
            to: Meta::from_string("/B/to:1").unwrap(),
            selector: Some(FlowSelector {
                source_status_include: set,
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        };

    }
}