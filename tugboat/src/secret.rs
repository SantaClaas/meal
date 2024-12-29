use std::{
    collections::HashMap,
    env::{self},
    rc::Rc,
    sync::Arc,
};

use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{secrets::SecretsGetRequest, ClientSecretsExt},
    Client,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub(super) enum ErrorType {
    #[error("Error loading secret id from environment variables: {0}")]
    VarError(#[from] env::VarError),
    #[error("Error parsing user secret id: {0}")]
    ParseError(#[from] uuid::Error),
}

#[derive(Debug)]
enum Secret {
    UserSecret,
    CookieSigningSecret,
}

#[derive(Error, Debug)]
#[error("Error loading secret id {variable}: {source}")]
pub(super) struct LoadSecretIdError {
    variable: Rc<str>,
    #[source]
    source: ErrorType,
}

#[derive(Error, Debug)]
pub(super) enum Error {
    #[error("Failed to load token from environment variables: {0}")]
    LoadTokenError(#[source] env::VarError),
    #[error("Error getting secrets from Bitwarden Secrets Manager")]
    BwsError(#[from] bitwarden::Error),
    #[error("Error authenticating with Bitwarden")]
    BwsAuthenticationFailed,
    #[error("Error loading secret id from environment variables: {0}")]
    LoadSecretIdError(#[from] LoadSecretIdError),
    #[error("Secret not provided by Bitwarden: {0:?}")]
    SecretNotProvided(Secret),
}

#[derive(Clone)]
pub(crate) struct Secrets {
    pub(crate) user_secret: Arc<str>,
    pub(crate) cookie_signing_secret: Arc<str>,
}

const USER_SECRET_ID_VARIABLE: &str = "USER_SECRET_ID";
const COOKIE_SIGNING_SECRET_ID: &str = "COOKIE_SIGNING_SECRET_ID";
const SECRET_ID_VARIABLES: &[&str] = &[USER_SECRET_ID_VARIABLE, COOKIE_SIGNING_SECRET_ID];

fn load_secret_ids() -> Result<HashMap<Uuid, &'static str>, LoadSecretIdError> {
    let mut secret_ids = HashMap::with_capacity(SECRET_ID_VARIABLES.len());
    for variable in SECRET_ID_VARIABLES {
        let value = env::var(variable).map_err(|error| LoadSecretIdError {
            variable: (*variable).into(),
            source: ErrorType::VarError(error),
        })?;

        let id = value.parse().map_err(|error| LoadSecretIdError {
            variable: (*variable).into(),
            source: ErrorType::ParseError(error),
        })?;

        secret_ids.insert(id, *variable);
    }

    Ok(secret_ids)
}

pub(super) async fn setup() -> Result<Secrets, Error> {
    let client = Client::new(None);

    let request = AccessTokenLoginRequest {
        access_token: env::var("BWS_TOKEN").map_err(Error::LoadTokenError)?,
        state_file: None,
    };

    let response = client.auth().login_access_token(&request).await?;

    if !response.authenticated {
        return Err(Error::BwsAuthenticationFailed);
    }

    let ids_by_variable = load_secret_ids()?;
    let request = SecretsGetRequest {
        ids: ids_by_variable.keys().copied().collect(),
    };

    let responses = client.secrets().get_by_ids(request).await?;

    let mut user_secret = None;
    let mut cookie_signing_secret = None;
    for secret in responses.data {
        let Some(variable) = ids_by_variable.get(&secret.id) else {
            tracing::warn!(
                "Received secret with id {} that was not requested",
                secret.id
            );
            continue;
        };

        match *variable {
            USER_SECRET_ID_VARIABLE => user_secret = Some(secret.value),
            COOKIE_SIGNING_SECRET_ID => cookie_signing_secret = Some(secret.value),
            //TODO make ids an enum to check compile time because this branch should not be reachable
            _ => {
                tracing::warn!(
                    "Received unknown secret with id {} and variable {}",
                    secret.id,
                    variable
                );
            }
        }
    }

    let user_secret = user_secret
        .ok_or_else(|| Error::SecretNotProvided(Secret::UserSecret))?
        .into();

    let cookie_signing_secret = cookie_signing_secret
        .ok_or_else(|| Error::SecretNotProvided(Secret::CookieSigningSecret))?
        .into();

    Ok(Secrets {
        user_secret,
        cookie_signing_secret,
    })
}
