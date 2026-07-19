/**
 * fandhe-frontend-wasm-full の既定方式（SSR + ハイドレーション）向けアプリ側 JS グルーの参照実装。
 *
 * `crates/wasm-full/src/entry.rs` の `hydrate(root_id)` は SSR 済み DOM の
 * `data-hydrate-*` 属性から状態を復元し、失敗時は初期状態での CSR 再描画へ
 * フォールバックする契約（同ファイル rustdoc 参照）。本ファイルはその契約を
 * 呼び出す最小のグルーであり、DOM 構築・HTML 文字列組み立て・状態管理を
 * 一切行わない。
 *
 * import 元は dist-server が配信する同一オリジンパス
 * `/static/wasm/fandhe_frontend_wasm_full.js` に固定し、外部 CDN を参照しない
 * （`.claude/rules/security.md` サプライチェーン対策）。
 *
 * セキュリティ不変条件: 本ファイルは `innerHTML` / `document.write` / HTML
 * 文字列組み立てを一切行わない。DOM 更新の XSS 安全性は Rust 側
 * （fandhe-frontend-core の既定エスケープ、REQ-1）に閉じたままとする契約を維持する。
 *
 * LOC ルーブリック: 実効行数 10 行以内を維持する（PoC-5 実績 3 行、REQ-11
 * 受け入れ基準 3。機械ゲート化はイシュー #156・`xtask check-loc`）。
 * 自コンポーネントを持つアプリケーションは本ファイルと同型のラッパーを
 * 自身の配信物として実装する想定の参照実装であり、実配線（デモページへの
 * `<script type="module">` 統合等）はスコープ外（イシュー #156 実装計画 §8）。
 */
import init, { hydrate } from "/static/wasm/fandhe_frontend_wasm_full.js";
await init();
hydrate("app");
