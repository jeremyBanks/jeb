/**
 * Ephemeral in-memory record of all known SKUs.
 * @type {{[skuId: string]: Sku}}
 */
export const skus = {};

export const upserlect = (
  /** @type {
  Pick<ASku, "skuId" | "type"> & Partial<Sku>
} */ newSku,
) => {
  const now = Date.now();
  const existingSku = skus[newSku.skuId];
  const sku = existingSku || makeSku(newSku.type);
  if (existingSku) {
    for (const [property, newValue] of Object.entries(newSku)) {
      const oldValue = existingSku[property];
      if (newValue !== oldValue && newValue != null) {
        sku[property] = newValue;
        sku.lastModified = now;
      }
    }
  } else {
    Object.assign(sku, newSku, { firstSeen: now });
    sku.firstSeen = now;
  }
  sku.lastSeen = now;
  skus[sku.skuId] = sku;
  return sku;
};

/** @typedef {Game | Subscription | Bundle | Addon} Sku */

/** @type {
 ((type: Game["type"]) => Game) &
 ((type: Bundle["type"]) => Bundle) &
 ((type: Addon["type"]) => Addon) &
 ((type: Subscription["type"]) => Subscription) &
 ((type: Sku["type"]) => Sku)
}} */
const makeSku = (/** @type {Sku["type"]} */ type) => {
  if (type === "game") return new Game();
  if (type === "bundle") return new Bundle();
  if (type === "addon") return new Addon();
  if (type === "subscription") return new Subscription();
  throw new TypeError();
};

class ASku {
  /** @type {string} */ skuId;
  /** @type {string} */ name;
  /** @type {string} */ type;
  /** @type {number} */ firstSeen = 0;
  /** @type {number} */ lastSeen = 0;
  /** @type {number} */ lastSpidered = 0;
  /** @type {number} */ lastModified = 0;
}

export class Game extends ASku {
  /** @type {"game"} */ type = "game";
  /** @type {string} */ appId;
  /** @type {string} */ slug;
  /** @type {string} */ cover720Url;
  /** @type {string} */ coverMicroData;
}

export class Subscription extends ASku {
  /** @type {"subscription"} */ type = "subscription";
  /** @type {{
   *    [skuId: string]: {
   *      firstSeen: number,
   *      lastSeen: number,
   *    }
   *  }} */ skus;
}

export class Bundle extends ASku {
  /** @type {"bundle"} */ type = "bundle";
  /** @type {{
   *    [skuId: string]: {
   *      firstSeen: number,
   *      lastSeen: number,
   *    }
   *  }} */ skus;
}

export class Addon extends ASku {
  /** @type {"addon"} */ type = "addon";
}
