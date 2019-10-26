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

    pub fn get_string(&self) -> String {
        format!("relation[{}->{}]", self.from_meta, self.to_meta)
    }
}

#[cfg(test)]
mod test{

    use super::*;

    #[test]
    fn get_string_test(){
        let result = RawRelation::new("a", "b", &RelationSettings::default()).unwrap();
        assert_eq!(result.get_string(), "relation[a->b]")
    }
}

