import { copyFileSync, cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';

const runtimeManifest = JSON.parse(readFileSync('node_modules/@effindomv2/runtime/dist/effindom.v2.manifest.json', 'utf8')) as {
  runtime_set_hash?: string;
};
if (typeof runtimeManifest.runtime_set_hash !== 'string' || runtimeManifest.runtime_set_hash.length === 0) {
  throw new Error('Installed EffinDOM runtime does not declare runtime_set_hash.');
}
const cdnManifestUrl = `https://runtimes.effindom.dev/v2/manifests/${runtimeManifest.runtime_set_hash}.json`;

rmSync('public', { recursive: true, force: true });
mkdirSync('public/runtime', { recursive: true });
cpSync('node_modules/@effindomv2/runtime/dist', 'public/runtime/dist', { recursive: true });
copyFileSync('node_modules/@effindomv2/runtime/dist/bridge.js', 'public/bridge.js');
copyFileSync('favicon.ico', 'public/favicon.ico');
writeFileSync(
  'public/effindom-runtime-config.js',
  `window.__effindomRuntime = Object.assign({}, window.__effindomRuntime, ${JSON.stringify({
    manifestUrls: [cdnManifestUrl, './runtime/dist/effindom.v2.manifest.json'],
    expectedRuntimeSetHash: runtimeManifest.runtime_set_hash,
    buildMode: 'release',
  })});\n`,
  'utf8',
);
writeFileSync(
  'public/index.html',
  readFileSync('index.html', 'utf8')
    .replace('{{LOADING_OVERLAY_STYLES}}', readFileSync('loading-overlay-styles.html', 'utf8'))
    .replace('{{LOADING_OVERLAY_BODY}}', readFileSync('loading-overlay-body.html', 'utf8')),
  'utf8',
);
