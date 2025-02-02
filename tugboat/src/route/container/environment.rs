pub(super) fn try_load_from_file(path: impl AsRef<str>) -> Option<Vec<String>> {
    let variables = dotenvy::from_path_iter(path.as_ref())
        .inspect_err(|error| {
            tracing::error!("Error loading environment variables from file: {error}")
        })
        .ok()?
        .filter_map(|item| match item {
            Ok((key, value)) => Some(format!("{key}={value}")),
            Err(error) => {
                tracing::error!("Error loading environment variable: {error}");
                None
            }
        })
        .collect();

    Some(variables)
}
