//! Listeners for incoming status updates arriving directly from monitored
//! devices. In reality these would come in all kinds of proprietary (mostly
//! binary) formats depending on the manufacturer, but we're using Bincode for
//! demonstration purposes.
//!
//! Both TCP and UDP listeners are provided. The UDP listener only supports one
//! status update per datagram, while the TCP listener can decode a stream of
//! one or more. Bincode-encoded payloads sent to the TCP listener must be
//! prefixed with their length in order for the decoder to properly determine
//! their bounds.

use std::{net::SocketAddr, time::Duration};

use async_bincode::AsyncBincodeReader;
use eyre::eyre;
use futures_util::stream::StreamExt;
use shared::data::Status;
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    sync::mpsc::Sender,
    time::timeout,
};
use tracing::{debug, info, warn};

#[tracing::instrument(skip(status_tx))]
pub async fn listen_tcp(
    addr: &SocketAddr,
    read_timeout: Duration,
    status_tx: Sender<Status>,
) -> eyre::Result<()> {
    info!("Starting TCP listener at http://{}:{}...", addr.ip(), addr.port());

    let listener = TcpListener::bind(addr).await?;

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, remote_addr)) => {
                    debug!(%remote_addr, "new incoming connection established");
                    let status_tx = status_tx.clone();
                    tokio::spawn(async move {
                        match process_status_stream(socket, read_timeout, status_tx, remote_addr)
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

#[tracing::instrument(skip(status_tx))]
async fn process_status_stream(
    stream: TcpStream,
    read_timeout: Duration,
    status_tx: Sender<Status>,
    remote_addr: SocketAddr,
) -> eyre::Result<()> {
    let mut reader = AsyncBincodeReader::<_, Status>::from(stream);
    while let Some(frame) = timeout(read_timeout, reader.next()).await? {
        let status = frame.map_err(|err| eyre!("failed to deserialize status: {}", err))?;
        debug!(
            %remote_addr,
            source_id = %status.source_id,
            timestamp = %status.timestamp,
            "received status: {:?}",
            status
        );
        status_tx.send(status).await.expect("Status channel");
    }
    Ok(())
}

#[tracing::instrument(skip(status_tx))]
pub async fn listen_udp(addr: &SocketAddr, status_tx: Sender<Status>) -> eyre::Result<()> {
    info!("Starting UDP listener at http://{}:{}...", addr.ip(), addr.port());

    let socket = UdpSocket::bind(addr).await?;
    // A valid `Status` with all fields specified is Bincode-encoded into ~70
    // bytes, so this buffer should be sufficient to store a single instance.
    let mut buf = [0; 128];

    tokio::spawn(async move {
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, remote_addr)) => match bincode::deserialize::<Status>(&buf[0..len]) {
                    Ok(status) => {
                        debug!(
                            %remote_addr,
                            source_id = %status.source_id,
                            timestamp = %status.timestamp,
                            "received status: {:?}",
                            status
                        );
                        status_tx.send(status).await.expect("Status channel");
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
