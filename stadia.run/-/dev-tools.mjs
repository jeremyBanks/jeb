import "./index.html/main.mjs";

const init = async() => {
  const root = document.getElementById('dev-tools');

  root.querySelector('#fetch-covers')
    .addEventListener('click', doFetchCovers);
  root.querySelector('#download-html')
    .addEventListener('click', doDownloadHtml);

  root.classList.remove('unloaded');
};

export const initialized = Promise.resolve().then(() => {
  console.group('🔧 initializing dev tools');
  return init();
}).finally(() => {
  console.groupEnd();
});

const runHost = document.location.host;
const stHost = runHost.endsWith(':57481') ? (
  runHost.replace(':57481', ':57480').replace('.run:', '.st:')
) : 'stadia.st';

const doFetchCovers = async() => {
  await reloadSkus();
};

const doDownloadHtml = async() => {
  const docToDownload = document.documentElement.cloneNode(true);

  docToDownload.querySelector('title').textContent = 'stadia.run';

  for (const el of docToDownload.querySelectorAll('[hidden]')) {
    el.removeAttribute('hidden');
  }

  for (const input of docToDownload.querySelectorAll('input[value]')) {
    el.removeAttribute('value');
  }

  for (const el of docToDownload.querySelectorAll('[style]')) {
    el.removeAttribute('style');
  }

  for (const el of docToDownload.querySelectorAll('[class=""],main [class]')) {
    el.removeAttribute('class');
  }

  const html = '<!doctype html><html>' + docToDownload.innerHTML.replace(/\s*<\/body>\s*$/, '\n');

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
};

const digits = '-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz';

const u6toRGB = (u6) => {
  const red =
    ((u6 & 0b000010) ? 0b10101010 : 0)
  + ((u6 & 0b000001) ? 0b01010101 : 0);
  const green =
    ((u6 & 0b001000) ? 0b10101010 : 0)
  + ((u6 & 0b000100) ? 0b01010101 : 0);
  const blue =
    ((u6 & 0b100000) ? 0b10101010 : 0)
  + ((u6 & 0b010000) ? 0b01010101 : 0);
  return [red, green, blue];
};

const microImageToURL = (microImage) => {
  const canvas = document.createElement('canvas');
  canvas.width = 8;
  canvas.height = 8;
  const g2d = canvas.getContext('2d');
  const pixels = g2d.getImageData(0, 0, canvas.width, canvas.height);

  const palette = new Map();
  for (let i = 0; i < 64; i++) {
    palette.set(i, u6toRGB(i));
  }

  const digitValues = new Map(
    Object.entries(digits).map(([index, char]) => [char, Number(index)]));

  for (let i = 0; i < microImage.length && i < 64; i++) {
    const color = palette.get(digitValues.get(microImage[i]));
    pixels.data[i * 4] = color[0];
    pixels.data[i * 4 + 1] = color[1];
    pixels.data[i * 4 + 2] = color[2];
    pixels.data[i * 4 + 3] = 0xFF;
  }

  g2d.putImageData(pixels, 0, 0);
  return canvas.toDataURL();
};

const loadedImage = async (/** @type string */ url) => {
  const image = new Image();
  await new Promise((resolve, reject) => {
    image.onload = resolve;
    image.onerror = reject;
    image.crossOrigin = 'anonymous';
    image.src = url;
  });
  return image;
}

/**
 * Returns an base-64 encoded 8x8 thumbnail the image at a given URL.
 * @returns {Promise<String>}
 */
const microImageFromURL = async (/** @type string */ url) => {
  const image = await loadedImage(url);
  const canvas = document.createElement('canvas');
  canvas.width = 8;
  canvas.height = 8;
  const g2d = canvas.getContext('2d');
  g2d.drawImage(image, 0, 0, canvas.width, canvas.height);
  const pixels = g2d.getImageData(0, 0, canvas.width, canvas.height);

  const microImage = new Array();
  for (let i = 0; i < 64; i++) {
    const rgb = pixels.data.slice(i * 4, i * 4 + 3);
    const u6 = rgbToU6(rgb);
    microImage.push(digits[u6]);
  }

  return microImage.join('');
}

