use crate::PlanInfo;

use super::*;

pub struct StorePlanDaoImpl;

impl StorePlanDaoTrait for StorePlanDaoImpl {
    fn save(&self, plan: &RawPlanInfo) -> Result<()> {
        use self::schema::plan;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::insert_into(plan::table)
            .values(plan)
            .execute(conn);
        match rtn {
            Ok(x) => match x {
                1 => Ok(()),
                num => Err(NatureError::DaoLogicalError(format!(
                    "should insert 1 but get {}",
                    num
                ))),
            },
            Err(e) => Err(DbError::from(e)),
        }
    }

    fn get(&self, key: &str) -> Result<Option<PlanInfo>> {
        use super::schema::plan::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let def = match plan.filter(upstream.eq(&key)).load::<RawPlanInfo>(conn) {
            Ok(rows) => rows,
            Err(e) => return Err(DbError::from(e)),
        };
        match def.len() {
            0 => Ok(None),
            1 => Ok(Some(def[0].to_plan_info()?)),
            x => Err(NatureError::DaoLogicalError(format!(
                "not 1 and 0 but get {}",
                x
            ))),
        }
    }
}

impl StorePlanDaoImpl {
    #[allow(dead_code)]
    fn delete(from_full_pall: &str) -> Result<usize> {
        use self::schema::plan::dsl::*;
        let conn: &SqliteConnection = &CONN.lock().unwrap();
        let rtn = diesel::delete(plan.filter(upstream.eq(from_full_pall))).execute(conn);
        match rtn {
            Ok(num) => Ok(num),
            Err(err) => Err(DbError::from(err)),
        }
    }
}

#[cfg(test)]
mod test {
    extern crate log;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::env;

    use super::*;

    #[test]
    fn save_and_get() {
        env::set_var("DATABASE_URL", "nature.sqlite");
        // save it
        let tester = StorePlanDaoImpl {};
        let info = PlanInfo {
            from_thing: Thing::new_with_version_and_type("/local_converter/from", 0, ThingType::Business).unwrap(),
            from_sn: 229195495639599414319914352480091205021,
            from_sta_ver: 0,
            to: Thing::new_with_version_and_type("/local_converter/to", 0, ThingType::Business).unwrap(),
            plan: vec![Instance {
                id: 217789594388339757346716979317903552035,
                data: InstanceNoID {
                    thing: Thing::new_with_version_and_type("/local_converter/to", 0, ThingType::Business).unwrap(),
                    event_time: 0,
                    execute_time: 0,
                    create_time: 0,
                    content: "one".to_string(),
                    context: HashMap::new(),
                    status: HashSet::new(),
                    status_version: 0,
                    from: None,
                },
            }],
        };
        let plan_info = RawPlanInfo::new(&info).unwrap();
        let _ = tester.save(&plan_info);

        // save twice will get `DaoDuplicated` error
        assert_eq!(
            tester.save(&plan_info).err(),
            Some(NatureError::DaoDuplicated("".to_string()))
        );

        // get it

        let rtn = tester.get(
            "/local_converter/from:0:229195495639599414319914352480091205021:0",
        ).unwrap().unwrap();
        assert_eq!(rtn.to.get_full_key(), "/B/local_converter/to");

        // delete it
        assert_eq!(
            StorePlanDaoImpl::delete(
                "/local_converter/from:0:229195495639599414319914352480091205021:0"
            ).unwrap(),
            1
        );
    }
}
