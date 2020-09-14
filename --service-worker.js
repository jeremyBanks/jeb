const cacheId = 'stadia.run/cache/v1';

self.addEventListener('fetch', (/** @type {any} */ event) =>
  event.respondWith((async (/** @type {FetchEvent} */ event) => {
    const request = event.request;
    try {
      const response = await fetch(request);
      if (response.ok && (
        request.method === 'GET' || request.method === 'HEAD'
      )) {
        const cache = await caches.open(cacheId);
        cache.put(event.request, response.clone());
      }
      return response;
    } catch (error) {
      return caches.match(request);
    }
  })(event)));
