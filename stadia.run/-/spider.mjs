import {
  Addon,
  Bundle,
  Game,
  List,
  Record,
  Sku,
  Subscription,
  getset as record,
} from "./data.mjs";

const stadiaPro = /** @type {Subscription} */ (record({
  type: "subscription",
  skuId: "59c8314ac82a456ba61d08988b15b550",
}));

const allGames = /** @type {List} */ (record({
  type: "list",
  listId: 4,
}));

const seeds = [allGames, stadiaPro];
