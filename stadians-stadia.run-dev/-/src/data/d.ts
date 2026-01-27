// TypeScript's JSDoc implementation doesn't let you express their entire type
// system, so some definitions need to go here instead.

export * from "@shopify/useful-types";

export interface RecordStore {
  readonly records: Readonly<Record<StadiaItem["key"], Readonly<StadiaItem>>>;

  readonly metadata: Readonly<
    Record<StadiaItem["key"], Readonly<ItemMetadata>>
  >;

  get(key: StadiaItem["key"]): ItemRef;
}

export interface ItemRef<T extends StadiaItem = StadiaItem> {
  readonly key: string & { [0]: "/" };

  readonly meta: Readonly<ItemMetadata>;

  readonly type?: T["type"];
  readonly item?: Readonly<T>;
}

/** Our spidering-related metadata for a given item.
 *  All timestamps in milliseconds since the epoch.
 */
export interface ItemMetadata {
  /** When we last spidered for this item directly. */
  lastSpidered: number;
  /** The most recent time we saw a property of this item change. */
  lastModified: number;

  /** The first time we saw this record's ID, directly or indirectly. */
  firstSeen: number;
  /** The most recent time we saw this record's ID, directly or indirectly. */
  lastSeen: number;
}

export type StadiaItem = Sku | List;

interface IStadiaItem {
  key: string;
  type: string;
}

export type Sku = Game | Bundle | Addon | Subscription;

interface ISku extends IStadiaItem {
  name: "string";
}

export interface Game extends ISku {
  type: "game";
}

export interface Bundle extends ISku {
  type: "bundle";
}

export interface Addon extends ISku {
  type: "addon";
}

export interface Subscription extends ISku {
  type: "subscription";
}

export interface List extends IStadiaItem {
  type: "list";
  listId: number;
}
