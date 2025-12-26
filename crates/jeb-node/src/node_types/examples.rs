#![allow(unused)]

use jeb_values::{Bytes, Item};
use macro_rules_attribute::apply;

use crate::{Receiver, Sender};

type ItemResult<T=Item> = Result<T, Item>;

macro_rules! node {
    {
        $(
            $( #[$attr:meta] )*
            $pub:vis
            $(async $async_vis:vis)?
            fn $name:ident
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
            $(async $async_vis)?
            fn $name
            (
                input: $input_ty,
                output: $output_ty
                $(, $rest_ident: $rest_ty)*
            )
            $(-> $return)?
            $body

            mod $name

        )+
    }
}

use node;

pub trait NodeFn {
    fn name() -> &'static str;
    fn max_inputs() -> Option<usize>;
    fn min_inputs() -> Option<usize>;
    fn max_outputs() -> Option<usize>;
    fn min_outputs() -> Option<usize>;

    const MAX_INPUTS: Option<usize> = Some(0);
    const MIN_INPUTS: Option<usize> = Self::MAX_INPUTS;

    const MAX_OUTPUTS: Option<usize> = Some(0);
    const MIN_OUTPUTS: Option<usize> = Self::MAX_OUTPUTS;

    type InputType;
    type OutputType;
}

// impl NodeFn for

node! {
    pub async fn merge(input: Vec<Receiver<Item>>, output: Sender<ItemResult>)  {

    }

    pub fn chunks(input: Receiver<Item>, output: Sender<ItemResult>, rest: (), size: usize) -> impl Future<Output = ()> {
        async { unimplemented!() }
    }

    pub fn stdin(input: (), output: Sender<Bytes>) -> impl Future<Output = ()> {
        async { unimplemented!() }
    }
}

#[test]
fn test() {
    // let _ = merge(vec![], Sender {});
    // let _ = chunks;
    // let _ = stdin;

    // tokio::runtime::Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap()
    //     .block_on(example());
}