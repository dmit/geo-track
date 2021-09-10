//! Listeners for incoming status updates arriving directly from monitored
//! devices. In reality these would come in all kinds of proprietary (mostly
//! binary) formats depending on the manufacturer, but we're using CBOR for
//! demonstration purposes.
//!
//! Both TCP and UDP listeners are provided. The UDP listener only supports one
//! status update per datagram, while the TCP listener can decode a stream of
//! one or more payloads.

use std::{net::SocketAddr, time::Duration};

use futures_util::stream::StreamExt;
use shared::data::Status;
use thiserror::Error;
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    time::timeout,
};
use tokio_util::codec::FramedRead;
use tracing::{debug, error, info, warn};

use crate::{
    cq::CqrsError,
    storage::{StorageCommand, StorageError, StorageHandler},
};

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("packet deserialization error")]
    Deserialize(#[from] tokio_serde_cbor::Error),
    #[error("internal communication error")]
    Internal(#[from] CqrsError),
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("internal storage error")]
    Storage(#[from] StorageError),
    #[error("timeout elapsed")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

pub type Result<T> = std::result::Result<T, IngestError>;

/// Bind to the specified network address and start listening for incoming
/// [`Status`] packets over TCP. Incoming packets are decoded and forwarded for
/// storage and further processing.
#[tracing::instrument(skip(handler))]
pub async fn listen_tcp(
    addr: &SocketAddr,
    read_timeout: Duration,
    handler: StorageHandler,
) -> Result<()> {
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
    handler: StorageHandler,
) -> Result<()> {
    let mut reader = FramedRead::new(stream, tokio_serde_cbor::Decoder::<Status>::new());
    while let Some(frame) = timeout(read_timeout, reader.next()).await? {
        let status = frame?;
        debug!(
            %remote_addr,
            source_id = %status.source_id,
            timestamp = %status.timestamp,
            "received status: {:?}",
            status
        );
        handler.command(StorageCommand::PersistStatus(status)).await??;
    }
    Ok(())
}

/// Bind to the specified network address and start listening for incoming
/// [`Status`] packets over UDP. Incoming packets are decoded and forwarded for
/// storage and further processing.
#[tracing::instrument(skip(handler))]
pub async fn listen_udp(addr: &SocketAddr, handler: StorageHandler) -> Result<()> {
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
                        if let Err(err) =
                            handler.command(StorageCommand::PersistStatus(status)).await
                        {
                            error!(%err, "failed to handle incoming status");
                        }
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
