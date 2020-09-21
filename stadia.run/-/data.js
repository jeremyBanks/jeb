import { slugify } from "./index.js";

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
      if (
        newValue != undefined &&
        newValue !== oldValue &&
        JSON.stringify(newValue) !== JSON.stringify(oldValue)
      ) {
        record[property] = newValue;
        if (oldValue != undefined && !/^(first|last)/.test(property)) {
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
  if (!type) throw new TypeError("no .type");
  if (type === "game") return new Game();
  if (type === "bundle") return new Bundle();
  if (type === "addon") return new Addon();
  if (type === "subscription") return new Subscription();
  if (type === "list") return new List();
  if (type === "preorder") return new Preorder();
  if (type === "organization") return new Organization();
  if (type === "user") return new User();

  if (type !== "addon-subscription") {
    console.warn(`weird type: ${type}`);
  }

  return new UnknownTypeSku();
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

  /**
   * A modifier on the speed with which records of this type get
   * stale.
   */
  get _spiderFrequencyCoefficient() {
    return 1.0;
  }

  age(now = Date.now()) {
    const knownStale = this.lastModified > this.lastSpidered + 10;
    let frequencyCoefficient = this._spiderFrequencyCoefficient;
    if (knownStale) {
      frequencyCoefficient *= 64;
    }
    return (Date.now() - this.lastSpidered) * frequencyCoefficient;
  }
}

export class List extends ARecord {
  /** @type {"list"} */ type = "list";

  get _spiderFrequencyCoefficient() {
    if (this.lastSpidered === 0 || this.childSkuIds.length > 0) {
      return 8.0;
    } else {
      return 1 / 8.0;
    }
  }

  /** @type {number} */ listId;
  get _key() {
    return `/list/${this.listId}`;
  }
}

export class User extends ARecord {
  /** @type {string} */ userId;
  /** @type {string} */ name;
  /** @type {string} */ number;
  /** @type {string} */ playedAppIds;

  /** @type {"list"} */ type = "list";
  get _spiderFrequencyCoefficient() {
    return 1 / 32.0;
  }

  get _key() {
    return `u${this.userId}`;
  }
}

class ASku extends ARecord {
  /** @type {string} */ skuId;
  /** @type {string} */ coverUrl;
  /** @type {number} */ releaseDateA;
  /** @type {number} */ releaseDateB;
  /** @type {string} */ coverMicroData;
  /** @type {Array<string>} */ developerOrganizationIds;
  /** @type {string} */ publisherOrganizationId;

  get _key() {
    return `${this.skuId}`;
  }

  get slug() {
    if (this.type === "list") {
      return undefined;
    } else if (this._slug) {
      return this._slug;
    } else if (true || this.type === "game" || this.type === "subscription") {
      return slugify(this.name);
    } else {
      return slugify(
        this._key.slice(0, 8) + "-" + slugify(this.name || "").slice(0, 23),
      );
    }
  }

  set slug(slug) {
    this._slug = slug;
  }
}

export class UnknownTypeSku extends ASku {}

export class Preorder extends ASku {
  /** @type {"preorder"} */ type = "preorder";
}

export class Game extends ASku {
  /** @type {"game"} */ type = "game";
  /** @type {string} */ appId;
}

export class Subscription extends ASku {
  /** @type {"subscription"} */ type = "subscription";
  /** @type {Array<string>} */ childSkuIds;

  get _spiderFrequencyCoefficient() {
    return 8.0;
  }
}

export class Bundle extends ASku {
  /** @type {"bundle"} */ type = "bundle";
  /** @type {Array<string>} */ childSkuIds;
}

export class Addon extends ASku {
  /** @type {"addon"} */ type = "addon";

  get _spiderFrequencyCoefficient() {
    return 1 / 16;
  }
}

export class NonDirectlySpiderable extends ARecord {
  get _spiderFrequencyCoefficient() {
    return 0;
  }
}

export class Organization extends NonDirectlySpiderable {
  /** @type {"addon"} */ type = "organization";
  /** @type {string} */ organizationId;

  get _key() {
    return `${this.organizationId}`;
  }
}
