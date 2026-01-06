#![cfg(windows)]
#![cfg_attr(debug_assertions, allow(unused))]

use std::any::type_name;
use std::any::Any;
use std::any::TypeId;
use std::fmt::Debug;

use tracing::instrument;
use tracing::metadata::LevelFilter;
use tracing::trace as log;
use tracing_subscriber::fmt::format::FmtSpan;
use windows_core::InParam;
use windows_core::Interface;
use windows_core::w;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::Buffer;
use windows::Storage::Streams::DataReader;
use windows::Storage::Streams::IBuffer;

#[derive(Debug, Clone, Default)]
pub struct Tts {
    voice_required: Option<Vec<String>>,
    voice_preferred: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Speech {
    text: String,
    audio: Vec<u8>,
}

pub async fn speak(text: &str) -> Result<Speech, eyre::Report> {
    Tts::default().speak(text).await
}

impl Tts {
    fn new() -> Self {
        Self::default()
    }

    async fn speak(&self, text: &str) -> Result<Speech, eyre::Report> {
        let synth = SpeechSynthesizer::new().wrap()?;

        let voice = synth.Voice().wrap()?;
        let options = synth.Options().wrap()?;

        log!(
            "      Language: {:>19}",
            voice.Language().wrap()?.to_string()
        );
        log!(
            "         Voice: {:>19}",
            voice.DisplayName().wrap()?.to_string()
        );
        log!("         Pitch: {:>19.2}", options.AudioPitch().wrap()?);
        log!("        Volume: {:>19.2}", options.AudioVolume().wrap()?);
        log!("         Speed: {:>19.2}", options.SpeakingRate().wrap()?);
        log!(
            "          Rest: {:>16?}",
            options.PunctuationSilence().wrap()?.0
        );
        log!(
            "           End: {:>16?}",
            options.AppendedSilence().wrap()?.0
        );
        log!(
            "         Words: {:>19}",
            options.IncludeWordBoundaryMetadata().wrap()?.to_string()
        );
        log!(
            "         Stops: {:>19}",
            options
                .IncludeSentenceBoundaryMetadata()
                .wrap()?
                .to_string()
        );

        let stream = synth
            .SynthesizeTextToStreamAsync(w!("hello, world!"))
            .wrap()?
            .await
            .wrap()?;

        let buffer = Buffer::Create(64 * 1024 * 1024)
            .wrap()?
            .cast::<IBuffer>()
            .wrap()?;

        stream
            .ReadAsync(
                InParam::from(Some(&buffer)),
                buffer.Capacity().unwrap(),
                Default::default(),
            )
            .wrap()?
            .await
            .wrap()?;

        let content_type = stream.ContentType().wrap()?;

        log!("        Length: {:>18}B", buffer.Length().wrap()?);
        log!("          Type: {:>19}", content_type.to_string());

        let mut bytes = vec![0u8; buffer.Length().wrap()? as usize];
        DataReader::FromBuffer(InParam::from(Some(&buffer)))
            .wrap()?
            .ReadBytes(&mut bytes)
            .wrap()?;

        Ok(Speech {
            text: text.to_owned(),
            audio: bytes,
        })
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("{0}{1}")]
pub(crate) struct WrappedError(&'static str, String);

impl WrappedError {
    pub fn new(msg: &'static str, err: impl Debug) -> Self {
        Self(msg, format!("{err:?}"))
    }
}

pub(crate) trait DebugResultExt {
    type Ok;
    fn wrap(self) -> Result<Self::Ok, WrappedError>;
}

impl<T, E: Debug + Any> DebugResultExt for Result<T, E> {
    type Ok = T;
    fn wrap(self) -> Result<T, WrappedError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                let prefix = if err.type_id() == TypeId::of::<windows_core::Error>() {
                    "windows_core::"
                } else {
                    let prefix = type_name::<E>();
                    &prefix[..prefix.rfind("::").map(|x| x + 2).unwrap_or(prefix.len())]
                };
                Err(WrappedError::new(prefix, err))
            },
        }
    }
}
