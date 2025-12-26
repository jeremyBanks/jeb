use std::marker::PhantomData;

use jeb_values::Item;

use crate::{
    channel::{Receiver, Sender, channel},
    node_types::TaskHandle,
};


pub struct TransformNode<
    In = Item,
    Out = Result<Item, &'static str>,
    F = fn(Receiver<In>, Sender<Out>) -> TaskHandle,
> where
    F: FnOnce(Receiver<In>, Sender<Out>) -> TaskHandle,
{
    f: F,
    #[allow(clippy::type_complexity)]
    t: PhantomData<fn(Receiver<In>, Sender<Out>) -> TaskHandle>,
}

impl<In, Out, F> TransformNode<In, Out, F>
where
    F: FnOnce(Receiver<In>, Sender<Out>) -> TaskHandle,
{
    pub fn new(f: F) -> Self {
        Self { f, t: PhantomData }
    }

    pub fn spawn(self, input: Receiver<In>) -> Receiver<Out> {
        let (sender, receiver) = channel::<Out>();
        let _handle = (self.f)(input, sender);
        receiver
    }
}

pub fn transform<In, Out, F, Fut>(
    f: F,
) -> TransformNode<In, Out, impl FnOnce(Receiver<In>, Sender<Out>) -> TaskHandle>
where
    F: FnOnce(Receiver<In>, Sender<Out>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
    In: Send + 'static,
    Out: Send + 'static,
{
    TransformNode::new(move |receiver, sender| tokio::spawn(f(receiver, sender)))
}
