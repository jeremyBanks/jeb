use std::path::Path;

use crate::{Receiver, SinkNode, TaskHandle, write_sink};

pub fn write_path<P>(
    path: P,
) -> SinkNode<jeb_values::Item, impl FnOnce(Receiver<jeb_values::Item>) -> TaskHandle>
where
    P: AsRef<Path> + Send + 'static,
{
    write_sink(move || async move { tokio::fs::File::create(path).await })
}
