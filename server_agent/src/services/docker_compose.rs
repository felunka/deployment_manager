use std::convert::Infallible;
use std::fs;

use serde::Deserialize;
use serde_json;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

use crate::util;

#[derive(Deserialize)]
struct DockerComposeRequest {
    path: String,
    compose: String,
}

pub async fn create_or_update_compose(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let body = match request.into_body().collect().await {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Cant read request body\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let body_bytes = body.to_bytes();
    // Parse JSON
    let setup: DockerComposeRequest = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Cant read request body\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    // Create dirs if needed
    if let Err(e) = std::fs::create_dir_all(&setup.path) {
        let res = Response::builder()
            .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::new(Bytes::from(format!(
                "{{\"error\": \"mkdir failed: {}\"}}",
                e
            ))))
            .unwrap();
        return Ok(res);
    }

    // Update compose file content
    match fs::write(format!("{}/docker-compose.yml", &setup.path), setup.compose) {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Cant read/write docker-compose.yml\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    // Update compose
    let compose_output = util::command_output(
        "docker",
        Some(vec!["compose", "up", "-d", "--remove-orphans", "--build"]),
        Some(&setup.path),
    );

    Ok(Response::new(Full::new(Bytes::from(format!(
        "{{\"ok\": true, \"compose\": \"{}\"}}",
        compose_output
    )))))
}

pub async fn logs(path: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let compose_logs_output = util::command_output(
        "docker",
        Some(vec!["compose", "logs"]),
        Some(path),
    );
    let lines: Vec<&str> = compose_logs_output.lines().collect();
    let last_100_lines = &lines[lines.len().saturating_sub(100)..];

    let serialized = serde_json::to_string(&last_100_lines).unwrap();

    Ok(Response::new(Full::new(Bytes::from(serialized))))
}
