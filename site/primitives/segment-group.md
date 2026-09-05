# Segment Group

`fandhe-frontend-headless-ui` の `segment_group` mod が提供する segmented control（セグメントコントロール）の unstyled 部品です。Root / Indicator / Item / ItemText / ItemControl / ItemHiddenInput の 6 anatomy パーツを持ちますが、状態機械・dispatch・hydration は `radio_group` へ全委譲しており独自の状態機械は持ちません（WAI-ARIA 上 segmented control は radio パターンそのものであるため）。`Indicator` は SSR 決定的な位置表現（CSS カスタムプロパティ 2 種）のみを提供します。

`/themes/` 側の `SegmentGroup`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`SegmentGroupProps`（`disabled`/`readonly`/`invalid`/`required`）は次のように各パーツへ反映されます。`root` は `data-disabled`/`data-invalid`/`data-required` を付与し、さらに `disabled`/`readonly`/`required` が `true` のときのみ対応する `aria-disabled`/`aria-readonly`/`aria-required="true"`（`radiogroup` ロールの Supported States）を付与します。`indicator` は `data-disabled` のみを付与します。`item`/`item-control`/`item-text` は `data-disabled`/`data-readonly`/`data-invalid` を付与します（`data-required` は item 系パーツに無い属性です）。`item-control` は意味論を持たない装飾パーツであることを明示するため `aria-hidden="true"` を常時付与します。`item-hidden-input` には `invalid: true` のときのみ `aria-invalid="true"` を、`required: true` のときのみネイティブ `required` 存在属性を付与します（`readonly` はネイティブ `<input type="radio">` に無効な属性のため反映しません）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | Item | グループへフォーカスを移します（選択済み項目があればその項目、無ければ先頭項目）。ネイティブ radio group のロービングタブインデックスに従います。 |
| Space | Item | フォーカス中の未選択項目を選択します。 |
| ArrowDown / ArrowRight | Item | 次の項目へ移動し選択します。 |
| ArrowUp / ArrowLeft | Item | 前の項目へ移動し選択します。 |

`item-hidden-input` が生成するネイティブ `<input type="radio">`（同一 `name` グループ）の標準操作のみで成立します。`radio_group` と異なり `fandhe-frontend-wasm-full` に segment-group の CSR 配線が一切無いため、Home/End 拡張・`data-readonly` の選択変更抑止といった `keynav` 由来の挙動はいずれも提供されません。

**参考サイトとの差分**

ark-ui 公式 Segment Group（API 節・Data Attributes 表）と突合し、上記の `data-*`/`aria-*` 語彙を是正しました。一方、以下は意図的に合わせていません。

- ark の API 節には `Label`（`data-orientation`/`data-disabled`/`data-invalid`/`data-required`）が載りますが、Anatomy 図（コードスニペット）には存在しません。`/themes/` 側の `SLOTS` 固定リストと CSS への波及を避けるため今回は採用を見送っています（外部ラベルとの関連付けは `root` の `labelled_by` で成立させます）。
- `indicator` の `data-state`/`style`（CSS カスタムプロパティ 2 種、SSR 決定的）は本フレームワーク固有として維持します（`/themes/` 側が `indicator[data-state="unchecked"]` セレクタで依存するため。ark の CSR 実測による追従は持ち込みません）。
- `data-orientation` は `root`/`indicator` のみへ付与し、子パーツへ伝播しません（`radio_group` と同判断）。
- `data-readonly` は `root` へ出力しません（ark の Root Data Attributes 表に無いためです）。ネイティブ `<input type="radio">` に `readonly` 属性は無効なため `item-hidden-input` へも反映しません。`fandhe-frontend-wasm-full` に segment-group の CSR 配線が一切無いため、`data-readonly`/`aria-readonly` は SSR 語彙にとどまり、CSR での選択変更抑止は `radio_group` と異なり未提供です。
- Radix Themes の Segmented Control は styled 部品のため視覚参考にとどめ、ARIA・キーボードの一次情報は ark-ui を採用しています。

スタイル済みの表示例は [Segment Group](../themes/segment-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="segment-group"][data-part="..."]` セレクタでスタイルを当てます。以下は root の外枠、item の余白、checked/disabled の見た目、invalid 時の枠色、フォーカスリング、indicator を CSS カスタムプロパティから描く最小例です。

```css
[data-scope="segment-group"][data-part="root"] {
  display: inline-flex;
  position: relative;
  border: 1px solid #ccc;
  border-radius: 6px;
}

[data-scope="segment-group"][data-part="root"][data-invalid] {
  border-color: #d33;
}

[data-scope="segment-group"][data-part="item"] {
  padding: 0.25rem 0.75rem;
  cursor: pointer;
}

[data-scope="segment-group"][data-part="item"][data-state="checked"] {
  color: #fff;
}

[data-scope="segment-group"][data-part="item"][data-disabled] {
  opacity: 0.5;
}

[data-scope="segment-group"][data-part="item"]:focus-within {
  outline: 2px solid #06c;
  outline-offset: 2px;
}

[data-scope="segment-group"][data-part="indicator"] {
  position: absolute;
  inset-block: 0;
  width: calc(100% / var(--fandhe-segment-group-count));
  transform: translateX(calc(100% * var(--fandhe-segment-group-index)));
  background: #06c;
  border-radius: 6px;
  transition: transform 0.15s ease;
}
```
