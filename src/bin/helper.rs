use clap::Parser;
use hyper::http::uri::Scheme;
use raw_ipa::cli::Verbosity;
use raw_ipa::net::{bind_mpc_helper_server, BindTarget};
use std::error::Error;
use std::net::SocketAddr;
use std::panic;
use tracing::info;

#[derive(Debug, Parser)]
#[clap(name = "mpc-helper", about = "CLI to start an MPC helper endpoint")]
struct Args {
    /// Configure logging.
    #[clap(flatten)]
    logging: Verbosity,

    /// Port to listen. If not specified, will ask Kernel to assign the port
    #[arg(short, long)]
    port: Option<u16>,

    /// Indicates whether to start HTTP or HTTPS endpoint
    #[arg(short, long, default_value = "http")]
    scheme: Scheme,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let _handle = args.logging.setup_logging();

    // decide what protocol we're going to use here
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port.unwrap_or(0)));
    let target = match args.scheme.as_str() {
        "http" => BindTarget::Http(addr),
        #[cfg(feature = "self-signed-certs")]
        "https" => {
            let config = raw_ipa::net::tls_config_from_self_signed_cert().await?;
            BindTarget::Https(addr, config)
        }
        _ => {
            panic!("\"{}\" protocol is not supported", args.scheme)
        }
    };

    // start server
    let (addr, server_handle) = bind_mpc_helper_server(target).await;
    info!(
        "listening to {}://{}, press Enter to quit",
        args.scheme, addr
    );
    let _ = std::io::stdin().read_line(&mut String::new())?;
    server_handle.abort();

    Ok(())
}
