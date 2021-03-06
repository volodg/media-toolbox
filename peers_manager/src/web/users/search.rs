use super::super::app::AppState;

use futures::Future;

use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, State};

use super::super::super::db::users::SearchWithKeyword;

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
