use crate::client::Client;

#[derive(Debug)]
pub struct Spider {
    pub client: Client,
}

impl Spider {
    pub fn new(credentials: crate::credentials::GoogleCookies) -> Self {
        Self {
            client: Client::new(credentials),
        }
    }
}
