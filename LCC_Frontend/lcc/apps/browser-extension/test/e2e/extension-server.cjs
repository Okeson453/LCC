// E2E harness server: serves the static dist/ for the extension popup / sidepanel
// so Playwright can load them in a regular page context.
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');

const PORT = process.env.E2E_PORT ? Number(process.env.E2E_PORT) : 8082;
const ROOT = path.resolve(__dirname, '../../../dist');

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
};

const server = http.createServer((req, res) => {
  let urlPath = (req.url ?? '/').split('?')[0];
  if (urlPath === '/') urlPath = '/sidepanel/index.html';
  const file = path.join(ROOT, urlPath);
  if (!file.startsWith(ROOT)) {
    res.writeHead(403).end();
    return;
  }
  fs.readFile(file, (err, data) => {
    if (err) {
      res.writeHead(404).end('not found');
      return;
    }
    const ext = path.extname(file);
    res.writeHead(200, { 'Content-Type': TYPES[ext] ?? 'application/octet-stream' });
    res.end(data);
  });
});

server.listen(PORT, () => {
  console.log(`e2e harness listening at http://localhost:${PORT}`);
});
