const http = require('http');
const httpProxy = require('http-proxy');

const proxy = httpProxy.createProxyServer({});

const server = http.createServer((req, res) => {
  // CORS headers
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  
  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }
  
  // Proxy to MERKLITH node
  proxy.web(req, res, { target: 'http://localhost:8545' });
});

console.log('CORS Proxy server listening on port 8540');
console.log('Proxying requests from http://localhost:8540 to http://localhost:8545');
server.listen(8540);
