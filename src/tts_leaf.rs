#![cfg(windows)]
use tracing::trace as log;
use windows::core::InParam;
use windows::core::Interface;
use windows::core::HSTRING;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::Buffer;
use windows::Storage::Streams::DataReader;
use windows::Storage::Streams::IBuffer;

use sapi_lite;

use super::*;

// Let's try to get sentence/word boundaries so we can slice them up!

pub async fn speak(text: &str) -> Result<Vec<u8>, panic> {
    speak_as(text, None).await
}

pub async fn speak_as(text: &str, voice_name: impl Into<Option<&str>>) -> Result<Vec<u8>, panic> {
    let synth = SpeechSynthesizer::new()?;

    std::thread::sleep(std::time::Duration::from_millis(2000));

    if let Some(voice_name) = voice_name.into() {
        let voice = SpeechSynthesizer::AllVoices()?
            .into_iter()
            .find(|voice| {
                voice.DisplayName().unwrap_or_default() == voice_name
                    || voice.Id().unwrap_or_default() == voice_name
                    || voice.Description().unwrap_or_default() == voice_name
                    || voice.Language().unwrap_or_default() == voice_name
            })
            .ok_or_else(|| eyre::eyre!("voice not found: {voice_name}"))?;
        synth.SetVoice(&voice)?;
    }

    let voices = Vec::from_iter(SpeechSynthesizer::AllVoices()?)
        .into_iter()
        .map(|voice| {
            format!(
                "\n  {}\n  {}\n  {}\n  {}",
                voice.DisplayName().unwrap(),
                voice.Id().unwrap(),
                voice.Description().unwrap(),
                voice.Language().unwrap(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n  ");

    log!("voices: {}", voices);
    std::process::exit(0);

    let voice = synth.Voice()?;
    let options = synth.Options()?;

    log!("      Language: {:>19}", voice.Language()?.to_string());
    log!("         Voice: {:>19}", voice.DisplayName()?.to_string());
    log!("                {:>19}", voice.Id()?.to_string());
    log!("                {:>19}", voice.Language()?.to_string());
    println!("Voices: {voices}");
    log!("         Pitch: {:>19.2}", options.AudioPitch()?);
    log!("        Volume: {:>19.2}", options.AudioVolume()?);
    log!("         Speed: {:>19.2}", options.SpeakingRate()?);
    log!("          Rest: {:>16?}", options.PunctuationSilence()?.0);
    log!("           End: {:>16?}", options.AppendedSilence()?.0);
    log!(
        "         Words: {:>19}",
        options.IncludeWordBoundaryMetadata()?.to_string()
    ); // <--- USE THIS YO
    log!(
        "         Stops: {:>19}",
        options.IncludeSentenceBoundaryMetadata()?.to_string()
    );

    let stream = synth
        .SynthesizeTextToStreamAsync(&HSTRING::from(text))?
        .await?;

    let buffer = Buffer::Create(64 * 1024 * 1024)?.cast::<IBuffer>()?;

    stream
        .ReadAsync(
            InParam::from(Some(&buffer)),
            buffer.Capacity().unwrap(),
            Default::default(),
        )?
        .await?;

    let content_type = stream.ContentType()?;

    log!("        Length: {:>18}B", buffer.Length()?);
    log!("          Type: {:>19}", content_type.to_string());

    let mut bytes = vec![0u8; buffer.Length()? as usize];
    DataReader::FromBuffer(InParam::from(Some(&buffer)))?.ReadBytes(&mut bytes)?;

    Ok(bytes)
}
