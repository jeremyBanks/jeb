function assertAsciiCharacter(character: string) {
  if (character.length !== 1) {
    throw new Error(
      `expected a single ASCII character, but got a string of length ${character.length}`,
    );
  }

  if (character >= "\x7F") {
    throw new Error(
      `expected an ASCII character, but got a non-ASCII character`,
    );
  }
}

function assertAsciiString(string: string) {
  for (let i = 0; i < string.length; i++) {
    const character = string[i]!;

    if (character >= "\x7F") {
      throw new Error(
        `expected an ASCII string, but contained a non-ASCII character at index ${i}`,
      );
    }
  }
}

export interface EncodingSpecification {
  /** Digits used by this encoding for fully-encoded data. */
  readonly digits: string;

  /** Number of bytes of per input data block. */
  readonly blockSize: number;

  /**
   * Character used to pad output to a multiple of the output block size, if
   * used for this encoding.
   */
  readonly padding?: string;

  /** Behaviour for escaping "safe" blocks as-is, if supported for this encoding. */
  readonly escaping?: {
    /**
     * Characters used to indicate a single escaped block.
     */
    readonly singleBlockEscape: string;

    /**
     * Characters used to indicate a variable number of blocks to be escaped.
     *
     * Two consecutive instances of this character indicate two blocks.
     *
     * Two instances of this character wrapping an integer value encoded using
     * this encoding's digits indicates that many escaped blocks.
     */
    readonly variableBlockEscape: string;

    /**
     * Character used to indicate that the remainder of the message is escaped.
     *
     * If unspecified, escaping the remainder will instead be indicated by two
     * instances of `variableBlockEscape` wrapping the `singleBlockEscape`
     * character.
     */
    readonly remainderEscape?: string;

    /**
     * Character used to pad the end of escaped blocks to maintain original
     * length.
     */
    readonly padding: string;

    /**
     * Extra characters that are safe for escaped data in this encoding, in
     * addition to the digits and escape characters specified above.
     */
    readonly extraSafeCharacters?: string;
  };
}

export interface EncodingConfiguration {
  readonly strict?: boolean;
}

export class Encoding {
  readonly specification: EncodingSpecification;
  readonly configuration?: EncodingConfiguration;

  #digitCount: number;
  #digitValues: Map<string, number>;
  #maxBlockValue: number;
  #digitCharacters: Set<string>;
  #allCharacters: Set<string>;
  #caseSensitive: boolean;
  #dataBlockSize: number;
  #encodedBlockSize: number;
  #partialBlockSizes: [data: number, encoded: number][];

  constructor(
    specification: EncodingSpecification,
    configuration?: EncodingConfiguration,
  ) {
    this.encode = this.encode.bind(this);
    this.decode = this.decode.bind(this);

    this.specification = specification;
    this.configuration = configuration;

    this.#digitCount = specification.digits.length;

    this.#dataBlockSize = specification.blockSize;
    this.#maxBlockValue = Math.pow(256, specification.blockSize) -
      1;
    this.#encodedBlockSize = Math.ceil(
      Math.log(this.#maxBlockValue + 1) / Math.log(this.#digitCount),
    );

    this.#partialBlockSizes = [];
    for (
      let partialBlockSize = 1;
      partialBlockSize < this.#dataBlockSize;
      partialBlockSize += 1
    ) {
      const encodedSize = Math.ceil(
        Math.log(Math.pow(256, partialBlockSize)) / Math.log(this.#digitCount),
      );
      this.#partialBlockSizes.push([partialBlockSize, encodedSize]);
    }

    assertAsciiString(specification.digits);
    this.#digitCharacters = new Set(specification.digits);

    this.#digitValues = new Map(
      [...specification.digits].map((s, i) => [s, i]),
    );

    this.#caseSensitive = specification.digits.length !==
      new Set(specification.digits.toLowerCase()).size;

    if (specification.escaping) {
      assertAsciiCharacter(specification.escaping.singleBlockEscape);
      assertAsciiCharacter(specification.escaping.variableBlockEscape);
      assertAsciiCharacter(specification.escaping.padding);
      if (specification.escaping.extraSafeCharacters) {
        assertAsciiString(specification.escaping.extraSafeCharacters);
      }
      this.#allCharacters = new Set([
        ...specification.digits,
        ...specification.escaping.singleBlockEscape,
        ...specification.escaping.variableBlockEscape,
        ...specification.escaping.padding,
        ...(specification.escaping.extraSafeCharacters ?? []),
      ]);
    } else {
      this.#allCharacters = this.#digitCharacters;
    }
  }

  withEscaping(escaping?: EncodingSpecification["escaping"]): Encoding {
    return new Encoding({
      ...this.specification,
      escaping,
    }, this.configuration);
  }

  withConfiguration(configuration?: EncodingConfiguration): Encoding {
    return new Encoding(this.specification, configuration);
  }

  strict(strict: boolean = true): Encoding {
    return this.withConfiguration({ strict });
  }

  // This sure ain't right.
  #encodeUint(value: number): string {
    const digits = [];
    while (value > 0) {
      const digitValue = value % this.#digitCount;
      value = (value - digitValue) / this.#digitCount;
      const digit = this.specification.digits[digitValue]!;
      digits.push(digit);
    }
    return digits.reverse().join("");
  }

  #decodeUint(encoded: string): number {
    let value = 0;
    for (const character of encoded) {
      value *= this.#digitCount;
      const digitValue = this.#digitValues.get(character);
      if (digitValue === undefined) {
        throw new Error("unexpected character in encoded data");
      }
      value += digitValue;
    }
    return value;
  }

  #encodeBlock(block: Uint8Array): string {
    let encodedSize;
    if (block.length === this.#dataBlockSize) {
      encodedSize = this.#encodedBlockSize;
    } else {
      encodedSize = this.#partialBlockSizes.find(([dataSize, _encodedSize]) =>
        dataSize === block.length
      )![1];
    }
    if (block.length < this.#dataBlockSize) {
      const maximumForBlockSize =
        this.#encodeUint(Math.pow(this.#digitCount, block.length)).length;
      const padded = new Uint8Array(this.#dataBlockSize);
      padded.set(block);
      return this.#encodeBlock(padded).slice(0, maximumForBlockSize);
    }

    let value = 0;
    for (const byte of block) {
      value *= 256;
      value += byte;
    }

    return this.#encodeUint(value).padStart(
      encodedSize,
      this.specification.digits[0]!,
    );
  }

  #decodeBlock(block: string): Uint8Array {
    throw new Error("not implemented");
  }

  encode(data: Uint8Array): string {
    let buffer = "";
    for (let i = 0; i < data.length; i += this.specification.blockSize) {
      const block = data.slice(i, i + this.specification.blockSize);
      buffer += this.#encodeBlock(block);
    }
    return buffer;
  }

  decode(encoded: string): Uint8Array {
    throw new Error("not implemented");
  }
}
