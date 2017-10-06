#![allow(unused_imports)]
#![allow(dead_code)]
#[allow(unused)]
#[allow(unused_variables)]

mod api;
mod server;
mod config;
mod consul;

#[cfg(test)]
mod config_test;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate grpcio;
extern crate futures;
extern crate protobuf;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate reqwest;
extern crate serde;
extern crate url;
extern crate md5;

use clap::{Arg, App, SubCommand};
use std::process::exit;
use std::sync::Arc;
use consul::Client as ConsulClient;
use consul::catalog::Catalog;
use std::borrow::Borrow;
use grpcio::{Environment, ChannelBuilder, UnarySink};
use config::Config;
use api::eds_grpc::{EndpointDiscoveryServiceClient};
use api::discovery::{DiscoveryRequest};

fn main() {
    let app = App::new("diplomat")
        .version(crate_version!())
        .about(
            "Provides the Envoy v2 API as a gRPC service and CLI application.",
        )
        .author("Timothy Perrett")
        .arg(
            Arg::with_name("config")
                .long("config")
                .value_name("config")
                .help("Path to the configuration for diplomat")
                .required(false)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("client")
                .about("Interact with diplomat using the cli",)
                .subcommand(
                    SubCommand::with_name("eds")
                        .about("given a service name, resolve the IPs providing that service",)
                        .arg(
                            Arg::with_name("service-name")
                                .long("service-name")
                                .value_name("service-name")
                                .required(true)
                                .takes_value(true),
                        )
                )
                .subcommand(
                    SubCommand::with_name("cds")
                        .about("cds",)
                        .arg(
                            Arg::with_name("service-name")
                                .long("service-name")
                                .value_name("service-name")
                                .required(true)
                                .takes_value(true),
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("serve")
                .about("Starts the diplomat server",)
        );

    // TIM: Not sure if cloning here is going to cause problems,
    // but given this is once at the edge of the world it probally isn't
    // too much of a big deal.
    let matches = app.clone().get_matches();

    let config_path: &str = matches.value_of("config").unwrap_or("diplomat.toml");
    info!(
        "==>> attempting to load configuration from '{}'",
        config_path
    );

    let config = config::load(config_path.to_string());
    if config.is_err() {
        error!("==>> failed loading the specified configuration file... exiting.")
    }

    let ccc = consul::Config::new().unwrap();
    let xxx = ConsulClient::new(ccc);

    match matches.subcommand() {
        ("client", Some(sub_m)) => {
            match sub_m.subcommand_name() {
                Some("eds") => {
                    let env = Arc::new(Environment::new(1));
                    let channel = ChannelBuilder::new(env).connect(&config.unwrap().client.address);
                    let client = EndpointDiscoveryServiceClient::new(channel);
                    let mut dr = DiscoveryRequest::new();
                    let res = client.fetch_endpoints(dr);
                    println!("eds {:?}", res);
                }
                Some("cds") => {
                    println!("cds is currently not implemented")
                }
                _ => {
                    let _ = app.clone().print_help();
                    println!("");
                }
            }
        }
        ("serve", Some(_)) => {
            ::server::start(config.unwrap(), xxx);
        }
        _ => {
            let _ = app.clone().print_help();
            println!("");
        }
    }
}

