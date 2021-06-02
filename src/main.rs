#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::collections;
use std::env;
use std::fs;
use std::io;
use std::process;
use std::result;
use std::str;

const NVSMI: &str = "nvidia-smi";

fn is_avail() -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p = format!("{}/{}", p, NVSMI);
            if fs::metadata(p).is_ok() {
                return true;
            }
        }
    }
    false
}

fn get_gpu_uuids() -> collections::HashSet<String> {
    let output = process::Command::new(NVSMI)
        .arg("--query-gpu=uuid")
        .arg("--format=csv,noheader")
        .output();
    parse_output(output)
}

fn get_used_gpu_uuids() -> collections::HashSet<String> {
    let output = process::Command::new(NVSMI)
        .arg("--query-compute-apps=gpu_uuid")
        .arg("--format=csv,noheader")
        .output();
    parse_output(output)
}

fn parse_output(
    output: result::Result<process::Output, io::Error>,
) -> collections::HashSet<String> {
    let output = match output {
        Ok(output) => output,
        Err(_) => {
            log::error!("failed to execute: {}", NVSMI);
            process::exit(1);
        }
    };
    let stdout = str::from_utf8(&output.stdout);
    let stdout = match stdout {
        Ok(output) => output,
        Err(_) => {
            log::error!("failed to parse output: {}", NVSMI);
            process::exit(1);
        }
    };
    let uuids: collections::HashSet<String> = stdout.lines().map(String::from).collect();
    uuids
}

fn main() {
    let matches = App::new("cvd")
        .version(crate_version!())
        .author("Hirokazu Kiyomaru <h.kiyomaru@gmail.com>")
        .about("Show CUDA visible devices")
        .arg(
            Arg::with_name("n")
                .short("n")
                .value_name("NUM")
                .help("Sets the number of devices to show")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("empty_only")
                .short("e")
                .long("empty-only")
                .help("Shows empty devices"),
        )
        .get_matches();

    env_logger::init();

    let n = value_t!(matches, "n", usize).unwrap_or(0);

    let empty_only = matches.is_present("empty_only");

    if !is_avail() {
        log::error!("command not found: {}", NVSMI);
        process::exit(1);
    }
    let gpu_uuids = get_gpu_uuids();
    let gpu_uuids = if empty_only {
        let used_gpu_uuids = get_used_gpu_uuids();
        gpu_uuids
            .difference(&used_gpu_uuids)
            .map(String::from)
            .collect()
    } else {
        gpu_uuids
    };
    let mut gpu_uuids: Vec<String> = gpu_uuids.into_iter().collect();
    gpu_uuids.sort();
    if n > 0 {
        if gpu_uuids.len() > n {
            log::error!("{} exceeds the number of GPUs, {}", n, gpu_uuids.len());
            process::exit(1);
        }
        gpu_uuids.truncate(n);
    }
    println!("{}", gpu_uuids.join(","));
}
