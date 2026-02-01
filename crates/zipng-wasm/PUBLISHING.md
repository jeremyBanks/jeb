# Publishing zipng-wasm

This package is configured for dual-purpose publishing to both NPM and JSR (Deno's JavaScript Registry).

## Build Process

```bash
# Build all packages with publication metadata
./build-publish.sh
```

This script:
1. Builds WASM for web, node, and bundler targets
2. Updates package.json files with proper metadata and bin fields
3. Copies CLI to Node.js package
4. Copies README to all packages

## Package Structure

### NPM - Node.js Package (`pkg/node/`)

**Dual-purpose**: Library + CLI

```json
{
  "name": "zipng-wasm",
  "main": "zipng_wasm.js",     // Library
  "bin": {
    "zipng": "./cli.mjs"        // CLI command
  }
}
```

**Usage as library:**
```javascript
import { encode } from 'zipng-wasm';
const polyglot = encode(JSON.stringify({ files, options }));
```

**Usage as CLI:**
```bash
npm install -g zipng-wasm
zipng -o output.png file1.txt file2.txt

# Or with npx (no install)
npx zipng-wasm -o output.png file1.txt file2.txt
```

### NPM - Web Package (`pkg/web/`)

**Library only** (no CLI - browsers can't run CLI)

```html
<script type="module">
  import init, { encode } from 'zipng-wasm';
  await init();
  const result = encode(filesJson);
</script>
```

### NPM - Bundler Package (`pkg/bundler/`)

**For webpack/rollup/vite users**

```javascript
import { encode } from 'zipng-wasm';
// Bundler handles WASM loading
```

### JSR - Deno Package

**Uses existing Deno CLI + web WASM**

```bash
# As CLI
deno install -A -n zipng jsr:@zipng/wasm/cli
zipng -o output.png file.txt

# As library
import { encode } from "jsr:@zipng/wasm";
```

## Publishing

### To NPM

```bash
# Publish the Node.js package (with CLI)
cd pkg/node
npm publish

# Optionally publish web/bundler variants under different names
# cd pkg/web && npm publish --tag web
```

### To JSR

```bash
# From crates/zipng-wasm/
deno publish --config jsr.json
```

## Testing Before Publishing

### Test NPM package locally

```bash
cd pkg/node
npm link
zipng -o test.png somefile.txt

# Or test directly
node cli.mjs -o test.png somefile.txt
```

### Test as library

```bash
# Node.js
cd pkg/node
node -e "const {encode} = require('./zipng_wasm.js'); console.log(typeof encode)"

# Deno
deno eval "import {encode} from './pkg/web/zipng_wasm.js'; console.log(typeof encode)"
```

## Version Management

Update version in three places:
1. `Cargo.toml` (source of truth)
2. Run `build-publish.sh` (auto-updates package.json files)

Or manually:
- `pkg/node/package.json`
- `pkg/web/package.json`
- `pkg/bundler/package.json`
- `jsr.json`

## Package Features

✅ **Works as library** - import and use programmatically
✅ **Works as CLI** - npx/npm install global
✅ **TypeScript types** - auto-generated .d.ts files
✅ **Multiple platforms** - Node.js, browsers, Deno
✅ **Multiple ecosystems** - NPM and JSR
✅ **Zero native deps** - Pure WebAssembly

## File Size

- WASM binary: ~474 KB (uncompressed), ~110 KB (gzipped)
- Total package: ~500 KB

## License

MIT OR Apache-2.0
