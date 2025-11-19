use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Read all input from stdin
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;

    // Process through pipeline
    let output = process_pipeline(&args, input)?;

    // Write to stdout
    io::stdout().write_all(&output)?;

    Ok(())
}

fn process_pipeline(commands: &[String], mut data: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut is_stream_of_streams = false;
    let mut streams: Vec<Vec<u8>> = vec![];

    for cmd in commands {
        match cmd.as_str() {
            "encode-z85" => {
                if is_stream_of_streams {
                    // Map over each sub-stream
                    streams = streams
                        .into_iter()
                        .map(|chunk| json_encoded_binary::encode(&chunk))
                        .collect();
                } else {
                    // Encode single stream
                    data = json_encoded_binary::encode(&data);
                }
            }
            "decode-z85" => {
                if is_stream_of_streams {
                    // Map over each sub-stream
                    streams = streams
                        .into_iter()
                        .map(|chunk| json_encoded_binary::decode(&chunk))
                        .collect();
                } else {
                    // Decode single stream
                    data = json_encoded_binary::decode(&data);
                }
            }
            "encode-jeb85" => {
                if is_stream_of_streams {
                    // Map over each sub-stream
                    streams = streams
                        .into_iter()
                        .map(|chunk| json_encoded_binary::encode_jeb85(&chunk))
                        .collect();
                } else {
                    // Encode single stream
                    data = json_encoded_binary::encode_jeb85(&data);
                }
            }
            "decode-jeb85" => {
                if is_stream_of_streams {
                    // Map over each sub-stream
                    streams = streams
                        .into_iter()
                        .map(|chunk| json_encoded_binary::decode_jeb85(&chunk))
                        .collect();
                } else {
                    // Decode single stream
                    data = json_encoded_binary::decode_jeb85(&data);
                }
            }
            "split-64k" => {
                if is_stream_of_streams {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Cannot split-64k on stream-of-streams",
                    ));
                }
                // Split into 64 KiB chunks
                const CHUNK_SIZE: usize = 64 * 1024;
                streams = data
                    .chunks(CHUNK_SIZE)
                    .map(|chunk| chunk.to_vec())
                    .collect();
                is_stream_of_streams = true;
            }
            "split-lines" => {
                if is_stream_of_streams {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Cannot split-lines on stream-of-streams",
                    ));
                }
                // Split by newlines
                streams = data
                    .split(|&b| b == b'\n')
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_vec())
                    .collect();
                is_stream_of_streams = true;
            }
            "join" => {
                if !is_stream_of_streams {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Cannot join a single stream",
                    ));
                }
                // Join with no delimiter
                data = streams.into_iter().flatten().collect();
                streams = vec![];
                is_stream_of_streams = false;
            }
            "join-lines" => {
                if !is_stream_of_streams {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Cannot join-lines a single stream",
                    ));
                }
                // Join with newlines
                let mut result = Vec::new();
                for (i, stream) in streams.iter().enumerate() {
                    result.extend_from_slice(stream);
                    if i < streams.len() - 1 {
                        result.push(b'\n');
                    }
                }
                data = result;
                streams = vec![];
                is_stream_of_streams = false;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown command: {}", cmd),
                ));
            }
        }
    }

    // Error if we end with stream-of-streams
    if is_stream_of_streams {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Pipeline ended with stream-of-streams (forgot to join?)",
        ));
    }

    Ok(data)
}
