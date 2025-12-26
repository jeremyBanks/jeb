use jeb_values::Item;

use crate::{Sender, SourceNode, TaskHandle, iter_source};

pub fn bytes_source<IntoBytesIterator>(
    bytes: IntoBytesIterator,
) -> SourceNode<
    Result<Item, &'static str>,
    impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle,
>
where
    IntoBytesIterator: IntoIterator<Item = Vec<u8>> + Send + 'static,
    IntoBytesIterator::IntoIter: Send,
{
    iter_source(bytes.into_iter().map(|v| Item::Bytes(v.into())))
}
