# Popover

トリガー起点のオーバーレイです。`fandhe-frontend-headless-ui` の
`popover` mod は Root / Trigger / Anchor / Positioner / Arrow / ArrowTip /
Content / Title / Description / CloseTrigger / Indicator の 11 anatomy
パーツを提供します。trigger は `aria-haspopup="dialog"`・`aria-expanded`・
（`controls` が指定されたときの）`aria-controls` を持ち、content は
`role="dialog"` を固定付与します。本部品は SSR での属性出力と状態機械の
みを担い、trigger/close-trigger の click → dispatch 配線・Escape キーでの
閉鎖・外側クリックでの閉鎖は `fandhe-frontend-wasm-full` が実 DOM 配線を
担います。`content` は `tabindex="-1"` を固定付与します（プログラム的
フォーカスのみを許可する WAI-ARIA dialog パターンの前提）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Enter / Space | Trigger | ポップオーバーを開きます（`fandhe-frontend-wasm-full` の headless part → action 対応表が `"toggle"` を dispatch）。 |
| Enter / Space | CloseTrigger | ポップオーバーを閉じます（同対応表が `"close"` を dispatch）。 |
| Escape | — | 最上位のポップオーバーを閉じます。`data-close-on-escape="false"` で無効化できます。 |
| Tab / Shift+Tab | — | フォーカストラップはありません（通常の文書順でフォーカスが移動します）。参考サイト（ark-ui/Radix）の「Esc 後に trigger へフォーカスを復帰する」挙動は未実装です。 |

**参考サイトとの差分**

ark-ui・Radix Primitives・chakra-ui と突合し、`content` の `tabindex="-1"`
固定付与を追加しました。anatomy（11 パート）・`data-*` 語彙の増減は
ありません。一方、以下は意図的に合わせていません。

- **Radix `Portal`**: DOM 配置の関心のため不採用です。
- **zag の `data-placement`**: `data-side`/`data-align`（`positioner` へ付与）が同役割を担うため置き換えていません。
- **zag の `data-expanded`（content）**: `data-state` + `aria-expanded` と重複するため不採用です。
- **zag の trigger `data-ownedby` / `data-value` / `data-current`**（複数トリガー識別）: `aria-controls` による id 関連付けが同等の役割を担うため不採用です（`dialog` と同判断）。
- **chakra-ui の Header / Body / Footer**: `fandhe-frontend-pre-styled-ui`（Themes 層）の関心のため headless anatomy には持ち込みません。
- **close-trigger の既定 `aria-label`**: アイコンボタン等の用途を想定し、内容を強制しない既存方針のため呼び出し側の責務のまま維持しています。
- **フォーカストラップ・開時の autoFocus・閉鎖時の trigger へのフォーカス復帰**: 参考サイトは Esc 後に trigger へフォーカスを復帰させますが、本リポジトリでは未実装です（`focus_trap::should_trap` は dialog scope のみを対象とします）。

スタイル済みの表示例は [Popover](../themes/popover.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="popover"][data-part="..."]`
セレクタでスタイルを当てます。author スタイルの `display` が UA の
`[hidden] { display: none }` を上書きして closed 状態が見えてしまわない
よう、`positioner`/`content` には `[hidden]` ガードを必ず含めます。

```css
[data-scope="popover"][data-part="root"] {
  position: relative;
  display: inline-block;
}

[data-scope="popover"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
}

[data-scope="popover"][data-part="positioner"][hidden],
[data-scope="popover"][data-part="content"][hidden] {
  display: none;
}

[data-scope="popover"][data-part="content"] {
  background: #fff;
  border: 1px solid rgb(0 0 0 / 0.1);
  border-radius: 8px;
  padding: 1rem;
  box-shadow: 0 0.25rem 1rem rgb(0 0 0 / 0.15);
}

[data-scope="popover"][data-part="arrow"] {
  position: absolute;
  width: 0.5rem;
  height: 0.5rem;
}
```
