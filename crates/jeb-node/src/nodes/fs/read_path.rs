use std::path::Path;

use crate::{Sender, SourceNode, TaskHandle, read_source};

pub fn read_path<P>(
    path: P,
) -> SourceNode<
    Result<crate::Item, &'static str>,
    impl FnOnce(Sender<Result<crate::Item, &'static str>>) -> TaskHandle,
>
where
    P: AsRef<Path> + Send + 'static,
{
    read_source(move || async move { tokio::fs::File::open(path).await })
}
