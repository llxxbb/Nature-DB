use std::str::FromStr;

use mysql_async::Value;

use nature_common::Executor;

use crate::{MetaCache, MetaDao, Relation, RelationSettings};
use crate::raw_models::RawRelation;

use super::*;

pub type Relations = Result<Vec<Relation>>;

lazy_static! {
    pub static ref D_R: RelationDaoImpl = RelationDaoImpl {};
}

#[async_trait]
pub trait RelationDao: Sync + Send {
    async fn get_relations<MC, M>(&self, from: &str, meta_cache_getter: &MC, meta_getter: &M) -> Relations
        where MC: MetaCache, M: MetaDao;
    async fn insert(&self, one: RawRelation) -> Result<usize>;
    async fn delete(&self, one: RawRelation) -> Result<usize>;
    async fn update_flag(&self, from: &str, to: &str, flag_f: i32) -> Result<usize>;
    async fn insert_by_biz(&self, from: &str, to: &str, url: &str, protocol: &str) -> Result<RawRelation>;
    async fn delete_by_biz(&self, from: &str, to: &str) -> Result<usize>;
}

pub struct RelationDaoImpl;

#[async_trait]
impl RelationDao for RelationDaoImpl {
    async fn get_relations<MC, M>(&self, from: &str, meta_cache_getter: &MC, meta_getter: &M) -> Relations
        where MC: MetaCache, M: MetaDao {
        let sql = r"SELECT from_meta, to_meta, settings, flag
            FROM nature.relation
            where from_meta = :from_meta and flag = 1";

        let p = params! {
            "from_meta" => from,
        };

        let raws = MySql::fetch(sql, p, RawRelation::from).await?;
        match raws.len() {
            0 => Ok(vec![]),
            x if x > 0 => {
                let mut rtn: Vec<Relation> = Vec::new();
                for d in raws {
                    match Relation::from_raw(d, meta_cache_getter, meta_getter).await {
                        Ok(r) => rtn.push(r),
                        Err(e) => return Err(e)
                    }
                }
                Ok(rtn)
            }
            _ => Err(NatureError::SystemError("unknown error occurred".to_string(),
            ))
        }
    }
    async fn insert(&self, one: RawRelation) -> Result<usize> {
        let sql = r"INSERT INTO nature.relation
            (from_meta, to_meta, settings, flag)
            VALUES(:from_meta, :to_meta, :settings, :flag)";

        let p: Vec<(String, Value)> = one.clone().into();
        let rtn: usize = MySql::idu(sql, p).await?;
        debug!("Saved relation : {} -> {}", one.from_meta, one.to_meta);
        Ok(rtn)
    }
    async fn delete(&self, one: RawRelation) -> Result<usize> {
        let sql = r"DELETE FROM nature.relation
            WHERE from_meta=:from_meta AND to_meta=:to_meta";

        let p = params! {
            "from_meta" => one.from_meta.to_string(),
            "to_meta" => one.to_meta.to_string(),
        };

        let rtn: usize = MySql::idu(sql, p).await?;
        debug!("relation deleted : {} -> {}", one.from_meta, one.to_meta);
        Ok(rtn)
    }

    /// `from` and `to`'s form are full_key:version
    async fn update_flag(&self, from: &str, to: &str, flag_f: i32) -> Result<usize> {
        let sql = r"UPDATE nature.relation
            SET settings='', flag=:flag
            WHERE from_meta=:from_meta AND to_meta=:to_meta";

        let p = params! {
            "from_meta" => from,
            "to_meta" => to,
            "flag" => flag_f,
        };

        let rtn = MySql::idu(sql, p).await?;
        debug!("relation flag updated: : {} -> {}", from, to);
        Ok(rtn)
    }

    /// `version` will be set to 0
    async fn insert_by_biz(&self, from: &str, to: &str, url: &str, protocol: &str) -> Result<RawRelation> {
        let one = RawRelation::new(
            from,
            to,
            &RelationSettings {
                selector: None,
                executor: Some(Executor {
                    protocol: nature_common::Protocol::from_str(protocol)?,
                    url: url.to_string(),
                    settings: "".to_string(),
                }),
                filter_before: vec![],
                filter_after: vec![],
                use_upstream_id: false,
                target: Default::default(),
                delay: 0,
                delay_on_para: (0, 0),
                id_bridge: false,
            },
        )?;
        let _ = D_R.insert(one.clone()).await;
        Ok(one)
    }

    async fn delete_by_biz(&self, from: &str, to: &str) -> Result<usize> {
        let row = RawRelation {
            from_meta: from.to_string(),
            to_meta: to.to_string(),
            settings: String::new(),
            flag: 1,
        };
        D_R.delete(row).await
    }
}

#[cfg(test)]
mod test {
    extern crate log;

    use std::env;

    use tokio::runtime::Runtime;

    use nature_common::{Meta, setup_logger};

    use crate::{C_M, CONN_STR};

    use super::*;

    /// need db connection
    #[test]
    #[ignore]
    fn relation_test() {
        env::set_var("DATABASE_URL", CONN_STR);
        let _ = setup_logger();

        let mut runtime = Runtime::new().unwrap();

        // clear before test
        debug!("--delete first-----------------");
        let _ = runtime.block_on(D_R.delete_by_biz("B:from:1", "B:to:1"));

        // get null
        debug!("--will get none-----------------");
        let meta = "B:from:1";
        let rtn = runtime.block_on(D_R.get_relations(meta, &*C_M, &*D_M)).unwrap();
        assert_eq!(rtn.is_empty(), true);

        // insert
        debug!("--insert one-----------------");
        let _ = runtime.block_on(D_R.insert_by_biz("B:from:1", "B:to:1", "url", "http"));
        let rtn = runtime.block_on(D_R.get_relations(meta, &MCMock {}, &*D_M)).unwrap();
        assert_eq!(rtn.len(), 1);

        // update flag
        debug!("--update it-----------------");
        let _ = runtime.block_on(D_R.update_flag("B:from:1", "B:to:1", 0));
        let rtn = runtime.block_on(D_R.get_relations(meta, &MCMock {}, &*D_M)).unwrap();
        assert_eq!(rtn.is_empty(), true);

        // delete after test
        debug!("--delete it after used-----------------");
        let _ = runtime.block_on(D_R.delete_by_biz("B:from:1", "B:to:1"));
    }

    #[derive(Copy, Clone)]
    struct MCMock;

    #[async_trait]
    impl MetaCache for MCMock {
        async fn get<M>(&self, meta_str: &str, _getter: &M) -> Result<Meta> where M: MetaDao {
            Meta::from_string(meta_str)
        }
    }
}
