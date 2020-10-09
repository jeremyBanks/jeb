import { getset, records, deriveDerivedDerivations } from "./data.js";
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

  const untitled = skuData[5];

  const publisherOrganizationId = skuData[15];
  const developerOrganizationIds = skuData[16];

  const coverUrl = skuData[2]?.[1]?.[0]?.[0]?.[1]?.split(/=/)[0];
  const coverMicroData = await microImageFromURL(coverUrl);
  const coverHash = await hashFromURL(coverUrl);

  const releaseDateA = 1000 * skuData[10]?.[0] || undefined;
  const releaseDateB = 1000 * skuData[26]?.[0] || undefined;

  let countries, languages, description;

  if (type === "game") {
    countries = [...skuData[25]].sort();
    languages = [...skuData[24]].sort();
  }

  description = skuData[9];

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
    coverHash,
    countries,
    languages,
    description,
    coverMicroData,
    releaseDateA,
    releaseDateB,
    untitled,
    publisherOrganizationId,
    developerOrganizationIds,
  };

  if (childSkuIds) {
    props.childSkuIds = childSkuIds;
  }

  return getset(props);
};

const keygen = record => {
  if (record.type === "list") {
    return "zzzl" + record._key.padStart(28, "-");
  }

  if (record.type === "organization") {
    return "zzzo" + record.organizationId.slice(0, 28);
  }

  if (record.type === "user") {
    return "zzzu" + record._key.padStart(28, "-");
  }

  let appId = record.appId.replace(/^([a-f0-9]{32})([a-z0-9]+)$/, "$1-$2");
  let skuId = record.skuId.replace(/^([a-f0-9]{32})([a-z0-9]+)$/, "$1-$2");
  let typeTag = "";

  if (record.type === "subscription") {
    appId = "0000";
    typeTag = "SB";
  } else if (record.type === "game") {
    typeTag = "GG";
  } else if (record.type === "addon-subscription") {
    typeTag = "GS";
  } else if (record.type === "addon") {
    typeTag = "GX";
  } else if (record.type === "bundle") {
    typeTag = "PA";
  } else if (record.type === "preorder") {
    typeTag = "PR";
  }

  let idLen = 6;
  let typeLen = 2;
  let nameLen = 32 - typeLen - idLen - idLen;

  let nameTag = (record.slug || slugify(record.name)).replace(/-/g, "");
  if (nameTag.length < nameLen) {
    nameTag += slugify(record.untitled).replace(/-/g, "");
  } else if (nameTag.length > nameLen) {
    // remove last instance of most-frequent letter
    while (nameTag.length > nameLen) {
      const frequencies = {
        0: 0.2,
        e: 0.1249,
        t: 0.0928,
        a: 0.0804,
        o: 0.0764,
        i: 0.0757,
        n: 0.0723,
        s: 0.0651,
        r: 0.0628,
        h: 0.0505,
        l: 0.0407,
        d: 0.0382,
        c: 0.0334,
        u: 0.0273,
        m: 0.0251,
        f: 0.024,
        p: 0.0214,
        g: 0.0187,
        w: 0.0168,
        y: 0.0166,
        b: 0.0148,
        v: 0.0105,
        k: 0.0054,
        x: 0.0023,
        j: 0.0016,
        q: 0.0012,
        z: 0.0009,
      };
      let mostFrequent = "";
      for (const character of nameTag) {
        const frequency = (frequencies[character] += 1);
        if (!mostFrequent || frequency > frequencies[mostFrequent]) {
          mostFrequent = character;
        }
      }

      const index = nameTag.lastIndexOf(mostFrequent);
      nameTag = nameTag.slice(0, index) + nameTag.slice(index + 1);
    }
  }

  const result = [
    appId.padEnd(idLen, 0).slice(0, idLen),
    typeTag.padEnd(typeLen, "?").slice(0, typeLen),
    skuId.padEnd(idLen, 0).slice(0, idLen),
    (nameTag + skuId.slice(idLen)).slice(0, nameLen),
  ].join("");

  if (result.length !== 32) {
    throw new TypeError("wrong id length");
  }
  return result;
};

const spider = async (/** @type {Record} */ record) => {
  const also = {};

  if (record.type === "list") {
    const page = await fetchStadiaPage(`store/list/${record.listId}`);

    if (page.list) {
      for (const sku of page.list) {
        await loadSkuData(sku[9]);
      }
      also.childSkuIds = page.list.map(sku => sku[9][0]);
      also.name = page.heading;
    } else {
      also.childSkuIds = [];
      also.name = page.title.replace(/ - Store - Stadia$/, "");
    }
  } else if (record.type === "user") {
    const page = await fetchStadiaPage(
      `profile/${record.userId}/gameactivities/all`,
    );
    also.playedAppIds = page.playedAppIds;
    also.name = page.user?.[0][0];
    also.number = page.user?.[0][1];
    also.coverUrl = page.user?.[1][1].replace("/mdpi/", "/xxhdpi/");
    also.coverMicroData = await microImageFromURL(also.coverUrl);
    also.coverHash = await hashFromURL(also.coverUrl);
  } else {
    const appId = record.appId || "-";
    const page = await fetchStadiaPage(
      `store/details/${appId}/sku/${record.skuId}`,
    );

    const organizations = [page.sku[22][0], ...page.sku[22][1]];
    for (const organization of organizations) {
      getset({
        type: "organization",
        organizationId: organization[0],
        name: organization[2][0],
      });
    }

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
    ...also,
    lastSpidered: Date.now(),
  });
};

