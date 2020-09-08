import "./jsons.js";

import {
  canFetchDevApi,
  canFetchStadiaHost,
  canFetchStadiaStore,
  fetchDevApi,
  fetchStadiaJsons,
} from "./net.js";
import { digits, loadedImage, microImageToURL, u6toRGB } from "./index.js";

const init = async () => {
  const root = document.getElementById("dev-tools");

  root.classList.remove("unloaded");
  document.querySelector("footer").classList.add("activated");

  root.querySelector(".loaded-games-status").textContent = 0;
  root.querySelector(".loaded-skus-status").textContent = 0;

  root.querySelector(".dev-server-status").textContent = (await canFetchDevApi)
    ? "✅ available"
    : "❌ unavailable";

  root.querySelector(
    ".stadia-proxy-status",
  ).textContent = (await canFetchStadiaStore)
    ? "✅ authenticated"
    : (await canFetchStadiaHost)
    ? "⚠️ unauthenticated"
    : "❌ unavailable";

  root.querySelector(".do-load-from-dev").disabled = !(await canFetchDevApi);
  root.querySelector(".do-save-to-dev").disabled = !(await canFetchDevApi);
  root.querySelector(
    ".do-load-from-store",
  ).disabled = !(await canFetchStadiaStore);
};

export const initialized = Promise.resolve()
  .then(() => {
    console.group("🔧 initializing dev tools");
    return init();
  })
  .finally(() => {
    console.groupEnd();
  });

const runHost = document.location.host;
const stHost = runHost.endsWith(":57481")
  ? runHost.replace(":57481", ":57480").replace(".run:", ".st:")
  : "stadia.st";

const doFetchCovers = async () => {
  await reloadSkus();
};

const doDownloadHtml = async () => {
  const docToDownload = document.documentElement.cloneNode(true);

  docToDownload.querySelector("title").textContent = "stadia.run";

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

  try {
    await fetch("//dev-api.stadia.st:57482/index.html", {
      method: "PUT",
      body: html,
    });
  } catch (error) {
    console.error("Failed to write index.html directly, downloading instead.");
    console.error(error);

    const href = URL.createObjectURL(
      new Blob([html], {
        type: "text/html",
      }),
    );
    const el = Object.assign(document.createElement("a"), {
      download: "index.html",
      href,
    });
    document.body.appendChild(el);
    el.click();
    document.body.removeChild(el);
  }
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

const checkStatus = (/** @type Response */ response) => {
  if (response.ok) {
    return response;
  } else {
    throw Object.assign(
      new Error(`${response.status} ${response.statusText}`),
      { response },
    );
  }
};

const reloadSkus = async () => {
  const skusData = await fetch(`//${stHost}/-/skus.json`)
    .then(checkStatus)
    .then(response => response.json());

  const skus = new Map();
  for (const sku of Object.values(skusData)) {
    skus.set(sku.sku, sku);
  }

  const proGameSkus = new Set();
  const addProGames = skuId => {
    const sku = skus.get(skuId);
    if (sku.type === "game") {
      proGameSkus.add(skuId);
    } else if (sku.skus) {
      sku.skus.forEach(addProGames);
    }
  };
  addProGames("59c8314ac82a456ba61d08988b15b550");

  const games = [...skus.values()]
    .filter(sku => sku.image)
    .map(game => ({
      name: game.name
        .replace(/™/g, " ")
        .replace(/®/g, " ")
        .replace(/[\:\-]? Early Access$/g, " ")
        .replace(/[\:\-]? \w+ Edition$/g, " ")
        .replace(/\(\w+ Ver(\.|sion)\)$/g, " ")
        .replace(/™/g, " ")
        .replace(/\s{2,}/g, " ")
        .replace(/^\s+|\s+$/g, ""),
      app: game.app,
      microImage: game.microImage,
      image: game.image,
      pro: proGameSkus.has(game.sku),
    }))
    .sort((gameA, gameB) => {
      const aName = gameA.name.toLowerCase();
      const bName = gameB.name.toLowerCase();

      if (gameA.pro && !gameB.pro) {
        return -1;
      } else if (!gameA.pro && gameB.pro) {
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

  for (const game of games) {
    let root = template.content.cloneNode(true).firstElementChild;
    let url = game.image;

    const fullImg = root.querySelector("img");
    fullImg.src = url;
    root.querySelector("st-cover-full").hidden = fullImg.complete;
    root.querySelector("st-cover-micro").hidden = !fullImg.complete;
    loadedImage(url)
      .then(() => {
        root.querySelector("st-cover-full").hidden = false;
        root.querySelector("st-cover-micro").hidden = true;
      })
      .catch(error => console.error(error));

    root.querySelector(
      "a",
    ).href = `https://stadia.google.com/player/${game.app}`;
    root.querySelector("st-name").textContent = game.name;

    if (!game.microImage) {
      const microImage = await microImageFromURL(url);
      game.microImage = microImage;
    }

    root.querySelector(
      "st-cover-micro",
    ).style.backgroundImage = `url(${microImageToURL(game.microImage)})`;

    root.querySelector("st-cover-micro").setAttribute("data", game.microImage);

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

  return data;
};

const padOpaqueKeys = object => {
  if (
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

canFetchStadiaStore.then(async () => {
  const paths = [
    "store/details/-/sku/59c8314ac82a456ba61d08988b15b550", // Stadia Pro
    "store/list/3", // All Games
    "store/list/45", // Pro Deals
    "store/details/-/sku/bd70626ec3834dedbc6dda5b956f7648", // bundle
    "store/details/-/sku/4950959380034dcda0aecf98f675e11f", // game
    "store/details/-/sku/5c1d84fe250a473e9d0313ed232508bc", // addon
  ];
  for (const path of paths) {
    console.log(path, await fetchStadiaPage(path));

    await new Promise(resolve => setTimeout(resolve, 4 * 1000));
  }
});
