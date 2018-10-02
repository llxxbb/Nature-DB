use chrono::prelude::*;


/// `Thing`'s extended information
/// `DateTime` is not implement `Default` trait
#[derive(Serialize, Deserialize, Debug, Queryable, Clone, PartialOrd, PartialEq)]
pub struct ThingDefine {
    pub key: String,

    /// For human readable what the `Thing` is.
    pub description: Option<String>,

    /// version of the `Thing`
    pub version: i32,

    pub states: Option<String>,

    /// Define whats the `Thing` should include
    pub fields: Option<String>,

    pub create_time: NaiveDateTime,
}


impl Default for ThingDefine {
    fn default() -> Self {
        ThingDefine {
            key: String::new(),
            description: None,
            version: 0,
            states: None,
            fields: None,
            create_time: Local::now().naive_local(),
        }
    }
}

impl ThingDefine {
    pub fn is_status(&self) -> bool {
        !self.states.is_none()
    }
}

#[cfg(test)]
mod test {
    use nature_common::*;

    #[test]
    fn standardize_empty() {
        println!("----------------- standardize_empty --------------------");
        let mut key = String::new();
        let rtn = Thing::key_standardize(&mut key);
        if let Err(NatureError::VerifyError(x)) = rtn {
            assert_eq!(x, "key length can't be zero");
        } else {
            panic!("should get error")
        }

        let mut key = "/".to_string();
        let rtn = Thing::key_standardize(&mut key);
        if let Err(NatureError::VerifyError(x)) = rtn {
            assert_eq!(x, "key length can't be zero");
        } else {
            panic!("should get error")
        }
    }

    /// also test for removing last separator and Business prefix
    #[test]
    fn standardize_no_separator_at_beginning() {
        println!("----------------- standardize_no_separator_at_beginning --------------------");
        let mut key = "a/b/c/".to_string();
        let _rtn = Thing::key_standardize(&mut key);
        assert_eq!(key, "/a/b/c");
    }
}