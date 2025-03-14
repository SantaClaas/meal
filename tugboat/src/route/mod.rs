mod container;
mod middleware;
mod token;

use std::time::Duration;

pub(crate) use container::collect_garbage;

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
    BoxError, Router,
};
use libsql::Connection;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};

use crate::TugState;

pub(super) fn get_for_machines(connection: Connection) -> Router<TugState> {
    Router::new().route(
        "/containers/:container_id/update",
        post(container::update_image)
            .route_layer(axum::middleware::from_fn_with_state(
                connection,
                middleware::require_container_token,
            ))
            // Rate limiting based on https://github.com/tokio-rs/axum/discussions/987#discussioncomment-2678595
            .route_layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(|err: BoxError| async move {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled error: {}", err),
                        )
                    }))
                    .layer(BufferLayer::new(1024))
                    .layer(RateLimitLayer::new(10, Duration::from_secs(60))),
            ),
    )
}
pub(super) fn get_for_humans() -> Router<TugState> {
    let container_router = Router::new()
        .route("/", get(container::get_index).post(container::create))
        .route("/:container_id/token", post(container::create_token))
        .route(
            "/:container_id/environment/variables",
            get(container::environment::get_variables).post(container::environment::update),
        )
        .route(
            "/:container_id/environment/variables/delete",
            post(container::environment::delete),
        )
        .route(
            "/:container_id/environment/variables/update",
            post(container::environment::update),
        )
        .route("/:container_id/stop", post(container::stop_container))
        .route("/:container_id/start", post(container::start_container))
        .route("/:container_id/delete", post(container::delete))
        .route(
            "/:container_id/edit",
            get(container::edit::get).post(container::edit::create),
        )
        .route("/:container_id/update/pull", post(container::pull_update));

    Router::new()
        .route("/", get(|| async { Redirect::to("/containers") }))
        .nest("/containers", container_router)
}
