import { canFetchStadiaStore, fetchStadia } from "./net.mjs";

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
      try {
        values.push(JSON.parse(currentJson));
      } catch (jsonError) {
        try {
          values.push(JSON5.parse(currentJson));
        } catch (json5error) {}
      }
      currentJsonStartIndex = null;
    }

    index += 1;
    if (index >= text.length) {
      return values;
    }
  }
};

canFetchStadiaStore.then(async () => {
  const response = await fetchStadia(
    "store/details/cc97434908874852a6705d255a605dc8rcp1/sku/1e4107605f83447fa8e04e0abdc578b3",
  );

  const body = await response.text();
  const doc = new DOMParser().parseFromString(body, "text/html");
  const scripts = [...doc.querySelectorAll("script")];

  const jsons = scripts.flatMap(script => jsonObjects(script.textContent));
  console.log(jsons);
});
