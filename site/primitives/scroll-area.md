# Scroll Area

カスタムスクロール領域です。`fandhe-frontend-headless-ui` の `scroll_area` mod は Root / Viewport / Content / Scrollbar / Thumb / Corner の 6 anatomy パーツを提供し、`viewport` にはキーボードスクロール操作のための `tabindex="0"` を固定出力します。Themes 版と異なりスクロールバーの見た目（`overflow` の CSS 表現）を持たず、構造とフォーカス制御属性のみを担います。

スタイル済みの表示例は [Scroll Area](../themes/scroll-area.md) を参照してください。

**アクセシビリティ・参考サイトとの対応**

- anatomy は ark-ui/Zag.js（root/viewport/content/scrollbar/thumb/corner の 6 パーツ）と完全一致します。Radix Primitives は `content` を持ちませんが、ark-ui/chakra-ui との一致を優先し維持しています。
- ark-ui/Zag.js が持つ `data-overflow-x`/`data-overflow-y`/`data-at-*`/`data-hover`/`data-scrolling`/`data-dragging`、Radix Primitives の `data-state="visible"|"hidden"` はいずれも DOM 計測・ポインタ操作由来の実行時状態であり、SSR の静的マークアップでは真の値を決定できないため採用していません。
- Zag.js が付与する `role="presentation"` は追加していません。`viewport` は `tabindex="0"` を固定付与しフォーカス可能であるため、WAI-ARIA 1.2 の Presentational Roles Conflict Resolution によりブラウザから無視される値であり、Radix Primitives（`role` 非付与）とも整合します。
- `viewport` の `tabindex="0"` 固定は維持しています。SSR では `overflow` の有無を判定できないため、WCAG 2.1.1（スクロール領域のキーボード到達性）に対して安全側に倒す設計です。
- `scrollbar`/`corner` の `aria-hidden="true"` は両参照サイトにはない本実装独自の付与ですが、ネイティブスクロールバーとの意味重複を明示する目的で付与しています。
- キーボード操作は独自のキーハンドラを持たず、矢印キー/PageUp・PageDown/Home・End/Space などはブラウザのネイティブスクロール挙動に委ねます（Radix Primitives docs も同方針を明記）。
- 自前 CSS で組み立てる最小例は [API Reference](../../docs/api/headless-ui-api.md) の Scroll Area 節を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
