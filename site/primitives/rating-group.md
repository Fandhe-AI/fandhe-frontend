# Rating Group

`fandhe-frontend-headless-ui` の `rating_group` mod が提供する星評価の unstyled 部品です。Root / Label / Control / Item / HiddenInput の 5 anatomy パーツを持ち、`1..=count` の数値評価値（未評価は `None`）と hover プレビューを表現する状態機械を備えます。`readonly` フラグにより表示専用の平均評価等も安全に描画できます。

`/themes/` 側の `RatingGroup`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`RatingGroupProps`（`disabled`/`readonly`/`required`）は次のように各パーツへ反映されます。`label` は `data-disabled`/`data-required` を、`control` は `data-disabled`/`data-readonly` に加えて `disabled`/`readonly` が真のときのみ `aria-disabled="true"`/`aria-readonly="true"` を付与します。`item` には roving `tabindex`（確定選択中の星、未評価なら 1 番目の星が `"0"`、それ以外は `"-1"`、`disabled` のときは属性自体を省略）が付きます。hidden-input の `required` は `type="hidden"` に効果がないため付与しません。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| ArrowRight | Item | 次の星へ roving フォーカスを移し値を 1 つ増やします（dispatch: `"set"`）。 |
| ArrowLeft | Item | 前の星へ roving フォーカスを移し値を 1 つ減らします（dispatch: `"set"`）。 |
| Enter | Item | フォーカス中の星番号を確定値として設定します（dispatch: `"set"`）。 |

ポインタ・キーボード操作の実際の DOM 配線（`keydown` イベントハンドラ、`fandhe-frontend-wasm-full`）は本部品のスコープ外です。本部品は SSR 静的マークアップと dispatch 契約のみを提供します。

**参考サイトとの差分**

ark-ui Rating Group と突合し、上記のキーボード操作・`data-*`/`aria-*` 語彙へ揃えました。一方、以下は意図的に合わせていません。

- `data-half`（`allow_half`、0.5 刻み評価）は状態機械・CSS が別設計のため未提供です（#742 以来の継続）。
- `aria-setsize`/`aria-posinset`（item）は全 item が DOM 上の兄弟要素として連続配置されるため、支援技術が自動算出できます。
- `aria-roledescription="rating"`（item）は `role="radio"` + `aria-label` で十分と判断し、追随していません。
- `aria-orientation="horizontal"`（control）は固定値かつ軸を持たないため、`data-orientation` も含めて不採用です。
- `data-hover`/`data-active`/`data-focus`（pointer/focus 系）は SSR 静的出力の関心外です（Checkbox #1602 と同じ判断）。

スタイル済みの表示例は [Rating Group](../themes/rating-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="rating-group"][data-part="..."]` セレクタでスタイルを当てます。以下は星の寸法・塗り分け（`data-highlighted`）・確定選択（`data-checked`）・無効化・フォーカスリングの最小例です。

```css
[data-scope="rating-group"][data-part="item"] {
  display: inline-block;
  width: 1.25rem;
  height: 1.25rem;
  cursor: pointer;
}

[data-scope="rating-group"][data-part="item"][data-highlighted] {
  color: #f5a623;
}

[data-scope="rating-group"][data-part="item"][data-checked] {
  font-weight: bold;
}

[data-scope="rating-group"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="rating-group"][data-part="item"]:focus-visible {
  outline: 2px solid #06c;
  outline-offset: 2px;
}
```
