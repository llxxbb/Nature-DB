use std::str::FromStr;

use diesel::prelude::*;

use crate::{CONN, CONNNECTION, MetaCacheGetter, Relation, RelationSettings};
use crate::raw_models::RawRelation;

use super::*;

pub struct RelationDaoImpl;

pub type RelationResult = Result<Option<Vec<Relation>>>;
pub type RelationGetter = fn(&str, MetaCacheGetter, MetaGetter) -> RelationResult;

impl RelationDaoImpl {
    pub fn get_relations(from: &str, meta_cache_getter: MetaCacheGetter, meta_getter: MetaGetter) -> RelationResult {
        use self::schema::relation::dsl::*;
        let rtn = { // {} used to release conn
            let conn: &CONNNECTION = &CONN.lock().unwrap();
            relation
                .filter(from_meta.eq(from.to_string()))
                .filter(flag.eq(1))
                .load::<RawRelation>(conn)
        };
        let def = match rtn {
            Ok(rows) => rows,
            Err(e) => return Err(DbError::from(e)),
        };
        match def.len() {
            0 => Ok(None),
            x if x > 0 => {
                let mut rtn: Vec<Relation> = Vec::new();
                for d in def {
                    match Relation::from_raw(d, meta_cache_getter, meta_getter) {
                        Ok(multi) => {
                            multi.into_iter().for_each(|e| rtn.push(e))
                        }
                        Err(e) => {
                            warn!("raw to `one_step_flow` occur error : {:?}", e);
                        }
                    }
                }
                if rtn.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(rtn))
                }
            }
            _ => Err(NatureError::SystemError(
                "unknown error occurred".to_string(),
            )),
        }
    }
    pub fn insert(one: RawRelation) -> Result<usize> {
        use self::schema::relation;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(relation::table)
            .values(&one)
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from_with_msg(e, &format!("{:?}", &one))),
        }
    }
    pub fn delete(one: RawRelation) -> Result<usize> {
        use self::schema::relation::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(
            relation
                .filter(from_meta.eq(one.from_meta))
                .filter(to_meta.eq(one.to_meta)),
        )
            .execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    /// `from` and `to`'s form are full_key:version
    pub fn update_flag(from: &str, to: &str, flag_f: i32) -> Result<usize> {
        use self::schema::relation::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::update(
            relation.filter(from_meta.eq(&from))
                .filter(to_meta.eq(&to)))
            .set(flag.eq(flag_f))
            .execute(conn);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from(e))
        }
    }

    /// `version` will be set to 0
    pub fn insert_by_biz(from: &str, to: &str, url: &str, protocol: &str) -> Result<RawRelation> {
        let one = RawRelation::new(
            from,
            to,
            &RelationSettings {
                selector: None,
                executor: Some(vec![Executor {
                    protocol: Protocol::from_str(protocol)?,
                    url: url.to_string(),
                    group: "".to_string(),
                    proportion: 1,
                }]),
                use_upstream_id: false,
                target_states: None,
            },
        )?;
        let _ = RelationDaoImpl::insert(one.clone());
        Ok(one)
    }

    pub fn delete_by_biz(from: &str, to: &str) -> Result<usize> {
        let row = RawRelation {
            from_meta: from.to_string(),
            to_meta: to.to_string(),
            settings: String::new(),
            flag: 1,
        };
        RelationDaoImpl::delete(row)
    }
}

#[cfg(test)]
mod test {
    extern crate log;

    use std::env;

    use crate::{CONN_STR, MetaCacheImpl};

    use super::*;

    #[test]
    fn relation_test() {
        env::set_var("DATABASE_URL", CONN_STR);
        let _ = setup_logger();

        // clear before test
        let _ = RelationDaoImpl::delete_by_biz("/B/from:1", "/B/to:1");

        // get null
        let meta = "/B/from:1";
        let rtn = RelationDaoImpl::get_relations(meta, MetaCacheImpl::get, MetaDaoImpl::get).unwrap();
        assert_eq!(rtn, None);

        // insert
        let _ = RelationDaoImpl::insert_by_biz("/B/from:1", "/B/to:1", "url", "http");
        let rtn = RelationDaoImpl::get_relations(meta, meta_cache, MetaDaoImpl::get).unwrap();
        assert_eq!(rtn.unwrap().len(), 1);

        // update flag
        let _ = RelationDaoImpl::update_flag("/B/from:1", "/B/to:1", 0);
        let rtn = RelationDaoImpl::get_relations(meta, meta_cache, MetaDaoImpl::get).unwrap();
        assert_eq!(rtn, None);

        // delete after test
        let _ = RelationDaoImpl::delete_by_biz("/B/from:1", "/B/to:1");
    }

    fn meta_cache(m: &str, _: MetaGetter) -> Result<Meta> {
        Meta::from_string(m)
    }
}
