import { accessSync, readFileSync } from 'node:fs';

for (const file of [
  'public/index.html',
  'public/favicon.ico',
  'public/harness.js',
  'public/app.wasm',
  'public/bridge.js',
  'public/effindom-runtime-config.js',
  'public/runtime/dist/effindom.v2.manifest.json',
]) {
  accessSync(file);
}

if (!readFileSync('public/index.html', 'utf8').includes('id="fui-canvas"')) {
  throw new Error('Generated shell is missing #fui-canvas.');
}
if (readFileSync('public/index.html', 'utf8').includes('{{LOADING_OVERLAY_')) {
  throw new Error('Generated shell contains unresolved loading-overlay placeholders.');
}
