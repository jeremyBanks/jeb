import * as records from "../records.js";

import { Sku } from "./models.js";

export class DataStore {
  private skusById = new Map<Sku["sku"], Sku>();

  private constructor() {}

  public static async open() {
    const data = new DataStore();
    await data.load();
    return data;
  }

  private async migrate() {
    // Update to clear existing data when making incompatible schema changes.
    const schema = 20_2002_05;

    const existingSchema = (await browser.storage.local.get("schema")).schema;
    if (schema !== existingSchema) {
      console.info(
        `Resetting extension storage (data schema was ${existingSchema} but we need ${schema}).`,
      );
      await browser.storage.local.clear();
      await browser.storage.local.set({ schema });
    }
  }

  public async load() {
    await this.migrate();
    const { skusById } = await browser.storage.local.get(["skusById"]);
    console.debug("i'd like to load", skusById);
  }

  public async save() {
    await this.migrate();
    await browser.storage.local.set({
      skusById: records.sorted(Object.fromEntries(this.skusById.entries())),
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
        const previous = { ...existing };
        console.info(
          "SKU updated, adding",
          sku,
          "to",
          previous,
          "producing",
          existing,
        );
        Object.assign(existing, sku);
      }
    }

    return existing;
  }
}
