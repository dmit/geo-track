//! A TCP-based listener for incoming status updates arriving directly from
//! monitored devices. In reality these would come in all kinds of weird
//! (mostly binary) formats depending on the manufacturer, but we're using
//! Bincode for demonstration purposes.

use std::{net::SocketAddr, time::Duration};

use async_bincode::AsyncBincodeReader;
use eyre::eyre;
use futures_util::stream::StreamExt;
use shared::data::Status;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
    time::timeout,
};
use tracing::{debug, info, warn};

#[tracing::instrument(skip(status_tx))]
pub async fn listen(
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
                        match process_status_stream(socket, read_timeout, status_tx).await {
                            Ok(()) => {
                                debug!("connection closed");
                            }
                            Err(err) => {
                                debug!(%err, "connection closed: {}", err);
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
) -> eyre::Result<()> {
    let mut stream = AsyncBincodeReader::from(stream);
    while let Some(frame) = timeout(read_timeout, stream.next()).await? {
        let frame = frame.map_err(|err| eyre!("failed to deserialize status: {}", err))?;
        status_tx.send(frame).await?;
    }
    Ok(())
}
