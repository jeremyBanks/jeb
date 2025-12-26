use crate::{
    Item,
    Receiver,
    Sender,
    TaskHandle,
    TransformNode,
    transform,
};

pub fn chunks(
    length: usize,
) -> TransformNode<
    Result<Item, &'static str>,
    Result<Item, &'static str>,
    impl FnOnce(Receiver<Result<Item, &'static str>>, Sender<Result<Item, &'static str>>) -> TaskHandle,
> {
    let chunk_size = if length == 0 { 65536 } else { length };

    transform(move |mut input, output| async move {
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

                            // Add to text buffer and chunk by character count
                            text_buffer.push_str(&text);

                            while text_buffer.chars().count() >= chunk_size {
                                let chunk_chars: String =
                                    text_buffer.chars().take(chunk_size).collect();
                                let byte_len = chunk_chars.len();
                                text_buffer.drain(..byte_len);

                                if output
                                    .push_value(Item::Text(chunk_chars.into()))
                                    .await
                                    .is_err()
                                {
                                    return;
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

                            // Add to bytes buffer and chunk by byte count
                            bytes_buffer.extend_from_slice(&bytes);

                            while bytes_buffer.len() >= chunk_size {
                                let chunk: Vec<u8> = bytes_buffer.drain(..chunk_size).collect();

                                if output.push_value(Item::Bytes(chunk.into())).await.is_err() {
                                    return;
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
