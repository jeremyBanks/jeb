/**
 * Ephemeral in-memory record of all known records.
 * @type {{[key: string]: Record}}
 */
const records = {};

export const getset = (
  /** @type {
  (Pick<ASku, "skuId" | "type"> & Partial<Sku>) |
  (Pick<List, "listId" | "type"> & Partial<List>)
} */ newProps,
) => {
  const newRecord = Object.assign(makeRecord(newProps.type), newProps);
  const now = Date.now();
  const existingRecord = records[newRecord._key];
  const record = existingRecord || makeRecord(newRecord.type);
  if (existingRecord) {
    for (const [property, newValue] of Object.entries(newRecord)) {
      const oldValue = existingRecord[property];
      if (newValue !== oldValue && newValue != null) {
        record[property] = newValue;
        record.lastModified = now;
      }
    }
    if (
      existingRecord.type !== newRecord.type ||
      existingRecord._key !== newRecord._key
    ) {
      throw new Error("data integrity failure");
    }
  } else {
    Object.assign(record, newRecord, { firstSeen: now });
    record.firstSeen = now;
  }
  record.lastSeen = now;

  records[record._key] = record;
  return record;
};

/** @typedef {Game | Subscription | Bundle | Addon} Sku */
/** @typedef {Sku | List} Record */
/** @typedef {{
  [skuId: string]: {
    firstSeen: number,
    lastSeen: number,
  }
}} SkuSet */

const makeRecord = (/** @type {Record["type"]} */ type) => {
  if (type === "game") return new Game();
  if (type === "bundle") return new Bundle();
  if (type === "addon") return new Addon();
  if (type === "subscription") return new Subscription();
  if (type === "list") return new List();
  throw new TypeError("unknown record type");
};

class ARecord {
  /** @type {string} */ type;
  /** @type {string} */ name;
  /** @type {number} */ lastSpidered = 0;
  /** @type {number} */ lastModified = 0;
  /** @type {number} */ firstSeen = 0;
  /** @type {number} */ lastSeen = 0;

  /**
   * A primary key uniquely identifying this record from all others.
   * @returns {string}
   * */
  get _key() {
    throw new TypeError("not implemented");
  }
}

export class List extends ARecord {
  /** @type {"list"} */ type = "list";

  /** @type {number} */ listId;
  get _key() {
    return `/list/${this.listId}`;
  }
}

class ASku extends ARecord {
  /** @type {string} */ skuId;
  get _key() {
    return `${this.skuId}`;
  }
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
    [skuId: string]: {
      firstSeen: number,
      lastSeen: number,
    }
  }} */ skus;
}

export class Bundle extends ASku {
  /** @type {"bundle"} */ type = "bundle";
  /** @type {{
    [skuId: string]: {
      firstSeen: number,
      lastSeen: number,
    }
  }} */ skus;
}

export class Addon extends ASku {
  /** @type {"addon"} */ type = "addon";
}
