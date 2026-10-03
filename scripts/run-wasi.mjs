// Run the same Rust conformance test binaries on WASI using Node's Preview 1 host.
import { readFile } from 'node:fs/promises';
import { WASI } from 'node:wasi';

const [binary, ...args] = process.argv.slice(2);
if (!binary) throw new Error('Usage: node scripts/run-wasi.mjs <binary.wasm> [args]');
const wasi = new WASI({ version: 'preview1', args: [binary, ...args], env: {}, preopens: {} });
const module = await WebAssembly.compile(await readFile(binary));
const instance = await WebAssembly.instantiate(module, { wasi_snapshot_preview1: wasi.wasiImport });
wasi.start(instance);
