# Accordion

`fandhe-frontend-pre-styled-ui` の `accordion` mod が提供するスタイル済み Accordion 部品です。
項目（Item）ごとに開閉するトリガー（ItemTrigger）とパネル（ItemContent）を持ち、
高々 1 項目だけが開く single モードと複数項目が同時に開ける multiple モードの
2 つの状態機械（`Accordion` / `MultiAccordion`）を選べます。ページ内に収まる
disclosure（開閉パネル）であり、他のセクションを覆うオーバーレイではないため、
掲示位置を中和する専用 CSS は不要です。項目パネル（ItemContent）の開閉は headless 層が
付与する `hidden` 属性で行いますが、JS 有効時は `fandhe-frontend-wasm-full` が実測した
高さを CSS 変数（`--fandhe-content-height`）へ書き込み、`@starting-style` +
`transition-behavior: allow-discrete` によって高さトランジションとして表現します
（[Collapsible](collapsible.md) と同型の仕組み）。JS 無効時は `auto` フォールバックで
従来どおり `hidden` による即時切り替えのまま動作します。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
