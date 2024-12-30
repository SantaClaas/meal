mod project;

use axum::{
    routing::{get, post},
    Router,
};

use crate::TugState;

pub(crate) fn create_router() -> Router<TugState> {
    Router::new()
        .route("/", get(project::get_index_page))
        .route("/new", get(project::get_new_page).post(project::create))
        .route("/:project_id", get(project::get_project_details))
        .route("/:project_id/token", post(project::create_token))
}
