use std::convert::Infallible;

use bollard::Docker;
use bollard::query_parameters::InspectContainerOptionsBuilder;
use bollard::query_parameters::ListContainersOptionsBuilder;
use bollard::query_parameters::StartContainerOptionsBuilder;
use bollard::query_parameters::StopContainerOptionsBuilder;
use bollard::query_parameters::LogsOptionsBuilder;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use bollard::container::LogOutput;
use futures_util::TryStreamExt;

pub async fn list_containers(
    _request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
              .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
              .body(Full::new(Bytes::from("{\"error\": \"Docker init failed\"}")))
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
              .body(Full::new(Bytes::from("{\"error\": \"Docker init failed\"}")))
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
              .body(Full::new(Bytes::from("{\"error\": \"Docker init failed\"}")))
              .unwrap();
            return Ok(res);
        }
    };

    match docker.start_container(&id, Some(options)).await {
        Ok(_) => return Ok(Response::new(Full::new(Bytes::from("{\"ok\": \"Docker container started\"}")))),
        Err(_) => {
            let res = Response::builder()
              .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
              .body(Full::new(Bytes::from("{\"error\": \"Docker container start failed\"}")))
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
              .body(Full::new(Bytes::from("{\"error\": \"Docker init failed\"}")))
              .unwrap();
            return Ok(res);
        }
    };

    match docker.stop_container(&id, Some(options)).await {
        Ok(_) => return Ok(Response::new(Full::new(Bytes::from("{\"ok\": \"Docker container stopped\"}")))),
        Err(_) => {
            let res = Response::builder()
              .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
              .body(Full::new(Bytes::from("{\"error\": \"Docker container stop failed\"}")))
              .unwrap();
            return Ok(res);
        }
    };
}

pub async fn container_logs(id: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let options = LogsOptionsBuilder::default().stdout(true).stderr(true).build();

    let docker = match Docker::connect_with_defaults() {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
              .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
              .body(Full::new(Bytes::from("{\"error\": \"Docker init failed\"}")))
              .unwrap();
            return Ok(res);
        }
    };

    let logs_stream = docker.logs(
        id,
        Some(options),
    );

    let lines = match logs_stream
        .map_ok(|log| match log {
            LogOutput::StdOut { message }
            | LogOutput::StdErr { message } => {
                String::from_utf8_lossy(&message).to_string()
            }
            _ => String::new(),
        })
        .try_collect::<Vec<String>>()
        .await {
            Ok(l) => l,
            Err(e) => {
                println!("{}", e);
                let res = Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::new(Bytes::from("{\"error\": \"Docker read logs failed\"}")))
                    .unwrap();
                return Ok(res);
            }
        };

    let serialized = serde_json::to_string(&lines).unwrap();

    Ok(Response::new(Full::new(Bytes::from(serialized))))
}
