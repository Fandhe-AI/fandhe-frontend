# Radio Group

`fandhe-frontend-headless-ui` の `radio_group` mod が提供する「高々 1 項目が選択される」ラジオグループの unstyled 部品です。Root / Label / Item / ItemControl / ItemText / ItemHiddenInput の 6 anatomy パーツを持ち、意味論・キーボード操作・排他選択はネイティブ `<input type="radio">`（`ItemHiddenInput`）に委ねる構造です。

`/themes/` 側の `RadioGroup`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`RadioGroupProps`（`disabled`/`readonly`/`invalid`/`required`）は次のように各パーツへ反映されます。`root`/`label` は `data-disabled`/`data-invalid`/`data-required` を付与し、`root` はさらに `disabled`/`readonly`/`required` が `true` のときのみ対応する `aria-disabled`/`aria-readonly`/`aria-required="true"`（`radiogroup` ロールの Supported States）を付与します。`item`/`item-control`/`item-text` は `data-disabled`/`data-readonly`/`data-invalid` を付与します（`data-required` は item 系パーツに無い属性です）。`item-control` は意味論を持たない装飾パーツであることを明示するため `aria-hidden="true"` を常時付与します。`item-hidden-input` には `invalid: true` のときのみ `aria-invalid="true"` を、`required: true` のときのみネイティブ `required` 存在属性を付与します（`readonly` はネイティブ `<input type="radio">` に無効な属性のため反映しません）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | Root | グループへフォーカスを移します（選択済み項目があればその項目、無ければ先頭項目）。ネイティブ radio group のロービングタブインデックスに従います。 |
| Space | Item | フォーカス中の未選択項目を選択します。 |
| ArrowDown / ArrowRight | Item | 次の項目へ移動し選択します（循環）。 |
| ArrowUp / ArrowLeft | Item | 前の項目へ移動し選択します（循環）。 |
| Home / End | Item | 先頭/末尾の項目へ移動し選択します。ark-ui / Radix Primitives の Keyboard 表には無い拡張ですが、WAI-ARIA APG のオプション挙動として提供します。 |

実 DOM 配線は `fandhe-frontend-wasm-full` の `keynav`（`radio_next_index`、orientation で軸制限・循環）が矢印キー/Home/End を担いますが、JS なしでも同一 `name` のネイティブ radio としてブラウザ標準操作（Tab/Space/矢印キー）が成立します。

**参考サイトとの差分**

ark-ui 公式 Data Attributes / Keyboard Support 表（zag `radio-group.connect.ts`）・Radix Primitives `radio-group` と突合し、上記のキーボード操作・`data-*`/`aria-*` 語彙・anatomy を是正しました。一方、以下は意図的に合わせていません。

- ark の `Indicator`（選択項目へ追従する浮動ビジュアル、位置 `style` 計算）はレイアウト計測の関心のため headless へ持ち込みません（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
- `data-orientation` は `root` のみへ付与し、子パーツへ伝播しません（`checkbox_group` と同判断。CSS は子孫セレクタで足ります）。
- `data-hover`/`data-active`/`data-focus`（ark）は SSR 静的出力に持たせません（`checkbox` と同じ契約。キーボード操作専用の `data-focus-visible` は `fandhe-frontend-wasm-full` が合成します）。
- `item-hidden-input` は `data-state` を維持します（ark は持ちませんが、`fandhe-frontend-wasm-full` の `keynav` が同期する既存契約のためです）。
- `readonly` は `root` へ `data-readonly` を出力しません（ark の Root Data Attributes 表に `data-readonly` は無いためです。表示契約としての `data-readonly` は `item`/`item-control`/`item-text` のみへ出力します）。

スタイル済みの表示例は [Radio Group](../themes/radio-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="radio-group"][data-part="..."]` セレクタでスタイルを当てます。以下は item-control の円形描画、checked/disabled/invalid の見た目、フォーカスリング、horizontal orientation のレイアウトの最小例です。

```css
[data-scope="radio-group"][data-part="item-control"] {
  display: inline-block;
  width: 1rem;
  height: 1rem;
  border-radius: 999px;
  border: 1px solid #ccc;
}

[data-scope="radio-group"][data-part="item-control"][data-state="checked"] {
  box-shadow: inset 0 0 0 3px #06c;
}

[data-scope="radio-group"][data-part="item-control"][data-disabled] {
  opacity: 0.5;
}

[data-scope="radio-group"][data-part="item-control"][data-invalid] {
  border-color: #d33;
}

[data-scope="radio-group"][data-part="item"]:focus-within {
  outline: 2px solid #06c;
  outline-offset: 2px;
}

[data-scope="radio-group"][data-part="root"][data-orientation="horizontal"] {
  display: flex;
  gap: 1rem;
}
```
