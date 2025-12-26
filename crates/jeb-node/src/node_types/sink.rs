use std::marker::PhantomData;

use derive_more::{Deref, DerefMut};
use jeb_values::Item;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{channel::Receiver, node_types::TaskHandle};



#[derive(Copy, Clone, Deref, DerefMut)]
pub struct SinkNode<T = Result<Item, &'static str>, F = fn(Receiver<T>) -> TaskHandle>
where
    F: FnOnce(Receiver<T>) -> TaskHandle,
{
    #[deref]
    #[deref_mut]
    f: F,
    t: PhantomData<fn(Receiver<T>) -> TaskHandle>,
}

impl<T, F> SinkNode<T, F>
where
    F: FnOnce(Receiver<T>) -> TaskHandle,
{
    pub fn new(f: F) -> Self {
        Self { f, t: PhantomData }
    }

    pub fn spawn(self, input: Receiver<T>) -> TaskHandle {
        (self.f)(input)
    }
}


pub fn write_sink<W, F, Fut>(
    writer_fn: F,
) -> SinkNode<Item, impl FnOnce(Receiver<Item>) -> TaskHandle>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = std::io::Result<W>> + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    sink(move |mut input| async move {
        let mut writer = writer_fn().await.expect("failed to open writer");

        while let Some(item) = input.pull().await {
            match item {
                Item::Bytes(bytes) => {
                    writer
                        .write_all(&bytes)
                        .await
                        .expect("failed to write bytes");
                }
                Item::Text(text) => {
                    writer
                        .write_all(text.as_bytes())
                        .await
                        .expect("failed to write text");
                }
                _ => panic!("write_sink received non-text/non-bytes item"),
            }
        }
    })
}


pub fn sink<T, F, Fut>(f: F) -> SinkNode<T, impl FnOnce(Receiver<T>) -> TaskHandle>
where
    F: FnOnce(Receiver<T>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
    T: Send + 'static,
{
    SinkNode::new(move |receiver| tokio::spawn(f(receiver)))
}
