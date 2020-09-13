const cacheId = 'stadia.run/cache/v1';

self.addEventListener('fetch', event => event.respondWith(async () => {
  try {
    const response = await fetch(event.request);
    if (response.ok) {
      const cache = await caches.open(cacheId);
      cache.put(event.request, response.clone());
    }
    return response;
  } catch (error) {
    return caches.match(event.request);
  }
}));
