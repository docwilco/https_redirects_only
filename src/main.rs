use clap::Parser;
use log::{LevelFilter, info};
use simple_logger::SimpleLogger;
use std::{net::SocketAddr, str::FromStr};
use warp::{host::Authority, http::Uri, path::FullPath, reply::Reply, Filter};

/// Redirect all incoming HTTP requests to HTTPS
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value_t = 80, value_parser)]
    port: u16,

    /// Bind address
    #[clap(short, long, default_value = "0.0.0.0", value_parser)]
    address: String,
}

// redirect all http requests to https
#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("https_redirects_only", LevelFilter::Info)
        .env()
        .init()
        .unwrap();

    // parse the command line arguments for bind address and port
    let args = Args::parse();
    let addr = SocketAddr::new(
        args.address.parse().expect("Invalid bind address"),
        args.port,
    );
    info!(target: "https_redirects_only::server", "Listening on {}", addr);

    // setup warp route for all http requests
    let route = warp::path::full()
        .and(warp::host::optional())
        .map(|path: FullPath, host: Option<Authority>| match host {
            Some(host) => Box::new(warp::redirect(
                Uri::from_str(&format!("https://{}{}", host.as_str(), path.as_str()))
                    .expect("invalid uri"),
            )) as Box<dyn Reply>,
            None => Box::new(warp::reply::with_status(
                "Missing Host or Authority header",
                warp::http::StatusCode::BAD_REQUEST,
            )) as Box<dyn Reply>,
        })
        .with(warp::log("https_redirects_only::request"));

    warp::serve(route).run(addr).await;
}
