use jeb_values::Item;

use crate::{Sender, SourceNode, TaskHandle, read_source};

pub fn stdin() -> SourceNode<
    Result<Item, &'static str>,
    impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle,
> {
    read_source(|| async { Ok(tokio::io::stdin()) })
}
