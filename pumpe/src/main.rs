use std::{collections::VecDeque, fs, path::Path, rc::Rc};

use axum::{
    body::Body,
    http::{self, header},
    Router,
};
use camino::Utf8PathBuf;
use clap::{error::ErrorKind, CommandFactory, Parser};
use http_body_util::BodyExt;
use thiserror::Error;
use tower::ServiceExt;
use tower_http::{compression::CompressionLayer, services::ServeDir};

#[derive(Parser, Debug)]
#[command()]
struct Arguments {
    #[arg(num_args = 1)]
    directory: Utf8PathBuf,
}

#[derive(Error, Debug)]
enum PumpeError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

fn visit_directory(path: Utf8PathBuf) -> Result<Rc<[Utf8PathBuf]>, PumpeError> {
    let mut files = Vec::new();
    let mut directory_queue = VecDeque::new();
    directory_queue.push_back(path.clone());

    // Breadth first traversal
    while let Some(path) = directory_queue.pop_front() {
        for entry in path.read_dir_utf8()? {
            let entry = entry?;
            let path = entry.path().to_path_buf();
            if path.is_dir() {
                directory_queue.push_back(path);

                continue;
            }

            files.push(path);
        }
    }

    Ok(files.into())
}

#[tokio::main]
async fn main() -> Result<(), PumpeError> {
    // Helpful: https://www.rustadventure.dev/introducing-clap/clap-v4/accepting-file-paths-as-arguments-in-clap
    let directory = Arguments::parse().directory;

    if !directory.exists() {
        let mut command = Arguments::command();
        command.error(
            ErrorKind::ValueValidation,
            format!("Path does not exist: {}", directory),
        );
    }

    if !directory.is_dir() {
        let mut command = Arguments::command();
        command.error(
            ErrorKind::ValueValidation,
            format!("Path is not a directory: {}", directory),
        );
    }

    // List all files in directory
    let files = visit_directory(directory.clone())?;

    let app: Router = Router::new()
        .fallback_service(ServeDir::new(directory.clone()))
        .layer(CompressionLayer::new());

    fn request(path: &str, encoding: &str) -> http::Request<Body> {
        http::Request::get(path)
            .header(header::ACCEPT_ENCODING, encoding)
            .body(Body::empty())
            .unwrap()
    }

    async fn write_response(response: http::Response<Body>, target: &Utf8PathBuf) {
        let data = response.into_body().collect().await.unwrap().to_bytes();
        fs::write(target, data).unwrap();
    }

    async fn compress(
        app: Router,
        file: &Utf8PathBuf,
        target_directory: impl AsRef<Path>,
        (encoding_extension, encoding): (&str, &str),
    ) {
        // Request
        let path = file.strip_prefix(target_directory).unwrap();
        let path = Utf8PathBuf::from("/").join(path);
        let request = request(path.as_str(), encoding);
        let response = app.clone().oneshot(request).await.unwrap();

        // Write
        let mut target = file.clone();
        let extension = target.extension().unwrap_or_default();
        target.set_extension(format!("{extension}.{encoding_extension}",));
        write_response(response, &target).await;
    }

    for file in files.iter() {
        // Detect if file is already compressed. This is not precise as it does not check the content but it is enough for our usecase
        if file
            .extension()
            .is_some_and(|extension| matches!(extension, "br" | "gz" | "zst" | "zz"))
        {
            continue;
        }

        compress(app.clone(), file, directory.as_path(), ("gz", "gzip")).await;
        compress(app.clone(), file, directory.as_path(), ("br", "br")).await;
        compress(app.clone(), file, directory.as_path(), ("zst", "zstd")).await;
        compress(app.clone(), file, directory.as_path(), ("zz", "deflate")).await;
    }

    Ok(())
}
