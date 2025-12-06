import { Encoding } from "./encoding.ts";

/**
 * Standard uppercase hex encoding.
 *
 * {@link https://datatracker.ietf.org/doc/html/rfc3548#section-6}
 */
export const HEX = new Encoding({
  digits: "0123456789ABCDEF",
  blockSize: 1,
});

/**
 * Lowercase hex encoding.
 */
export const HEX_LOWERCASE = new Encoding({
  digits: "0123456789abcdef",
  blockSize: 1,
});

/**
 * Standard uppercase base32 encoding.
 *
 * {@link https://datatracker.ietf.org/doc/html/rfc3548#section-5}
 */
export const BASE_32 = new Encoding({
  digits: "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
  blockSize: 5,
  padding: "=",
});

/**
 * Lowercase base32 encoding.
 *
 * {@link https://datatracker.ietf.org/doc/html/rfc3548#section-5}
 */
export const BASE_32_LOWERCASE = new Encoding({
  digits: "abcdefghijklmnopqrstuvwxyz234567",
  blockSize: 5,
  padding: "=",
});

/**
 * Standard base64 encoding.
 *
 * {@link https://datatracker.ietf.org/doc/html/rfc3548#section-3}
 */
export const BASE_64 = new Encoding({
  digits: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
  blockSize: 3,
  padding: "=",
});

/**
 * Standard base64url encoding.
 *
 * {@link https://datatracker.ietf.org/doc/html/rfc3548#section-4}
 */
export const BASE_64_URL = new Encoding({
  digits: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
  blockSize: 3,
});

/**
 * Z85 encoding with added support for incomplete trailing blocks.
 *
 * {@link https://rfc.zeromq.org/spec/32/}
 */
export const Z85 = new Encoding({
  digits:
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#",
  blockSize: 4,
});
