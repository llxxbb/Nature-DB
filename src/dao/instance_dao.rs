use std::str::FromStr;

use diesel::prelude::*;

use nature_common::*;

use crate::{DbError, get_conn, Mission};
use crate::raw_models::RawInstance;

pub struct InstanceDaoImpl;

pub type InstanceParaGetter = fn(&QueryByID) -> Result<Option<Instance>>;
pub type InstanceKeyGetter = fn(&str, &str) -> Result<Option<Instance>>;

pub static INS_PARA_GETTER: InstanceParaGetter = InstanceDaoImpl::get_by_id;
pub static INS_KEY_GETTER: InstanceKeyGetter = InstanceDaoImpl::get_by_key;

impl InstanceDaoImpl {
    pub fn insert(instance: &Instance) -> Result<usize> {
        use super::schema::instances;
        let new = RawInstance::new(instance)?;
        match diesel::insert_into(instances::table)
            .values(new)
            .execute(&get_conn()?) {
            Ok(rtn) => {
                debug!("Saved instance : {}", instance.get_key());
                Ok(rtn)
            }
            Err(err) => Err(DbError::from_with_msg(err, &instance.id.to_string()))
        }
    }

    /// check whether source stored earlier
    pub fn get_by_from(f_para: &ParaForIDAndFrom) -> Result<Option<Instance>> {
        use super::schema::instances::dsl::*;
        let def = instances
            .filter(ins_key.like(&f_para.para_like())
                .and(from_key.eq(&f_para.from_key))
            )
            .order(state_version.desc())
            .limit(1)
            .load::<RawInstance>(&get_conn()?);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].to()?)),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    fn get_last_state(f_para: &QueryByID) -> Result<Option<Instance>> {
        use super::schema::instances::dsl::*;
        let def = instances
            .filter(ins_key.like(&f_para.para_like()))
            .order(state_version.desc())
            .limit(f_para.limit as i64)
            .load::<RawInstance>(&get_conn()?);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].to()?)),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn get_by_key(key: &str, spliter: &str) -> Result<Option<Instance>> {
        let temp: Vec<&str> = key.split(spliter).collect();
        if temp.len() != 4 {
            return Err(NatureError::VerifyError("error key format for task".to_string()));
        }
        let para = QueryByID {
            id: temp[1].to_string(),
            meta: temp[0].to_string(),
            para: temp[2].to_string(),
            state_version: i32::from_str(temp[3])?,
            limit: 1,
        };
        Self::get_by_id(&para)
    }

    pub fn get_by_id(f_para: &QueryByID) -> Result<Option<Instance>> {
        use super::schema::instances::dsl::*;
        let def = instances
            .filter(ins_key.eq(f_para.get_key()))
            .filter(state_version.eq(f_para.state_version))
            .load::<RawInstance>(&get_conn()?);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].to()?)),
                _ => {
                    Err(NatureError::SystemError("should less than 2 record return".to_string()))
                }
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn delete(ins: &Instance) -> Result<usize> {
        debug!("delete instance, id is : {:?}", ins.id);
        use super::schema::instances::dsl::*;
        let rows = instances.filter(ins_key.eq(ins.get_key()));
        match diesel::delete(rows).execute(&get_conn()?) {
            Ok(rtn) => Ok(rtn),
            Err(e) => Err(DbError::from(e))
        }
    }

    /// get downstream instance through upstream instance
    pub fn get_last_taget(from: &Instance, mission: &Mission) -> Result<Option<Instance>> {
        if !mission.to.is_state() {
            return Ok(None);
        }
        let para_part = &mission.target_demand.upstream_para;
        let para_id = get_para_and_key_from_para(&from.para, para_part)?.0;
        let mut id: u128 = match from.sys_context.get(&*CONTEXT_TARGET_INSTANCE_ID) {
            // context have target id
            Some(state_id) => u128::from_str(state_id)?,
            None => 0,
        };
        if id == 0 {
            if mission.use_upstream_id {
                id = from.id
            } else if mission.to.check_master(&from.meta) {
                id = from.id
            }
        }
        let meta = mission.to.meta_string();
        debug!("get last state for meta {}", &meta);
        let qc = QueryByID::new(id, &meta, &para_id, 0);
        Self::get_last_state(&qc)
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[test]
    #[allow(dead_code)]
    fn query_by_id() {
        env::set_var("DATABASE_URL", "mysql://root@localhost/nature");
        let para = QueryByID {
            id: "3827f37003127855b32ea022daa04cd".to_string(),
            meta: "B:sale/order:1".to_string(),
            para: "".to_string(),
            state_version: 0,
            limit: 1,
        };
        let result = InstanceDaoImpl::get_by_id(&para);
        let _ = dbg!(result);
    }
}