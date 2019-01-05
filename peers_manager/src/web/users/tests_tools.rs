
extern crate mime;

use super::super::app::AppState;
use db::users::DbExecutor;
use actix_web::client::ClientResponse;
use actix_web::test::TestServer;
use diesel::prelude::*;
use super::create::{NewUserInput, create_user};

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
        let addr1 = SyncArbiter::start(3, || create_db_executor());
        let addr2 = SyncArbiter::start(3, || {
            super::super::email_validator::ValidateExecutor(None)
        });
        AppState {
            db: addr1,
            email_validator: addr2,
        }
    })
    .start(|app| {
        app.resource("/users/create_user", |r| r.with(create_user));
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
}
