use std::{
    io::{self, Read, Write},
    process,
};

/// A binary stream with an associated error flag
#[derive(Clone, Debug)]
struct BinaryStream {
    data: Vec<u8>,
    error: bool,
}

impl BinaryStream {
    fn new(data: Vec<u8>) -> Self {
        Self { data, error: false }
    }
}

/// A stream of binary streams
#[derive(Debug)]
struct StreamOfStreams {
    streams: Vec<BinaryStream>,
}

impl StreamOfStreams {
    fn from_single(data: Vec<u8>) -> Self {
        Self {
            streams: vec![BinaryStream::new(data)],
        }
    }

    fn has_errors(&self) -> bool {
        self.streams.iter().any(|s| s.error)
    }

    fn to_single(self) -> io::Result<BinaryStream> {
        if self.streams.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Pipeline ended with stream-of-streams (forgot to join?)",
            ));
        }
        Ok(self.streams.into_iter().next().unwrap())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Read all input from stdin
    let mut input = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut input) {
        eprintln!("Error reading stdin: {}", e);
        process::exit(1);
    }

    // Process through pipeline
    match process_pipeline(&args, input) {
        Ok(output_stream) => {
            // Write to stdout
            if let Err(e) = io::stdout().write_all(&output_stream.data) {
                eprintln!("Error writing to stdout: {}", e);
                process::exit(1);
            }

            // Exit with error code if stream has error flag
            if output_stream.error {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn process_pipeline(commands: &[String], input: Vec<u8>) -> io::Result<BinaryStream> {
    let mut streams = StreamOfStreams::from_single(input);

    for (cmd_idx, cmd) in commands.iter().enumerate() {
        let mut error_logged_this_command = false;

        match cmd.as_str() {
            "encode-z85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .map(|stream| BinaryStream {
                        data: json_encoded_binary::encode(&stream.data),
                        error: stream.error, // Propagate error flag
                    })
                    .collect();
            }
            "decode-z85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .enumerate()
                    .map(|(stream_idx, stream)| {
                        let (decoded, had_error) = decode_z85_with_errors(&stream.data);
                        if had_error && !error_logged_this_command {
                            eprintln!(
                                "[cmd {} (decode-z85)]: error in stream {}",
                                cmd_idx, stream_idx
                            );
                            error_logged_this_command = true;
                        }
                        BinaryStream {
                            data: decoded,
                            error: stream.error || had_error, // Propagate or set error
                        }
                    })
                    .collect();
            }
            "encode-jeb85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .map(|stream| BinaryStream {
                        data: json_encoded_binary::encode_jeb85(&stream.data),
                        error: stream.error,
                    })
                    .collect();
            }
            "decode-jeb85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .enumerate()
                    .map(|(stream_idx, stream)| {
                        let (decoded, had_error) = decode_jeb85_with_errors(&stream.data);
                        if had_error && !error_logged_this_command {
                            eprintln!(
                                "[cmd {} (decode-jeb85)]: error in stream {}",
                                cmd_idx, stream_idx
                            );
                            error_logged_this_command = true;
                        }
                        BinaryStream {
                            data: decoded,
                            error: stream.error || had_error,
                        }
                    })
                    .collect();
            }
            "split-64k" => {
                // Split each stream into 64 KiB chunks
                const CHUNK_SIZE: usize = 64 * 1024;
                let mut new_streams = Vec::new();
                for stream in streams.streams {
                    for chunk in stream.data.chunks(CHUNK_SIZE) {
                        new_streams.push(BinaryStream {
                            data: chunk.to_vec(),
                            error: stream.error, // Propagate error to all chunks
                        });
                    }
                }
                streams.streams = new_streams;
            }
            "split-64b" => {
                // Split each stream into 64 byte chunks
                const CHUNK_SIZE: usize = 64;
                let mut new_streams = Vec::new();
                for stream in streams.streams {
                    for chunk in stream.data.chunks(CHUNK_SIZE) {
                        new_streams.push(BinaryStream {
                            data: chunk.to_vec(),
                            error: stream.error, // Propagate error to all chunks
                        });
                    }
                }
                streams.streams = new_streams;
            }
            "first" => {
                // Keep only the first substream
                if !streams.streams.is_empty() {
                    let first = streams.streams.into_iter().next().unwrap();
                    streams.streams = vec![first];
                }
            }
            "split-lines" => {
                // Split each stream by newlines
                let mut new_streams = Vec::new();
                for stream in streams.streams {
                    for line in stream.data.split(|&b| b == b'\n') {
                        if !line.is_empty() {
                            new_streams.push(BinaryStream {
                                data: line.to_vec(),
                                error: stream.error, // Propagate error
                            });
                        }
                    }
                }
                streams.streams = new_streams;
            }
            "join" => {
                // Join all streams with no delimiter
                let has_error = streams.has_errors();
                let data: Vec<u8> = streams.streams.into_iter().flat_map(|s| s.data).collect();
                streams = StreamOfStreams {
                    streams: vec![BinaryStream {
                        data,
                        error: has_error,
                    }],
                };
            }
            "join-lines" => {
                // Join all streams with newlines
                let has_error = streams.has_errors();
                let mut data = Vec::new();
                let stream_count = streams.streams.len();
                for (i, stream) in streams.streams.into_iter().enumerate() {
                    data.extend_from_slice(&stream.data);
                    if i < stream_count - 1 {
                        data.push(b'\n');
                    }
                }
                streams = StreamOfStreams {
                    streams: vec![BinaryStream {
                        data,
                        error: has_error,
                    }],
                };
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown command: {}", cmd),
                ));
            }
        }
    }

    streams.to_single()
}

