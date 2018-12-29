
use super::app::AppState;

use futures::Future;

use actix_web::{
    AsyncResponder, FutureResponse, HttpResponse,
    State, Json,
};

use super::super::db::users::{CreateUser, LoginWithEmail, SearchWithKeyword};

#[derive(Deserialize)]
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
