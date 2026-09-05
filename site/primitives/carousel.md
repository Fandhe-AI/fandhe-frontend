# Carousel

スライド送り UI です。`fandhe-frontend-headless-ui` の `carousel` mod は Root / Control / PrevTrigger / NextTrigger / ItemGroup / Item / IndicatorGroup / Indicator の 8 anatomy パーツと、`role="region"`・`aria-roledescription`・自動生成の `aria-label` を提供します。Themes 版と異なりトランジション CSS やレイアウトを持たず、SSR での属性出力と決定的な状態機械のみを担います。

**アクセシビリティ・参考サイトとの対応**（イシュー #1660）: zag.js（ark-ui の内部実装）と突合し、`data-orientation` を全 8 パーツへ拡張、`item`/`indicator` に `data-index`（0-origin）、`item` に `data-inview` を追加しました。一方、非表示スライドへの `aria-hidden` 付与は本モジュールが CSS 前提の見た目を持たない SSR 静的マークアップのため意図的に見送っています（非 current を実際に隠さない構成で `aria-hidden` のみ付けると全スライドが可視のまま支援技術からのみ隠れる不整合になるため）。非 current を CSS で隠す場合は呼び出し側で `attrs` に `("aria-hidden", "true")` を渡してください。同様に `aria-controls`（trigger → item-group）・各パーツの `id`/`dir` も headless-ui に生成機構がないため、必要な場合は `item_group` の `attrs` に `id`、trigger の `attrs` に `aria-controls` を明示的に渡す運用とします。

**キーボード操作（現状の対応範囲）**: 状態機械 `Carousel` は決定的な dispatch action を提供します（`"next"`/`"prev"` が横向き ArrowRight/ArrowLeft・縦向き ArrowDown/ArrowUp 相当、`"first"`/`"last"` が Home/End 相当）。ただし実際のキーボードイベント配線（`keydown` リスナー登録・`orientation` に応じたキー選別）は `fandhe-frontend-wasm-full`（クライアントランタイム）側の後続責務であり、本イシューのスコープ外です。trigger/indicator は native `button` のためクリック（Enter/Space）は標準の DOM 挙動でカバーされます。

スタイル済みの表示例は [Carousel](../themes/carousel.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
