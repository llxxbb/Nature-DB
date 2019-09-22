use std::convert::TryInto;

use chrono::prelude::*;

use nature_common::{Meta, NatureError, State};

use super::super::schema::meta;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable)]
#[derive(Insertable)]
#[table_name = "meta"]
pub struct RawMeta {
    pub full_key: String,

    /// For human readable what the `Meta` is.
    pub description: Option<String>,

    /// version of the `Meta`
    pub version: i32,

    pub states: Option<String>,

    /// Define whats the `Meta` should include
    pub fields: Option<String>,

    pub config: String,

    pub flag: i32,

    pub create_time: NaiveDateTime,
}

impl Default for RawMeta {
    fn default() -> Self {
        RawMeta {
            full_key: String::new(),
            description: None,
            version: 1,
            states: None,
            fields: None,
            config: "{}".to_string(),
            flag: 1,
            create_time: Local::now().naive_local(),
        }
    }
}

impl From<Meta> for RawMeta {
    fn from(m: Meta) -> Self {
        RawMeta {
            full_key: m.get_full_key(),
            description: None,
            version: m.version,
            states: match m.state {
                None => None,
                Some(x) => Some(State::states_to_string(&x, ","))
            },
            fields: None,
            config: "".to_string(),
            flag: 0,
            create_time: Local::now().naive_local(),
        }
    }
}

impl TryInto<Meta> for RawMeta {
    type Error = NatureError;

    fn try_into(self) -> Result<Meta, Self::Error> {
        let mut rtn = Meta::from_full_key(&self.full_key, self.version)?;
        if let Some(s) = self.states {
            let (s, _) = State::string_to_states(&s)?;
            rtn.state = Some(s);
        }
        Ok(rtn)
    }
}

impl RawMeta {
    pub fn has_states(&self) -> bool {
        self.states.is_some()
    }
}

