// フレームワーク横断 CSR ベンチマークの比較対象一覧（正）。
//
// bench/PROTOCOL.md §1 の CSR 対象（7 種）と一致させる。run_csr.mjs
// （実行時間計測）と payload/measure.mjs（バンドルサイズ計測）の双方が
// この配列を import して使う。2 箇所で別々にハードコードすると、
// 一方だけ対象を増減したときに「一部だけ計測されているのに気付かない」
// fail-open（PR #1370 codex 再レビュー指摘、P1 x2）を招くため、
// 正本をここへ一元化する。
export const ALL_FRAMEWORKS = ["vanilla", "react", "preact", "vue", "svelte", "lit", "fandhe"];
