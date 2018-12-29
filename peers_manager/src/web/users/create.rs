use super::super::app::AppState;

use futures::Future;

use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, State};

use super::super::super::db::users::{CreateUser, CreateUserError, LoginResponse};

#[derive(Deserialize, Serialize)]
pub struct NewUserInput {
    name: String,
    email: String,
    about: String,
}

enum CreateUserErrorCode {
    UserAlreadyExists,
}

#[derive(Serialize, Deserialize)]
struct CreateUserHttpError {
    code: u32,
    details: String,
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
        .and_then(|res| match res {
            Ok(user) => {
                let response = LoginResponse {
                    token: Some(user.id),
                };
                Ok(HttpResponse::Ok().json(response))
            }
            Err(error) => {
                use http::StatusCode;
                Ok(match error {
                    CreateUserError::UserAlreadyExists => {
                        let response = HttpResponse::new(StatusCode::BAD_REQUEST);
                        let mut builder = response.into_builder();

                        let error = CreateUserHttpError {
                            code: CreateUserErrorCode::UserAlreadyExists as u32,
                            details: "user already exists".to_string(),
                        };

                        builder.json(error)
                    }
                    CreateUserError::DbError(_) => HttpResponse::InternalServerError().into(),
                })
            }
        })
        .from_err()
        .responder()
}

#[cfg(test)]
mod tests_tools {
    extern crate mime;

    use super::*;
    use actix_web::client::ClientResponse;
    use actix_web::test::TestServer;
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
            let addr = SyncArbiter::start(3, || create_db_executor());
            AppState { db: addr }
        })
        .start(|app| {
            app.resource("/users/create_user", |r| r.with(create_user));
        })
    }

    pub fn db_clear_users() {
        let srv = create_db_executor();
        let conn = &srv.0.get().unwrap();
        use super::super::super::super::schema::users::dsl::*;

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
                .timeout(Duration::from_secs(2))
                .json(new_user)
                .unwrap();

            self.execute(request.send()).unwrap()
        }
    }
}

#[cfg(test)]
mod create_user_tests {

    use super::*;
    use actix_web::HttpMessage;

    #[test]
    fn test_create_user() {
        use super::tests_tools::*;

        db_clear_users();

        let mut srv = create_test_server();

        let email = "test@gmail.com";

        let new_user = NewUserInput {
            name: "name 1".to_string(),
            email: email.to_string(),
            about: "about 1".to_string(),
        };

        let response = srv.create_user(new_user);
        let bytes = srv.execute(response.body()).unwrap();
        let token_data: LoginResponse = serde_json::from_slice(&bytes).unwrap();
        let token = token_data.token.unwrap();

        assert!(token > 0);
        assert!(response.status().is_success());

        let new_user = NewUserInput {
            name: "name 2".to_string(),
            email: email.to_string(),
            about: "about 2".to_string(),
        };

        let response = srv.create_user(new_user);
        let bytes = srv.execute(response.body()).unwrap();
        let error_data: CreateUserHttpError = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            error_data.code,
            CreateUserErrorCode::UserAlreadyExists as u32
        );

        assert!(response.status().is_client_error());
    }
}
