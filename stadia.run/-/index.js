/** @generated from index.html */

// We may import an copy of this module while using dev tools, so we use
// this to share any mutable state between the module instances.
const mut = (globalThis["index.js#mut"] =
  globalThis["index.js#mut"] || Object.create(null));

export const initialized = (mut.initialized =
  mut.initialized ||
  Promise.resolve().then(async () => {
    console.group("🔧 initializing");
    try {
      await initialize();
    } finally {
      console.groupEnd();
    }
  }));

export const initialize = async () => {
  if (typeof window === "undefined") {
    return;
  }

  let desktopPWA = false;

  try {
    desktopPWA =
      navigator.userAgentData &&
      navigator.userAgentData.mobile === false &&
      (window.navigator.standalone ||
        window.matchMedia("(display-mode: standalone)").matches);
  } catch (error) {
    console.warn(error);
  }

  if (desktopPWA) {
    document.querySelector("base").target = "_blank";
  }

  searchInput.addEventListener("input", event => onInput(event));
  searchForm.addEventListener("submit", event => onSubmit(event));
  window.addEventListener("popstate", checkUrl);

  checkUrl(true);

  await Promise.all([initDevToolsLoader(), unpackMicroCovers()]);

  // Just used for PWA offline fallback, because Chrome requires it.
  // if (navigator.serviceWorker) {
  //   navigator.serviceWorker.register('/--service-worker.js', {scope: '/'});
  // }
};

export const searchForm =
  globalThis.document && document.querySelector("st-search form");
export const searchInput = searchForm && searchForm.querySelector("input");
export const searchButton = searchForm && searchForm.querySelector("button");
export const gameTiles =
  globalThis.document && document.querySelector("st-games");

export const skus = (mut.skus = mut.skus || new Map());

export const digits =
  "-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

export const cleanName = name =>
  name
    .replace(/™/g, " ")
    .replace(/®/g, " ")
    .replace(/[\:\-]? Early Access$/g, " ")
    .replace(/[\:\-]? \w+ Edition$/g, " ")
    .replace(/\(\w+ Ver(\.|sion)\)$/g, " ")
    .replace(/™/g, " ")
    .replace(/\s{2,}/g, " ")
    .replace(/^\s+|\s+$/g, "");

