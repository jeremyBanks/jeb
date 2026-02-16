import { encode, decode, z855Binary, Z855DecodeError, type Z855EncodeOptions } from "./z855.ts";

// Re-export library functions for external use
export { encode, decode, z855Binary, Z855DecodeError, type Z855EncodeOptions } from "./z855.ts";

// CLI entry point
if (import.meta.main) {
  const args = Deno.args;

  if (args.length !== 1) {
    console.error("error: expected exactly one argument: 'encode' or 'decode'");
    console.error("Usage: deno run main.ts <encode|decode>");
    Deno.exit(1);
  }

  const command = args[0];

  if (command === "encode") {
    // Read all bytes from stdin
    const input = await readAllStdin();

    // Encode to Z855
    const encoded = encode(input);

    // Write to stdout
    const encoder = new TextEncoder();
    await Deno.stdout.write(encoder.encode(encoded));

    Deno.exit(0);
  } else if (command === "decode") {
    // Read all bytes from stdin
    const input = await readAllStdin();

    // Convert to string (Z855 is ASCII, so this should be valid UTF-8)
    const decoder = new TextDecoder();
    const inputStr = decoder.decode(input);

    try {
      // Decode from Z855
      const decoded = decode(inputStr);

      // Write raw bytes to stdout
      await Deno.stdout.write(decoded);

      Deno.exit(0);
    } catch (e) {
      if (e instanceof Z855DecodeError) {
        console.error(`error: ${e.message}`);
      } else {
        console.error(`error: ${e}`);
      }
      Deno.exit(1);
    }
  } else {
    console.error(`error: unknown command '${command}'. Use 'encode' or 'decode'.`);
    Deno.exit(1);
  }
}

/**
 * Read all bytes from stdin
 */
async function readAllStdin(): Promise<Uint8Array> {
  const chunks: Uint8Array[] = [];
  const reader = Deno.stdin.readable.getReader();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    if (value) chunks.push(value);
  }

  // Concatenate all chunks
  const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }

  return result;
}
