use std::convert::Infallible;
use std::path::Path;
use std::process::Command;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use regex::Regex;
use serde::Deserialize;

use crate::util;

#[derive(Deserialize)]
struct SetupRequest {
    token: String,
    path: String,
    git_url: String,
}

pub async fn setup_new(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let body = match request.into_body().collect().await {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Cant read request body\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let body_bytes = body.to_bytes();
    // Parse JSON
    let setup: SetupRequest = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(_) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"Cant read request body\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    // ensure directory does not already exist
    if Path::new(&setup.path).exists() {
        let res = Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from(
                "{\"error\": \"path already exists\"}",
            )))
            .unwrap();
        return Ok(res);
    }

    // create directory
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

    // create an HTTP client that does NOT follow redirects so we can read the Location header
    let client_no_redirect = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"client init failed: {}\"}}",
                    e
                ))))
                .unwrap();
            return Ok(res);
        }
    };

    // get latest release redirect
    let latest_url = "https://github.com/actions/runner/releases/latest";
    let resp = match client_no_redirect.get(latest_url).send().await {
        Ok(r) => r,
        Err(e) => {
            let res = Response::builder()
                .status(hyper::StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"request failed: {}\"}}",
                    e
                ))))
                .unwrap();
            return Ok(res);
        }
    };

    let status = resp.status().as_u16();
    if !(status == 302 || status == 301) {
        let res = Response::builder()
            .status(hyper::StatusCode::BAD_GATEWAY)
            .body(Full::new(Bytes::from(format!(
                "{{\"error\": \"unexpected redirect status: {}\"}}",
                status
            ))))
            .unwrap();
        return Ok(res);
    }

    let location = match resp.headers().get(reqwest::header::LOCATION) {
        Some(v) => match v.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        None => String::new(),
    };

    if location.is_empty() {
        let res = Response::builder()
            .status(hyper::StatusCode::BAD_GATEWAY)
            .body(Full::new(Bytes::from(
                "{\"error\": \"no location header\"}",
            )))
            .unwrap();
        return Ok(res);
    }

    // expect format .../tag/vX.Y.Z
    let re = Regex::new(r"/tag/(v[0-9].*)$").unwrap();
    let tag = match re
        .captures(&location)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
    {
        Some(t) => t,
        None => {
            let res = Response::builder()
                .status(hyper::StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(
                    "{\"error\": \"could not extract version tag\"}",
                )))
                .unwrap();
            return Ok(res);
        }
    };

    let file_version = tag.trim_start_matches('v');

    let file_name = format!("actions-runner-linux-x64-{}.tar.gz", file_version);
    let download_url = format!(
        "https://github.com/actions/runner/releases/download/{}/{}",
        tag, file_name
    );

    // download file
    let download_resp = match reqwest::get(&download_url).await {
        Ok(r) => r,
        Err(e) => {
            let res = Response::builder()
                .status(hyper::StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"download failed: {}\"}}",
                    e
                ))))
                .unwrap();
            return Ok(res);
        }
    };

    if !download_resp.status().is_success() {
        let res = Response::builder()
            .status(hyper::StatusCode::BAD_GATEWAY)
            .body(Full::new(Bytes::from(format!(
                "{{\"error\": \"download returned {}\"}}",
                download_resp.status()
            ))))
            .unwrap();
        return Ok(res);
    }

    let bytes = match download_resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            let res = Response::builder()
                .status(hyper::StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"reading download failed: {}\"}}",
                    e
                ))))
                .unwrap();
            return Ok(res);
        }
    };

    let target_file_path = Path::new(&setup.path).join(&file_name);
    if let Err(e) = std::fs::write(&target_file_path, &bytes) {
        let res = Response::builder()
            .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::new(Bytes::from(format!(
                "{{\"error\": \"write file failed: {}\"}}",
                e
            ))))
            .unwrap();
        return Ok(res);
    }

    // extract tar
    let extract = Command::new("tar")
        .arg("xzf")
        .arg(target_file_path.file_name().unwrap())
        .current_dir(&setup.path)
        .status();

    match extract {
        Ok(s) if s.success() => {}
        Ok(s) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"tar exited with {}\"}}",
                    s
                ))))
                .unwrap();
            return Ok(res);
        }
        Err(e) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"tar failed: {}\"}}",
                    e
                ))))
                .unwrap();
            return Ok(res);
        }
    }

    // run config script
    let cfg = Command::new("./config.sh")
        .arg("--url")
        .arg(&setup.git_url)
        .arg("--token")
        .arg(&setup.token)
        .arg("--unattended")
        .current_dir(&setup.path)
        .status();

    match cfg {
        Ok(s) if s.success() => {}
        Ok(s) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"config exited with {}\"}}",
                    s
                ))))
                .unwrap();
            return Ok(res);
        }
        Err(e) => {
            let res = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(format!(
                    "{{\"error\": \"config failed: {}\"}}",
                    e
                ))))
                .unwrap();
            return Ok(res);
        }
    }

    // try to install and start service
    let mut svc_result = String::new();
    let svc_install = Command::new("sudo")
        .arg("-n")
        .arg("./svc.sh")
        .arg("install")
        .current_dir(&setup.path)
        .status();
    match svc_install {
        Ok(s) if s.success() => svc_result.push_str("installed "),
        Ok(s) => svc_result.push_str(&format!("install-exit:{} ", s)),
        Err(e) => svc_result.push_str(&format!("install-err:{} ", e)),
    }

    let svc_start = Command::new("sudo")
        .arg("-n")
        .arg("./svc.sh")
        .arg("start")
        .current_dir(&setup.path)
        .status();
    match svc_start {
        Ok(s) if s.success() => svc_result.push_str("started"),
        Ok(s) => svc_result.push_str(&format!("start-exit:{}", s)),
        Err(e) => svc_result.push_str(&format!("start-err:{}", e)),
    }

    let res = Response::builder()
        .status(hyper::StatusCode::OK)
        .body(Full::new(Bytes::from(format!(
            "{{\"ok\": true, \"service\": \"{}\"}}",
            svc_result
        ))))
        .unwrap();
    Ok(res)
}

pub fn get_status(
    svc_path: &str,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let service_status = util::command_output("sudo", Some(vec!["-n", "./svc.sh", "status"]), Some(&svc_path));

    Ok(Response::new(Full::new(Bytes::from(service_status))))
}
