// This module shouldn't know about spidering, except
// in enough to import the spider metadata class.

class RecordStore {
  /** @type {Map<
    StadiaItem['key'],
    Readonly<SpideredItem>
  >} */ #itemByKey = new Map();
}

// TypeScript's JSDoc implementation doesn't let you express their entire type
// system, so some definitions need to go in a .ts file instead.
/** @typedef {import('./d').Addon} Addon */
/** @typedef {import('./d').Bundle} Bundle */
/** @typedef {import('./d').Game} Game */
/** @typedef {import('./d').List} List */
/** @typedef {import('./d').Sku} Sku */
/** @typedef {import('./d').SpideredItem} SpideredItem */
/** @typedef {import('./d').SpiderMetadata} SpiderMetadata */
/** @typedef {import('./d').StadiaItem} StadiaItem */
