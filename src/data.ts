import * as records from "./records.js";

export type Sku = Game | AddOn | Bundle | Subscription;

export class DataStore {
  // Update to clear existing data when making incompatible schema changes.
  readonly schema = 20_2002_05;

  private skusById = new Map<Sku['sku'], Sku>();

  private constructor() {}

  public static async open() {
    const data = new DataStore();
    await data.load();
    return data;
  }

  private async migrate() {
    const existingSchema = (await browser.storage.local.get("schema")).schema;
    if (this.schema !== existingSchema) {
      console.info(
        `Resetting extension storage (data schema was ${existingSchema} but we need ${this.schema}).`,
      );
      await browser.storage.local.clear();
      await browser.storage.local.set({ schema: this.schema });
    }
  }

  public async load() {
    await this.migrate();
    const { skusByLocalKey } = await browser.storage.local.get([
      "skusByLocalKey",
    ]);
    console.debug("i'd like to load", skusByLocalKey);
  }

  public async save() {
    await this.migrate();
    await browser.storage.local.set({
      skusByLocalKey: records.sorted(
        Object.fromEntries(this.skusByLocalKey.entries()),
      ),
    });
  }

  public values(): Iterable<Sku> {
    return this.skusById.values();
  }

  public get(id: string): Sku | undefined {
    return this.skusById.get(id);
  }

  public upsert(sku: Sku): Sku {
    const existing = this.skusById.get(sku.sku);
    if (existing === undefined) {
      this.skusById.set(sku.sku, sku);
      console.info("SKU added", sku);
      return sku;
    }
    
    if (existing !== sku) {
      if (JSON.stringify(existing) !== JSON.stringify(sku)) {
        const previous = {...existing};
        console.info("SKU updated, adding", sku, "to", previous, "producing", existing);
        Object.assign(existing, sku);
      }
    }
    
    return existing;
  }

    

    if (existing) {
      const existingJson = JSON.stringify(existing);
      const newJson = JSON.stringify(sku);
      if (existingJson !== newJson) {
        console.warn(`skus had same ids but different properties.`);
        Object.assign(existing, sku);
      }
      return existing;
    } else {
      this.skus[sku.sku] = sku;
      return sku;
    }
  }
}

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

export class CommonSku {
  readonly proPriceCents: number | null = null;
  readonly proSalePriceCents: number | null = null;
  readonly basePriceCents: number | null = null;
  readonly baseSalePriceCents: number | null = null;

  constructor(
    readonly app: string,
    readonly sku: string,
    readonly type: "game" | "addon" | "bundle" | "subscription",
    readonly name: string,
    readonly internalSlug: string,
    readonly description: string,
  ) {
    this.localKey = localKey(this);
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
    readonly description: string,
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
    readonly description: string,
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
    readonly skus: Array<string>,
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
    readonly skus: Array<string>,
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}
