use actix::prelude::Addr;
use db::users::DbExecutor;

/// State with DbExecutor address
pub struct AppState {
    pub db: Addr<DbExecutor>,
}
