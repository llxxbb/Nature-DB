use diesel::prelude::*;

use crate::{CONN, CONNNECTION};
use crate::raw_models::RawPlanInfo;

use super::*;

pub struct StorePlanDaoImpl;

impl StorePlanDaoImpl {
    pub fn save(plan: &RawPlanInfo) -> Result<()> {
        use self::schema::plan;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(plan::table)
            .values(plan)
            .execute(conn);
        match rtn {
            Ok(x) => match x {
                1 => Ok(()),
                num => Err(NatureError::SystemError(format!(
                    "should insert 1 but get {}",
                    num
                ))),
            },
            Err(e) => Err(DbError::from_with_msg(e, &format!("upstream : {}, downstream : {}", plan.upstream, plan.downstream))),
        }
    }

    pub fn get(key: &str) -> Result<Option<RawPlanInfo>> {
        use super::schema::plan::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let def = match plan.filter(upstream.eq(&key)).load::<RawPlanInfo>(conn) {
            Ok(rows) => rows,
            Err(e) => return Err(DbError::from(e)),
        };
        match def.len() {
            0 => Ok(None),
            1 => Ok(Some(def[0].clone())),
            x => Err(NatureError::SystemError(format!(
                "not 1 or 0 but get {}",
                x
            ))),
        }
    }
}

impl StorePlanDaoImpl {
    pub fn delete(from: &str, to: &str) -> Result<usize> {
        use self::schema::plan::dsl::*;
        let conn: &CONNNECTION = &CONN.lock().unwrap();
        let rtn = diesel::delete(plan.filter(
            upstream.eq(from)
                .and(downstream.eq(to))
        )).execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }
}
