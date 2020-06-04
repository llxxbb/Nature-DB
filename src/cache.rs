use tokio::runtime::Runtime;

use nature_common::{NatureError, Result};

pub use self::meta_cache::*;
pub use self::relation_cache::*;

lazy_static! {
    pub static ref RUNTIME: Result<Runtime> = get_runtime();
}

fn get_runtime() -> Result<Runtime> {
    match Runtime::new() {
        Ok(r) => Ok(r),
        Err(e) => {
            let msg = format!("get tokio runtime error : {}", e.to_string());
            warn!("{}", msg);
            return Err(NatureError::LogicalError(msg));
        }
    }
}

mod meta_cache;
mod relation_cache;
