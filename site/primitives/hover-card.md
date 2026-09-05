# Hover Card

リンク先プレビュー等、hover / focus で開閉するオーバーレイです。
`fandhe-frontend-headless-ui` の `hover_card` mod は Root / Trigger /
Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツを提供します。
trigger はリンク先プレビュー用途の `a` 要素であり、`HoverCardDelays`
（`open_ms`/`close_ms`、ark-ui 既定の 600/300 ms）を `data-open-delay`/
`data-close-delay` として決定的に出力します。WAI-ARIA に hover card 専用
パターンは存在しないため、`aria-expanded`/`aria-controls`/`aria-haspopup`
及び固定 `role` を一切付与しません（Tooltip との違い）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | Trigger | trigger はネイティブ `a` 要素のため、`href` を渡している場合はブラウザ標準でフォーカス到達します。Radix の「Tab で hover card を開閉」に相当する focus/blur 駆動の開閉配線は `fandhe-frontend-wasm-full` 側の責務で未配線です。 |
| Enter | Trigger | `href` を渡している場合、リンク先へ遷移します（ブラウザ標準）。hover card 自体の開閉はこのキー操作では行いません。 |

**参考サイトとの差分**

Zag.js（`hover-card.connect.ts`）・ark-ui・Radix Primitives・Radix Themes・
chakra-ui と突合し、**是正すべき欠落は見つかりませんでした**（イシュー
#1641）。anatomy（6 パート）・`data-state` 語彙（`open`/`closed`）・ARIA
非付与方針はいずれも一致しています。一方、以下は意図的に合わせていません。

- **`data-side`/`data-align` の付与先**: Radix は `content` へ、Zag は
  `trigger`/`content` の双方へ出しますが、本クレートは `positioning`
  （#590）の規約どおり `positioner` へ透過させます（Tooltip/Popover と
  同型。hover-card だけ変えると規約が分裂するため）。
- **`tabindex="-1"`（content）・`dir`・自動 `id`**（Zag）: hover/focus の
  タイマー・DOM 配線が `fandhe-frontend-wasm-full` に未実装の段階では
  固定付与しません。
- **複数トリガー識別（`data-value`/`data-current`/`data-ownedby`、Zag）**:
  機能拡張でありスコープ外候補です（下記参照）。

上記の差分に伴う `data-*`・パートの増減は無いため、Themes 側イシュー
#1523（closed）への追加コメントは行っていません。

スコープ外候補（是正対象ではなく、別途起票を検討する余地がある事項）:

- 複数トリガー対応（Zag の `data-value`/`data-current`/`data-ownedby`）
- `fandhe-frontend-wasm-full` 側の hover/focus タイマー配線・
  `PositionedKind::from_scope("hover-card")` への追加
- 配線後の content `tabindex="-1"` 付与の再評価

スタイル済みの表示例は [Hover Card](../themes/hover-card.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

`[data-scope="hover-card"][data-part="..."]` セレクタでスタイルを当てます。
以下は content の枠・影、closed 時の非表示、trigger のフォーカスリング、
positioner の配置調整の最小例です。

```css
[data-scope="hover-card"][data-part="content"] {
  border: 1px solid #888;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

[data-scope="hover-card"][data-part="content"][data-state="closed"] {
  display: none;
}

[data-scope="hover-card"][data-part="trigger"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

[data-scope="hover-card"][data-part="positioner"][data-side="top"] {
  margin-bottom: 8px;
}
```
