//! PWA web shell — manifest, service worker, HTML for WASM deployment.

/// PWA manifest JSON string.
pub fn manifest_json() -> String {
    r##"{"name":"2009Scape","short_name":"2009Scape","description":"RuneScape 2009 client rebuilt in Rust","start_url":"/","display":"fullscreen","orientation":"landscape","background_color":"#0a0a1a","theme_color":"#1a1a3a","icons":[{"src":"/icons/icon-192.png","sizes":"192x192","type":"image/png"},{"src":"/icons/icon-512.png","sizes":"512x512","type":"image/png"}]}"##.to_string()
}

/// Service worker JS.
pub fn service_worker_js() -> &'static str {
    "const CACHE_NAME='rs2-v1';const URLS=['/','/index.html','/rs2_client_bg.wasm','/rs2_client.js'];self.addEventListener('install',e=>e.waitUntil(caches.open(CACHE_NAME).then(c=>c.addAll(URLS))));self.addEventListener('fetch',e=>{if(e.request.method!=='GET')return;e.respondWith(caches.match(e.request).then(r=>r||fetch(e.request)))});"
}

/// HTML shell for WASM client.
pub fn index_html() -> &'static str {
    r##"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1"><title>2009Scape</title><link rel="manifest" href="/manifest.json"><style>*{margin:0;padding:0}html,body{width:100%;height:100%;overflow:hidden;background:#0a0a1a}canvas{width:100%;height:100%;display:block;touch-action:none}#loading{position:fixed;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;background:#000;color:#c4a832;font-family:serif;font-size:24px;z-index:100}.bar{width:300px;height:24px;border:2px solid #3a3a5a;margin-top:20px}.fill{height:100%;background:#2a7a2a;width:0%;transition:width .3s}</style></head><body><div id="loading"><div>2009Scape</div><div class="bar"><div class="fill" id="p"></div></div></div><canvas id="canvas"></canvas><script type="module">import init from'./rs2_client.js';document.getElementById('p').style.width='30%';init().then(()=>{document.getElementById('p').style.width='100%';setTimeout(()=>document.getElementById('loading').style.display='none',500)});if('serviceWorker'in navigator)navigator.serviceWorker.register('/sw.js');</script></body></html>"##
}
