// Based on:
// https://github.com/tokio-rs/tokio/blob/master/examples/proxy.rs
// https://github.com/tokio-rs/tokio/blob/tokio-1.43.0/examples/proxy.rs
// https://github.com/rust-vsock/tokio-vsock
// https://github.com/rust-vsock/vsock-rs

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use futures::FutureExt;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_vsock::{VsockAddr, VsockStream};

use pf_proxy::{addr_info::AddrInfo, utils};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// ip address of the listener side (e.g. 127.0.0.1:1200)
    #[clap(short, long, value_parser)]
    ip_addr: String,

    /// vsock address of the upstream side, usually the other side of the transparent proxy (e.g. 3:1200)
    #[clap(short, long, value_parser)]
    vsock_addr: String,
}

pub async fn proxy(listen_addr: &str, server_addr: VsockAddr) -> Result<()> {
    println!("Listening on: {:?}", listen_addr);
    println!("Proxying to: {:?}", server_addr);

    let listener = TcpListener::bind(listen_addr)
        .await
        .context("Failed to bind listener: malformed listening address:port")?;

    while let Ok((inbound, _)) = listener.accept().await {
        let transfer = transfer(inbound, server_addr).map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer data: error={:?}", e);
            }
        });

        tokio::spawn(transfer);
    }

    Ok(())
}

async fn transfer(mut inbound: TcpStream, proxy_addr: VsockAddr) -> Result<()> {
    let inbound_addr = inbound
        .peer_addr()
        .context("could not fetch inbound address from TCP stream")?
        .to_string();

    // Read original destination from inbound TCP stream
    let orig_dst = inbound
        .get_original_dst()
        .ok_or(anyhow!("Failed to retrieve original destination from TCP stream"))?;
    println!("Original destination: {:?}", orig_dst);

    println!("Proxying to: {:?}", proxy_addr);

    let mut outbound = VsockStream::connect(proxy_addr)
        .await
        .context("failed to connect to vsock endpoint")?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    // send original destination ip and port to upstream proxy on host
    match orig_dst {
        std::net::SocketAddr::V4(v4) => {
            wo.write_u8(4_u8).await?;
            wo.write_u32_le(v4.ip().to_bits()).await?
        },
        std::net::SocketAddr::V6(v6) => {
            wo.write_u8(6_u8).await?;
            wo.write_u128_le(v6.ip().to_bits()).await?
        },
    };
    wo.write_u16_le(orig_dst.port()).await?;

    /*
    // send original destination ip and port to upstream proxy on host
    wo.write_u32_le(if let std::net::SocketAddr::V4(v4) = orig_dst {
        Ok((*v4.ip()).into())
    } else {
        Err(anyhow!("Can't send original_dst: received ipv6 address for original_dst"))
    }?)
    .await?;
    wo.write_u16_le(orig_dst.port()).await?;
    */

    // Send request to upstream resource
    let client_to_server = async {
        io::copy(&mut ri, &mut wo)
            .await
            .context("error in ip to vsock copy")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        println!("ip to vsock IO copy done, from {:?} to {:?}, with original_dst={:?} from inbound TCP stream",
                 inbound_addr, proxy_addr, orig_dst);
        wo.shutdown().await
    };

    // Receive response from upstream resource and write it to inbound connection input stream
    let server_to_client = async {
        io::copy(&mut ro, &mut wi)
            .await
            .context("error in vsock to ip copy")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        println!("vsock to ip IO copy done, from {:?} to {:?}, with original_dst={:?} from inbound TCP stream",
                 proxy_addr, inbound_addr, orig_dst);
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client).with_context(|| {
        format!(
            "error in connection between inbound ip address {:?} with original_dst={:?} and outbound vsock endpoint {:?}",
            inbound_addr, orig_dst, proxy_addr
        )
    })?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let vsock_addr = utils::split_vsock(&cli.vsock_addr)?;
    proxy(&cli.ip_addr, vsock_addr).await?;

    Ok(())
}
