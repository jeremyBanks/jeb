use crate::{Item, Receiver, SinkNode, TaskHandle, write_sink};

pub fn stdout() -> SinkNode<Item, impl FnOnce(Receiver<Item>) -> TaskHandle> {
    write_sink(|| async { Ok(tokio::io::stdout()) })
}
