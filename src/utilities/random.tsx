/**
 * Placeholder value used to distinguish missing arguments from `undefined`
 * arguments. (Infrequent, in most cases we want to treat them equivalently.)
 */
let Missing = Symbol("Missing");

/**
 * Generates a random number between 0 (inclusive) and 1 (exclusive).
 */
export let randomFraction = () => {
  return Math.random();
};

/**
 * Generates a random integer in the specified inclusive range.
 *
 * This cryptographically-safe uniformly-distributed implementation is overkill
 * but it should still be more than fast enough for our purposes.
 *
 * Based on https://stackoverflow.com/a/55544949.
 */
export let randomInteger = (min: number, max: number): number => {
  let range = max - min;
  let maxGeneratedValue = 0xffffffff;
  let possibleResultValues = range + 1;
  let possibleGeneratedValues = maxGeneratedValue + 1;
  let remainder = possibleGeneratedValues % possibleResultValues;
  let maxUnbiased = maxGeneratedValue - remainder;

  if (
    !Number.isInteger(min) ||
    !Number.isInteger(max) ||
    max > Number.MAX_SAFE_INTEGER ||
    min < Number.MIN_SAFE_INTEGER
  ) {
    throw new Error("Arguments must be safe integers, but were out of bounds.");
  } else if (range > maxGeneratedValue) {
    throw new Error(
      `Range of ${range} (from ${min} to ${max}) > ${maxGeneratedValue}.`
    );
  } else if (max < min) {
    throw new Error(`max (${max}) must be >= min (${min}).`);
  } else if (min === max) {
    return min;
  }

  let generated;
  do {
    generated = crypto.getRandomValues(new Uint32Array(1))[0];
  } while (generated > maxUnbiased);

  return min + (generated % possibleResultValues);
};

/**
 * Chooses and returns an element at random from an array or other iterable.
 *
 * If the iterable contains no elements, throws an error unless an
 * `emptyDefault` is provided.
 */
export let randomChoice = <Value extends unknown = unknown>(
  values: Array<Value> | Iterable<Value>,
  emptyDefault: Value | typeof Missing = Missing
): Value => {
  let valueArray = [...values];

  if (valueArray.length === 0) {
    if (emptyDefault === Missing) {
      throw new Error(
        "attempted to take randomChoice() from an empty iterable"
      );
    } else {
      return emptyDefault as Value;
    }
  }

  return valueArray[randomInteger(0, valueArray.length - 1)];
};
