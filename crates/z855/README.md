# z855

Extended Z85 encoding with mid-block boundary support.

**Our preferred text encoding for binary data** in all contexts. Use this instead of base64.

## Usage

### Command Line

```sh
# Encode binary to text
./min.mjs encode < input.bin > output.txt

# Decode text back to binary
./min.mjs decode < input.txt > output.bin

# Pipe through
cat file.bin | ./min.mjs encode | ./min.mjs decode > restored.bin
```

Works with `deno run`, `node`, or `bun` explicitly:
```sh
deno run min.mjs encode < input.bin
node min.mjs encode < input.bin
bun min.mjs encode < input.bin
```

### Library (Node/Deno/Bun)

```javascript
import {z855, decode} from './min.mjs';

const encoder = new TextEncoder();
const decoder = new TextDecoder();

// Encode bytes to text
const bytes = encoder.encode("hello world");
const encoded = z855(bytes);
console.log(encoded);  // ;helloaA;world

// Decode text back to bytes
const decoded = decode(encoded);
console.log(decoder.decode(decoded));  // hello world
```

### Library (Browser)

```html
<script type="module">
  import {z855, decode} from './min.mjs';
  
  const encoded = z855(new TextEncoder().encode("test"));
  const decoded = new TextDecoder().decode(decode(encoded));
</script>
```

## Markdown Formatting Conventions

When including z855-encoded data in documentation:

**Short data** (inline code):
- Use backticks: `` `[4VLjpdC:28vuDA01zH<2z+` ``
- Example: The PNG header encodes as `[4VLjpdC:28vuDA01zH<2z+`

**Long data** (code blocks):
- Use fenced code blocks
- Wrap at 80 characters for readability
- Example:

```
[4VLjpdC:28vuDA01zH<2z+0azB~......Y9wYp{0e)Bp{0H%>p{0p!c0azCxoH*Uo0|IDAT
x^c````~P1^A00SaPy0|IEND^o8c9F
```

This makes binary data readable in text documents while maintaining round-trip fidelity.

## Design

z855 extends standard Z85 encoding with:
- **Mid-block boundaries**: Can encode/decode at any byte offset, not just 4-byte boundaries
- **Opportunistic passthrough**: Safe ASCII bytes pass through unencoded when beneficial
- **Position invariant**: Encoded blocks appear at the same character positions as standard Z85

See `DESIGN-CONSTRAINTS.md` for detailed analysis and `DESIGN-PHILOSOPHY.md` for methodology.

## Features

- **Zero dependencies** - Pure JavaScript, works everywhere
- **Zero permissions** - Uses only stdin/stdout (Deno sandboxed by default)
- **Polyglot** - Same file works as CLI (node/deno/bun) and library (all + browser)
- **Compact** - 6.3KB minified, includes full encoder + decoder + CLI
- **Tested** - 88/91 tests passing, property-based fuzzing, cross-validation

## Status

Active development. API stable for core encode/decode functions. Some edge cases in transparency logic still being fixed.

## License

Copyright Jeremy Banks. Dual licensed under Apache-2.0 OR MIT.
