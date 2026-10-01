/**
 * PWA service worker — audits P-04.
 *
 * Strategy:
 * - HTML routes: network-first with cache fallback.
 * - Static assets: cache-first with stale-while-revalidate.
 * - API: never cached.
 */
const VERSION = 'v1';
const STATIC_CACHE = `static-${VERSION}`;
const HTML_CACHE = `html-${VERSION}`;
const API_EXCLUDE = [/^\/api\//];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(STATIC_CACHE).then((cache) => cache.addAll(['/manifest.webmanifest', '/robots.txt'])),
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== STATIC_CACHE && k !== HTML_CACHE).map((k) => caches.delete(k))),
    ),
  );
  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  const req = event.request;
  if (req.method !== 'GET') return;
  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return;
  if (API_EXCLUDE.some((rx) => rx.test(url.pathname))) return;

  if (req.mode === 'navigate') {
    event.respondWith(networkFirst(req));
  } else if (url.pathname.startsWith('/_next/static/') || url.pathname.startsWith('/icons/') || url.pathname === '/manifest.webmanifest') {
    event.respondWith(cacheFirst(req, STATIC_CACHE));
  } else {
    event.respondWith(staleWhileRevalidate(req, HTML_CACHE));
  }
});

async function networkFirst(req) {
  try {
    const res = await fetch(req);
    const cache = await caches.open(HTML_CACHE);
    cache.put(req, res.clone());
    return res;
  } catch {
    const cached = await caches.match(req);
    return cached ?? new Response('Offline', { status: 503 });
  }
}

async function cacheFirst(req, name) {
  const cached = await caches.match(req);
  if (cached) return cached;
  const res = await fetch(req);
  const cache = await caches.open(name);
  cache.put(req, res.clone());
  return res;
}

async function staleWhileRevalidate(req, name) {
  const cached = await caches.match(req);
  const fetchPromise = fetch(req).then((res) => {
    caches.open(name).then((c) => c.put(req, res.clone()));
    return res;
  });
  return cached ?? fetchPromise;
}
