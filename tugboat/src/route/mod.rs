mod container;
mod middleware;
pub(crate) mod project;
mod token;


pub(crate) use container::collect_garbage;

use axum::{
    routing::{get, post},
    Router,
};
use libsql::Connection;

use crate::TugState;

pub(crate) fn create_router(connection: Connection) -> Router<TugState> {
    let project_router = Router::new()
        .route("/", get(project::get_index_page))
        .route("/new", get(project::get_new_page).post(project::create))
        .route("/:project_id", get(project::get_details))
        .route("/:project_id/token", post(project::create_token))
        .route("/:project_id/delete", post(project::delete));

    let container_router = Router::new()
        .route("/", get(container::get_index).post(container::create))
        .route("/:container_id/token", post(container::create_token))
        .route(
            "/:container_id/update",
            post(container::update).route_layer(axum::middleware::from_fn_with_state(
                connection,
                middleware::require_container_token,
            )),
        );

    Router::new()
        .nest("/projects", project_router)
        .nest("/containers", container_router)
}