/// Decode Z85 with error handling
/// Returns (output, had_error)
fn decode_z85_with_errors(input: &[u8]) -> (Vec<u8>, bool) {
    let mut output = Vec::new();
    let mut had_error = false;
    let mut i = 0;

    while i + 5 <= input.len() {
        let block = [
            input[i],
            input[i + 1],
            input[i + 2],
            input[i + 3],
            input[i + 4],
        ];

        match json_encoded_binary::decode_z85_block(block) {
            Ok(decoded) => {
                output.extend_from_slice(&decoded);
            }
            Err(_) => {
                // Error: output "E(" + original 5 bytes + ")" = 8 bytes
                output.extend_from_slice(b"E(");
                output.extend_from_slice(&block);
                output.push(b')');
                had_error = true;
            }
        }
        i += 5;
    }

    // Handle remaining bytes (partial block)
    if i < input.len() {
        let remaining = input.len() - i;
        let mut block = [b'0'; 5];
        let start_pos = 5 - remaining;
        block[start_pos..].copy_from_slice(&input[i..]);

        match json_encoded_binary::decode_z85_block(block) {
            Ok(decoded) => {
                let bytes_decoded = json_encoded_binary::BLOCK_BYTES_BY_DIGITS[remaining];
                if bytes_decoded != usize::MAX {
                    let start_byte = 4 - bytes_decoded;
                    output.extend_from_slice(&decoded[start_byte..]);
                }
            }
            Err(_) => {
                // Error on partial block: output "E(" + original bytes + ")" (padded to 8
                // total)
                output.extend_from_slice(b"E(");
                output.extend_from_slice(&input[i..]);
                // Pad to make total 8 bytes (2 + remaining + padding + 1)
                let padding_needed = 5 - remaining;
                output.extend(std::iter::repeat_n(b'0', padding_needed));
                output.push(b')');
                had_error = true;
            }
        }
    }

    (output, had_error)
}

/// Decode JEB85 with error handling
/// Returns (output, had_error)
fn decode_jeb85_with_errors(input: &[u8]) -> (Vec<u8>, bool) {
    // For now, JEB85 decode uses same error handling as Z85
    // If input contains . markers, it's binary mode
    // If it's text mode, it can't really fail

    if input.is_empty() {
        return (Vec::new(), false);
    }

    // Check if it's text mode (no . markers and text-safe)
    if !input.contains(&b'.') {
        let all_z85 = input
            .iter()
            .all(|&b| json_encoded_binary::Z85_DECODE[b as usize] != 255);

        // Simple check for text-safe
        if let Ok(s) = core::str::from_utf8(input)
            && !all_z85 && s.len() <= 65536 {
                // Text mode passthrough
                return (input.to_vec(), false);
            }
    }

    // Binary mode - for now just use the non-error version
    // TODO: Implement proper error handling for JEB85 binary mode
    (json_encoded_binary::decode_jeb85(input), false)
}
