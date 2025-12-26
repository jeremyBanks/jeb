#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use derive_more::{Deref, DerefMut};
use jeb_values::{Bytes, Item};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::{Receiver, Sender, channel};

type TaskHandle = tokio::task::JoinHandle<()>;


trait Node: Sized {
    fn stack_spawn(self, stack: &mut Vec<Receiver>) -> Result<(), &'static str>;
}

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

pub fn read_source<R>(reader: R) -> SourceNode<Result<Item, &'static str>, impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    source(move |output| async move {
        let mut reader = reader;
        let mut buffer = [0u8; 65_536];

        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let bytes = Bytes::from(&buffer[..n]);
                    if output.send(Ok(Item::Bytes(bytes))).await.is_err() {
                        break;
                    }
                }
                Err(_err) => {
                    let _ = output.send(Err("Failed to read")).await;
                    break;
                }
            }
        }
    })
}

pub fn write_sink<W>(writer: W) -> SinkNode<Result<Item, &'static str>, impl FnOnce(Receiver<Result<Item, &'static str>>) -> TaskHandle>
where
    W: AsyncWrite + Unpin + Send + 'static,
{
    sink(move |mut input| async move {
        let mut writer = writer;

        while let Some(item) = input.recv().await {
            match item {
                Ok(Item::Bytes(bytes)) => {
                    if writer.write_all(&bytes).await.is_err() {
                        break;
                    }
                }
                Ok(_) => {
                    // Ignore non-bytes items
                }
                Err(_err) => {
                    break;
                }
            }
        }
    })
}


impl<T, E: std::fmt::Debug> Receiver<Result<T, E>> where E: Send + 'static, T: Send + 'static {
    /// Takes a stream of Result<T, E> and splits it into two separate streams,
    /// one for the Ok values and one for the Err values.
    pub fn out_and_err(self) -> (Receiver<T>, Receiver<E>) {
        let (ok_sender, ok_receiver) = channel::<T>();
        let (err_sender, err_receiver) = channel::<E>();

        tokio::spawn(async move {
            let mut receiver = self;
            while let Some(item) = receiver.recv().await {
                match item {
                    Ok(ok) => {
                        if ok_sender.send(ok).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        if err_sender.send(err).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        (ok_receiver, err_receiver)
    }

    /// Takes a stream of Result<T, E> and closes after the first Err.
    pub fn fail_fast(self) -> Receiver<T> {
        let (ok_sender, ok_receiver) = channel::<T>();

        tokio::spawn(async move {
            let mut receiver = self;
            while let Some(item) = receiver.recv().await {
                match item {
                    Ok(ok) => {
                        if ok_sender.send(ok).await.is_err() {
                            break;
                        }
                    }
                    Err(_err) => {
                        break;
                    }
                }
            }
        });

        ok_receiver
    }

    /// Takes a stream of Result<T, E> and unwraps the Ok values, panicking on Err.
    pub fn unwrapping(self) -> Receiver<T> {
        let (ok_sender, ok_receiver) = channel::<T>();

        tokio::spawn(async move {
            let mut receiver = self;
            while let Some(item) = receiver.recv().await {
                ok_sender.send(item.unwrap()).await.unwrap()
            }
        });

        ok_receiver
    }
}

pub fn stdin() -> SourceNode<Result<Item, &'static str>, impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle> {
    read_source(tokio::io::stdin())
}

pub fn stdout() -> SinkNode<Result<Item, &'static str>, impl FnOnce(Receiver<Result<Item, &'static str>>) -> TaskHandle> {
    write_sink(tokio::io::stdout())
}

pub fn stderr() -> SinkNode<Result<Item, &'static str>, impl FnOnce(Receiver<Result<Item, &'static str>>) -> TaskHandle> {
    write_sink(tokio::io::stderr())
}

// async fn example() {
//     let stdin = stdin();
//     let stdin = stdin.spawn();

//     stdout().spawn(input).await;
// }

// impl<T, F> Node for SourceNode<T, F>
// where F: Fn(Sender<T>) -> TaskHandle   {
//     fn spawn(&mut self) -> TaskHandle {
//         let (sender receiver, input) = channel::<T>();
//         (self.f)(output)
//     }
// }

// static_assertions::assert_obj_safe!(Node);

// pub trait SourceNode<T = Result<Item, &'static str>> {
//     fn spawn_source(&mut self) -> (TaskHandle, Output<T>);
// }

// impl<T> SourceNode<T> for fn(Input<T>) -> TaskHandle {
//     fn spawn_source(&mut self) -> (TaskHandle, Output<T>) {
//         let (output, input) = channel::<T>();
//         let handle = (self)(input);
//         (handle, output)
//     }
// }

// pub trait SinkNode<T = Item> {
//     fn spawn_source(&mut self) -> (TaskHandle, Input<T>);
// }

// impl<T> SinkNode<T> for fn(Output<T>) -> TaskHandle {
//     fn spawn_source(&mut self) -> (TaskHandle, Input<T>) {
//         let (output, input) = channel::<T>();
//         let handle = (self)(output);
//         (handle, input)
//     }
// }
