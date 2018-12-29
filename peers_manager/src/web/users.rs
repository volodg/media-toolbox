use super::app::AppState;

use futures::Future;

use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, State};

use super::super::db::users::{CreateUser, LoginWithEmail, SearchWithKeyword};

#[derive(Deserialize, Serialize)]
pub struct NewUserInput {
    name: String,
    email: String,
    about: String,
}

/// Async request handler
pub fn create_user(
    (new_user, state): (Json<NewUserInput>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    // send async `CreateUser` message to a `DbExecutor`
    state
        .db
        .send(CreateUser {
            name: new_user.name.clone(),
            email: new_user.email.clone(),
            about: new_user.about.clone(),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

#[cfg(test)]
mod tests {
    extern crate mime;

    use super::*;

    #[test]
    fn test_create_user() {
        use actix::sync::SyncArbiter;
        use actix_web::test::TestServer;
        use actix_web::http;

        use db::users::DbExecutor;
        use diesel::prelude::PgConnection;
        use diesel::r2d2::ConnectionManager;

        let mut srv = TestServer::build_with_state(|| {
            // start diesel actors
            let addr = SyncArbiter::start(3, || {
                let database_url = "test.db";
                let manager = ConnectionManager::<PgConnection>::new(database_url);
                let pool = r2d2::Pool::builder()
                    .build(manager)
                    .expect("Failed to create pool.");
                DbExecutor(pool.clone())
            });
            // then we can construct custom state, or it could be `()`
            AppState { db: addr }
        })
        .start(|app| {
            app.resource("/users/create_user", |r| r.with(create_user));
        });

        let new_user = NewUserInput {
            name: String::from(""),
            email: String::from(""),
            about: String::from(""),
        };

        let request = srv.client(http::Method::POST, "/users/create_user")
             .header(http::header::CONTENT_TYPE, "application/json")
             .json(new_user)
             .unwrap();
        let response = srv.execute(request.send()).unwrap();

        assert!(response.status().is_success());
        // now we can run our test code
    }
}

pub fn login_user(
    (login, state): (Json<LoginWithEmail>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    // send async `LoginWithEmail` message to a `DbExecutor`
    state
        .db
        .send(login.into_inner())
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn user_search(
    (search, state): (Json<SearchWithKeyword>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    // send async `SearchWithKeyword` message to a `DbExecutor`
    state
        .db
        .send(search.into_inner())
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}
