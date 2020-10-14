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
  // HACK
  if (newProps.playedAppIds && newProps.playedAppIds.length === 0) {
    delete newProps.playedAppIds;
  }

  const newRecord = Object.assign(makeRecord(newProps.type), newProps);
  const now = Date.now();
  const existingRecord = records[newRecord._key];
  const record = existingRecord ?? makeRecord(newRecord.type);

  let modified = false;
  if (existingRecord) {
    if (
      existingRecord.coverHash &&
      existingRecord.coverHash === newProps.coverHash
    ) {
      // ignore URL changes if the content is the same
      newProps.coverUrl = existingRecord.coverUrl;
      // this should be identical, but canvas behaviour can slightly
      // vary so let's also preserve it.
      newProps.coverMicroData = existingRecord.coverMicroData;
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
    throw new TypeError("record corrupt, missing key or type");
  }

  record.firstSeen = record.firstSeen ?? now;
  record.lastSeen = now;
  record.lastModified = modified ? now : record.lastModified ?? 0;
  record.lastSpidered = record.lastSpidered ?? Math.random() * 0.5 * now;

  records[record._key] = record;
  return record;
};

export const deriveDerivedDerivations = async () => {
  let proGamesLoaded = 0;

  const recordsOfType = {};
  for (const record of Object.values(records)) {
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
  const activePlayerSince = Date.now() - 1000 * 60 * 60 * 24 * 24 - Infinity;

  const recentPlayerCountByGameAppId = {};
  for (const user of recordsOfType.user) {
    if (Math.max(user.firstSeen, user.lastModified) > activePlayerSince)
      for (const appId of (user.playedAppIds || []).slice(0, Infinity)) {
        recentPlayerCountByGameAppId[appId] =
          (recentPlayerCountByGameAppId[appId] || 0) + 1;
      }
  }

  const howPopular =
    Object.values(recentPlayerCountByGameAppId)
      .sort((a, b) => a - b)
      .slice(-1)[0] * 0.75;
  const popularEnough = Object.entries(recentPlayerCountByGameAppId)
    .filter(a => a[1] >= howPopular)
    .map(a => a[0]);

  console.debug({ howPopular, popularEnough, recentPlayerCountByGameAppId });

  const slugs = new Set();
  for (const game of recordsOfType.game) {
    if (slugs.has(game.slug)) {
      throw new Error("duplicate game slug: ", game.slug);
    }
    slugs.add(game.slug);

    game.popular = undefined; // popularEnough.includes(game.appId) ? true : undefined;

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
    return (Date.now() - this.lastSpidered) * frequencyCoefficient;
  }
}

export class List extends ARecord {
  /** @type {"list"} */ type = "list";

  get _spiderFrequencyCoefficient() {
    if (this.childSkuIds && this.childSkuIds.length > 0) {
      return 2.0;
    } else {
      return 1 / 16.0;
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

  /** @type {"user"} */ type = "user";
  get _spiderFrequencyCoefficient() {
    if (this.playedAppIds?.length) {
      return 1 / 32;
    } else {
      return 0;
    }
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
    if (this.type !== "game") {
      return undefined;
    } else if (this._slug) {
      return this._slug;
    } else {
      return slugify(this.name);
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
  /** @type {string} */ appId;
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

export class Avatar extends NonDirectlySpiderable {
  /** @type {"addon"} */ type = "avatar";
  /** @type {string} */ avatarId;

  get _key() {
    return `${this.avatarId}`;
  }
}
