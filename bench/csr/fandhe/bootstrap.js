// フレームワーク横断 CSR ベンチマーク（bench/PROTOCOL.md §2.2）向けの
// fandhe-frontend ブート処理。build.sh が __WASM_GLUE__ を wasm-bindgen
// 生成物の実ファイル名（fandhe_bench.js）へ置換したうえで esbuild
// （--minify --format=esm）を適用し、bench/csr/dist/fandhe/bootstrap.js
// として配置する。
//
// inline <script> ではなく独立 .js ファイルにしているのは、この起動コードを
// payload 計測（bench/payload/measure.mjs、.js/.wasm が対象で index.html は
// 対象外）へ含めるため。他フレームワークは起動コード込みの bundle.js を
// 計測しており、fandhe だけ起動コードを index.html 側へ逃すと payload
// 比較が非対称になる（bench/PROTOCOL.md §2.3）。
import init, { bench_create, bench_update, bench_clear } from "./__WASM_GLUE__";

await init();

// window.__bench: playwright ハーネス（bench/csr/run_csr.mjs）が直接呼ぶ
// 計測対象 API（bench/PROTOCOL.md §2.2）。ハーネスは waitForFunction で
// window.__bench.{create,update,clear} の 3 関数が揃うことを検証してから
// 計測を始めるため、wasm 初期化完了（上の await init()）後にのみ束縛する
// この代入自体が「準備完了」の合図を兼ねる（別途の ready フラグは不要）。
// ボタンクリックは手動動作確認用の配線であり、計測経路とは独立
// （同じ Rust 関数を呼ぶのみ）。
window.__bench = {
  create: () => bench_create(),
  update: () => bench_update(),
  clear: () => bench_clear(),
};

document.getElementById("create").addEventListener("click", () => window.__bench.create());
document.getElementById("update").addEventListener("click", () => window.__bench.update());
document.getElementById("clear").addEventListener("click", () => window.__bench.clear());
