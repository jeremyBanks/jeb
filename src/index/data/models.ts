import { localKey } from "./local-key.js";

export type Sku = Game | AddOn | Bundle | Subscription;

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
    readonly description: string
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
