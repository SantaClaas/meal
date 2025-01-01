mod container;
pub(crate) mod project;

use axum::{
    routing::{get, post},
    Router,
};

use crate::TugState;

pub(crate) fn create_router() -> Router<TugState> {
    let project_router = Router::new()
        .route("/", get(project::get_index_page))
        .route("/new", get(project::get_new_page).post(project::create))
        .route("/:project_id", get(project::get_details))
        .route("/:project_id/token", post(project::create_token))
        .route("/:project_id/delete", post(project::delete));

    let container_router =
        Router::new().route("/", get(container::get_index).post(container::create));
    Router::new()
        .nest("/projects", project_router)
        .nest("/containers", container_router)
}
