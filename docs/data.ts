export type Sku = Game | AddOn | Bundle | Subscription;

export const internalKey = (sku: Sku | CommonSku) => {
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

export class CommonSku {
  constructor(
    readonly app: string,
    readonly sku: string,
    readonly type: "game" | "addon" | "bundle" | "subscription",
    readonly name: string,
    readonly internalSlug: string,
    readonly description: string
  ) {
    this.localKey = internalKey(this);
  }
  readonly localKey: string;
}

export class Game extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "game" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}

export class AddOn extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "addon" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}

export class Bundle extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "bundle" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string,
    readonly skus: Array<string>
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}

export class Subscription extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "subscription" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string,
    readonly skus: Array<string>
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}
