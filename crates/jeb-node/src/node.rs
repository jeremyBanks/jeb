#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use derive_more::{Deref, DerefMut};
use jeb_values::{Bytes, Item};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::{Receiver, Sender, channel};

pub type TaskHandle = tokio::task::JoinHandle<()>;

#[derive(Copy, Clone, Deref, DerefMut)]
pub struct SourceNode<T = Result<Item, &'static str>, F = fn(Sender<T>) -> TaskHandle  >
where F: FnOnce(Sender<T>) -> TaskHandle {
    #[deref]
    #[deref_mut]
    f: F,
    t: PhantomData<fn(Sender<T>) -> TaskHandle  >,
}

impl<T, F> SourceNode<T, F>
where F: FnOnce(Sender<T>) -> TaskHandle   {
    pub fn new(f: F) -> Self {
        Self {
            f,
            t: PhantomData,
        }
    }

    pub fn spawn(self) -> Receiver<T> {
        let (sender, receiver) = channel::<T>();
        let _handle = (self.f)(sender);
        receiver
    }
}

#[derive(Copy, Clone, Deref, DerefMut)]
pub struct SinkNode<T = Result<Item, &'static str>, F = fn(Receiver<T>) -> TaskHandle  >
where F: FnOnce(Receiver<T>) -> TaskHandle {
    #[deref]
    #[deref_mut]
    f: F,
    t: PhantomData<fn(Receiver<T>) -> TaskHandle>,
}

impl<T, F> SinkNode<T, F>
where F: FnOnce(Receiver<T>) -> TaskHandle   {
    pub fn new(f: F) -> Self {
        Self {
            f,
            t: PhantomData,
        }
    }

    pub fn spawn(self, input: Receiver<T>) -> TaskHandle {
        (self.f)(input)
    }
}

pub struct TransformNode<In = Item, Out = Result<Item, &'static str>, F = fn(Receiver<In>, Sender<Out>) -> TaskHandle  >
where F: FnOnce(Receiver<In>, Sender<Out>) -> TaskHandle {
    f: F,
    #[allow(clippy::type_complexity)]
    t: PhantomData<fn(Receiver<In>, Sender<Out>) -> TaskHandle>,
}

impl<In, Out, F> TransformNode<In, Out, F>
where F: FnOnce(Receiver<In>, Sender<Out>) -> TaskHandle   {
    pub fn new(f: F) -> Self {
        Self {
            f,
            t: PhantomData,
        }
    }

    pub fn spawn(self, input: Receiver<In>) -> Receiver<Out> {
        let (sender, receiver) = channel::<Out>();
        let _handle = (self.f)(input, sender);
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

pub fn sink<T, F, Fut>(f: F) -> SinkNode<T, impl FnOnce(Receiver<T>) -> TaskHandle>
where
    F: FnOnce(Receiver<T>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
    T: Send + 'static,
{
    SinkNode::new(move |receiver| tokio::spawn(f(receiver)))
}

pub fn transform<In, Out, F, Fut>(f: F) -> TransformNode<In, Out, impl FnOnce(Receiver<In>, Sender<Out>) -> TaskHandle>
where
    F: FnOnce(Receiver<In>, Sender<Out>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
    In: Send + 'static,
    Out: Send + 'static,
{
    TransformNode::new(move |receiver, sender| tokio::spawn(f(receiver, sender)))
}

pub fn read_source<R, F, Fut>(reader_fn: F) -> SourceNode<Result<Item, &'static str>, impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle>
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

pub fn write_sink<W, F, Fut>(writer_fn: F) -> SinkNode<Item, impl FnOnce(Receiver<Item>) -> TaskHandle>
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
                    writer.write_all(&bytes).await.expect("failed to write bytes");
                }
                Item::Text(text) => {
                    writer.write_all(text.as_bytes()).await.expect("failed to write text");
                }
                _ => panic!("write_sink received non-text/non-bytes item"),
            }
        }
    })
}

pub fn iter_source<I, T>(items: I) -> SourceNode<Result<T, &'static str>, impl FnOnce(Sender<Result<T, &'static str>>) -> TaskHandle>
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
