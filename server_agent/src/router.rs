use std::collections::HashMap;
use std::convert::Infallible;

use http_body_util::Full;
use hyper::{Method, Request, Response};
use hyper::body::Bytes;
use bcrypt::{verify};

use regex::Regex;

use crate::services::{self};


pub async fn router(
    request: Request<hyper::body::Incoming>,
    key_hash: String
) -> Result<Response<Full<Bytes>>, Infallible> {
    // Auth check
    let token_result = match request.headers().get("X-Api-Key") {
        Some(t) => t,
        None => return forbidden()
    };
    let token = match token_result.to_str() {
        Ok(t) => t,
        Err(_) => return forbidden()
    };
    let valid = match verify(token, &key_hash) {
        Ok(res) => res,
        Err(e) => {
            println!("Hash verification error!");
            println!("{}", e);
            return forbidden()
        }
    };
    if !valid {
        return forbidden();
    }

    // Deconstuct request path and params
    let path = request.uri().path();
    let params: HashMap<String, String> = request
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);
    println!("{}", path);

    // Setup regex routes
    let containers_re = Regex::new(r"\/docker\/container\/(?P<id>[a-z0-9]{64})\/(?P<action>\w+)").unwrap();

    // Do routing
    if path == "/health" && request.method() == Method::GET {
        services::health::health(request)
    } else if path == "/docker/containers/list" && request.method() == Method::GET {
        services::docker::list_containers(request).await
    } else if path == "/docker/container" && request.method() == Method::POST {
        services::docker::create_or_update_container(request).await
    } else if let Some(caps) = containers_re.captures(path) {
        let id = &caps["id"];
        let action = &caps["action"];

        match action {
            "inspect" => services::docker::container_inspect(id).await,
            "start" => services::docker::container_start(id).await,
            "stop" => services::docker::container_stop(id).await,
            "logs" => services::docker::container_logs(id).await,
            _ => not_found()
        }
    } else if path == "/runner/status" && params.contains_key("path") && request.method() == Method::GET {
        let svc_path = params.get("path").unwrap();
        services::github_runners::get_status(svc_path)
    } else if path == "/runner" && request.method() == Method::POST {
        services::github_runners::setup_new(request).await
    } else if path == "/docker/compose" && request.method() == Method::POST {
        services::docker_compose::create_or_update_compose(request).await
    } else if path == "/docker/compose/status" && params.contains_key("path") && request.method() == Method::GET {
        let compose_path = params.get("path").unwrap();
        services::docker_compose::logs(compose_path).await
    } else {
        not_found()
    }
}

fn not_found() -> Result<Response<Full<Bytes>>, Infallible> {
    let resp = Response::builder()
        .status(hyper::StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("NOT FOUND")))
        .unwrap();
    Ok(resp)
}

fn forbidden() -> Result<Response<Full<Bytes>>, Infallible> {
    let resp = Response::builder()
        .status(hyper::StatusCode::FORBIDDEN)
        .body(Full::new(Bytes::from("FORBIDDEN")))
        .unwrap();
    Ok(resp)
}
