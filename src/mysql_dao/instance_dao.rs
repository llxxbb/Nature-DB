use std::str::FromStr;

use chrono::{Local, TimeZone};
use mysql_async::Value;

use nature_common::*;

use crate::{Mission, QUERY_SIZE_LIMIT};
use crate::mysql_dao::MySql;
use crate::raw_models::RawInstance;

pub struct InstanceDaoImpl;

impl InstanceDaoImpl {
    pub async fn insert(instance: &Instance) -> Result<usize> {
        let new = RawInstance::new(instance)?;
        let sql = r"INSERT INTO instances
            (ins_key, content, context, states, state_version, create_time, sys_context, from_key)
            VALUES(:ins_key, :content,:context,:states,:state_version,:create_time,:sys_context,:from_key)";
        let vec: Vec<(String, Value)> = new.into();
        let rtn: usize = match MySql::idu(sql, vec).await {
            Ok(n) => n,
            Err(e) => {
                return Err(e);
            }
        };
        debug!("Saved instance : {}", instance.get_key());
        Ok(rtn)
    }

    //noinspection RsLiveness
    /// check whether source stored earlier
    pub async fn get_by_from(f_para: &IDAndFrom) -> Result<Option<Instance>> {
        let sql = r"SELECT ins_key, content, context, states, state_version, create_time, sys_context, from_key
            FROM instances
            where ins_key like :para_like and from_key = :from_key
            order by state_version desc
            limit 1";
        let p = params! {
            "para_like" => f_para.para_like().to_string(),
            "from_key" => f_para.from_key.to_string(),
        };

        let rtn = MySql::fetch(sql, p, RawInstance::from).await?;
        match rtn.len() {
            1 => Ok(Some(rtn[0].to()?)),
            0 => Ok(None),
            _ => Err(NatureError::LogicalError("should not return more than one rows".to_string()))
        }
    }

    //noinspection RsLiveness
    async fn get_last_state(f_para: &KeyCondition) -> Result<Option<Instance>> {
        let sql = r"SELECT ins_key, content, context, states, state_version, create_time, sys_context, from_key
            FROM instances
            where ins_key = :ins_key
            order by state_version desc
            limit 1";
        let p = params! {
            "ins_key" => f_para.get_key(),
        };
        let rtn = MySql::fetch(sql, p, RawInstance::from).await?;
        match rtn.len() {
            1 => Ok(Some(rtn[0].to()?)),
            0 => Ok(None),
            _ => Err(NatureError::LogicalError("should not return more than one rows".to_string()))
        }
    }

    pub async fn get_by_key(key: String, spliter: String) -> Result<Option<Instance>> {
        let temp: Vec<&str> = key.split(&spliter).collect();
        if temp.len() != 4 {
            return Err(NatureError::VerifyError("error key format for task".to_string()));
        }
        let para = KeyCondition {
            id: temp[1].to_string(),
            meta: temp[0].to_string(),
            key_gt: "".to_string(),
            para: temp[2].to_string(),
            state_version: i32::from_str(temp[3])?,
            time_ge: None,
            time_lt: None,
            limit: 1,
        };
        Self::get_by_id(para).await
    }

    //noinspection RsLiveness
    pub async fn get_by_id(f_para: KeyCondition) -> Result<Option<Instance>> {
        let sql = r"SELECT ins_key, content, context, states, state_version, create_time, sys_context, from_key
            FROM instances
            where ins_key = :ins_key and state_version = :state_version
            order by state_version desc
            limit 1";
        let p = params! {
            "ins_key" => f_para.get_key().to_string(),
            "state_version" => f_para.state_version,
        };
        let rtn = MySql::fetch(sql, p, RawInstance::from).await?;
        match rtn.len() {
            1 => Ok(Some(rtn[0].to()?)),
            0 => Ok(None),
            _ => Err(NatureError::LogicalError("should not return more than one rows".to_string()))
        }
    }

