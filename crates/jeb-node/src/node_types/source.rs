use std::marker::PhantomData;

use derive_more::{Deref, DerefMut};
use jeb_values::{Bytes, Item};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{channel::{Receiver, Sender, channel}, node_types::TaskHandle};



#[derive(Copy, Clone, Deref, DerefMut)]
pub struct SourceNode<T = Result<Item, &'static str>, F = fn(Sender<T>) -> TaskHandle>
where
    F: FnOnce(Sender<T>) -> TaskHandle,
{
    #[deref]
    #[deref_mut]
    f: F,
    t: PhantomData<fn(Sender<T>) -> TaskHandle>,
}

impl<T, F> SourceNode<T, F>
where
    F: FnOnce(Sender<T>) -> TaskHandle,
{
    pub fn new(f: F) -> Self {
        Self { f, t: PhantomData }
    }

    pub fn spawn(self) -> Receiver<T> {
        let (sender, receiver) = channel::<T>();
        let _handle = (self.f)(sender);
        receiver
    }
}


pub fn source<T, F, Fut>(f: F) -> SourceNode<T, impl FnOnce(Sender<T>) -> TaskHandle>
where
    F: FnOnce(Sender<T>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
    T: Send + 'static,
{
    SourceNode::new(move |sender| tokio::spawn(f(sender)))
}
pub fn read_source<R, F, Fut>(
    reader_fn: F,
) -> SourceNode<
    Result<Item, &'static str>,
    impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle,
>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = std::io::Result<R>> + Send + 'static,
    R: AsyncRead + Unpin + Send + 'static,
{
    source(move |output| async move {
        match reader_fn().await {
            Ok(mut reader) => {
                let mut buffer = [0u8; 65_536];

                loop {
                    match reader.read(&mut buffer).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let bytes = Bytes::from(&buffer[..n]);
                            if output.push_value(Item::Bytes(bytes)).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => {
                            let _ = output.push_error("failed to read").await;
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                let _ = output.push_error("failed to open").await;
            }
        }
    })
}


pub fn iter_source<I, T>(
    items: I,
) -> SourceNode<Result<T, &'static str>, impl FnOnce(Sender<Result<T, &'static str>>) -> TaskHandle>
where
    I: IntoIterator<Item = T> + Send + 'static,
    I::IntoIter: Send,
    T: Send + 'static,
{
    source(|sender| async move {
        for item in items {
            if sender.push_value(item).await.is_err() {
                break;
            }
        }
    })
}
