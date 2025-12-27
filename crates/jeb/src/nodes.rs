#[cfg(
    any(
        feature = "stdio",
        feature = "fs"
    )
)]
use crate::{
    Panic,
    model::{
        Bytes,
        channel,
    },
};
#[cfg(
    any(
        feature = "stdio",
        feature = "fs"
    )
)]
use tokio::io::AsyncWriteExt;
#[cfg(
    any(
        feature = "stdio",
        feature = "fs"
    )
)]
use tokio_stream::StreamExt;
#[cfg(
    any(
        feature = "stdio",
        feature = "fs"
    )
)]
use tokio_util::codec::{
    BytesCodec,
    FramedRead,
};
use {
    crate::model::{
        Node,
        Receiver,
        Task,
    },
    jeb_streaming::Item,
    std::borrow::Cow,
};

pub trait NodeDef: Node + Send + Sync + 'static {
    const NAME: &'static str;

    fn name(&self) -> Cow<str> {
        Self::NAME.into()
    }

    fn spawn(&self, stack: Vec<Receiver>) -> (Vec<Receiver>, Task);
}

impl<T: NodeDef> Node for T {
    fn spawn(&self, stack: Vec<Receiver>) -> (Vec<Receiver>, Task) {
        NodeDef::spawn(self, stack)
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg(feature = "stdio")]
struct Stdin;

#[cfg(feature = "stdio")]
impl NodeDef for Stdin {
    const NAME: &'static str = "stdin";

    fn spawn(&self, mut stack: Vec<Receiver>) -> (Vec<Receiver>, Task) {
        let (sender, receiver) = channel();
        let stdin = tokio::io::stdin();

        let handle = tokio::spawn(async move {
            let mut stdin_bytes: FramedRead<tokio::io::Stdin, BytesCodec> =
                FramedRead::new(stdin, BytesCodec::new());

            while let Some(value) = stdin_bytes.next().await {
                let vec = value?.to_vec();
                let bytes = Bytes::from(vec);
                sender.send(bytes.into()).await?;
            }

            Ok(())
        });

        stack.push(receiver);

        (stack, handle)
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg(feature = "fs")]
struct ReadPath<T: AsRef<std::path::Path>>(T);

#[cfg(feature = "fs")]
impl<T: AsRef<std::path::Path> + Send + Sync + 'static> NodeDef for ReadPath<T> {
    const NAME: &'static str = "read:";

    fn name(&self) -> Cow<str> {
        format!("read:{}", self.0.as_ref().display()).into()
    }

    fn spawn(&self, mut stack: Vec<Receiver>) -> (Vec<Receiver>, Task) {
        let (sender, receiver) = channel();
        let path = self.0.as_ref().to_owned();

        let handle = tokio::spawn(async move {
            let file = tokio::fs::File::open(path).await?;

            let mut file_bytes: FramedRead<tokio::fs::File, BytesCodec> =
                FramedRead::new(file, BytesCodec::new());

            while let Some(value) = file_bytes.next().await {
                let vec = value?.to_vec();
                let bytes = Bytes::from(vec);
                sender.send(bytes.into()).await?;
            }

            Ok(())
        });

        stack.push(receiver);

        (stack, handle)
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg(feature = "stdio")]
struct Stdout;

#[cfg(feature = "stdio")]
impl NodeDef for Stdout {
    const NAME: &'static str = "stdout";

    fn spawn(&self, mut stack: Vec<Receiver>) -> (Vec<Receiver>, Task) {
        let mut receiver = stack.pop().expect("stdout node must receive an input");
        let mut stdout = tokio::io::stdout();

        let handle = tokio::spawn(async move {
            while let Some(value) = receiver.next().await {
                match value {
                    Item::Bytes(bytes) => {
                        stdout.write_all(&bytes).await?;
                    }
                    Item::Text(text) => {
                        stdout.write_all(text.as_bytes()).await?;
                    }
                    Item::Value(_) => {
                        unimplemented!("stdout does not support Value items");
                    }
                }
            }

            Ok(())
        });

        (stack, handle)
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg(feature = "stdio")]
struct Stderr;
#[cfg(feature = "stdio")]
impl NodeDef for Stderr {
    const NAME: &'static str = "stderr";

    fn spawn(&self, mut stack: Vec<Receiver>) -> (Vec<Receiver>, Task) {
        let mut receiver = stack.pop().expect("stderr node must receive an input");
        let mut stderr = tokio::io::stderr();

        let handle = tokio::spawn(async move {
            while let Some(value) = receiver.next().await {
                match value {
                    Item::Bytes(bytes) => {
                        stderr.write_all(&bytes).await?;
                    }
                    Item::Text(text) => {
                        stderr.write_all(text.as_bytes()).await?;
                    }
                    Item::Value(_) => {
                        unimplemented!("stderr does not support Value items");
                    }
                }
            }

            Ok(())
        });

        (stack, handle)
    }
}

// TODO: move or remove
#[cfg(
    any(
        feature = "stdio",
        feature = "fs"
    )
)]
pub async fn wip_example_pseudo_main() -> Result<(), Panic> {
    let nodes: Vec<&dyn Node> = vec![
        #[cfg(feature = "stdio")]
        &Stdin,
        #[cfg(feature = "stdio")]
        &Stdout,
        #[cfg(feature = "fs")]
        &ReadPath("/etc/hosts"),
    ];

    let mut stack = vec![];
    let mut tasks = vec![];

    for node in nodes {
        let task;
        (stack, task) = node.spawn(stack);
        tasks.push(task);
    }

    assert!(stack.is_empty());

    let mut complete_tasks = futures::stream::FuturesUnordered::from_iter(tasks);

    while let Some(result) = tokio_stream::StreamExt::next(&mut complete_tasks).await {
        result??;
    }

    Ok(())
}
