use std::convert::Infallible;

use bollard::container::LogOutput;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::CreateContainerOptionsBuilder;
use bollard::query_parameters::CreateImageOptionsBuilder;
use bollard::query_parameters::InspectContainerOptionsBuilder;
use bollard::query_parameters::ListContainersOptionsBuilder;
use bollard::query_parameters::LogsOptionsBuilder;
use bollard::query_parameters::StartContainerOptionsBuilder;
use bollard::query_parameters::StopContainerOptionsBuilder;
use bollard::query_parameters::RemoveContainerOptionsBuilder;
use bollard::Docker;

use serde::Deserialize;
use serde_json;

use futures_util::TryStreamExt;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

#[derive(Deserialize)]
struct DockerRequest {
    container_name: String,
    container_config: ContainerCreateBody,
}

pub async fn create_or_update_container(
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
    let setup: DockerRequest = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            let res = Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Cant read request body. JSON parse error\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker init failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let options = CreateContainerOptionsBuilder::default()
        .name(&setup.container_name)
        .build();
    // clone config because we may try create twice (ContainerCreateBody derives Clone)
    let cfg = setup.container_config.clone();
    match docker
        .create_container(Some(options.clone()), cfg.clone())
        .await
    {
        Ok(result) => {
            let serialized = serde_json::to_string(&result).unwrap();
            Ok(Response::new(Full::new(Bytes::from(serialized))))
        }
        Err(e) => {
            let err_str = e.to_string();
            // if missing image, try to pull it then retry create
            if err_str.contains("No such image") || err_str.contains("404") {
                let image = cfg.image.clone().unwrap_or_default();
                if image.is_empty() {
                    let res = Response::builder()
                        .status(hyper::StatusCode::BAD_REQUEST)
                        .body(Full::new(Bytes::from("{\"error\":\"No image specified\"}")))
                        .unwrap();
                    return Ok(res);
                }
                // naive split into repo:tag (falls back to "latest")
                let (repo, tag) = match image.rsplit_once(':') {
                    Some((r, t)) => (r.to_string(), t.to_string()),
                    None => (image.clone(), "latest".to_string()),
                };
                let create_image_opts = CreateImageOptionsBuilder::default()
                    .from_image(repo.as_str())
                    .tag(tag.as_str())
                    .build();
                let mut pull_stream = docker.create_image(Some(create_image_opts), None, None);
                loop {
                    match pull_stream.try_next().await {
                        Ok(Some(_progress)) => continue,
                        Ok(None) => break,
                        Err(pe) => {
                            let res = Response::builder()
                                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Full::new(Bytes::from(format!(
                                    "{{\"error\":\"Image pull failed\",\"message\":\"{}\"}}",
                                    pe
                                ))))
                                .unwrap();
                            return Ok(res);
                        }
                    }
                }
                // retry create
                match docker.create_container(Some(options), cfg).await {
                    Ok(result) => {
                        let serialized = serde_json::to_string(&result).unwrap();
                        Ok(Response::new(Full::new(Bytes::from(serialized))))
                    }
                    Err(e2) => {
                        let res = Response::builder()
                            .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Full::new(Bytes::from(
                                format!("{{\"error\": \"Docker create container failed\",\"message\":\"{}\"}}", e2),
                            )))
                            .unwrap();
                        return Ok(res);
                    }
                }
            } else {
                let res = Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::new(Bytes::from(format!(
                        "{{\"error\": \"Docker create container failed\",\"message\":\"{}\"}}",
                        e
                    ))))
                    .unwrap();
                return Ok(res);
            }
        }
    }
}

pub async fn list_containers(
    _request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker init failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let images = match docker.list_containers(Some(options)).await {
        Ok(v) => v,
        Err(_) => {
            return Ok(Response::new(Full::new(Bytes::from(
                "{\"error\": \"Docker init failed\"}",
            ))));
        }
    };

    let serialized = serde_json::to_string(&images).unwrap();

    Ok(Response::new(Full::new(Bytes::from(serialized))))
}

pub async fn container_inspect(id: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = InspectContainerOptionsBuilder::default().build();
    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker init failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let docker_container_inspect = match docker.inspect_container(&id, Some(options)).await {
        Ok(v) => v,
        Err(_) => {
            return Ok(Response::new(Full::new(Bytes::from(
                "{\"error\": \"Docker init failed\"}",
            ))));
        }
    };

    let serialized = serde_json::to_string(&docker_container_inspect).unwrap();

    Ok(Response::new(Full::new(Bytes::from(serialized))))
}

pub async fn container_start(id: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = StartContainerOptionsBuilder::default().build();
    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker init failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    match docker.start_container(&id, Some(options)).await {
        Ok(_) => {
            return Ok(Response::new(Full::new(Bytes::from(
                "{\"ok\": \"Docker container started\"}",
            ))))
        }
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker container start failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };
}

pub async fn container_stop(id: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = StopContainerOptionsBuilder::default().build();
    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker init failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    match docker.stop_container(&id, Some(options)).await {
        Ok(_) => {
            return Ok(Response::new(Full::new(Bytes::from(
                "{\"ok\": \"Docker container stopped\"}",
            ))))
        }
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker container stop failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };
}

pub async fn container_rm(id: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = RemoveContainerOptionsBuilder::default().build();
    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker init failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    match docker.remove_container(&id, Some(options)).await {
        Ok(_) => {
            return Ok(Response::new(Full::new(Bytes::from(
                "{\"ok\": \"Docker container removed\"}",
            ))))
        }
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker container stop failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };
}

pub async fn container_logs(id: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .build();

    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker init failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let logs_stream = docker.logs(id, Some(options));

    let lines = match logs_stream
        .map_ok(|log| match log {
            LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
                String::from_utf8_lossy(&message).to_string()
            }
            _ => String::new(),
        })
        .try_collect::<Vec<String>>()
        .await
    {
        Ok(l) => l,
        Err(e) => {
            println!("{}", e);
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Docker read logs failed\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };
    let last_100_lines: &[String] = &lines[lines.len().saturating_sub(100)..];

    let serialized = serde_json::to_string(&last_100_lines).unwrap();

    Ok(Response::new(Full::new(Bytes::from(serialized))))
}
