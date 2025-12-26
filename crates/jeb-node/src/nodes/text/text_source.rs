use crate::{Item, Sender, SourceNode, TaskHandle, iter_source};

pub fn text_source<IntoTextIterator>(
    text: IntoTextIterator,
) -> SourceNode<
    Result<Item, &'static str>,
    impl FnOnce(Sender<Result<Item, &'static str>>) -> TaskHandle,
>
where
    IntoTextIterator: IntoIterator<Item = String> + Send + 'static,
    IntoTextIterator::IntoIter: Send,
{
    iter_source(text.into_iter().map(|s| Item::Text(s.into())))
}