const inclusive = (a, b) => {
  const r = new Array();
  for (let i = a; i >= a && i <= b; i++) {
    r.push(i);
  }
  return r;
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
    type: "list",
    listId: 3,
  });

  getset({
    type: "subscription",
    skuId: "59c8314ac82a456ba61d08988b15b550",
  });

  try {
    await withTimeout(16, canFetchDevApi);

    const skus = await (await fetchDevApi("skus.json")).json();
    const meta = await (await fetchDevApi("skus-meta.json")).json();
    for (const key of Object.keys(skus)) {
      Object.assign(skus[key], meta[key]);
    }
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
    const allRecords = Object.values(records).sort((a, b) => {
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
    });

    const ageLimit = 24 * 60 * 60 * 1000;

    const staleRecords = allRecords.filter(
      record => record.age(now) > ageLimit,
    );

    const record = allRecords[0];

    const icon =
      allRecords.length > 8 ? (staleRecords.length > 0 ? "⚠️" : "✅") : "❌";

    document.querySelector("#dev-tools .record-count").textContent = `${icon} ${
      staleRecords.length
    } stale, ${allRecords.length - staleRecords.length} fresh, ${
      allRecords.length
    } total`;

    if (staleRecords.length % 4 === 0) {
      await updateDocument();
      await downloadDocument();
    }

    if (staleRecords.length === 0) {
      console.info(
        `Everything has been spidered recently (at most ${
          record.age(now) / 1000 / 60 / 60
        } hours ago).`,
      );
      await sleep(Math.random() * 600.0);
      continue;
    }

    let skus = {};
    let meta = {};
    if (await canFetchDevApi) {
      for (const key of Object.keys(records).sort()) {
        const item = records[key];
        const newKey = keygen(item);

        if (skus.hasOwnProperty(newKey)) {
          throw new Error(`duplicate key ${newKey}`);
        }

        skus[newKey] = {
          appId: item.appId,
          childSkuIds: item.childSkuIds,
          countries: item.countries,
          coverHash: item.coverHash,
          coverMicroData: item.coverMicroData,
          coverUrl: item.coverUrl,
          description: item.description,
          developerOrganizationIds: item.developerOrganizationIds,
          isPro: item.isPro,
          languages: item.languages,
          listId: item.listId,
          name: item.name,
          number: item.number,
          organizationId: item.organizationId,
          playedAppIds: item.playedAppIds,
          popular: item.popular,
          publisherOrganizationId: item.publisherOrganizationId,
          releaseDateA: item.releaseDateA,
          releaseDateB: item.releaseDateB,
          skuId: item.skuId,
          slug: item.slug,
          type: item.type,
          untitled: item.untitled,
          userId: item.userId,
          wasPro: item.wasPro,
        };

        meta[newKey] = {
          firstSeen: item.firstSeen,
          lastModified: item.lastModified,
          lastSeen: item.lastSeen,
          lastSpidered: item.lastSpidered,
        };
      }
    }

    let a = staleRecords.slice(0, Math.max(8, staleRecords.length / 100));
    const chosenRecord = a[Math.floor(Math.random() * a.length)];
    await spider(chosenRecord);
    console.info("🕷️ spidered", chosenRecord);

    if (await canFetchDevApi) {
      fetchDevApi("skus.json", {
        method: "PUT",
        body: JSON.stringify(skus, null, 2),
      });
      fetchDevApi("skus-meta.json", {
        method: "PUT",
        body: JSON.stringify(meta, null, 2),
      });
    }

    console.debug(`${Object.keys(records).length} records.`, records);
    const s = Math.random() * 24.0;
    console.debug("sleeping for", s, "seconds");
    await sleep(s);
  }
};

