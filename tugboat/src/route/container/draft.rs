use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use axum::{
    extract::{self, Path},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
// Using axum extra form extractor because it supports multiple values with the same key
use axum_extra::extract::Form;
use nanoid::nanoid;
use serde::Deserialize;
use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::{redirect_to, TugState};

#[derive(Clone, Hash, Eq, PartialEq, Deserialize, Debug)]
pub(in crate::route) struct Id(Arc<str>);

impl Display for Id {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Default)]
pub(crate) struct State(Arc<Mutex<HashMap<Id, Draft>>>);

impl<T> From<T> for Id
where
    T: AsRef<str>,
{
    fn from(id: T) -> Self {
        Self(Arc::from(id.as_ref()))
    }
}

#[derive(Clone, Deserialize)]
pub(in crate::route) struct Draft {
    id: Id,
    environment_variable_keys: Vec<String>,
    environment_variable_values: Vec<String>,
    created_at: OffsetDateTime,
}

pub(in crate::route) async fn create(extract::State(state): extract::State<TugState>) -> Redirect {
    let mut drafts = state.container_drafts.0.lock().await;

    let id: Id = nanoid!().into();

    let draft = Draft {
        id: id.clone(),
        environment_variable_keys: Vec::new(),
        environment_variable_values: Vec::new(),
        created_at: OffsetDateTime::now_utc(),
    };

    drafts.insert(id.clone(), draft);

    let url = format!("/containers/drafts/{id}");
    Redirect::to(&url)
}

#[derive(askama::Template)]
#[template(path = "container/draft.html")]
pub(in crate::route) struct Template {
    draft: Draft,
}

#[derive(thiserror::Error, Debug)]
pub(in crate::route) enum GetDraftError {
    #[error("Draft not found: {0}")]
    NotFound(Id),
}

impl IntoResponse for GetDraftError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(_id) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

#[axum::debug_handler]
pub(in crate::route) async fn get(
    extract::State(state): extract::State<TugState>,
    Path(id): Path<Id>,
) -> Result<Template, GetDraftError> {
    let drafts = state.container_drafts.0.lock().await;

    let draft = drafts.get(&id).ok_or_else(|| GetDraftError::NotFound(id))?;

    Ok(Template {
        draft: draft.clone(),
    })
}

/// Completes the draft and creates a container
pub(in crate::route) async fn create_container(
    extract::State(state): extract::State<TugState>,
    Form(draft): Form<Draft>,
) -> Result<Redirect, ()> {
    Ok(Redirect::to("/containers"))
}

pub(in crate::route) async fn update(
    extract::State(state): extract::State<TugState>,
    Form(draft): Form<Draft>,
) -> Result<Redirect, ()> {
    Ok(redirect_to!("/containers/drafts/{}", draft.id))
}
