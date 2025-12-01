// @generated file from wasmbuild -- do not edit
// deno-lint-ignore-file
// deno-fmt-ignore-file

export function encode_z85(bytes: Uint8Array): Uint8Array;
export function encode_jeb85(bytes: Uint8Array): Uint8Array;
/**
 * The kind of error encountered during shell tokenization.
 */
export enum ErrorKind {
  /**
   * An unclosed single quote was encountered.
   */
  UnclosedSingleQuote = 0,
  /**
   * An unclosed double quote was encountered.
   */
  UnclosedDoubleQuote = 1,
  /**
   * A trailing backslash was encountered at end of input.
   */
  TrailingBackslash = 2,
  /**
   * Dollar sign for variable expansion (not interpreted).
   */
  DollarSign = 3,
  /**
   * Backtick for command substitution (not interpreted).
   */
  Backtick = 4,
  /**
   * Pipe for piping (not interpreted).
   */
  Pipe = 5,
  /**
   * Ampersand for background/AND (not interpreted).
   */
  Ampersand = 6,
  /**
   * Semicolon as command separator (not interpreted).
   */
  Semicolon = 7,
  /**
   * Newline as command separator (not interpreted).
   */
  Newline = 8,
  /**
   * Open parenthesis for subshell (not interpreted).
   */
  OpenParen = 9,
  /**
   * Close parenthesis for subshell (not interpreted).
   */
  CloseParen = 10,
  /**
   * Less-than for input redirection (not interpreted).
   */
  LessThan = 11,
  /**
   * Greater-than for output redirection (not interpreted).
   */
  GreaterThan = 12,
  /**
   * Hash for comment (not interpreted).
   */
  Hash = 13,
  /**
   * Asterisk glob wildcard (not interpreted).
   */
  Asterisk = 14,
  /**
   * Question mark glob wildcard (not interpreted).
   */
  QuestionMark = 15,
  /**
   * Open bracket for glob bracket expression (not interpreted).
   */
  OpenBracket = 16,
  /**
   * Tilde at word start for tilde expansion (not interpreted).
   */
  Tilde = 17,
}
export class Bytes {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
}
export class Float {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
}
export class Text {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
}
