#![allow(unused)]

use {
    crate::{
        Item,
        Receiver,
        Sender,
        channel::channel,
    },
    async_stream::stream,
    futures::{
        Stream,
        StreamExt,
    },
    jeb_value::Bytes,
    macro_rules_attribute::apply,
    tokio::task::JoinHandle,
};

type ItemResult<T = Item> = Result<T, Item>;

pub async fn merge(
    input: Vec<Receiver<Item>>,
    output: Sender<ItemResult>,
) -> impl Stream<Item = ItemResult> {
    stream! {
        yield Err(Item::from("unimplemented!"));
    }
}

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
    // let d = merge.spawn(vec![stdin.spawn(), stdin.spawn()]);

    // let _ = merge(vec![], Sender {});
    // let _ = chunks;
    // let _ = stdin;

    // tokio::runtime::Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap()
    //     .block_on(example());
}
