import { CommonSku, Sku } from "./models.js";

/// Generates a local key that orders SKUs by type, truncated app ID, and
/// truncated SKU ID, and includes a potentially-truncated copy of the SKU's
/// name. The ordering should be adequate for tree indexing if we ever use it,
/// while potentially providing some minimal human-readability.
export const localKey = (sku: Sku | CommonSku) => {
  const length = 32;
  const maxNameLength = 23;
  const typeTag =
    ({ game: "g", addon: "o", bundle: "x", subscription: "c" } as any)[
      sku.type
    ] ?? `?${sku.type}?`;
  const idsPrefix = sku.app.slice(0, 6) + sku.sku.slice(0, 2);
  const idsRest = sku.app.slice(6) + sku.sku.slice(2);

  let name = (sku.name + sku.internalSlug)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "");

  if (name.length > maxNameLength) {
    const letterCounts: Record<string, number> = {};
    for (const letter of name) {
      letterCounts[letter] = (letterCounts[letter] || 0) + 1;
    }
    while (name.length > maxNameLength) {
      const mostFrequentCount = Math.max(...Object.values(letterCounts));
      const mostFrequent = Object.entries(letterCounts)
        .filter(([_letter, count]) => count == mostFrequentCount)
        .map(([letter, _count]) => letter);
      for (let i = name.length - 1; i >= 0; i -= 1) {
        const letter = name[i];
        if (mostFrequent.includes(letter)) {
          name = name.slice(0, i) + name.slice(i + 1);
          letterCounts[letter] -= 1;
          break;
        }
      }
    }
  }

  return (typeTag + idsPrefix + name + idsRest).slice(0, length);
};
