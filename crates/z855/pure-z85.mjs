// Pure Z85 encoder (no Z855 escapes)
const Z85_ALPHABET = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

export function encodeZ85(data) {
  let encoded = "";
  let i = 0;

  // Process 4-byte blocks
  while (i + 4 <= data.length) {
    const value = (
      (data[i] << 24) |
      (data[i + 1] << 16) |
      (data[i + 2] << 8) |
      data[i + 3]
    ) >>> 0;

    let block = "";
    let v = value;
    for (let j = 0; j < 5; j++) {
      block = Z85_ALPHABET[v % 85] + block;
      v = Math.floor(v / 85);
    }
    encoded += block;
    i += 4;
  }

  // Handle trailing bytes (1-3 bytes)
  const remaining = data.length - i;
  if (remaining > 0) {
    let value = 0;
    for (let j = 0; j < remaining; j++) {
      value = (value << 8) | data[i + j];
    }

    // Encode with remaining+1 characters
    let block = "";
    let v = value;
    for (let j = 0; j < remaining + 1; j++) {
      block = Z85_ALPHABET[v % 85] + block;
      v = Math.floor(v / 85);
    }
    encoded += block;
  }

  return encoded;
}
