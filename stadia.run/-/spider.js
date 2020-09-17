import {
  Addon,
  Bundle,
  Game,
  List,
  Subscription,
  getset,
  records,
} from "./data.js";
import {
  digits,
  loadedImage,
  microImageToURL,
  cleanName,
  slugify,
} from "./index.js";
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
    6: "addon-subscription",
    10: "preorder",
  }[skuData[6]];
  const skuId = skuData[0];
  const appId = skuData[4];
  const name = skuData[1];

  const coverUrl = skuData[2]?.[1]?.[0]?.[0]?.[1]?.split(/=/)[0];
  const coverMicroData = await microImageFromURL(coverUrl);

  const releasedOnStadia = 1000 * skuData[10]?.[0];
  const releasedAnywhere = 1000 * skuData[26]?.[0];

  const childSkuIds = skuData[14]?.[0]?.map(x => x[0]);
  const childData = skuData[14]?.[0]?.map(x => x[2]);
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
    releasedOnStadia,
    releasedAnywhere,
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
    const appId = record.appId || "-";
    const page = await fetchStadiaPage(
      `store/details/${appId}/sku/${record.skuId}`,
    );
    await loadSkuData(page.sku[16]);
    if (page.gameAddons) {
      for (const sku of page.gameAddons) {
        await loadSkuData(sku[9]);
      }
    }
    if (page.gameBundles) {
      for (const sku of page.gameBundles) {
        await loadSkuData(sku[9]);
      }
    }
    if (page.gameSubscriptions) {
      for (const sku of page.gameSubscriptions) {
        await loadSkuData(sku[9]);
      }
    }
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
    const now = Date.now();
    const record = Object.values(records).sort((a, b) => {
      if (a.age(now) < b.age(now)) {
        return +1;
      } else if (b.age(now) < a.age(now)) {
        return -1;
      } else if (a.lastModified < b.lastModified) {
        return -1;
      } else if (b.lastModified < a.lastModified) {
        return +1;
      } else {
        return 0;
      }
    })[0];

    if (record.age(now) < 24 * 60 * 60 * 1000) {
      console.info(
        `Everything has been spidered recently (at most ${
          record.age(now) / 1000 / 60 / 60
        } hours ago).`,
      );
      await sleep(128.0);
      continue;
    }

    await updateDocument();

    if (await canFetchDevApi) {
      const sorted = {};
      for (const key of Object.keys(records).sort()) {
        sorted[key] = records[key];
      }
      fetchDevApi("skus.json", {
        method: "PUT",
        body: JSON.stringify(sorted, null, 2),
      });
      downloadDocument();
    }

    await spider(record);
    console.info("🕷️ spidered", record);
    console.debug(`${Object.keys(records).length} records.`, records);
    await sleep(6.0);
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
  data.gameAddons = opaque.ZAm7Wesooooooob?.[0];
  data.gameBundles = opaque.SYcsTdsb?.[1];
  data.gameSubscriptions = opaque.SYcsTdsb?.[2];

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

const downloadDocument = async () => {
  const docToDownload = document.documentElement.cloneNode(true);

  docToDownload.querySelector("title").textContent = "stadia.run";

  docToDownload.querySelector("base").removeAttribute("target");

  for (const el of docToDownload.querySelectorAll("[hidden]")) {
    el.removeAttribute("hidden");
  }

  for (const input of docToDownload.querySelectorAll("input[value]")) {
    el.removeAttribute("value");
  }

  for (const el of docToDownload.querySelectorAll("[style]")) {
    el.removeAttribute("style");
  }

  for (const el of docToDownload.querySelectorAll('[class=""],main [class]')) {
    el.removeAttribute("class");
  }

  const html =
    "<!doctype html>" +
    docToDownload.innerHTML
      .replace(/\s*<\/body>\s*$/, "\n")
      .replace(/^<head>/, "")
      .replace(/<\/head><body>/, "")
      .replace(/(\s)(disabled|autofocus)(="")([>\s])<\/body>/g, "$1$2$4");

  await fetch("//dev-api.stadia.st:57482/index.html", {
    method: "PUT",
    body: html,
  });
};

