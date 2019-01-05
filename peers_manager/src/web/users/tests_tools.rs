extern crate mime;

use actix_web::client::ClientResponse;
use actix_web::test::TestServer;
use actix_web::HttpMessage;

use super::super::super::db::users::LoginResponse;
use super::super::app::AppState;
use super::create::{create_user, NewUserInput};
use super::login::login_user;
use db::users::DbExecutor;
use diesel::prelude::*;

fn create_db_executor() -> DbExecutor {
    use diesel::prelude::PgConnection;
    use diesel::r2d2::ConnectionManager;

    let database_url = "postgres://postgres:docker@localhost:5432/peers_test";
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    DbExecutor(pool.clone())
}

pub fn create_test_server() -> TestServer {
    use actix::sync::SyncArbiter;

    TestServer::build_with_state(|| {
        let addr1 = SyncArbiter::start(1, || create_db_executor());
        let addr2 = SyncArbiter::start(1, || super::super::email_validator::ValidateExecutor(None));
        AppState {
            db: addr1,
            email_validator: addr2,
        }
    })
    .start(|app| {
        app.resource("/users/create_user", |r| r.with(create_user))
            .resource("/users/login", |r| r.with(login_user));
    })
}

pub fn db_clear_users() {
    let srv = create_db_executor();
    let conn = &srv.0.get().unwrap();
    use super::super::super::schema::users::dsl::*;

    let _ = diesel::delete(users).execute(conn);
}

pub trait UsersWebMethods {
    fn create_user(&mut self, new_user: NewUserInput) -> ClientResponse;
    fn test_create_new_user(&mut self, new_user: NewUserInput) -> i64;
}

impl UsersWebMethods for TestServer {
    fn create_user(&mut self, new_user: NewUserInput) -> ClientResponse {
        use actix_web::http;
        use std::time::Duration;

        let request = self
            .client(http::Method::POST, "/users/create_user")
            .header(http::header::CONTENT_TYPE, "application/json")
            .timeout(Duration::from_secs(10))
            .json(new_user)
            .unwrap();

        self.execute(request.send()).unwrap()
    }

    fn test_create_new_user(&mut self, new_user: NewUserInput) -> i64 {
        let response = self.create_user(new_user);
        let bytes = self.execute(response.body()).unwrap();
        let token_data: LoginResponse = serde_json::from_slice(&bytes).unwrap();
        let token = token_data.token;

        assert!(token > 0);
        assert!(response.status().is_success());

        token
    }
}
