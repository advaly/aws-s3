extern crate s3;
use s3::bucket::Bucket;
use s3::creds::Credentials;

use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use clap::{App, Arg, ArgGroup};
use anyhow::{Result, anyhow};
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
struct Configs {
    region: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
    local: String,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = App::new("aws-s3")
        // headers
        .version(env!("CARGO_PKG_VERSION"))
        .author("ADVALY SYSTEM Inc.")
        .about("AWS S3 file uploader and downloader")

        // mode
        .arg(Arg::with_name("list").help("List objects on remote"))
        .arg(Arg::with_name("get").help("Get a file from remote"))
        .arg(Arg::with_name("put").help("Put a file to remote"))
        .arg(Arg::with_name("delete").help("Delete a file from remote"))
        .group(ArgGroup::with_name("mode")
            .args(&["list","get", "put", "delete"])
            .required(true)
        )

        // options
        .arg(Arg::with_name("local")
           .short("l").long("local")
            .help("Local file path to put/get")
            .takes_value(true)
        )
        .arg(Arg::with_name("remote")
           .short("r").long("remote")
            .help("Remote file path on AWS S3")
            .takes_value(true)
        )
        .arg(Arg::with_name("region")
            .short("g").long("region")
            .help("AWS S3 region name")
            .takes_value(true)
        )
        .arg(Arg::with_name("bucket")
            .short("b").long("bucket")
            .help("AWS S3 bucket name")
            .takes_value(true)
        )
        .arg(Arg::with_name("access key id")
            .short("a").long("access_key_id")
            .help("AWS_ACCESS_KEY_ID")
            .takes_value(true)
        )
        .arg(Arg::with_name("secret access key")
            .short("s").long("secret_access_key")
            .help("AWS_SECRET_ACCESS_KEY")
            .takes_value(true)
        )
        .arg(Arg::with_name("config")
            .short("c").long("config")
            .help("Config file path")
            .takes_value(true)
            .default_value("aws-s3.json")
        )
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Enable debug print")
        )
        .get_matches();

    // Read config parameters if exist
    let mut cfg: Configs = match File::open(args.value_of("config").unwrap()) {
        Ok(file) => serde_json::from_reader(BufReader::new(file))?,
        Err(_) => Default::default()
    };

    // Overwrite config parameters by command line options
    args.value_of("access key id").map(|v| cfg.access_key_id = v.into());
    args.value_of("secret access key").map(|v| cfg.secret_access_key = v.into());
    args.value_of("bucket").map(|v| cfg.bucket = v.into());
    args.value_of("region").map(|v| cfg.region = v.into());
    args.value_of("local").map(|v| cfg.local = v.into());

    // debug print
    if args.is_present("debug") {
        println!("{:#?}", cfg);
    }

    // Set AWS keys as environment variable or use existing env settings
    set_env("AWS_ACCESS_KEY_ID", cfg.access_key_id);
    set_env("AWS_SECRET_ACCESS_KEY", cfg.secret_access_key);

    // Create a bucket object
    let bucket = Bucket::new(
        cfg.bucket.as_str(),
        cfg.region.parse()?,
        Credentials::new(None, None, None, None, None)?
    )?;

    // Perform S3 access
    let local = if cfg.local != "" { Some(cfg.local.as_str()) } else { None };
    aws_s3(bucket, args.value_of("mode"), args.value_of("remote"), local, args.is_present("debug"))?;

    Ok(())
}

fn aws_s3(bucket: Bucket, mode: Option<&str>, remote: Option<&str>, local: Option<&str>, debug: bool) -> Result<()> {
    if debug {
        println!("{:#?}", bucket);
        println!("mode = {:?}", mode);
        println!("remote path = {:?}", remote);
        println!("local path = {:?}", local);
    }

    match mode {
        // List remote objects
        Some("l" | "list") | None => {
            let results = bucket.list_blocking("".to_string(), None)?;
            for (list, code) in results {
                check_code(code)?;
                for content in list.contents {
                    println!("{:?}", content);
                }
            }        
        },

        // Put a file to remote
        Some("p" | "put") => {
            // Check path
            let local_path = local.ok_or(anyhow!("No local path specified"))?;
            let remote_path = remote.ok_or(anyhow!("No remote path specified"))?;

            // Read data from file
            let mut buffer = Vec::new();
            File::open(local_path).and_then(|mut f| f.read_to_end(&mut buffer))?;

            // Put to remote
            let (_, code) = bucket.put_object_blocking(&remote_path, &buffer)?;
            check_code(code)?;
        },

        // Get a file from remote
        Some("g" | "get") => {
            // Check remote path
            let remote_path = remote.ok_or(anyhow!("No remote path specified"))?;

            // Check local_path. Add the remote file name if local path is directory.
            let local_path = local.ok_or(anyhow!("No local path specified"))
                .map(|v| {
                    let mut path = PathBuf::from(v);
                    if path.exists() && path.is_dir() {
                        path = path.join(Path::new(remote_path).file_name().unwrap());
                        if debug {
                            println!("local path (complemented) = {:?}", path);
                        }
                    }
                    path
                })?;
            
            // Get the remote file
            let (data, code) = bucket.get_object_blocking(remote_path)?;
            check_code(code)?;

            // Write to a file
            File::create(local_path).and_then(|mut f| f.write_all(&data))?;
        },

        // Delete a file from remote
        Some("d" | "del" | "delete") => {
            // Check remote path
            let remote_path = remote.ok_or(anyhow!("No remote path specified"))?;

            // Delete a file
            bucket.delete_object_blocking(remote_path)?;
        }

        // Error
        Some(_) => return Err(anyhow!("Invalid mode"))
    }

    Ok(())
}

fn check_code(code: u16) -> Result<()> {
    match code {
        200 => Ok(()),
        n => Err(anyhow!("Remote file error. result code is {}.", n))
    }
}

fn set_env(name: &str, value: String) {
    if value != "" {
        env::set_var(name, value);
    }
}
