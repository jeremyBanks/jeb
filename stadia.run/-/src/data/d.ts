// TypeScript's JSDoc implementation doesn't let you express their entire type
// system, so some definitions need to go here instead.

export * from "@shopify/useful-types";

export interface SpiderMetadata {
  lastSpidered?: number;
  lastModified?: number;
  firstSeen?: number;
  lastSeen?: number;
}

export type StadiaItem = Sku | List;

export type SpideredItem<T extends StadiaItem = StadiaItem> = T & {
  meta: SpiderMetadata;
};

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
