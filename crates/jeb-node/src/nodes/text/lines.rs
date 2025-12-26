use jeb_values::Item;

use crate::{Receiver, Sender, TaskHandle, TransformNode, transform};

pub fn lines() -> TransformNode<
    Result<Item, &'static str>,
    Result<Item, &'static str>,
    impl FnOnce(Receiver<Result<Item, &'static str>>, Sender<Result<Item, &'static str>>) -> TaskHandle,
> {
    transform(|mut input, output| async move {
        let mut text_buffer = String::new();
        let mut bytes_buffer = Vec::<u8>::new();

        while let Some(result) = input.pull().await {
            match result {
                Ok(item) => {
                    match item {
                        Item::Text(text) => {
                            // Flush bytes buffer if switching types
                            if !bytes_buffer.is_empty() {
                                if output
                                    .push_value(Item::Bytes(bytes_buffer.into()))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                bytes_buffer = Vec::new();
                            }

                            // Add to text buffer and split on newlines
                            text_buffer.push_str(&text);

                            loop {
                                if let Some(newline_pos) = text_buffer.find('\n') {
                                    let line = text_buffer[..=newline_pos].to_string();
                                    text_buffer.drain(..=newline_pos);

                                    if output.push_value(Item::Text(line.into())).await.is_err() {
                                        return;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        Item::Bytes(bytes) => {
                            // Flush text buffer if switching types
                            if !text_buffer.is_empty() {
                                if output
                                    .push_value(Item::Text(text_buffer.clone().into()))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                text_buffer.clear();
                            }

                            // Add to bytes buffer and split on newline bytes
                            bytes_buffer.extend_from_slice(&bytes);

                            loop {
                                if let Some(newline_pos) =
                                    bytes_buffer.iter().position(|&b| b == b'\n')
                                {
                                    let line: Vec<u8> =
                                        bytes_buffer.drain(..=newline_pos).collect();

                                    if output.push_value(Item::Bytes(line.into())).await.is_err() {
                                        return;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        other => {
                            // Flush both buffers before passing through
                            if !text_buffer.is_empty() {
                                if output
                                    .push_value(Item::Text(text_buffer.clone().into()))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                text_buffer.clear();
                            }
                            if !bytes_buffer.is_empty() {
                                if output
                                    .push_value(Item::Bytes(bytes_buffer.clone().into()))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                bytes_buffer.clear();
                            }

                            // Pass through other item types
                            if output.push_value(other).await.is_err() {
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    // Flush both buffers before passing through error
                    if !text_buffer.is_empty() {
                        if output
                            .push_value(Item::Text(text_buffer.clone().into()))
                            .await
                            .is_err()
                        {
                            return;
                        }
                        text_buffer.clear();
                    }
                    if !bytes_buffer.is_empty() {
                        if output
                            .push_value(Item::Bytes(bytes_buffer.clone().into()))
                            .await
                            .is_err()
                        {
                            return;
                        }
                        bytes_buffer.clear();
                    }

                    // Pass through error
                    if output.push_error(e).await.is_err() {
                        return;
                    }
                }
            }
        }

        // Flush remaining buffers at end of stream
        if !text_buffer.is_empty() {
            let _ = output.push_value(Item::Text(text_buffer.into())).await;
        }
        if !bytes_buffer.is_empty() {
            let _ = output.push_value(Item::Bytes(bytes_buffer.into())).await;
        }
    })
}
