import JSON5 from "./json5.mjs";

/** @typedef {
 Array<JsonValue> |
 Record<string, JsonValue> |
 string | number | null
} JsonValue */

export const jsonObjects = (/** @type {string} */ text) => {
  /** @type {Array<JsonValue & object>} */ const values = [];

  if (!text) {
    return values;
  }

  let index = 0;
  let currentJsonStartIndex = null;

  /** @type {Array<'[' | '{' | '"' | '\\'>} */
  const delimiterStack = [];

  for (;;) {
    const character = text[index];

    if (currentJsonStartIndex === null) {
      if (character === "{" || character === "[") {
        delimiterStack.push(character);
        currentJsonStartIndex = index;
      }
    } else {
      const topDelimiter = delimiterStack[delimiterStack.length - 1];
      if (topDelimiter === "\\") {
        delimiterStack.pop();
      } else if (topDelimiter === '"') {
        if (character === '"') {
          delimiterStack.pop();
        }
      } else if (character === "{" || character === "[") {
        delimiterStack.push(character);
      } else if (topDelimiter === "{" && character === "}") {
        delimiterStack.pop();
      } else if (topDelimiter === "[" && character === "]") {
        delimiterStack.pop();
      }
    }

    if (currentJsonStartIndex !== null && delimiterStack.length === 0) {
      const currentJson = text.slice(currentJsonStartIndex, index + 1);
      let parsed = undefined;
      try {
        parsed = JSON.parse(currentJson);
      } catch (jsonError) {
        try {
          parsed = JSON5.parse(currentJson);
        } catch (json5error) {}
      }
      if (parsed !== undefined) {
        if (parsed instanceof Array && parsed.length > 0) {
          values.push({ values: parsed });
        } else if (Object.keys(parsed).length > 0) {
          values.push(parsed);
        }
      }
      currentJsonStartIndex = null;
    }

    index += 1;
    if (index >= text.length) {
      return values;
    }
  }
};
