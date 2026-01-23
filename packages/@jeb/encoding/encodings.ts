import { BASE_64_URL, HEX_LOWERCASE, Z85 } from "./standards.ts";

/**
 * Extended base64url encoding preserving blocks of characters unreserved by
 * RFC 3986.
 */
export const ENCODING_66 = BASE_64_URL.withEscaping({
  singleBlockEscape: "~",
  variableBlockEscape: ".",
  padding: "_",
});

/**
 * Extended base64url encoding preserving blocks of characters unreserved by
 * RFC 2396.
 */
export const ENCODING_66_70 = BASE_64_URL.withEscaping({
  ...ENCODING_66.specification.escaping!,
  extraSafeCharacters: "!()'",
});

/**
 * Extended Z85 encoding preserving blocks of characters that are safe as-is
 * in any type of JavaScript string literal.
 */
export const ENCODING_87_91 = Z85.withEscaping({
  singleBlockEscape: "|",
  variableBlockEscape: " ",
  remainderEscape: ";",
  padding: " ",
  extraSafeCharacters: ",_~",
});

/** Extended hex encoding preserving blocks of alphanumeric characters. */
export const ENCODING_18_36 = HEX_LOWERCASE.withEscaping({
  singleBlockEscape: "y",
  variableBlockEscape: "z",
  padding: "z",
  extraSafeCharacters: "ghijklmnopqrstuvwx",
});
