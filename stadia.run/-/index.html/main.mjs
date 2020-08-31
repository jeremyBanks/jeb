// @generated from index.html, do not edit directly.
//   <script type="module" id="main.mjs">
    export const digits = '-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz';

    const slugify = s => {
      return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^\-+|\-+$/g, '');
    };

    const init = async () => {
      await Promise.all([
        unpackMicroCovers(),
      ]);
    };

    const nonCanonicalDevToolsImport =
      import.meta.url.includes('/index.html/') && import.meta.url.endsWith('.mjs');

    const initialized = Promise.resolve().then(() => {
      if (nonCanonicalDevToolsImport) {
        return;
      }

      console.group('🔧 initializing document');
      return init().finally(() => {
        console.groupEnd();
      });
    });

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

    const unpackMicroCovers = async () => {
      for (const game of document.querySelectorAll('st-game')) {
        const full = game.querySelector('st-cover-full img');
        const micro = game.querySelector('st-cover-micro');

        if (full.complete) {
          // full image already in cache
          micro.hidden = true;
        } else {
          micro.classList.add('rendered');
          micro.style.backgroundImage = `url(${microImageToURL(micro.getAttribute('data'))})`;
          full.hidden = true;
          micro.hidden = false;
        }

        loadedImage(full.src).then(() => {
          full.hidden = false;
          micro.hidden = true;
        });
      }
    };

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

    const form = document.querySelector('st-search form');
    const input = form.querySelector('input');
    const button = form.querySelector('button');
    const gameList = document.querySelector('st-games');

    let filterElementsPending = null;
    const filterElements = async () => {
      if (filterElementsPending) {
        return filterElementsPending
      }

      return filterElementsPending = new Promise(
        resolve => resolve()
      ).then(() => {
        filterElementsPending = false;

        const q = slugify(input.value);

        const elements = [];

        for (const child of gameList.querySelectorAll('st-game')) {
          let name = child.querySelector('st-name').textContent;
          const matches = slugify(name).includes(q);
          child.firstElementChild.hidden = !matches;
          if (matches) {
            elements.push(child);
          }
        }

        document.documentElement.setAttribute('data-st-matches', elements.length);

        return elements;
      });
    };

    input.addEventListener('input', event => onInput(event));

    const onInput = (event) => {
      filterElements().then(elements => {
        const params = new URLSearchParams(location.search);

        params.set('q', input.value.toLowerCase());

        history.replaceState(
          null,
          '',
          input.value ? ('/?' + params.toString()) : '/'
        );
      })
    };

    const onSubmit = (event) => {
      event?.preventDefault?.();

      filterElements().then(elements => {
        const params = new URLSearchParams(location.search);

        if (elements.length === 1) {
          const name = elements[0].querySelector('st-name').textContent;
          const slug = slugify(name);
          if (!event?.first) {
            history.pushState(
              null,
              '',
              '/' + slug
            );
          }
          input.value = name;
          document.title = 'stadia.run/' + slug;
          elements[0].querySelector('a').click();
        } else {
          params.set("q", input.value);
          history.replaceState(
            null,
            '',
            input.value ? ('/?' + params.toString()) : '/'
          );
          document.title = 'stadia.run';
        }
      });
    };

    form.addEventListener('submit', event => onSubmit(event));

    const checkUrl = (first = false) => {
      const slug = slugify(decodeURIComponent(document.location.pathname.slice(1))) || null;
      const query = new URLSearchParams(document.location.search).get('q') || null;

      if (query) {
        input.value = query;
        onInput();
      } else if (slug) {
        input.value = slug.replace(/-/g, ' ');
        onSubmit({first});
      }
    };

    window.addEventListener('popstate', checkUrl);

    checkUrl(true);
