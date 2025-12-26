use std::path::Path;

use crate::{Receiver, SinkNode, SourceNode, Sender, TaskHandle, read_source, write_sink};

pub fn read_path<P>(path: P) -> SourceNode<Result<jeb_values::Item, &'static str>, impl FnOnce(Sender<Result<jeb_values::Item, &'static str>>) -> TaskHandle>
where
    P: AsRef<Path> + Send + 'static,
{
    read_source(move || async move { tokio::fs::File::open(path).await })
}

pub fn write_path<P>(path: P) -> SinkNode<jeb_values::Item, impl FnOnce(Receiver<jeb_values::Item>) -> TaskHandle>
where
    P: AsRef<Path> + Send + 'static,
{
    write_sink(move || async move { tokio::fs::File::create(path).await })
}
