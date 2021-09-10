use std::future::Future;

use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Error)]
pub enum CqrsError {
    #[error("communication channel closed")]
    ChannelClosed,
    #[error("unable to respond to sender")]
    SenderUnavailable,
}

impl<T> From<mpsc::error::SendError<T>> for CqrsError {
    fn from(_: mpsc::error::SendError<T>) -> Self {
        Self::ChannelClosed
    }
}

impl From<oneshot::error::RecvError> for CqrsError {
    fn from(_: oneshot::error::RecvError) -> Self {
        Self::ChannelClosed
    }
}

pub trait Request: Sized {
    type Result: Sized;
}

impl Request for () {
    type Result = ();
}

enum Envelope<C: Request, Q: Request> {
    Command { payload: C, tx: oneshot::Sender<C::Result> },
    Query { payload: Q, tx: oneshot::Sender<Q::Result> },
}

#[derive(Debug)]
pub struct Address<C: Request = (), Q: Request = ()> {
    tx: mpsc::Sender<Envelope<C, Q>>,
}

impl<C: Request, Q: Request> Address<C, Q> {
    pub async fn command(&self, payload: C) -> Result<C::Result, CqrsError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Envelope::Command { payload, tx }).await?;
        let result = rx.await?;
        Ok(result)
    }

    pub async fn query(&self, payload: Q) -> Result<Q::Result, CqrsError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Envelope::Query { payload, tx }).await?;
        let result = rx.await?;
        Ok(result)
    }
}

// Need an explicit impl of [`Clone`] because otherwise the compiler requires
// the generic parameters to also implement it.
impl<C: Request, Q: Request> Clone for Address<C, Q> {
    fn clone(&self) -> Self {
        Self { tx: self.tx.clone() }
    }
}

#[derive(Debug)]
pub struct Mailbox<C, Q, CFn, CFut, QFn, QFut>
where
    C: Request,
    Q: Request,
    CFn: Fn(C) -> CFut,
    CFut: Future<Output = C::Result>,
    QFn: Fn(Q) -> QFut,
    QFut: Future<Output = Q::Result>,
{
    rx: mpsc::Receiver<Envelope<C, Q>>,
    on_command: CFn,
    on_query: QFn,
}

impl<C, Q, CFn, CFut, QFn, QFut> Mailbox<C, Q, CFn, CFut, QFn, QFut>
where
    C: Request,
    Q: Request,
    CFn: Fn(C) -> CFut,
    CFut: Future<Output = C::Result>,
    QFn: Fn(Q) -> QFut,
    QFut: Future<Output = Q::Result>,
{
    pub async fn next(&mut self) -> Result<(), CqrsError> {
        match self.rx.recv().await {
            Some(Envelope::Command { payload, tx }) => {
                let resp = (self.on_command)(payload).await;
                tx.send(resp).map_err(|_| CqrsError::SenderUnavailable)
            }
            Some(Envelope::Query { payload, tx }) => {
                let resp = (self.on_query)(payload).await;
                tx.send(resp).map_err(|_| CqrsError::SenderUnavailable)
            }
            None => Err(CqrsError::ChannelClosed),
        }
    }
}

pub fn bounded<C, Q, CFn, CFut, QFn, QFut>(
    bound: usize,
    on_command: CFn,
    on_query: QFn,
) -> (Address<C, Q>, Mailbox<C, Q, CFn, CFut, QFn, QFut>)
where
    C: Request,
    Q: Request,
    CFn: Fn(C) -> CFut,
    CFut: Future<Output = C::Result>,
    QFn: Fn(Q) -> QFut,
    QFut: Future<Output = Q::Result>,
{
    let (tx, rx) = mpsc::channel(bound);
    (Address { tx }, Mailbox { rx, on_command, on_query })
}
