#![allow(unused)]

use jeb_values::{Bytes, Item};
use macro_rules_attribute::apply;
use tokio::task::JoinHandle;

use crate::{Receiver, Sender, channel::channel};

type ItemResult<T = Item> = Result<T, Item>;

macro_rules! node {
    {
        $(
            $( #[$attr:meta] )*
            $pub:vis
            $(async fn $async_name:ident)?
            $(fn $name:ident)?
            (
                input: $input_ty:ty,
                output: $output_ty:ty
                $(, $rest_ident:ident: $rest_ty:ty)*
            )
            $(-> $return:ty)?
            $body:block
        )+
    } => {
        $(
            $( #[$attr] )*
            $pub
            $(async fn $async_name)?
            $(fn $name)?
            (
                input: $input_ty,
                output: $output_ty
                $(, $rest_ident: $rest_ty)*
            )
            $(-> $return)?
            $body

            // mod $($name)? $($async_name)? {
            //     use super::*;

            //     pub fn spawn(
            //         input: $input_ty,
            //         $( $rest_ident: $rest_ty ),*
            //     ) -> $output_ty {
            //         let (sender, receiver) = $crate::channel::<$output_ty>();
            //         tokio::spawn(
            //             super::$($name)?$($async_name)?(
            //                 input,
            //                 sender,
            //                 $( $rest_ident ),*
            //             )
            //         );
            //         receiver
            //     }
            // }
        )+
    }
}

use node;

trait InputManyItemsOutputOneItemResult {
    fn spawn(self, input: Vec<Receiver<Item>>) -> Receiver<ItemResult>;
}
impl<T, F> InputManyItemsOutputOneItemResult for T
where
    T: Fn(Vec<Receiver<Item>>, Sender<ItemResult>) -> F,
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn spawn(self, input: Vec<Receiver<Item>>) -> Receiver<ItemResult> {
        let (sender, receiver) = channel::<ItemResult>();
        tokio::spawn((self)(input, sender));
        receiver
    }
}

type InputOneBytes {
    fn spawn(self, input: Receiver<Item>) -> JoinHandle<()>;
}

trait OutputOneBytesResult {
    fn spawn(self) -> JoinHandle<()>;
}
impl<T, F> OutputOneBytesResult for T
where
    T: Fn(Vec<Receiver<Item>>, Sender<ItemResult>) -> F,
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn spawn(self) -> Receiver<ItemResult<()>> {
        let (sender, receiver) = channel::<ItemResult>();
        tokio::spawn((self)(input, sender));
        receiver
    }
}

// impl NodeFn for

// node! {
pub async fn merge(input: Vec<Receiver<Item>>, output: Sender<ItemResult>) {}

pub fn chunks(
    input: Receiver<Item>,
    output: Sender<ItemResult>,
    rest: (),
    size: usize,
) -> impl Future<Output = ()> {
    async { unimplemented!() }
}

pub fn stdin(input: (), output: Sender<Bytes>) -> impl Future<Output = ()> {
    async { unimplemented!() }
}
// }

#[test]
fn test() {
    let d = merge.spawn(vec![stdin.spawn(), stdin.spawn()]);

    // let _ = merge(vec![], Sender {});
    // let _ = chunks;
    // let _ = stdin;

    // tokio::runtime::Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap()
    //     .block_on(example());
}
