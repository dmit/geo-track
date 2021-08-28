//! Listeners for incoming status updates arriving directly from monitored
//! devices. In reality these would come in all kinds of proprietary (mostly
//! binary) formats depending on the manufacturer, but we're using CBOR for
//! demonstration purposes.
//!
//! Both TCP and UDP listeners are provided. The UDP listener only supports one
//! status update per datagram, while the TCP listener can decode a stream of
//! one or more payloads.

use std::future::Future;
use std::{net::SocketAddr, time::Duration};

use async_trait::async_trait;
use eyre::eyre;
use futures_util::stream::StreamExt;
use shared::data::Status;
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    time::timeout,
};
use tokio_util::codec::FramedRead;
use tracing::{debug, info, warn};

/// This describes an interface that allows ingestion points to pass on incoming
/// well-formed [`Status`] packets further into the processing system without
/// exposing any implementation details.
#[async_trait]
pub trait StatusHandler: Clone + Send + Sync + 'static {
    /// Hand off the [`Status`] packet for processing.
    async fn run(&self, status: Status) -> eyre::Result<()>;
}

#[async_trait]
impl<FN, FUT> StatusHandler for FN
where
    FN: Fn(Status) -> FUT + Clone + Send + Sync + 'static,
    FUT: Future<Output = eyre::Result<()>> + Send,
{
    async fn run(&self, status: Status) -> eyre::Result<()> {
        self(status).await
    }
}

/// Bind to the specified network address and start listening for incoming
/// [`Status`] packets over TCP. Incoming packets are decoded and forwarded for
/// storage and further processing.
#[tracing::instrument(skip(handler))]
pub async fn listen_tcp(
    addr: &SocketAddr,
    read_timeout: Duration,
    handler: impl StatusHandler,
) -> eyre::Result<()> {
    info!("Starting TCP listener at http://{}:{}...", addr.ip(), addr.port());

    let listener = TcpListener::bind(addr).await?;

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, remote_addr)) => {
                    debug!(%remote_addr, "new incoming connection established");
                    let handler = handler.clone();
                    tokio::spawn(async move {
                        match process_status_stream(socket, read_timeout, remote_addr, handler)
                            .await
                        {
                            Ok(()) => {
                                debug!("connection closed");
                            }
                            Err(err) => {
                                debug!(%err, "connection closed");
                            }
                        }
                    });
                }
                Err(err) => {
                    warn!(%err, "failed to establish connection");
                }
            }
        }
    });

    Ok(())
}

#[tracing::instrument(skip(handler))]
async fn process_status_stream(
    stream: TcpStream,
    read_timeout: Duration,
    remote_addr: SocketAddr,
    handler: impl StatusHandler,
) -> eyre::Result<()> {
    let mut reader = FramedRead::new(stream, tokio_serde_cbor::Decoder::<Status>::new());
    while let Some(frame) = timeout(read_timeout, reader.next()).await? {
        let status = frame.map_err(|err| eyre!("failed to deserialize status: {}", err))?;
        debug!(
            %remote_addr,
            source_id = %status.source_id,
            timestamp = %status.timestamp,
            "received status: {:?}",
            status
        );
        handler.run(status).await.expect("Status channel");
    }
    Ok(())
}

/// Bind to the specified network address and start listening for incoming
/// [`Status`] packets over UDP. Incoming packets are decoded and forwarded for
/// storage and further processing.
#[tracing::instrument(skip(handler))]
pub async fn listen_udp(addr: &SocketAddr, handler: impl StatusHandler) -> eyre::Result<()> {
    info!("Starting UDP listener at http://{}:{}...", addr.ip(), addr.port());

    let socket = UdpSocket::bind(addr).await?;
    // A valid `Status` with all fields specified is CBOR-encoded into ~100
    // bytes, so this buffer should be sufficient to store a single instance
    // while also not blowing the stack.
    let mut buf = [0; 128];

    tokio::spawn(async move {
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, remote_addr)) => match serde_cbor::from_slice::<Status>(&buf[0..len]) {
                    Ok(status) => {
                        debug!(
                            %remote_addr,
                            source_id = %status.source_id,
                            timestamp = %status.timestamp,
                            "received status: {:?}",
                            status
                        );
                        handler.run(status).await.expect("Status channel");
                    }
                    Err(err) => {
                        debug!(%remote_addr, %err, "failed to deserialize status");
                    }
                },
                Err(err) => {
                    debug!(%err, "failed to read datagram");
                }
            }
        }
    });

    Ok(())
}
