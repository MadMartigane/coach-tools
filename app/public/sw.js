const CACHE_NAME = "coach-tools-tabata-v1";
const CORE_ASSETS = [
  "./",
  "./index.html",
  "./manifest.webmanifest",
  "./icon-192.svg",
  "./icon-512.svg",
  "./icon-maskable-512.svg",
  "./apple-touch-icon.svg",
];
const HASHED_ASSET_PATTERN = /(?:href|src|module_or_path:\s*['"])(\/[^'"]+\.(?:wasm|js|css))/g;

self.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);
      await cache.addAll(CORE_ASSETS);
      const indexResponse = await fetch("./index.html");
      const indexHtml = await indexResponse.text();
      const hashedAssets = [...indexHtml.matchAll(HASHED_ASSET_PATTERN)].map((match) => match[1]);
      await cache.addAll(hashedAssets);
      self.skipWaiting();
    })(),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const cacheKeys = await caches.keys();
      await Promise.all(
        cacheKeys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key)),
      );
      await self.clients.claim();
    })(),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) {
    return;
  }
  event.respondWith(
    (async () => {
      const cachedResponse = await caches.match(request);
      if (cachedResponse) {
        return cachedResponse;
      }
      try {
        const networkResponse = await fetch(request);
        if (networkResponse.status === 200) {
          const cache = await caches.open(CACHE_NAME);
          cache.put(request, networkResponse.clone());
        }
        return networkResponse;
      } catch (error) {
        if (request.mode === "navigate") {
          return caches.match("./index.html");
        }
        throw error;
      }
    })(),
  );
});
