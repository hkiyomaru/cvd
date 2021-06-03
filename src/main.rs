use clap::{crate_version, value_t, App, AppSettings, Arg};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::process;
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

fn get_gpu_uuids() -> HashSet<String> {
    let output = process::Command::new(NVSMI)
        .arg("--query-gpu=uuid")
        .arg("--format=csv,noheader")
        .output();
    parse_output(output)
}

fn get_used_gpu_uuids() -> HashSet<String> {
    let output = process::Command::new(NVSMI)
        .arg("--query-compute-apps=gpu_uuid")
        .arg("--format=csv,noheader")
        .output();
    parse_output(output)
}

fn parse_output(output: io::Result<process::Output>) -> HashSet<String> {
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
    let uuids: HashSet<String> = stdout.lines().map(String::from).collect();
    uuids
}

fn main() {
    env_logger::init();

    let matches = App::new("cvd")
        .setting(AppSettings::ColoredHelp)
        .version(crate_version!())
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

    if !is_avail() {
        log::error!("command not found: {}", NVSMI);
        process::exit(1);
    }

    let gpu_uuids = get_gpu_uuids();

    let gpu_uuids = if matches.is_present("empty_only") {
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

    let n_avail = gpu_uuids.len();
    if n_avail == 0 {
        log::error!("no available GPUs");
        process::exit(1);
    }

    let n = value_t!(matches, "n", usize).unwrap_or(n_avail);
    if n > n_avail {
        log::error!("{} exceeds the number of available GPUs, {}", n, n_avail);
        process::exit(1);
    }

    gpu_uuids.truncate(n);

    println!("{}", gpu_uuids.join(","));
}