export const slugify = (name, separator = "-") =>
  cleanName(name)
    .normalize("NFKD")
    .replace(/\p{Mark}/gu, "")
    .toLowerCase()
    .replace(/'/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^\-+|\-+$/g, "")
    .replace(/\-/g, separator);

export const loadedImage = async (/** @type string */ url) => {
  const image = new Image();
  await new Promise((resolve, reject) => {
    image.onload = resolve;
    image.onerror = reject;
    image.crossOrigin = "anonymous";
    image.src = url;
  });
  return image;
};

export const unpackMicroCovers = async () => {
  for (const game of document.querySelectorAll("st-game")) {
    const full = game.querySelector("st-cover-full img");
    const micro = game.querySelector("st-cover-micro");

    if (full.complete) {
      // full image already in cache
      micro.hidden = true;
    } else {
      micro.classList.add("rendered");
      micro.style.backgroundImage = `url(${microImageToURL(
        micro.getAttribute("data"),
      )})`;
      full.hidden = true;
      micro.hidden = false;
    }

    loadedImage(full.src).then(() => {
      full.hidden = false;
      micro.hidden = true;
    });
  }
};

export const u6toRGB = u6 => {
  const red =
    (u6 & 0b000010 ? 0b10101010 : 0) + (u6 & 0b000001 ? 0b01010101 : 0);
  const green =
    (u6 & 0b001000 ? 0b10101010 : 0) + (u6 & 0b000100 ? 0b01010101 : 0);
  const blue =
    (u6 & 0b100000 ? 0b10101010 : 0) + (u6 & 0b010000 ? 0b01010101 : 0);
  return [red, green, blue];
};

export const microImageToURL = microImage => {
  const canvas = document.createElement("canvas");
  canvas.width = 8;
  canvas.height = 8;
  const g2d = canvas.getContext("2d");
  const pixels = g2d.getImageData(0, 0, canvas.width, canvas.height);

  const palette = new Map();
  for (let i = 0; i < 64; i++) {
    palette.set(i, u6toRGB(i));
  }

  const digitValues = new Map(
    Object.entries(digits).map(([index, char]) => [char, Number(index)]),
  );

  for (let i = 0; i < microImage.length && i < 64; i++) {
    const color = palette.get(digitValues.get(microImage[i]));
    pixels.data[i * 4] = color[0];
    pixels.data[i * 4 + 1] = color[1];
    pixels.data[i * 4 + 2] = color[2];
    pixels.data[i * 4 + 3] = 0xff;
  }

  g2d.putImageData(pixels, 0, 0);
  return canvas.toDataURL();
};

let filterElementsPending = null;
const filterElements = async () => {
  if (filterElementsPending) {
    return filterElementsPending;
  }

  return (filterElementsPending = new Promise(resolve => resolve()).then(() => {
    filterElementsPending = false;

    const query = searchInput.value.toLowerCase();
    const slugQuery = slugify(query);

    const looseMatches = [];
    const exactMatches = [];

    for (const child of gameTiles.querySelectorAll("st-game")) {
      child.firstElementChild.hidden = true;

      let name = child.querySelector("st-name").textContent.toLowerCase();
      let slug = child.querySelector("st-slug").textContent.replace(/\/+/, "");

      if (query === slug) {
        exactMatches.push(child);
      } else if (query === name) {
        exactMatches.push(child);
      } else if (slugify(name).includes(slugQuery)) {
        looseMatches.push(child);
      } else if (slug.includes(slugQuery)) {
        looseMatches.push(child);
      }
    }

    const elements = exactMatches.length > 0 ? exactMatches : looseMatches;

    for (const el of elements) {
      el.firstElementChild.hidden = false;
    }

    document.documentElement.setAttribute("data-st-matches", elements.length);

    return elements;
  }));
};

const onInput = () => filterElements();

const onSubmit = event => {
  if (event && event.preventDefault) {
    event.preventDefault();
  }

  filterElements().then(elements => {
    const params = new URLSearchParams(location.search);

    if (elements.length === 1) {
      searchInput.value = elements[0].querySelector("st-slug").textContent;
      searchInput.select();
      elements[0].querySelector("a").click();
    }
  });
};

const checkUrl = (first = false) => {
  const slug =
    slugify(decodeURIComponent(document.location.pathname.slice(1))) || null;
  const query = new URLSearchParams(document.location.search).get("q") || null;

  if (query) {
    searchInput.value = query;
    onInput();
  } else if (slug) {
    searchInput.value = slug.replace(/-/g, " ");
    onSubmit({ first });
  }
};

let prevented = false;

const initDevToolsLoader = async () => {
  document.addEventListener("keydown", event => {
    if (event.key === "F12") {
      if (document.location.hash !== "#dev-tools") {
        const scrollTop = document.documentElement.scrollTop;
        document.location.hash = "#dev-tools";
        window.import("/-/dev.js");
        prevented = true;
        document.documentElement.scrollTop = scrollTop;
        event.preventDefault();
      } else {
        if (prevented) {
          prevented = false;
        } else {
          const scrollTop = document.documentElement.scrollTop;
          document.location.hash = "";
          history.replaceState(null, "", " ");
          document.documentElement.scrollTop = scrollTop;
          event.preventDefault();
        }
      }
    }
  });

  if (document.location.hash === "#dev-tools") {
    window.import("/-/dev.js");
    prevented = true;
  }

  document.addEventListener("hashchange", () => {
    if (document.location.hash === "#dev-tools") {
      window.import("/-/dev.js");
      prevented = true;
    }
  });

  document.querySelector("footer a").addEventListener("click", event => {
    window.import("/-/dev.js");
    const scrollTop = document.documentElement.scrollTop;
    document.location.hash = "#dev-tools";
    document.documentElement.scrollTop = scrollTop;
    event.preventDefault();
  });

  document
    .querySelector("#dev-tools header .close")
    .addEventListener("click", event => {
      const scrollTop = document.documentElement.scrollTop;
      document.location.hash = "";
      history.replaceState(null, "", " ");
      event.preventDefault();
      document.documentElement.scrollTop = scrollTop;
    });
};
