import {
  Addon,
  Bundle,
  Game,
  List,
  Subscription,
  getset,
  records,
} from "./data.js";
import { loadedImage, digits } from "./index.js";
import { jsonObjects } from "./jsons.js";
import {
  fetchStadia,
  checkStatus,
  canFetchStadiaStore,
  canFetchDevApi,
  fetchDevApi,
} from "./net.js";
/** @typedef {import("./data.js").Sku} Sku */
/** @typedef {import("./data.js").Record} Record */

import { sleep, withTimeout } from "./async.js";

const loadSkuData = async (/** @type {Array<unknown>} */ skuData) => {
  const type = {
    1: "game",
    2: "addon",
    3: "bundle",
    5: "subscription",
  }[skuData[6]];
  const skuId = skuData[0];
  const appId = skuData[4];
  const name = skuData[1];

  const coverUrl = skuData?.[2]?.[1]?.[0]?.[0]?.[1]?.split(/=/)[0];
  const coverMicroData = await microImageFromURL(coverUrl);

  const childSkuIds = skuData?.[14]?.[0]?.map(x => x[0]);
  const childData = skuData?.[14]?.[0]?.map(x => x[2]);
  if (childData?.filter(Boolean).length) {
    // if this is a shallow view it these will be null
    await Promise.all(childData.map(loadSkuData));
  }

  const props = {
    type,
    skuId,
    appId,
    name,
    coverUrl,
    coverMicroData,
  };

  if (childSkuIds) {
    props.childSkuIds = childSkuIds;
  }

  return getset(props);
};

const spider = async (/** @type {Record} */ record) => {
  if (record.type === "list") {
    const page = await fetchStadiaPage(`store/list/${record.listId}`);
    for (const sku of page.list) {
      await loadSkuData(sku[9]);
    }
  } else {
    const page = await fetchStadiaPage(`store/details/-/sku/${record.skuId}`);
    await loadSkuData(page.sku[16]);
  }

  getset({
    ...record,
    lastSpidered: Date.now(),
  });
};

/** @returns {Promise<unknown>} */
export const spiderThread = async () => {
  try {
    await withTimeout(16, canFetchStadiaStore);
  } catch (error) {
    console.debug(
      "Failed to connect to Stadia store, abandoning spider.",
      error,
    );
    return;
  }

  getset({
    name: "All Games",
    type: "list",
    listId: 3,
  });

  getset({
    name: "Stadia Pro",
    type: "subscription",
    skuId: "59c8314ac82a456ba61d08988b15b550",
  });

  try {
    await withTimeout(16, canFetchDevApi);

    const skus = await (await fetchDevApi("skus.json")).json();
    Object.values(skus).forEach(getset);
    console.info(`${Object.keys(records).length} records loaded.`, records);
  } catch (error) {
    console.debug(
      "Failed to connect to local dev server, skipping load.",
      error,
    );
  }

  for (;;) {
    const record = Object.values(records).sort((a, b) => {
      if (a.lastSpidered < b.lastSpidered) {
        return -1;
      } else if (b.lastSpidered < a.lastSpidered) {
        return +1;
      } else if (a.lastModified < b.lastModified) {
        return -1;
      } else if (b.lastModified < a.lastModified) {
        return +1;
      } else {
        return 0;
      }
    })[0];

    if (record.lastSpidered > Date.now() - 12 * 60 * 60 * 1000) {
      console.info("Everything has been spidered recently.");
      await sleep(128.0);
      continue;
    }

    if (await canFetchDevApi) {
      const sorted = {};
      for (const key of Object.keys(records).sort()) {
        sorted[key] = records[key];
      }
      fetchDevApi("skus.json", {
        method: "PUT",
        body: JSON.stringify(sorted, null, 2),
      });
    }

    await spider(record);
    console.info("🕷️ spidered", record);
    console.debug(`${Object.keys(records).length} records.`, records);
    await sleep(16.0);
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

/**
 * Returns an base-64 encoded 8x8 thumbnail the image at a given URL.
 * @returns {Promise<String>}
 */
const microImageFromURL = async (/** @type string */ url) => {
  const image = await loadedImage(url);
  const canvas = document.createElement("canvas");
  canvas.width = 8;
  canvas.height = 8;
  const g2d = canvas.getContext("2d");
  g2d.drawImage(image, 0, 0, canvas.width, canvas.height);
  const pixels = g2d.getImageData(0, 0, canvas.width, canvas.height);

  const microImage = new Array();
  for (let i = 0; i < 64; i++) {
    const rgb = pixels.data.slice(i * 4, i * 4 + 3);
    const u6 = rgbToU6(rgb);
    microImage.push(digits[u6]);
  }

  return microImage.join("");
};

/**
 * Rounds a 24-bit RGB value to the nearest 6-bit RGB value.
 * @returns {number}
 */
const rgbToU6 = (/** @type [number, number, number] */ rgb) => {
  const red = Math.round((0b11 * rgb[0]) / 0xff);
  const green = Math.round((0b11 * rgb[1]) / 0xff);
  const blue = Math.round((0b11 * rgb[2]) / 0xff);
  return (red << 0) + (green << 2) + (blue << 4);
};
