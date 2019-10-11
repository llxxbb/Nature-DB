use std::collections::HashSet;
use std::convert::TryInto;

use chrono::prelude::*;

use nature_common::{Meta, MetaSetting, NatureError, Result, State, States};

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
            config: match m.setting {
                None => "".to_string(),
                Some(s) => serde_json::to_string(&s).unwrap(),
            },
            flag: 0,
            create_time: Local::now().naive_local(),
        }
    }
}

impl TryInto<Meta> for RawMeta {
    type Error = NatureError;

    fn try_into(self) -> std::result::Result<Meta, Self::Error> {
        let mut rtn = Meta::from_full_key(&self.full_key, self.version)?;
        if let Some(s) = &self.states {
            if !s.is_empty() {
                match State::string_to_states(&s) {
                    Ok((ss, _)) => {
                        self.check_name(&ss)?;
                        rtn.state = Some(ss);
                    }
                    Err(e) => {
                        warn!("meta : {} init error: {:?}", &self.full_key, e);
                        return Err(e);
                    }
                }
            }
        }
        if !self.config.is_empty() {
            let setting: MetaSetting = serde_json::from_str(&self.config)?;
            if setting.is_state {
                rtn.is_state = true;
            }
            rtn.setting = Some(setting);
        }
        if rtn.state.is_some() {
            rtn.is_state = true;
        }
        debug!("get meta:{}", rtn.get_string());
        Ok(rtn)
    }
}

impl RawMeta {
    fn check_name(&self, s: &States) -> Result<()> {
        let mut set: HashSet<String> = HashSet::new();
        for one in s {
            if !set.insert(one.get_name()) {
                return Err(NatureError::VerifyError(format!("repeated state name: {:?}, for `Meta`: {:?}", one.get_name(), self.full_key)));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_into_test() {
        // error full_key
        let meta = RawMeta::default();
        let result: Result<Meta> = meta.try_into();
        assert_eq!(result.err().unwrap(), NatureError::VerifyError("illegal format for `full_key` : ".to_string()));

        let meta = RawMeta::from(Meta::from_full_key("/B/hello", 1).unwrap());
        let result: Meta = meta.try_into().unwrap();
        assert_eq!(result.get_full_key(), "/B/hello")
    }

    #[test]
    fn try_into_state_check_test() {
        let mut meta = RawMeta::from(Meta::from_full_key("/B/hello", 1).unwrap());
        meta.states = Some("a,b".to_string());
        let result: Result<Meta> = meta.try_into();
        assert_eq!(result.is_ok(), true);

        let mut meta = RawMeta::from(Meta::from_full_key("/B/hello", 1).unwrap());
        meta.states = Some("b,b".to_string());
        let result: Result<Meta> = meta.try_into();
        assert_eq!(result.err().unwrap(), NatureError::VerifyError("repeated state name: \"b\", for `Meta`: \"/B/hello\"".to_string()));

        let mut meta = RawMeta::from(Meta::from_full_key("/B/hello", 1).unwrap());
        meta.states = Some("".to_string());
        let result: Result<Meta> = meta.try_into();
        assert_eq!(result.is_ok(), true);
    }
}

