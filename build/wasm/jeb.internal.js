// @generated file from wasmbuild -- do not edit
// @ts-nocheck: generated
// deno-lint-ignore-file
// deno-fmt-ignore-file

let wasm;
export function __wbg_set_wasm(val) {
  wasm = val;
}

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
  if (
    cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0
  ) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder("utf-8", {
  ignoreBOM: true,
  fatal: true,
});

cachedTextDecoder.decode();

const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
  numBytesDecoded += len;
  if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
    cachedTextDecoder = new TextDecoder("utf-8", {
      ignoreBOM: true,
      fatal: true,
    });
    cachedTextDecoder.decode();
    numBytesDecoded = len;
  }
  return cachedTextDecoder.decode(
    getUint8ArrayMemory0().subarray(ptr, ptr + len),
  );
}

function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return decodeText(ptr, len);
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
  const ptr = malloc(arg.length * 1, 1) >>> 0;
  getUint8ArrayMemory0().set(arg, ptr / 1);
  WASM_VECTOR_LEN = arg.length;
  return ptr;
}

function getArrayU8FromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
 * @param {Uint8Array} bytes
 * @returns {Uint8Array}
 */
export function encode_z85(bytes) {
  const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
  const len0 = WASM_VECTOR_LEN;
  const ret = wasm.encode_z85(ptr0, len0);
  var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
  wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
  return v2;
}

/**
 * @param {Uint8Array} bytes
 * @returns {Uint8Array}
 */
export function encode_jeb85(bytes) {
  const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
  const len0 = WASM_VECTOR_LEN;
  const ret = wasm.encode_jeb85(ptr0, len0);
  var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
  wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
  return v2;
}

/**
 * The kind of error encountered during shell tokenization.
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17}
 */
export const ErrorKind = Object.freeze({
  /**
   * An unclosed single quote was encountered.
   */
  UnclosedSingleQuote: 0,
  "0": "UnclosedSingleQuote",
  /**
   * An unclosed double quote was encountered.
   */
  UnclosedDoubleQuote: 1,
  "1": "UnclosedDoubleQuote",
  /**
   * A trailing backslash was encountered at end of input.
   */
  TrailingBackslash: 2,
  "2": "TrailingBackslash",
  /**
   * Dollar sign for variable expansion (not interpreted).
   */
  DollarSign: 3,
  "3": "DollarSign",
  /**
   * Backtick for command substitution (not interpreted).
   */
  Backtick: 4,
  "4": "Backtick",
  /**
   * Pipe for piping (not interpreted).
   */
  Pipe: 5,
  "5": "Pipe",
  /**
   * Ampersand for background/AND (not interpreted).
   */
  Ampersand: 6,
  "6": "Ampersand",
  /**
   * Semicolon as command separator (not interpreted).
   */
  Semicolon: 7,
  "7": "Semicolon",
  /**
   * Newline as command separator (not interpreted).
   */
  Newline: 8,
  "8": "Newline",
  /**
   * Open parenthesis for subshell (not interpreted).
   */
  OpenParen: 9,
  "9": "OpenParen",
  /**
   * Close parenthesis for subshell (not interpreted).
   */
  CloseParen: 10,
  "10": "CloseParen",
  /**
   * Less-than for input redirection (not interpreted).
   */
  LessThan: 11,
  "11": "LessThan",
  /**
   * Greater-than for output redirection (not interpreted).
   */
  GreaterThan: 12,
  "12": "GreaterThan",
  /**
   * Hash for comment (not interpreted).
   */
  Hash: 13,
  "13": "Hash",
  /**
   * Asterisk glob wildcard (not interpreted).
   */
  Asterisk: 14,
  "14": "Asterisk",
  /**
   * Question mark glob wildcard (not interpreted).
   */
  QuestionMark: 15,
  "15": "QuestionMark",
  /**
   * Open bracket for glob bracket expression (not interpreted).
   */
  OpenBracket: 16,
  "16": "OpenBracket",
  /**
   * Tilde at word start for tilde expansion (not interpreted).
   */
  Tilde: 17,
  "17": "Tilde",
});

const BytesFinalization = (typeof FinalizationRegistry === "undefined")
  ? { register: () => {}, unregister: () => {} }
  : new FinalizationRegistry((ptr) => wasm.__wbg_bytes_free(ptr >>> 0, 1));

export class Bytes {
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    BytesFinalization.unregister(this);
    return ptr;
  }

  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_bytes_free(ptr, 0);
  }
}
if (Symbol.dispose) Bytes.prototype[Symbol.dispose] = Bytes.prototype.free;

const FloatFinalization = (typeof FinalizationRegistry === "undefined")
  ? { register: () => {}, unregister: () => {} }
  : new FinalizationRegistry((ptr) => wasm.__wbg_float_free(ptr >>> 0, 1));

export class Float {
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    FloatFinalization.unregister(this);
    return ptr;
  }

  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_float_free(ptr, 0);
  }
}
if (Symbol.dispose) Float.prototype[Symbol.dispose] = Float.prototype.free;

const TextFinalization = (typeof FinalizationRegistry === "undefined")
  ? { register: () => {}, unregister: () => {} }
  : new FinalizationRegistry((ptr) => wasm.__wbg_text_free(ptr >>> 0, 1));

export class Text {
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    TextFinalization.unregister(this);
    return ptr;
  }

  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_text_free(ptr, 0);
  }
}
if (Symbol.dispose) Text.prototype[Symbol.dispose] = Text.prototype.free;

export function __wbg___wbindgen_throw_b855445ff6a94295(arg0, arg1) {
  throw new Error(getStringFromWasm0(arg0, arg1));
}

export function __wbindgen_init_externref_table() {
  const table = wasm.__wbindgen_externrefs;
  const offset = table.grow(4);
  table.set(0, undefined);
  table.set(offset + 0, undefined);
  table.set(offset + 1, null);
  table.set(offset + 2, true);
  table.set(offset + 3, false);
}
