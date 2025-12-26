use jeb_values::Item;

pub struct Sender<T = Result<Item, &'static str>> {
    sender: tokio::sync::mpsc::Sender<T>,
}

impl<T> Sender<T> {
    pub async fn push(&self, item: T) -> Result<(), T> {
        self.sender.send(item).await.map_err(|e| e.0)
    }
}

impl<T, E> Sender<Result<T, E>> {
    pub async fn push_value(&self, item: T) -> Result<(), T> {
        self.sender.send(Ok(item)).await.map_err(|e| e.0.ok().expect("unreachable"))
    }

    pub async fn push_error(&self, item: E) -> Result<(), E> {
        self.sender.send(Err(item)).await.map_err(|e| e.0.err().expect("unreachable"))
    }
}

pub struct Receiver<T = Item> {
    receiver: tokio::sync::mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub async fn pull(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}

#[must_use]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    (Sender { sender }, Receiver { receiver })
}


impl<T, E: std::fmt::Debug> Receiver<Result<T, E>> where E: Send + 'static, T: Send + 'static {
    /// Takes a stream of Result<T, E> and splits it into two separate streams,
    /// one for the Ok values and one for the Err values.
    pub fn out_and_err(self) -> (Receiver<T>, Receiver<E>) {
        let (ok_sender, ok_receiver) = channel::<T>();
        let (err_sender, err_receiver) = channel::<E>();

        tokio::spawn(async move {
            let mut receiver = self;
            while let Some(item) = receiver.pull().await {
                match item {
                    Ok(ok) => {
                        if ok_sender.push(ok).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        if err_sender.push(err).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        (ok_receiver, err_receiver)
    }

    /// Takes a stream of Result<T, E> and closes after the first Err.
    pub fn fail_fast(self) -> Receiver<T> {
        let (ok_sender, ok_receiver) = channel::<T>();

        tokio::spawn(async move {
            let mut receiver = self;
            while let Some(item) = receiver.pull().await {
                match item {
                    Ok(ok) => {
                        if ok_sender.push(ok).await.is_err() {
                            break;
                        }
                    }
                    Err(_err) => {
                        break;
                    }
                }
            }
        });

        ok_receiver
    }

    /// Takes a stream of Result<T, E> and unwraps the Ok values, panicking on Err.
    pub fn unwrapping(self) -> Receiver<T> {
        let (ok_sender, ok_receiver) = channel::<T>();

        tokio::spawn(async move {
            let mut receiver = self;
            while let Some(item) = receiver.pull().await {
                if ok_sender.push(item.unwrap()).await.is_err() {
                    break;
                }
            }
        });

        ok_receiver
    }
}
