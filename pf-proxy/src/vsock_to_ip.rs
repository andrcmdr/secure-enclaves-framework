// Based on:
// https://github.com/tokio-rs/tokio/blob/master/examples/proxy.rs
// https://github.com/tokio-rs/tokio/blob/tokio-1.43.0/examples/proxy.rs
// https://github.com/rust-vsock/tokio-vsock
// https://github.com/rust-vsock/vsock-rs

use anyhow::{Context, Result};
use clap::Parser;
use futures::FutureExt;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_vsock::{VsockAddr, VsockListener, VsockStream};

use pf_proxy::utils;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// vsock address of the listener side (e.g. 88:4000)
    #[clap(short, long, value_parser)]
    vsock_addr: String,

    /// ip address of the upstream side (e.g. 127.0.0.1:4000)
    #[clap(short, long, value_parser)]
    ip_addr: String,
}

pub async fn proxy(listen_addr: VsockAddr, server_addr: String) -> Result<()> {
    println!("Listening on: {:?}", listen_addr);
    println!("Proxying to: {:?}", server_addr);

    let mut listener = VsockListener::bind(listen_addr)
        .context("Failed to bind listener to vsock: incorrect CID:port")?;

    while let Ok((inbound, _)) = listener.accept().await {
        let transfer = transfer(inbound, server_addr.clone()).map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer data: error={:?}", e);
            }
        });

        tokio::spawn(transfer);
    }

    Ok(())
}

async fn transfer(mut inbound: VsockStream, proxy_addr: String) -> Result<()> {
    let inbound_addr = inbound
        .local_addr() // .peer_addr()
        .context("could not fetch inbound address from vsock stream")?
        .to_string();

    println!("Proxying to: {:?}", proxy_addr);

    let mut outbound = TcpStream::connect(proxy_addr.clone())
        .await
        .context("failed to connect to TCP endpoint")?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    // Send request to upstream resource
    let client_to_server = async {
        io::copy(&mut ri, &mut wo)
            .await
            .context("error in vsock to ip copy")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        println!("vsock to ip IO copy done, from {:?} to {:?}", inbound_addr, proxy_addr);
        wo.shutdown().await
    };

    // Receive response from upstream resource and write it to inbound connection input stream
    let server_to_client = async {
        io::copy(&mut ro, &mut wi)
            .await
            .context("error in ip to vsock copy")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        println!("ip to vsock IO copy done, from {:?} to {:?}", proxy_addr, inbound_addr);
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client).with_context(|| {
        format!(
            "error in connection between inbound vsock endpoint {:?} and outbound ip address {:?}",
            inbound_addr, proxy_addr
        )
    })?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let vsock_addr = utils::split_vsock(&cli.vsock_addr)?;
    proxy(vsock_addr, cli.ip_addr).await?;

    Ok(())
}