const updateDocument = async () => {
  const proGameSkus = new Set();
  const addProGames = skuId => {
    const sku = records[skuId];
    if (sku.type === "game") {
      proGameSkus.add(skuId);
    } else if (sku.childSkuIds) {
      sku.childSkuIds.forEach(addProGames);
    }
  };
  addProGames("59c8314ac82a456ba61d08988b15b550");

  const games = [...Object.values(records)]
    // only include games that are to be released within the next week
    .filter(
      sku =>
        sku.type === "game" &&
        sku.releasedOnStadia < Date.now() + 1000 * 60 * 60 * 24 * 7,
    )
    .map(game => ({
      ...game,
      name: cleanName(game.name),
      pro: proGameSkus.has(game.skuId),
    }))
    .sort((gameA, gameB) => {
      const aName = gameA.name.toLowerCase();
      const bName = gameB.name.toLowerCase();

      const aReleased = Math.max(
        gameA.releasedOnStadia,
        gameA.releasedAnywhere,
      );
      const bReleased = Math.max(
        gameB.releasedOnStadia,
        gameB.releasedAnywhere,
      );

      if (gameA.pro && !gameB.pro) {
        return -1;
      } else if (!gameA.pro && gameB.pro) {
        return +1;
      } else if (aReleased > bReleased) {
        return -1;
      } else if (aReleased < bReleased) {
        return +1;
      } else if (aName < bName) {
        return -1;
      } else if (aName > bName) {
        return +1;
      } else {
        return 0;
      }
    });

  const template = document.querySelector("st-games template");

  const fragment = document.createDocumentFragment();

  const request = await fetch("//dev-api.stadia.st:57482/manifest.json", {
    method: "GET",
  });
  const manifest = await request.json();

  manifest.shortcuts = [];

  for (const game of games) {
    let root = template.content.cloneNode(true).firstElementChild;
    let url = game.coverUrl;

    manifest.shortcuts.push({
      name: game.name,
      url: `/${slugify(game.name)}`,
      icons: [
        {
          src: url + "=s192-p-rp",
          sizes: "192x192",
        },
      ],
    });

    const fullImg = root.querySelector("img");
    fullImg.src = url + "=w640-h360-rw";
    root.querySelector("st-cover-full").hidden = fullImg.complete;
    root.querySelector("st-cover-micro").hidden = !fullImg.complete;
    loadedImage(url)
      .then(() => {
        root.querySelector("st-cover-full").hidden = false;
        root.querySelector("st-cover-micro").hidden = true;
      })
      .catch(error => console.error(error));

    const link = root.querySelector("a");
    link.href = `https://stadia.google.com/player/${game.appId}`;
    root.querySelector("st-name").textContent = game.name;

    root.querySelector(
      "st-cover-micro",
    ).style.backgroundImage = `url(${microImageToURL(game.coverMicroData)})`;

    root
      .querySelector("st-cover-micro")
      .setAttribute("data", game.coverMicroData);

    if (game.pro) {
      root.querySelector("a").appendChild(
        Object.assign(document.createElement("st-pro"), {
          textContent: "PRO",
        }),
      );
    }

    fragment.appendChild(document.createTextNode("\n    "));
    fragment.appendChild(root);
  }

  template.remove();
  const gamesEl = document.querySelector("st-games");
  gamesEl.textContent = "";
  gamesEl.appendChild(template);
  gamesEl.appendChild(fragment);

  gamesEl.appendChild(document.createTextNode("\n  "));

  await fetch("//dev-api.stadia.st:57482/manifest.json", {
    method: "PUT",
    body: JSON.stringify(manifest, null, 2),
  });
};