    pub async fn get_by_meta(f_para: &KeyCondition) -> Result<Vec<Instance>> {
        let key_gt = if f_para.key_gt.eq("") { "".to_string() } else {
            format!(" and ins_key > '{}'", f_para.key_gt)
        };
        let time_ge = match f_para.time_ge {
            Some(ge) => format!(" and create_time >= '{}'", Local.timestamp_millis(ge)),
            None => "".to_string()
        };
        let time_lt = match f_para.time_lt {
            Some(lt) => format!(" and create_time < '{}'", Local.timestamp_millis(lt)),
            None => "".to_string()
        };
        let limit = if f_para.limit < *QUERY_SIZE_LIMIT {
            f_para.limit
        } else { *QUERY_SIZE_LIMIT };
        let sql = format!("SELECT ins_key, content, context, states, state_version, create_time, sys_context, from_key
            FROM instances
            where ins_key like :id_like{}{}{}
            order by ins_key
            limit {}", key_gt, time_ge, time_lt, limit);
        let p = params! {
            "id_like" => f_para.id_like().to_string(),
        };
        let result = MySql::fetch(sql, p, RawInstance::from).await?;
        let mut rtn: Vec<Instance> = vec![];
        for one in result {
            rtn.push(one.to()?)
        }
        Ok(rtn)
    }

    pub async fn delete(ins: &Instance) -> Result<usize> {
        let sql = r"DELETE FROM instances
            WHERE ins_key=:ins_key";
        let p = params! {
            "ins_key" => ins.get_key(),
        };
        let rtn: usize = MySql::idu(sql, p).await?;
        debug!("instance deleted, id is : {:?}", ins.id);
        Ok(rtn)
    }

    /// get downstream instance through upstream instance
    pub async fn get_last_taget(from: &Instance, mission: &mut Mission) -> Result<Option<Instance>> {
        if !mission.to.is_state() {
            return Ok(None);
        }
        let para_part = &mission.target_demand.copy_para;
        let para_id = if para_part.len() > 0 {
            let id = get_para_and_key_from_para(&from.para, para_part)?.0;
            mission.sys_context.insert(CONTEXT_TARGET_INSTANCE_PARA.to_string(), id.to_string());
            id
        } else {
            "".to_string()
        };
        let mut id: u128 = match mission.sys_context.get(&*CONTEXT_TARGET_INSTANCE_ID) {
            // context have target id
            Some(state_id) => u128::from_str_radix(state_id, 16)?,
            None => 0,
        };
        if id == 0 {
            if mission.use_upstream_id || mission.to.check_master(&from.meta) {
                mission.sys_context.insert(CONTEXT_TARGET_INSTANCE_ID.to_string(), format!("{:x}", from.id));
                id = from.id
            }
        }
        let meta = mission.to.meta_string();
        debug!("get last state for meta {}", &meta);
        let qc = KeyCondition::new(id, &meta, &para_id, 0);
        Self::get_last_state(&qc).await
    }
}


#[cfg(test)]
mod test {
    use std::env;

    use tokio::runtime::Runtime;

    use super::*;

    #[test]
    #[allow(dead_code)]
    fn get_last_state_test() {
        env::set_var("DATABASE_URL", "mysql://root@localhost/nature");
        let para = KeyCondition::new(0, "B:score/trainee/all-subject:1", "002", 0);
        let result = Runtime::new().unwrap().block_on(InstanceDaoImpl::get_last_state(&para));
        let _ = dbg!(result);
    }

    #[test]
    #[allow(dead_code)]
    fn query_by_id() {
        env::set_var("DATABASE_URL", "mysql://root@localhost/nature");
        let para = KeyCondition {
            id: "3827f37003127855b32ea022daa04cd".to_string(),
            meta: "B:sale/order:1".to_string(),
            key_gt: "".to_string(),
            para: "".to_string(),
            state_version: 0,
            time_ge: None,
            time_lt: None,
            limit: 1,
        };
        let result = Runtime::new().unwrap().block_on(InstanceDaoImpl::get_by_id(para));
        let _ = dbg!(result);
    }

    #[test]
    #[allow(dead_code)]
    fn query_by_meta() {
        let ge_t = 1588508143000;
        let ge = Local.timestamp_millis(ge_t);
        dbg!(ge);
        env::set_var("DATABASE_URL", "mysql://root@localhost/nature");
        let para = KeyCondition {
            id: "".to_string(),
            meta: "B:sale/order:1".to_string(),
            key_gt: "".to_string(),
            para: "".to_string(),
            state_version: 0,
            time_ge: Some(ge_t),
            time_lt: Some(1588508144000),
            limit: 100,
        };
        let result = Runtime::new().unwrap().block_on(InstanceDaoImpl::get_by_meta(&para));
        let _ = dbg!(result);
    }
}