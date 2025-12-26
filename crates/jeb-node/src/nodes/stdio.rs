use jeb_values::Item;

use crate::{Receiver, Sender, SinkNode, SourceNode, TaskHandle, read_source, write_sink};

pub fn stdin() -> SourceNode<Result<Item, &'static str>, impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle> {
    read_source(|| async { Ok(tokio::io::stdin()) })
}

pub fn stdout() -> SinkNode<Item, impl FnOnce(Receiver<Item>) -> TaskHandle> {
    write_sink(|| async { Ok(tokio::io::stdout()) })
}

pub fn stderr() -> SinkNode<Item, impl FnOnce(Receiver<Item>) -> TaskHandle> {
    write_sink(|| async { Ok(tokio::io::stderr()) })
}