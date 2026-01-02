use std::collections::HashMap;
use std::convert::Infallible;

use http_body_util::Full;
use hyper::{Method, Request, Response};
use hyper::body::Bytes;
use bcrypt::{verify};

use regex::Regex;

use crate::services::{self, github_runners};


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
    if path == "/health" {
        services::health::health(request)
    } else if path == "/docker/containers/list" {
        services::docker::list_containers(request).await
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
    } else if path == "/runner/status" && params.contains_key("path") {
        let svc_path = params.get("path").unwrap();
        github_runners::get_status(svc_path)
    } else if path == "/runner" && request.method() == Method::POST {
        github_runners::setup_new(request).await
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
