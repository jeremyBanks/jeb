use crate::client::Client;

#[derive(Debug)]
pub struct Spider {
    pub client: Client,
}

impl Spider {
    pub fn new(cookie_header: String) -> Self {
        Self {
            client: Client::new(cookie_header),
        }
    }
}