/**
 * Rounds a 24-bit RGB value to the nearest 6-bit RGB value.
 * @returns {number}
 */
const rgbToU6 = (/** @type [number, number, number] */ rgb) => {
  const red = Math.round(0b11 * rgb[0] / 0xFF);
  const green = Math.round(0b11 * rgb[1] / 0xFF);
  const blue = Math.round(0b11 * rgb[2] / 0xFF);
  return (red << 0) + (green << 2) + (blue << 4);
};

const checkStatus = (/** @type Response */ response) => {
  if (response.ok) {
    return response;
  } else {
    throw Object.assign(
      new Error(`${response.status} ${response.statusText}`),
      { response });
  }
};

const reloadSkus = async() => {
  const skusData = await
    fetch(`https://${stHost}/-/skus.json`).then(checkStatus)
      .then(response => response.json());

  const skus = new Map();
  for (const sku of Object.values(skusData)) {
    skus.set(sku.sku, sku);
  }

  const proGameSkus = new Set();
  const addProGames = (skuId) => {
    const sku = skus.get(skuId);
    if (sku.type === 'game') {
      proGameSkus.add(skuId);
    } else if (sku.skus) {
      sku.skus.forEach(addProGames);
    }
  }
  addProGames('59c8314ac82a456ba61d08988b15b550');

  const games = [...skus.values()].filter(sku => sku.image).map(game => ({
    name: game.name
      .replace(/™/g, ' ')
      .replace(/®/g, ' ')
      .replace(/[\:\-]? Early Access$/g, ' ')
      .replace(/[\:\-]? \w+ Edition$/g, ' ')
      .replace(/\(\w+ Ver(\.|sion)\)$/g, ' ')
      .replace(/™/g, ' ')
      .replace(/\s{2,}/g, ' ')
      .replace(/^\s+|\s+$/g, '')
    ,
    app: game.app,
    microImage: game.microImage,
    image: game.image,
    pro: proGameSkus.has(game.sku)
  })).sort((gameA, gameB) => {

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

  const template = document.querySelector('st-games template');

  const fragment = document.createDocumentFragment();

  for (const game of games) {
    let root = template.content.cloneNode(true).firstElementChild;
    let url = game.image;

    const fullImg = root.querySelector('img');
    fullImg.src = url;
    root.querySelector('st-cover-full').hidden = fullImg.complete;
    root.querySelector('st-cover-micro').hidden = !fullImg.complete;
    loadedImage(url).then(() => {
      root.querySelector('st-cover-full').hidden = false;
      root.querySelector('st-cover-micro').hidden = true;
    }).catch(error => console.error(error));

    root.querySelector('a').href = `https://stadia.google.com/player/${game.app}`;
    root.querySelector('st-name').textContent = game.name;

    if (!game.microImage) {
      const microImage = await microImageFromURL(url);
      game.microImage = microImage;
    }

    root.querySelector('st-cover-micro').style.backgroundImage = `url(${
      microImageToURL(game.microImage)})`;

    root.querySelector('st-cover-micro').setAttribute('data', game.microImage);

    if (game.pro) {
      root.querySelector('a').appendChild(Object.assign(
        document.createElement('st-pro'), {
          textContent: 'PRO'
        }
      ))
    }

    fragment.appendChild(document.createTextNode("\n    "));
    fragment.appendChild(root);
  }

  template.remove();
  const gamesEl = document.querySelector('st-games');
  gamesEl.textContent = '';
  gamesEl.appendChild(template);
  gamesEl.appendChild(fragment);

  gamesEl.appendChild(document.createTextNode("\n  "));
};
