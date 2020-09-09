/**
 * Ephemeral in-memory record of all known records.
 * @type {{[key: string]: Record}}
 */
export const records = {};

export const getset = (
  /** @type {
  (Pick<ASku, "skuId" | "type"> & Partial<Sku>) |
  (Pick<List, "listId" | "type"> & Partial<List>)
} */ newProps,
) => {
  const newRecord = Object.assign(makeRecord(newProps.type), newProps);
  const now = Date.now();
  const existingRecord = records[newRecord._key];
  const record = existingRecord ?? makeRecord(newRecord.type);

  let modified = false;
  if (existingRecord) {
    for (const [property, newValue] of Object.entries(newRecord)) {
      const oldValue = existingRecord[property];
      if (newValue !== oldValue && newValue != undefined) {
        record[property] = newValue;
        if (oldValue != undefined && property !== "lastSpidered") {
          console.debug(
            `modified ${property} of ${record.type} ${record._key} from ${oldValue} to ${newValue}`,
          );
          modified = true;
        }
      }
    }
    if (
      existingRecord.type !== newRecord.type ||
      existingRecord._key !== newRecord._key
    ) {
      throw new Error("data integrity failure");
    }
  } else {
    Object.assign(record, newRecord);
  }

  if (!record.type || !record._key || record._key === "undefined") {
    throw new TypeError("record corrupt, missing key or type");
  }

  record.firstSeen = record.firstSeen ?? now;
  record.lastSeen = now;
  record.lastModified = modified ? now : record.lastModified ?? 0;
  record.lastSpidered = record.lastSpidered ?? 0;

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
  throw new TypeError(`unknown record type: ${type}`);
};

class ARecord {
  /** @type {string} */ type;
  /** @type {string} */ name;
  /** @type {number} */ lastSpidered;
  /** @type {number} */ lastModified;
  /** @type {number} */ firstSeen;
  /** @type {number} */ lastSeen;

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
