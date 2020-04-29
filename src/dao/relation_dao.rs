use std::str::FromStr;

use diesel::prelude::*;

use crate::{get_conn, MetaCacheGetter, Relation, RelationSettings};
use crate::raw_models::RawRelation;

use super::*;

pub struct RelationDaoImpl;

pub type Relations = Result<Vec<Relation>>;
pub type RelationGetter = fn(&str, MetaCacheGetter, &MetaGetter) -> Relations;

impl RelationDaoImpl {
    pub fn get_relations(from: &str, meta_cache_getter: MetaCacheGetter, meta_getter: &MetaGetter) -> Relations {
        use self::schema::relation::dsl::*;
        let rtn = { // {} used to release conn
            relation
                .filter(from_meta.eq(from.to_string()))
                .filter(flag.eq(1))
                .load::<RawRelation>(&get_conn()?)
        };
        let def = match rtn {
            Ok(rows) => rows,
            Err(e) => return Err(DbError::from(e)),
        };
        match def.len() {
            0 => Ok(vec![]),
            x if x > 0 => {
                let mut rtn: Vec<Relation> = Vec::new();
                for d in def {
                    match Relation::from_raw(d, meta_cache_getter, meta_getter) {
                        Ok(r) => rtn.push(r),
                        Err(e) => return Err(e)
                    }
                }
                Ok(rtn)
            }
            _ => Err(NatureError::SystemError("unknown error occurred".to_string(),
            )),
        }
    }
    pub fn insert(one: RawRelation) -> Result<usize> {
        use self::schema::relation;
        let rtn = diesel::insert_into(relation::table)
            .values(&one)
            .execute(&get_conn()?);
        match rtn {
            Ok(x) => Ok(x),
            Err(e) => Err(DbError::from_with_msg(e, &format!("{:?}", &one))),
        }
    }
    pub fn delete(one: RawRelation) -> Result<usize> {
        use self::schema::relation::dsl::*;
        let rtn = diesel::delete(
            relation
                .filter(from_meta.eq(one.from_meta))
                .filter(to_meta.eq(one.to_meta)),
        )
            .execute(&get_conn()?);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }

    /// `from` and `to`'s form are full_key:version
    pub fn update_flag(from: &str, to: &str, flag_f: i32) -> Result<usize> {
        use self::schema::relation::dsl::*;
        let rtn = diesel::update(
            relation.filter(from_meta.eq(&from))
                .filter(to_meta.eq(&to)))
            .set(flag.eq(flag_f))
            .execute(&get_conn()?);
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
                executor: Some(Executor {
                    protocol: Protocol::from_str(protocol)?,
                    url: url.to_string(),
                    settings: "".to_string(),
                }),
                filter_before: vec![],
                filter_after: vec![],
                use_upstream_id: false,
                target: Default::default(),
                delay: 0,
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

    /// need db connection
    #[test]
    fn relation_test() {
        env::set_var("DATABASE_URL", CONN_STR);
        let _ = setup_logger();

        // clear before test
        debug!("--delete first-----------------");
        let _ = RelationDaoImpl::delete_by_biz("B:from:1", "B:to:1");

        // get null
        debug!("--will get none-----------------");
        let meta = "B:from:1";
        let rtn = RelationDaoImpl::get_relations(meta, MetaCacheImpl::get, MG).unwrap();
        assert_eq!(rtn.is_empty(), true);

        // insert
        debug!("--insert one-----------------");
        let _ = RelationDaoImpl::insert_by_biz("B:from:1", "B:to:1", "url", "http");
        let rtn = RelationDaoImpl::get_relations(meta, meta_cache, MG).unwrap();
        assert_eq!(rtn.len(), 1);

        // update flag
        debug!("--update it-----------------");
        let _ = RelationDaoImpl::update_flag("B:from:1", "B:to:1", 0);
        let rtn = RelationDaoImpl::get_relations(meta, meta_cache, MG).unwrap();
        assert_eq!(rtn.is_empty(), true);

        // delete after test
        debug!("--delete it after used-----------------");
        let _ = RelationDaoImpl::delete_by_biz("B:from:1", "B:to:1");
    }

    fn meta_cache(m: &str, _: &MetaGetter) -> Result<Meta> {
        Meta::from_string(m)
    }
}
