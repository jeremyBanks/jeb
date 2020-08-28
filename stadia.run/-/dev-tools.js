const init = async() => {
  const root = document.getElementById('dev-tools');
  root.classList.remove('unloaded');

  while (true) {
    stadiaMirror = open(
      'https://stadia.google.com/robots.txt',
      'stadiaMirror',
      `left=${screen.width},top=${screen.height},width=1,height=1`);
    if (stadiaMirror === null) {
      console.warn("Please allow pop-ups.");
      await new Promise(resolve => setTimeout(resolve, 500));
    } else {
      break;
    }
  }

  const spiderCode = `
    window.addEventListener("message", ({origin, data}) => {
      if (origin === ${JSON.stringify(window.origin)} && data.eval) {
        eval(event.data.eval);
      }
    })
  `;

  stadiaMirror.focus();

  while (true) {
    if (!await stadiaAlive()) {
    console.debug("Please press F12 to open the developer tools in the Stadia window, then paste the following to let us access it.\n", spiderCode);
      await new Promise(resolve => setTimeout(resolve, 500));
    } else {
      break;
    }
  }

  console.debug("Thanks! We're good to go!");

  window.focus();
};

let stadiaMirror = null;
const pendingRequests = new Map();
let nextRequestId = 0;

const stadiaAlive = () => {
  const id = nextRequestId++;

  stadiaMirror.postMessage({
    eval: `
      event.source.postMessage({
        id: ${JSON.stringify(id)}
      })
    `
  }, "https://stadia.google.com")

  return Promise.race([
    new Promise(resolve => {
      pendingRequests.set(id, resolve);
    }).then(_ => true),
    new Promise(resolve => {
      setTimeout(() => resolve(false), 100)
    })]);
};

const stadiaFetch = (...args) => {
  const id = nextRequestId++;

  stadiaMirror.postMessage({
    eval: `
      fetch(...JSON.parse(${JSON.stringify(args)}))
        .then(response => response.text())
        .then(body => {
          event.source.postMessage({
            id: ${JSON.stringify(id)},
            return: body
          })
        }).catch(error => {
          event.source.postMessage({
            id: ${JSON.stringify(id)},
            error
          })
        })
    `
  }, "https://stadia.google.com")

  return new Promise(resolve => {
    pendingRequests.set(id, resolve);
  });
};

window.addEventListener('message', ({origin, data}) => {
  if (origin === "https://stadia.google.com" && data?.id) {
    const id = data.id;
    const resolver = pendingRequests.get(id);
    if (data.error) {
      resolver(Promise.reject(data.error));
    } else {
      resolver(data.return);
    }
    pendingRequests.delete(id);
  }
})

export const initialized = Promise.resolve().then(() => {
  console.group('🔧 initializing dev tools');
  return init();
}).finally(() => {
  console.groupEnd();
});

setTimeout(() => reloadSkus());

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

const hydrateMicroImages = (root) => {
  for (const element of root.querySelectorAll("[data-micro-image]")) {
    element.style.backgroundImage = `url(${microImageToURL(element.dataset.microImage)})`;
  }
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
  const skus = await
    fetch('https://stadia.st/-/games.json').then(checkStatus)
      .catch(() => fetch('https://stadia.st/-/skus.json').then(checkStatus))
      .then(response => response.json());

  const games = Object.values(skus).filter(sku => sku.image).map(game => ({
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
  })).sort((gameA, gameB) => {
    const a = gameA.name.toLowerCase();
    const b = gameB.name.toLowerCase();
    return (a < b) ? -1 : (a > b) ? +1 : 0;
  });

  document.getElementById('games').data = games;

  const template = document.querySelector('template#stGameTemplate');

  const container = document.createDocumentFragment();

  for (const game of games) {
    let root = template.content.cloneNode(true).firstElementChild;
    let url = game.image;

    const fullImg = root.querySelector('img');
    fullImg.loading = 'lazy';
    fullImg.src = url;
    root.querySelector('st-cover-full').hidden = fullImg.complete;
    root.querySelector('st-cover-lite').hidden = !fullImg.complete;
    loadedImage(url).then(() => {
      root.querySelector('st-cover-full').hidden = false;
      root.querySelector('st-cover-lite').hidden = true;
    }).catch(error => console.error(error));

    root.querySelector('a').href = `https://stadia.google.com/player/${game.app}`;
    root.querySelector('st-name').textContent = game.name;

    if (!game.microImage) {
      const microImage = await microImageFromURL(url);
      game.microImage = microImage;
    }

    root.querySelector('st-cover-lite').style.backgroundImage = `url(${
      microImageToURL(game.microImage)})`;

    root.setAttribute('data-name', game.name);
    root.setAttribute('data-app-id', game.app);
    root.setAttribute('data-micro-image', game.microImage);
    root.setAttribute('data-full-cover', game.image);

    container.appendChild(root);
  }

  document.querySelector('st-games').appendChild(container)
};
