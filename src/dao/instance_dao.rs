use diesel::prelude::*;

use nature_common::*;

use crate::{CONN, CONNNECTION, DbError};
use crate::raw_models::RawInstance;

pub struct InstanceDaoImpl;

impl InstanceDaoImpl {
    pub fn insert(instance: &Instance) -> Result<usize> {
        use super::schema::instances;
        let new = RawInstance::new(instance)?;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        match diesel::insert_into(instances::table)
            .values(new)
            .execute(conn) {
            Ok(rtn) => {
                debug!("Saved instance for `Meta` {}, id : {:?}", instance.meta, instance.id);
                Ok(rtn)
            }
            Err(err) => Err(DbError::from_with_msg(err, &instance.id.to_string()))
        }
    }

    /// check whether source stored earlier
    pub fn get_by_from(f_para: &ParaForIDAndFrom) -> Result<Option<Instance>> {
        use super::schema::instances::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = instances
            .filter(instance_id.eq(u128_to_vec_u8(f_para.id))
                .and(meta.eq(&f_para.meta))
                .and(from_id.eq(u128_to_vec_u8(f_para.from_id)))
                .and(from_meta.eq(&f_para.from_meta))
                .and(from_state_version.eq(f_para.from_state_version))
                .and(from_para.eq(&f_para.from_para))
            )
            .order(state_version.desc())
            .limit(1)
            .load::<RawInstance>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].to()?)),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn get_last_state(f_para: &ParaForQueryByID) -> Result<Option<Instance>> {
        use super::schema::instances::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = instances
            .filter(instance_id.eq(u128_to_vec_u8(f_para.id))
                .and(meta.eq(&f_para.meta))
                .and(para.eq(&f_para.para))
            )
            .order(state_version.desc())
            .limit(f_para.limit as i64)
            .load::<RawInstance>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].to()?)),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }
    pub fn get_by_id(f_para: &ParaForQueryByID) -> Result<Option<Instance>> {
        use super::schema::instances::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = instances
            .filter(instance_id.eq(u128_to_vec_u8(f_para.id))
                .and(meta.eq(&f_para.meta))
                .and(state_version.ge(f_para.state_version))
                .and(para.eq(&f_para.para))
            )
            .load::<RawInstance>(conn);
        match def {
            Ok(rtn) => match rtn.len() {
                0 => Ok(None),
                1 => Ok(Some(rtn[0].to()?)),
                _ => Err(NatureError::SystemError("should less than 2 record return".to_string())),
            }
            Err(e) => Err(DbError::from(e))
        }
    }

    pub fn delete(ins: &Instance) -> Result<usize> {
        debug!("delete instance, id is : {:?}", ins.id);
        use super::schema::instances::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rows = instances
            .filter(instance_id.eq(ins.id.to_ne_bytes().to_vec()))
            .filter(meta.eq(&ins.meta))
            .filter(state_version.eq(ins.state_version));
        //        debug!("rows filter : {:?}", rows);
        match diesel::delete(rows).execute(conn) {
            Ok(rtn) => Ok(rtn),
            Err(e) => Err(DbError::from(e))
        }
    }
}
