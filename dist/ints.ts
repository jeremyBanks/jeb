// Unique symbol used for our type brands. Unused at runtime.
const type = Symbol("type");
type OptionallyBranded<Base, Name, Default> = Base &
  ({ [type]?: Name } | Default);
type Branded<Base, Name, Default> = Base & ({ [type]: Name } | Default);

// Standard number, 64 bit floating point.
export type f64 = OptionallyBranded<number, "f64", 0.0>;

// number constrained to u32 range.
export type u32 = Branded<number, "u32", 0>;

export const u32 = (i: number): u32 => {
  if (!u32.check(i)) {
    throw new TypeError(`expected u32, got ${typeof i} ${i}`);
  }
  return i;
};

u32.guard = u32;

u32.check = (i: number): i is u32 =>
  typeof i === "number" && i >= 0 && i <= 0xffffffff && Number.isSafeInteger(i);

// Standard bigint, signed.
export type iBig = OptionallyBranded<bigint, "iBig", 0n>;

// bigint constrained to u64 range.
export type u64 = Branded<bigint, "u64", 0n>;

export const u64 = (i: bigint): u64 => {
  if (!u64.check(i)) {
    throw new TypeError(`expected u64, got ${typeof i} ${i}`);
  }
  return i;
};

u64.guard = u64;

u64.check = (i: bigint): i is u64 =>
  typeof i === "bigint" && i >= 0n && i <= 0xffffffffffffffffn;

u64.wrap = (i: bigint): u64 => BigInt.asUintN(64, i) as u64;

u64.clamp = (i: bigint): u64 => {
  if (i < 0n) {
    return 0n as u64;
  } else if (i > 0xffffffffffffffffn) {
    return 0xffffffffffffffffn as u64;
  } else {
    return i as u64;
  }
};

// u64.toBase64 = (i: u64): string => "0";
// u64.tryFromBase64 = (s: string): u64 | undefined => {
//     if (s.length !== 11) {
//         return undefined;
//     }
//     const byteChars = atob(s.replace(/\+/g, '-').replace(/\//g, '_'));
//     for (let i = 0; i < byteChars.length; i++) {
//         bytes[i] = byteChars.charCodeAt(i);
//     }
// };
// u64.tryFromBase16 = (s: string): u64 | undefined => 0n;
// u64.tryFromBase10 = (s: string): u64 | undefined => {
//     const u = BigInt(i);
//     if (u < 0n || u > 0xFFFFFFFFFFFFFFFF) {
//         throw new TypeError(`u64 out of bounds: ${u}`);
//     }
//     return u;
// };
// u64.tryFrom = (s: string): u64 | undefined =>
//     u64.tryFromBase64(s) ??
//     u64.tryFromBase16(s) ??
// u64.tryFromBase10(s);
