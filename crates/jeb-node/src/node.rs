
use std::{marker::PhantomData, sync::LazyLock};

use derive_more::{Deref, DerefMut, From};
use jeb_values::{Bytes, Item};
use tokio::io::AsyncReadExt;
use crate::{Receiver, Sender, channel};

type TaskHandle = tokio::task::JoinHandle<()>;

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

pub fn source<T, F, FutureT>(f: F) -> SourceNode<T, impl FnOnce(Sender<T>) -> FutureT  >
where F: FnOnce(Sender<T>) -> FutureT,
T: 'static + Send,
FutureT: std::future::Future<Output = ()> + 'static + Send,
F: 'static + Send {
    // SourceNode::new(|output| {
    //     tokio::spawn(async move {
    //         (f)(output).await.unwrap()
    //     })
    // })
}

pub fn stdin() -> SourceNode {
    source(|output| async {
        let mut stdin = tokio::io::stdin();
        let mut buffer = [0u8; 65_536];

        loop {
            match stdin.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let bytes = Bytes::from(&buffer[..n]);
                    if output.send(Ok(Item::Bytes(bytes))).await.is_err() {
                        break;
                    }
                }
                Err(_err) => {
                    let _ = output.send(Err("Failed to read from stdin")).await;
                    break;
                }
            }
        }
    })
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
