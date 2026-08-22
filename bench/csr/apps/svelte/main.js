// Svelte 5 による CSR ベンチアプリのエントリ。App.svelte は build.mjs の
// esbuild プラグイン（svelte/compiler の generate:"client"）が事前に
// JS へコンパイルしてからバンドルする。
import { mount, flushSync } from "svelte";
import App from "./App.svelte";

function main() {
  const tbody = document.querySelector("#bench-table tbody");
  const app = mount(App, { target: tbody });

  // Svelte 5 の $state 更新は既定でマイクロタスクへバッチされるため、
  // __bench 呼び出し完了時点で DOM 反映済みを保証するには flushSync()
  // で pending な更新を同期的に確定させる（計測境界の変更、
  // bench/PROTOCOL.md §2.2 参照）。
  const create = () => flushSync(() => app.create());
  const update = () => flushSync(() => app.update());
  const clear = () => flushSync(() => app.clear());

  window.__bench = { create, update, clear };
  document.querySelector("#create").addEventListener("click", create);
  document.querySelector("#update").addEventListener("click", update);
  document.querySelector("#clear").addEventListener("click", clear);
}

main();
