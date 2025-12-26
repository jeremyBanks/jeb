use std::path::Path;

use crate::{Sender, SourceNode, TaskHandle, read_source};

pub fn read_path<P>(
    path: P,
) -> SourceNode<
    Result<jeb_values::Item, &'static str>,
    impl FnOnce(Sender<Result<jeb_values::Item, &'static str>>) -> TaskHandle,
>
where
    P: AsRef<Path> + Send + 'static,
{
    read_source(move || async move { tokio::fs::File::open(path).await })
}
