use {
    crate::{
        Receiver,
        SinkNode,
        TaskHandle,
        write_sink,
    },
    std::path::Path,
};

pub fn write_path<P>(
    path: P,
) -> SinkNode<crate::Item, impl FnOnce(Receiver<crate::Item>) -> TaskHandle>
where
    P: AsRef<Path> + Send + 'static,
{
    write_sink(move || async move { tokio::fs::File::create(path).await })
}
