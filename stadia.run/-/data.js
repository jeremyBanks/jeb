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
  now = Date.now(),
) => {
  if (!newProps.type) {
    throw new Error("no .type in " + JSON.stringify(newProps));
  }

  if (newProps.gameIds && newProps.gameIds.length === 0) {
    delete newProps.gameIds;
  } else if (newProps.gameIds) {
    for (const gameId of newProps.gameIds) {
      getset({ type: "game", gameId });
    }
  }

  const newRecord = Object.assign(makeRecord(newProps.type), newProps);

  const existingRecord = records[newRecord._key];
  const record = existingRecord ?? makeRecord(newRecord.type);

  let modified = false;
  if (existingRecord) {
    if (
      existingRecord.imageHash &&
      existingRecord.imageHash === newProps.imageHash
    ) {
      // ignore URL changes if the content is the same
      newProps.imageUrl = existingRecord.imageUrl;
      // this should be identical, but canvas behaviour can slightly
      // vary so let's also preserve it.
      newProps.thumbnail = existingRecord.thumbnail;
    }

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
    debugger;
    throw new TypeError("record corrupt, missing key or type");
  }

  record.lastSeen = now;
  record.lastModified = modified ? now : record.lastModified ?? 0;

  if (
    !record.lastSpidered ||
    record.lastSpidered < now - 1000 * 60 * 60 * 24 * 512
  ) {
    record.lastSpidered = 0;
  }

  if (!record.firstSeen || record.firstSeen < now - 1000 * 60 * 60 * 24 * 512) {
    record.firstSeen = now;
  }

  records[record._key] = record;
  if (record.type === "game" && record.skuId) {
    records[record.skuId] = record;
  }
  return record;
};

export const deriveDerivedDerivations = async () => {
  let proGamesLoaded = 0;

  const recordsOfType = {};
  for (const record of new Set(Object.values(records))) {
    (recordsOfType[record.type] = recordsOfType[record.type] || []).push(
      record,
    );
  }

  let proGameSkus = new Set();
  let addProGames = skuId => {
    let sku = records[skuId];
    if (!sku) {
      console.error("could not find pro game", skuId);
      proGamesLoaded = -Infinity;
    } else if (sku.type === "game") {
      proGamesLoaded += 1;
      proGameSkus.add(skuId);
    } else if (sku.childSkuIds) {
      sku.childSkuIds.forEach(addProGames);
    }
  };
  addProGames("59c8314ac82a456ba61d08988b15b550");

  const slugs = new Set();
  for (const game of recordsOfType.game) {
    if (slugs.has(game.slug)) {
      throw new Error("duplicate game slug: ", game.slug);
    }
    slugs.add(game.slug);

    if (proGamesLoaded > 0) {
      game.isPro = proGameSkus.has(game.skuId);
    }

    game.wasPro = game.wasPro || game.isPro;
  }

  // TODO: publishers?
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
  if (type === "avatar") return new Avatar();
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
    return (now - this.lastSpidered) * frequencyCoefficient;
  }
}

export class List extends ARecord {
  /** @type {"list"} */ type = "list";

  get _spiderFrequencyCoefficient() {
    if (this.childSkuIds && this.childSkuIds.length > 0) {
      return 2;
    } else {
      return 1 / 16;
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
  /** @type {string} */ gameIds;

  /** @type {"user"} */ type = "user";
  get _spiderFrequencyCoefficient() {
    if (this.gameIds?.length && !this.games?.length) {
      // stale! or has become private?
      return 1;
    } else if (this.lastActive && this.games?.length) {
      return 1 / 8;
    } else if (this.lastActive || this.games?.length || this.gameIds?.length) {
      return 1 / 32;
    } else {
      return 1 / 64;
    }
  }

  get _key() {
    return `u${this.userId}`;
  }
}

class ASku extends ARecord {
  /** @type {string} */ skuId;
  /** @type {string} */ imageUrl;
  /** @type {number} */ releaseDateA;
  /** @type {number} */ releaseDateB;
  /** @type {string} */ thumbnail;
  /** @type {Array<string>} */ developerOrganizationIds;
  /** @type {string} */ publisherOrganizationId;

  get _key() {
    return `${this.type === "game" ? this.gameId : this.skuId}`;
  }

  get slug() {
    if (this.type !== "game") {
      return undefined;
    } else if (this._slug) {
      return this._slug;
    } else if (this.name) {
      return slugify(this.name);
    } else {
      return slugify(this._key);
    }
  }

  set slug(slug) {
    if (slug && slug !== "undefined" && slug !== "null") {
      this._slug = slug;
    }
  }
}

export class UnknownTypeSku extends ASku {}

export class Preorder extends ASku {
  /** @type {"preorder"} */ type = "preorder";

  get _spiderFrequencyCoefficient() {
    return 4.0;
  }
}

export class Game extends ASku {
  /** @type {"game"} */ type = "game";
  /** @type {string} */ gameId;
  /** @type {boolean} */ isPro;
  /** @type {boolean} */ wasPro;

  get _spiderFrequencyCoefficient() {
    return 2.0;
  }
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
    return 1 / 32;
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

export class Avatar extends NonDirectlySpiderable {
  /** @type {"addon"} */ type = "avatar";
  /** @type {string} */ avatarId;

  get _key() {
    return `${this.avatarId}`;
  }
}
