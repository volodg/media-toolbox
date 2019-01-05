use super::super::app::AppState;

use futures::Future;
use http::StatusCode;

use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, State};

use super::super::super::db::users::{LoginWithEmail, LoginError};

pub enum LoginErrorCode {
    InvalidCredentials,
}

#[derive(Serialize, Deserialize)]
pub struct LoginHttpError {
    pub code: u32,
    details: String,
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
            Ok(token) => Ok(HttpResponse::Ok().json(token)),
            Err(error) => Ok(match error {
                LoginError::InvalidCredentials => {
                    let response = HttpResponse::new(StatusCode::BAD_REQUEST);
                    let mut builder = response.into_builder();

                    let error = LoginHttpError {
                        code: LoginErrorCode::InvalidCredentials as u32,
                        details: "invalid credentials".to_string(),
                    };

                    builder.json(error)
                }
                LoginError::DbError(_) => HttpResponse::InternalServerError().into(),
            }),
        })
        .responder()
}

#[cfg(test)]
mod create_user_tests {

    use super::*;
    use super::super::create::*;
    use super::super::tests_tools::*;
    use actix_web::client::ClientResponse;
    use actix_web::test::TestServer;

    fn login_user(srv: &mut TestServer, email: &str) -> ClientResponse {
        use super::super::super::super::db::users::LoginWithEmail;
        use actix_web::http;
        use std::time::Duration;

        let login = LoginWithEmail {
            email: email.to_string(),
        };

        let request = srv
            .client(http::Method::POST, "/users/login")
            .header(http::header::CONTENT_TYPE, "application/json")
            .timeout(Duration::from_secs(10))
            .json(login)
            .unwrap();

        srv.execute(request.send()).unwrap()
    }

    #[test]
    fn test_succees_login() {
        use super::super::super::super::db::users::LoginResponse;
        use actix_web::HttpMessage;

        db_clear_users();

        let mut srv = create_test_server();

        let email = "test_login_1@gmail.com";

        let new_user = NewUserInput {
            name: "name 1".to_string(),
            email: email.to_string(),
            about: "about 1".to_string(),
        };

        let new_user_token = srv.test_create_new_user(new_user);

        let response = login_user(&mut srv, email);
        let bytes = srv.execute(response.body()).unwrap();
        let token_data: LoginResponse = serde_json::from_slice(&bytes).unwrap();
        let token = token_data.token;
        assert_eq!(token, new_user_token);
    }

    #[test]
    fn test_failed_login() {
        use actix_web::HttpMessage;

        db_clear_users();

        let mut srv = create_test_server();

        let email = "test_login_10@gmail.com";

        let response = login_user(&mut srv, email);
        let bytes = srv.execute(response.body()).unwrap();
        let error_data: LoginHttpError = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(error_data.code, LoginErrorCode::InvalidCredentials as u32);

        assert!(response.status().is_client_error());
    }
}
