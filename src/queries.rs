use std::fmt::Debug;

use derive_more::From;
use derive_more::TryInto;
use paste::paste;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use crate::context::Context;
use crate::generic::default;
use crate::generic::never;

// we can eliminate a lot of overhead if we
// serialize the individual Response type directly, instead of
// through the AnyResponse enum.
// but first make it work?

macro_rules! request_and_response {
    () => {
        request_and_response! {
            // content-addressed blob storage
            // blob_from_id     = 0x00;

            // alternate content-addressings
            // blob_from_sha1   = 0x11;
            // blob_from_sha256 = 0x12;
            // blob_from_blake3 = 0x1E; // (length = 32)
            // blob_from_gitsha = 0x04; // (bytes, object type = "blob")

            // arbitrary key(-path)-value blob storage
            // blob_from_key    = 0x08;

            // container formats
            // blobs_by_keys_zipped = 0x0B; // non-compressed zip from sorted map of paths
            // mkv_by_concatenation = 0x0C; // mkv by concatenating audio or video

            // data fetching
            http_get = 0x09;
            // http_archive_get = 0x1A;
            // http_archive_query = 0x1B;

            // fic_by_id = 0x1F;

            // royalroad_fic = 0x46;
            // royalroad_spine = 0x47;
            // royalroad_chapter = 0x48;

            // text to speech
            // speech_voices    = 0x30;
            // speech_from_text = 0x31;
            // speech_from_ssml = 0x32;
            // speech_from_html = 0x33;
            // exercised_voices = 0x36;

            reserved = 0x7F;
        }
    };
    ( $( $name:ident = $tag:literal; )* ) => {
        paste! {
            request_and_response! {
                $( [<$name:snake>]: [<$name:camel>] = $tag; )*
            }
        }
    };
    ($(
        $name:ident: $Name:ident = $tag:literal;
    )*) => {
$(pub mod $name;)*

#[derive(Debug, Serialize, Deserialize, Clone, TryInto, From)]
#[repr(u32)]
pub enum AnyRequest {
    $(
        $Name($name::Request) = $tag,
    )*
}

#[derive(Debug, Serialize, Deserialize, Clone, TryInto, From)]
#[repr(u32)]
pub enum AnyResponse {
    $(
        $Name($name::Response) = $tag,
    )*
}

impl Request for AnyRequest {
    type Response = AnyResponse;

    #[instrument]
    fn query(&self, context: &mut Context) -> Result<Self::Response, never> {
        Ok(match self {
            $(
                Request::$Name(request) => Response::$Name(request.query(context)?),
            )*
        })
    }
}

$(
    impl Request for $name::Request {
        type Response = $name::Response;

        #[instrument]
        fn query(&self, context: &mut Context) -> Result<Self::Response, never> {
            Ok($name::query(self, context)?)
        }
    }

    impl Response for $name::Response {
        type Request = $name::Request;
    }
)*

} }

impl Response for AnyResponse {
    type Request = AnyRequest;
}

request_and_response! {}

impl Default for AnyRequest {
    fn default() -> Self {
        Self::BlobFromId(default())
    }
}

impl Default for AnyResponse {
    fn default() -> Self {
        Self::BlobFromId(default())
    }
}

pub trait Request:
    Debug + Default + Serialize + DeserializeOwned + Send + 'static + Into<AnyRequest>
{
    type Response: Response<Request = Self>;

    fn query(&self, context: &mut Context) -> Result<Self::Response, never>;
}

pub trait Response:
    Debug + Default + Serialize + DeserializeOwned + Send + 'static + Into<AnyResponse>
{
    type Request: Request<Response = Self>;
}
