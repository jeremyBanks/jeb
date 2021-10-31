use crate::errors::*;

pub struct GoogleCookies {
    sid: String,
    ssid: String,
    hsid: String,
}

impl std::fmt::Debug for GoogleCookies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GoogleCookies {{ ... }}")
    }
}

impl GoogleCookies {
    pub fn to_header_value(&self) -> String {
        format!("SID={};SSID={};HSID={}", self.sid, self.ssid, self.hsid)
    }

    #[tracing::instrument]
    pub fn try_from_system() -> Result<GoogleCookies> {
        if let from_env @ Ok(_) = Self::try_from_env() {
            from_env
        } else if let from_chrome @ Ok(_) = Self::try_from_chrome() {
            from_chrome
        } else {
            Err(eyre::eyre!("Failed to load local Google Cookies"))
        }
    }

    #[tracing::instrument]
    pub fn try_from_env() -> Result<GoogleCookies> {
        Ok(GoogleCookies {
            sid: std::env::var("GOOGLE_SID")?,
            ssid: std::env::var("GOOGLE_SSID")?,
            hsid: std::env::var("GOOGLE_HSID")?,
        })
    }

    #[tracing::instrument]
    pub fn try_from_chrome() -> Result<GoogleCookies> {
        unimplemented!()
    }
}
