mod container;
mod middleware;
pub(crate) mod project;
mod token;

use askama_axum::IntoResponse;
pub(crate) use container::collect_garbage;

use axum::{
    debug_handler,
    response::Redirect,
    routing::{get, post},
    Router,
};
use libsql::Connection;

use crate::TugState;

pub(super) struct Routes {
    pub(super) public: Router<TugState>,
    pub(super) private: Router<TugState>,
}

pub(super) fn get_for_machines(connection: Connection) -> Router<TugState> {
    Router::new().route(
        "/containers/:container_id/update",
        post(container::update).route_layer(axum::middleware::from_fn_with_state(
            connection,
            middleware::require_container_token,
        )),
    )
}
pub(super) fn get_for_humans() -> Router<TugState> {
    let project_router = Router::new()
        .route("/", get(project::get_index_page))
        .route("/new", get(project::get_new_page).post(project::create))
        .route("/:project_id", get(project::get_details))
        .route("/:project_id/token", post(project::create_token))
        .route("/:project_id/delete", post(project::delete));

    let container_router = Router::new()
        .route("/", get(container::get_index).post(container::create))
        .route("/:container_id/token", post(container::create_token));

    Router::new()
        .route("/", get(|| async { Redirect::to("/containers") }))
        .nest("/containers", container_router)
}
