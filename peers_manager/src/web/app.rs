use super::email_validator::ValidateExecutor;
use actix::prelude::Addr;
use db::users::DbExecutor;

/// State with DbExecutor address
pub struct AppState {
    pub db: Addr<DbExecutor>,
    pub email_validator: Addr<ValidateExecutor>,
}
