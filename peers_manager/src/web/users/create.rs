use super::super::app::AppState;

use futures::Future;

use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, State};
use http::StatusCode;

use super::super::super::db::users::{CreateUser, CreateUserError, LoginResponse};

#[derive(Deserialize, Serialize)]
pub struct NewUserInput {
    pub name: String,
    pub email: String,
    pub about: String,
}

pub enum CreateUserErrorCode {
    UserAlreadyExists,
    InvalidEmail,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserHttpError {
    pub code: u32,
    details: String,
}

use super::super::super::web::email_validator::{ValidateEmail, ValidateExecutor};
use db::users::DbExecutor;

pub fn create_user(
    (new_user, state): (Json<NewUserInput>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    let db = state.db.clone();

    Box::new(
        validate_email_request(state.email_validator.clone(), &new_user.email).and_then(
            move |email_is_valid| {
                if email_is_valid {
                    return db_create_user(db, new_user);
                }
                let response = HttpResponse::new(StatusCode::BAD_REQUEST);
                let mut builder = response.into_builder();

                let error = CreateUserHttpError {
                    code: CreateUserErrorCode::InvalidEmail as u32,
                    details: "email is not valid".to_string(),
                };

                Box::new(futures::future::ok(builder.json(error)))
            },
        ),
    )
}

fn validate_email_request(
    validator: actix::Addr<ValidateExecutor>,
    email: &str,
) -> impl Future<Item = bool, Error = actix_web::error::Error> {
    let validate_email = ValidateEmail {
        email: email.to_string(),
    };
    validator.send(validate_email).from_err()
}

/// Async request handler
fn db_create_user(
    db: actix::Addr<DbExecutor>,
    new_user: Json<NewUserInput>,
) -> FutureResponse<HttpResponse> {
    // send async `CreateUser` message to a `DbExecutor`
    db.send(CreateUser {
        name: new_user.name.clone(),
        email: new_user.email.clone(),
        about: new_user.about.clone(),
    })
    .and_then(|res| match res {
        Ok(user) => {
            let response = LoginResponse { token: user.id };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(error) => Ok(match error {
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
        }),
    })
    .from_err()
    .responder()
}

#[cfg(test)]
mod create_user_tests {

    use super::super::tests_tools::*;
    use super::*;
    use actix_web::HttpMessage;

    #[test]
    fn test_create_user() {
        db_clear_users();

        let mut srv = create_test_server();

        let email = "test_create@gmail.com";

        let new_user = NewUserInput {
            name: "name 1".to_string(),
            email: email.to_string(),
            about: "about 1".to_string(),
        };
        srv.test_create_new_user(new_user);

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

    #[test]
    fn test_invalid_email() {
        db_clear_users();

        let mut srv = create_test_server();

        let new_user = NewUserInput {
            name: "name 1".to_string(),
            email: "email 1".to_string(),
            about: "about 1".to_string(),
        };

        let response = srv.create_user(new_user);
        let bytes = srv.execute(response.body()).unwrap();
        let error_data: CreateUserHttpError = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(error_data.code, CreateUserErrorCode::InvalidEmail as u32);

        assert!(response.status().is_client_error());
    }
}
