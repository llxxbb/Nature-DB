use data::PlanInfo;
use diesel::prelude::*;
use nature_common::*;
use super::*;

pub struct StorePlanDaoImpl;

impl StorePlanDaoTrait for StorePlanDaoImpl {
    fn save(plan: &PlanInfo) -> Result<(), NatureError> {
        use self::schema::plan;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let will_save = RawPlanInfo::new(plan)?;
        let rtn = diesel::insert_into(plan::table)
            .values(will_save)
            .execute(conn);
        match rtn {
            Ok(x) => match x {
                1 => Ok(()),
                num => Err(NatureError::DaoLogicalError(format!("should insert 1 but get {}", num))),
            },
            Err(e) => Err(DbError::from(e)),
        }
    }

    fn get(key: &str) -> Result<Option<PlanInfo>, NatureError> {
        use super::schema::plan::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let def = match plan.filter(upstream.eq(&key))
            .load::<RawPlanInfo>(conn) {
            Ok(rows) => rows,
            Err(e) => return Err(DbError::from(e))
        };
        match def.len() {
            0 => Ok(None),
            1 => Ok(Some(def[0].to_plan_info()?)),
            x => Err(NatureError::DaoLogicalError(format!("not 1 and 0 but get {}", x))),
        }
    }
}
