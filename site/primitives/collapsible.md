# Collapsible

単一の開閉パネルです。`fandhe-frontend-headless-ui` の `collapsible` mod は
Root / Trigger / Indicator / Content の 4 anatomy パーツと、開閉状態を表す
`data-state`・`aria-expanded`・`aria-controls` を提供します。開閉状態の
遷移は `open`/`close`/`toggle` の dispatch で行い、closed のときは
`hidden` 存在属性を付与して JS なしの SSR でも閉状態を表現します。
`indicator`/`content` は `disabled` 引数を受け取り、`data-disabled` を
root/trigger と同じく 4 パートすべてへ反映します（ネイティブ `disabled`
存在属性は `button` にのみ付与し、`span`/`div` には付与しません）。
`fandhe-frontend-pre-styled-ui` にはまだ対応するスタイル済み部品がありません。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Space / Enter | Trigger | ネイティブ `<button type="button">` のため、フォーカス時の Space/Enter によるクリック相当の発火はブラウザ標準操作として成立します。クリックから開閉切替への dispatch 配線（`"toggle"`）は `fandhe-frontend-wasm-full` の責務です。 |
| Tab | Trigger | trigger のみがタブ順に含まれます。closed の content は `hidden` 存在属性によりタブ順・支援技術双方から除外されます。 |

**参考サイトとの差分**

ark-ui・Radix Primitives の Collapsible と突合し、`content`/`indicator` へ
の `data-disabled` 追加・キーボード操作の明示を是正しました（イシュー
#1637）。一方、以下は意図的に合わせていません。

- `content` の `data-collapsible` 存在属性（ark-ui）は採用しません。本
  クレートは `data-scope="collapsible" data-part="content"` を常に出力する
  ため、同じ情報を重複して持たせるだけで状態値でもないからです。
- サイズ計測・アニメーション系（ark-ui の `data-has-collapsed-size`・
  `--height`/`--width` 等の CSS 変数、Radix の
  `--radix-collapsible-content-*`）は headless-ui へ持ち込みません。
  レイアウト計測・アニメーションの関心であり
  （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）、必要なら
  スタイル済み部品（Themes 側、`fandhe-frontend-pre-styled-ui`）の責務です。
- `root` の `data-disabled` は維持します。ark-ui の Root には無い属性です
  が、Radix Primitives の Root には存在し、`fandhe-frontend-wasm-full` の
  祖先 root disabled 判定が既存契約として依存しているためです。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版が無いため、本部品を直接使う場合は `[data-scope="collapsible"][data-part="..."]` セレクタでスタイルを当てます。以下は indicator の回転アニメーション、content の折り畳み、disabled の見た目、フォーカスリングの最小例です（開閉アニメーション用の高さ計測は含みません。Themes 側の責務として #1670 で計画中です）。

```css
[data-scope="collapsible"][data-part="trigger"] {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 0.375rem 0.75rem;
  background: none;
  cursor: pointer;
}

[data-scope="collapsible"][data-part="trigger"]:focus-visible {
  outline: 2px solid #06c;
  outline-offset: 2px;
}

[data-scope="collapsible"][data-part="indicator"] {
  transition: transform 0.15s ease;
}

[data-scope="collapsible"][data-part="indicator"][data-state="open"] {
  transform: rotate(180deg);
}

[data-scope="collapsible"][data-part="content"][data-state="closed"] {
  display: none;
}

[data-scope="collapsible"][data-part="content"][data-state="open"] {
  padding: 0.5rem 0;
}

[data-scope="collapsible"] [data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}
```