const fetchStadiaPage = async url => {
  const opaque = await fetchStadiaOpaque(url);
  const data = Object.create(opaque);

  data.heading = opaque.HZ5mJ;
  data.title = opaque.title;
  data.self = opaque.D0Amudob?.[5];
  data.user = opaque.D0Amudoboos?.[5];
  data.list = opaque.WwD3rbnob?.[2];
  data.gameStats = opaque.e7h9qdoss?.[0]?.[8];
  data.playedAppIds = opaque.Q6jt8cooos?.[0];
  if (!data.playedAppIds?.length) {
    data.playedAppsIds = undefined;
  }
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

const fetchStadiaOpaque = async url => {
  const response = await fetchStadia(url);
  console.debug("Got Stadia response", response);
  checkStatus(response);

  const body = await response.text();
  const doc = new DOMParser().parseFromString(body, "text/html");

  const scripts = [...doc.querySelectorAll("script")];

  const jsons = scripts.flatMap(script => jsonObjects(script.textContent));

  const data = Object.create(jsons);

  for (const el of doc.querySelectorAll("[role][class]")) {
    data[el.className] = el.textContent;
  }

  data.title = doc.querySelector("title")?.textContent;

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
      const response = preloadResponse?.data;
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
        ?.values.map((value, i) => [
          ((i + 7577) / 7919)
            .toString(36)
            .replace(/[^A-Za-z]+/, "")
            .slice(0, 6)
            .padEnd(6, "s"),
          value,
        ]) || [],
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

const hashFromURL = async (/** @type string */ url) => {
  const response = await fetch(url);
  const body = await response.arrayBuffer();
  const hash = await crypto.subtle.digest("SHA-512", body);
  const byteLength = 8;
  const hexHash = Array.from(new Uint8Array(hash))
    .slice(0, byteLength)
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
  return `f12${byteLength.toString(16).padStart(2, "0")}${hexHash}`;
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

  for (const el of docToDownload.querySelectorAll(
    ".dev-server-status,.stadia-proxy-status,.record-count",
  )) {
    el.textContent = "❓";
  }

  const html =
    "<!doctype html>" +
    docToDownload.innerHTML
      .replace(/\s*<\/body>\s*$/, "\n")
      .replace(/^<head>/, "")
      .replace(/<\/head><body>/, "")
      .replace(
        /(\s)(disabled|autofocus|pre-order|pro|previously-pro|popular)(="")([>\s])/g,
        "$1$2$4",
      );

  await fetch("//dev-api.stadia.st:57482/index.html", {
    method: "PUT",
    body: html,
  });
};

const updateDocument = async () => {
  deriveDerivedDerivations();

  const games = [...Object.values(records)]
    // only include games that are to be released within the next week
    .filter(sku => sku.type === "game")
    .map(game => ({
      ...game,
      slug: game.slug,
      name: cleanName(game.name),
      preOrder:
        Math.max(game.releaseDateA, game.releaseDateB) >
        Date.now() + 1000 * 60 * 60 * 24 * 2,
    }))
    .sort((gameA, gameB) => {
      const aFirst = -1;
      const bFirst = +1;

      const aName = gameA.name.toLowerCase();
      const bName = gameB.name.toLowerCase();

      const aReleased = Math.max(gameA.releaseDateA, gameA.releaseDateB);
      const bReleased = Math.max(gameB.releaseDateA, gameB.releaseDateB);

      if (gameA.popular && !gameB.popular) {
        return aFirst;
      } else if (!gameA.popular && gameB.popular) {
        return bFirst;
      } else if (gameA.preOrder && !gameB.preOrder) {
        return bFirst;
      } else if (!gameA.preOrder && gameB.preOrder) {
        return aFirst;
      } else if (gameA.preOrder && gameB.preOrder) {
        if (aReleased > bReleased) {
          return bFirst;
        } else if (aReleased < bReleased) {
          return AFirst;
        }
      } else if (gameA.isPro && !gameB.isPro) {
        return aFirst;
      } else if (!gameA.isPro && gameB.isPro) {
        return bFirst;
      } else if (aReleased > bReleased) {
        return aFirst;
      } else if (aReleased < bReleased) {
        return bFirst;
      } else if (gameA.wasPro && !gameB.wasPro) {
        return aFirst;
      } else if (!gameA.wasPro && gameB.wasPro) {
        return bFirst;
      } else if (aName < bName) {
        return aFirst;
      } else if (aName > bName) {
        return bFirst;
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

    if (manifest.shortcuts.length < 16) {
      manifest.shortcuts.push({
        name: game.name,
        url: `/${game.slug}`,
        icons: [
          {
            src: url + "=s192-p-rp",
            sizes: "192x192",
          },
        ],
      });
    }

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

    const name = root.querySelector("st-name");
    name.textContent = game.name;

    const slug = root.querySelector("st-slug");
    slug.textContent = "/" + game.slug;

    root.querySelector(
      "st-cover-micro",
    ).style.backgroundImage = `url(${microImageToURL(game.coverMicroData)})`;

    root
      .querySelector("st-cover-micro")
      .setAttribute("data", game.coverMicroData);

    if (game.isPro) {
      const badge = Object.assign(document.createElement("st-badge"), {
        textContent: "PRO",
        title: `${game.name} is currently included with Stadia Pro.`,
      });
      badge.setAttribute("pro", "");
      link.appendChild(badge);
    } else if (game.wasPro) {
      const badge = Object.assign(document.createElement("st-badge"), {
        innerHTML: "previously<br />PRO",
        title: `${game.name} was previously included with Stadia Pro.`,
      });
      badge.setAttribute("previously-pro", "");
      link.appendChild(badge);
    }

    if (game.popular) {
      const badge = Object.assign(document.createElement("st-badge"), {
        innerHTML: "🔥",
        title: `${game.name} is popular!`,
      });
      badge.setAttribute("popular", "");
      link.appendChild(badge);
    }

    if (game.preOrder) {
      const badge = Object.assign(document.createElement("st-badge"), {
        textContent: "pre-order",
        title: `${game.name} is available for pre-order, but not yet released.`,
      });
      badge.setAttribute("pre-order", "");
      link.appendChild(badge);
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
