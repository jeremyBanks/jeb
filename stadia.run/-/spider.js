import {
  Addon,
  Bundle,
  Game,
  List,
  Subscription,
  getset,
  records,
} from "./data.js";
import { jsonObjects } from "./jsons.js";
import { fetchStadia, checkStatus } from "./net.js";
/** @typedef {import("./data.js").Sku} Sku */
/** @typedef {import("./data.js").Record} Record */

import { sleep } from "./async.js";

const seeds = [
  getset({
    name: "All Games",
    type: "list",
    listId: 3,
  }),
  getset({
    name: "Stadia Pro",
    type: "subscription",
    skuId: "59c8314ac82a456ba61d08988b15b550",
  }),
  getset({
    name: "Celeste",
    type: "game",
    skuId: "68fb07a7c4ac41f1afb21d742c717538",
  }),
];

const loadSkuData = (/** @type {Array<unknown>} */ skuData) => {
  // https://github.com/stadians/stadians/blob/spider/src/foreground/spider.ts#L113
  const skuId = skuData[0];
  const appId = skuData[4];
  const name = skuData[1];
  const type = {
    1: "game",
    2: "addon",
    3: "bundle",
    5: "subscription",
  }[skuData[6]];

  return getset({
    type,
    skuId,
    appId,
    name,
  });
};

const spider = async (/** @type {Record} */ record) => {
  if (record.type === "list") {
    const page = await fetchStadiaPage(`store/list/${record.listId}`);
    for (const sku of page.list) {
      loadSkuData(sku[9]);
    }
  } else {
    const page = await fetchStadiaPage(`store/details/-/sku/${record.skuId}`);
    loadSkuData(page.sku[16]);
  }

  getset({
    ...record,
    lastSpidered: Date.now(),
  });
};

/** @returns {Promise<never>} */
export const spiderThread = async () => {
  for (const seed of seeds) {
    await sleep(4.0);
    await spider(seed);
    console.info("🌱 seeded", seed);
  }

  console.info(
    `${Object.keys(records).length} records after seeding.`,
    records,
  );

  for (;;) {
    await sleep(16.0);
    const record = Object.values(records).sort((a, b) => {
      if (a.lastSpidered ?? NaN < b.lastSpidered ?? NaN) {
        return +1;
      } else if (b.lastSpidered ?? NaN < a.lastSpidered ?? NaN) {
        return -1;
      } else {
        return 0;
      }
    })[0];

    await spider(record);
    console.info("🕷️ spidered", record);
  }
};

const fetchStadiaPage = async url => {
  const opaque = await fetchStadiaOpaque(url);
  const data = Object.create(opaque);

  data.self = opaque.D0Amudob?.[5];
  data.list = opaque.WwD3rbnob?.[2];
  data.gameStats = opaque.e7h9qdoss?.[0]?.[8];
  data.playerGames = opaque.Q6jt8cooos?.[0];
  data.sku = opaque.FWhQVssb;
  data.storefront = opaque.xjyeoc?.[3].flatMap(x => x?.[1]);

  for (const key of Object.keys(data)) {
    if (data[key] === undefined) {
      delete data[key];
    }
  }

  console.debug("Got Stadia page", data);

  return data;
};

const padOpaqueKeys = (/** @type {unknown} */ object) => {
  if (
    object &&
    typeof object === "object" &&
    !(object instanceof Array) &&
    Object.keys(object).length >= 4 &&
    Object.keys(object).every(key => /^[a-zA-Z0-9]{1,6}$/.test(key))
  ) {
    return Object.fromEntries(
      Object.entries(object).map(([key, value]) => [key.padEnd(6, "s"), value]),
    );
  } else {
    return object;
  }
};

export const fetchStadiaJsons = async (path, options = {}) => {
  const response = await fetchStadia(path, options);
  console.debug("Got Stadia response", response);
  checkStatus(response);
  const body = await response.text();
  const doc = new DOMParser().parseFromString(body, "text/html");
  const scripts = [...doc.querySelectorAll("script")];

  return scripts.flatMap(script => jsonObjects(script.textContent));
};

const fetchStadiaOpaque = async url => {
  const jsons = await fetchStadiaJsons(url);

  const data = Object.create(jsons);

  Object.assign(
    data,
    padOpaqueKeys(
      jsons.find(
        x =>
          Object.keys(x).length >= 8 &&
          Object.keys(x).every(key => /^[a-zA-Z0-9]{1,6}$/.test(key)),
      ),
    ),
  );

  const preloadQueries = jsons.find(x => x?.["ds:0"]?.["id"]);

  if (preloadQueries) {
    for (const [key, { id, request }] of Object.entries(preloadQueries)) {
      const preloadResponse = jsons.find(x => x.key === key);
      const response = preloadResponse.data;
      const name =
        request.length > 0
          ? id + request.map(x => (typeof x).slice(0, 1)).join("")
          : id;
      data[name] = response;
    }
  }

  Object.assign(
    data,
    Object.fromEntries(
      jsons
        .find(
          ({ values }) =>
            values instanceof Array &&
            values.length >= 4 &&
            values.includes("stadia.google.com") &&
            values.includes("https://stadia.google.com/"),
        )
        .values.map((value, i) => [
          ((i + 7577) / 7919)
            .toString(36)
            .replace(/[^A-Za-z]+/, "")
            .slice(0, 6)
            .padEnd(6, "s"),
          value,
        ]),
    ),
  );

  if (data.nQyAEs) {
    Object.assign(data, padOpaqueKeys(data.nQyAEs));
    delete data.nQyAEs;
  }

  return data;
};
