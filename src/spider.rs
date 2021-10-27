use crate::client::Client;

#[derive(Debug)]
pub struct Spider {
    pub client: Client,
}

impl Spider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}
