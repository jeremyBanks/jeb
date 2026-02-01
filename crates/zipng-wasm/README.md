# zipng-wasm

WebAssembly bindings for the zipng polyglot PNG+ZIP encoder.

## Features

- **WASM-friendly API**: JSON-based interface with concrete types instead of generic iterators
- **File validation**: Automatic 60KB per-file limit enforcement (IDAT boundary constraint)
- **Predefined sort strategies**: Five built-in sorting modes to replace closures
- **Cross-platform**: Works in web browsers, Node.js, and Deno
- **Pure Rust**: No native dependencies, compiles to WASM cleanly

## Builds

The build script generates three targets:

- **Web** (`pkg/web/`): For browsers using ES modules
- **Node.js** (`pkg/node/`): For Node.js with CommonJS
- **Bundler** (`pkg/bundler/`): For webpack/rollup/vite

## Building

```bash
# Requires wasm-pack
./build.sh
```

## Usage

### Node.js

```javascript
const { encode_simple, version } = require('./pkg/node/zipng_wasm.js');

const files = [
    {
        path: "hello.txt",
        content: Array.from(Buffer.from("Hello, World!"))
    },
    {
        path: "data.json",
        content: Array.from(Buffer.from('{"test": true}'))
    }
];

const filesJson = JSON.stringify(files);
const polyglot = encode_simple(filesJson);

// polyglot is a Uint8Array containing a valid PNG+ZIP
fs.writeFileSync('output.png', Buffer.from(polyglot));
```

### Deno

See `../deno/` for the CLI wrapper:

```bash
deno run --allow-read --allow-write ../deno/zipng-cli.ts -o output.png file1.txt file2.txt
```

### Web Browser

```html
<script type="module">
import init, { encode_simple } from './pkg/web/zipng_wasm.js';

await init();

const files = [
    {
        path: "hello.txt",
        content: Array.from(new TextEncoder().encode("Hello!"))
    }
];

const result = encode_simple(JSON.stringify(files));
const blob = new Blob([result], { type: 'image/png' });
// Download or display...
</script>
```

## API

### `encode_simple(files_json: string) -> Uint8Array`

Encode files with default options (auto mode, lexicographic sort).

**Input JSON format:**
```json
[
  {"path": "file1.txt", "content": [72, 101, 108, 108, 111]},
  {"path": "file2.txt", "content": [87, 111, 114, 108, 100]}
]
```

### `encode(input_json: string) -> Uint8Array`

Encode files with custom options.

**Input JSON format:**
```json
{
  "files": [
    {"path": "file1.txt", "content": [72, 101, 108, 108, 111]}
  ],
  "options": {
    "mode": "auto",
    "font": "swiss",
    "sort_mode": "lexicographic"
  }
}
```

**Options:**
- `mode`: `"auto"` (default), `"indexed"`, or `"rgba"`
- `font`: `"swiss"`, `"sixth"`, `"sky"`, `"monte"`, `"sugimori"`, `"mini"`, `"micro"`, or `null` (auto)
- `sort_mode`: `"lexicographic"` (default), `"reverse"`, `"by_size"`, `"by_extension"`, or `"none"`

### `version() -> string`

Returns the zipng-wasm version.

## Sort Modes

- **lexicographic**: Sort paths A-Z (default)
- **reverse**: Sort paths Z-A
- **by_size**: Smallest to largest files
- **by_extension**: Group by file extension
- **none**: Preserve insertion order

## File Size Limits

- **Per-file maximum**: 60KB (enforced, will error)
- **Total archive**: Unlimited (spans multiple IDAT blocks)
- **RGBA threshold**: Auto mode switches to RGBA for archives >2MB

## Architecture

### Type Conversion Strategy

Instead of using generic `IntoIterator` and closures, the WASM API uses concrete JSON types:

- Files as `Vec<FileInput>` (path: String, content: Vec<u8>)
- Sort strategies as enum strings
- Options as a dedicated struct

This avoids wasm-bindgen limitations with generics and closures.

### Dependency Management

The core `zipng` crate's I/O-dependent features are disabled for WASM:
- `image` crate: Optional, uses pure-Rust `png` crate for font loading
- `brotli`, `zip`: Optional, disabled in WASM builds
- `walkdir`: Feature-gated, not available in WASM

### Font Loading

Fonts are loaded at compile-time using the `png` crate with transformations:
- Indexed/palette PNGs → RGB → Luminance
- RGBA → Luminance
- Grayscale → Direct use

No `image` crate needed in WASM builds.

## Testing

### Deno CLI Test
```bash
cd ../deno
deno run --allow-read --allow-write zipng-cli.ts -o test.png file1.txt file2.txt
file test.png  # Verify it's a PNG
unzip -l test.png  # Verify it's a ZIP
```

### Node.js Test
```bash
node test-node.js
file test_node_output.png
unzip -l test_node_output.png
```

### Web Test
```bash
python3 -m http.server 8000
# Open http://localhost:8000/test.html
```

## Technical Details

### Build Process

1. `wasm-pack build` compiles Rust → WASM
2. Generates JavaScript bindings with wasm-bindgen
3. Optimizes with wasm-opt
4. Creates three platform-specific packages

### WASM Size

- **Raw WASM**: ~474 KB (release, optimized)
- **gzipped**: ~110 KB

### Performance

Encoding performance is comparable to native builds. Font rendering and palette generation happen at compile-time.

## License

MIT OR Apache-2.0
