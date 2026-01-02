use std::convert::Infallible;

use serde::Serialize;
use serde_json;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

use crate::util;

#[derive(Serialize)]
struct SystemStats {
    memory_total: i32,
    memory_swapped: i32,
    memory_free: i32,
    memory_buffer: i32,
    memory_cache: i32,
    io_bytes_in: i32,
    io_bytes_out: i32,
    cpu_usage: i32,
}

pub fn health(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let vmstat = util::command_output("vmstat", None, None);
    let vmstat_split: Vec<Vec<&str>> = vmstat
        .lines()
        .map(|line| line.split_whitespace().collect())
        .collect();
    let total_mem = util::command_output("awk", Some(vec!["/^MemTotal:/ {printf $2}", "/proc/meminfo"]), None);

    let mut result = SystemStats {
        memory_total: total_mem.parse::<i32>().unwrap(),
        memory_swapped: -1,
        memory_free: -1,
        memory_buffer: -1,
        memory_cache: -1,
        io_bytes_in: -1,
        io_bytes_out: -1,
        cpu_usage: -1,
    };

    for (pos, e) in vmstat_split[1].iter().enumerate() {
        match *e {
            "swpd" => result.memory_swapped = vmstat_split[2][pos].parse::<i32>().unwrap(),
            "free" => result.memory_free = vmstat_split[2][pos].parse::<i32>().unwrap(),
            "buff" => result.memory_buffer = vmstat_split[2][pos].parse::<i32>().unwrap(),
            "cache" => result.memory_cache = vmstat_split[2][pos].parse::<i32>().unwrap(),
            "bi" => result.io_bytes_in = vmstat_split[2][pos].parse::<i32>().unwrap(),
            "bo" => result.io_bytes_out = vmstat_split[2][pos].parse::<i32>().unwrap(),
            "us" => result.cpu_usage = vmstat_split[2][pos].parse::<i32>().unwrap(),
            _ => {}
        }
    }

    let serialized = serde_json::to_string(&result).unwrap();

    Ok(Response::new(Full::new(Bytes::from(serialized))))
}
