# Accordion

高々 1 項目が開く single モードと、複数項目が同時に開く multiple モードの
2 状態機械を提供する開閉可能な項目リストです。`fandhe-frontend-headless-ui`
の `accordion` mod は Root / Item / ItemTrigger / ItemIndicator /
ItemContent の 5 anatomy パーツと、開閉状態を表す `data-state`・
`aria-expanded`・`aria-controls`・（ラベル付き時のみ）`role="region"` を
提供します。`AccordionProps`（`orientation`/`disabled`、イシュー #1636）を
全パーツへ通すことで `data-orientation` を全パーツへ、全項目一括
`disabled` を項目単位の `disabled` と OR 合成した実効値として反映します。
item-trigger は実効 disabled が true のときのみ `aria-disabled="true"` を、
item-indicator は常時 `aria-hidden="true"` を付与します。キーボード操作
（`orientation` に応じた ArrowDown/ArrowUp または ArrowRight/ArrowLeft・
Home/End・非循環）の実 DOM 配線は `fandhe-frontend-wasm-full` の
`keynav.rs` が担い、詳細は下記 Keyboard/Accessibility 節を参照してください。

スタイル済みの表示例は [Accordion](../themes/accordion.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
