/**
 * 全フレームワーク共通の行ラベル生成ヘルパー。
 *
 * `crates/xtask/src/bench_ssr.rs` のワークロード定義（`row()`）と同一の
 * フォーマットを再現する。既定エスケープ対象 5 文字（`&` `<` `>` `"` `'`）を
 * すべて含み、`<script>alert(1)</script>` という意図的な XSS ペイロードを
 * 埋め込む。各フレームワークの renderer（`renderers/*.mjs`）は、この文字列を
 * 必ず「テキスト補間・テキストノード」の経路（React の子要素・Vue の `h()`
 * テキスト子・Solid の `escape()`・Svelte の `{expression}`・Lit の
 * `html` タグ付きテンプレートの `${}` 差し込み）でのみ出力し、
 * `dangerouslySetInnerHTML` / `v-html` / `{@html}` / `unsafeHTML` などの
 * エスケープ迂回 API を使わない（既定エスケープの計測が目的のため）。
 */
export function rowLabel(i) {
  return `Row ${i} & "quoted" 'single' <script>alert(1)</script>`;
}
