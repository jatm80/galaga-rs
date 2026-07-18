import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';

function optimizeReleaseWasm(path: string): void {
  const result = spawnSync('wasm-opt', ['-O3', '--strip-debug', '--strip-producers', path, '-o', path], { stdio: 'inherit' });
  if (result.error !== undefined && (result.error as NodeJS.ErrnoException).code === 'ENOENT') {
    console.warn('wasm-opt not found; skipping optional Binaryen release optimization.');
    return;
  }
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const targetIndex = process.argv.indexOf('--target');
const target = targetIndex >= 0 ? process.argv[targetIndex + 1] : 'release';
if (target !== 'debug' && target !== 'release') {
  throw new Error('--target must be debug or release.');
}
const releaseArgs = target === 'release' ? ['--release'] : [];
const result = spawnSync('cargo', ['build', '--target', 'wasm32-unknown-unknown', ...releaseArgs], {
  stdio: 'inherit',
});
if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

mkdirSync('public', { recursive: true });
copyFileSync(`target/wasm32-unknown-unknown/${target}/galaga_rs.wasm`, 'public/app.wasm');
if (target === 'release') optimizeReleaseWasm('public/app.wasm');
