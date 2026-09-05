# Toggle Tip

クリック開閉の小型ヒントです。`fandhe-frontend-headless-ui` の
`toggle_tip` mod は Root / Trigger / Positioner / Content / Arrow /
ArrowTip の 6 anatomy パーツを提供します。見た目は Tooltip（小型・
非モーダル）、挙動は Popover（クリックで開閉し明示的に閉じるまで持続）の
変種と位置づけられ、trigger は `aria-expanded` を持ちますが
`aria-haspopup` は付与せず、content も `role="tooltip"` を持ちません
（Tooltip・Popover いずれとも異なる 3 者境界）。本部品は SSR での属性
出力と状態機械のみを担い、click → dispatch・Escape 閉鎖・外側クリック
閉鎖・placement 計算は `fandhe-frontend-wasm-full` 側の責務ですが、
`"toggle-tip"` scope は未登録のため現時点では未配線です。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | Trigger | trigger はネイティブ `button` 要素のため、ブラウザ標準でフォーカス到達します（`disabled` 時は除外）。 |
| Enter / Space | Trigger | click は発火しますが、`fandhe-frontend-wasm-full` の `MAPPING_TABLE` に `toggle-tip` scope の行が無いため開閉 dispatch は未配線です。 |
| Escape | — | 参考サイト（chakra-ui、既定 `closeOnEscape: true`）では最上位のツールチップを閉じますが、本リポジトリでは未配線です。 |

**参考サイトとの差分**

ark-ui に Toggle Tip コンポーネントは存在せず（`ark-ui.com/docs/components/toggle-tip`
は 2026-09-06 時点で HTTP 404）、Radix Primitives / Radix Themes にも
該当部品はありません。唯一の直接参照は chakra-ui ToggleTip（Ark
`Popover` を内包したラッパー）で、これと突合した結果、**是正すべき
具体的な欠落は見つかりませんでした**（イシュー #1644）。anatomy（6
パート）・`data-state`/`data-disabled` 語彙はいずれも一致しています。
一方、以下は意図的に合わせていません。

- **`aria-haspopup="dialog"` / `role="dialog"` / `tabindex="-1"`**:
  chakra-ui ToggleTip は Ark Popover 基盤のためこれらを持ちますが、本
  部品の content は非対話の短文テキストであり `dialog` ロールは不適合と
  判断し、非付与を維持しています。`tabindex="-1"` は
  `fandhe-frontend-wasm-full` 配線後に再評価します。
- **Zag/chakra の `data-placement`**: `positioner` の `data-side`/
  `data-align`（`positioning`、#590）が同役割を担うため置き換えていません。
- **Zag の `data-expanded`（content）**: `data-state` + `aria-expanded` と
  重複するため不採用です。
- **複数トリガー識別（`data-value`/`data-current`/`data-ownedby`）**:
  機能拡張でありスコープ外候補です（下記参照）。
- **`Portal`**: DOM 配置の関心のため headless anatomy には持ち込みません。
- **chakra-ui の Sizes / InfoTip**: `fandhe-frontend-pre-styled-ui`
  （Themes 層）の関心のため headless anatomy には持ち込みません。

上記の差分に伴う `data-*`・パートの増減は無いため、Themes 側イシュー
#1546（closed）への追加コメントは行っていません。

スコープ外候補（是正対象ではなく、別途起票を検討する余地がある事項）:

- `fandhe-frontend-wasm-full` への `"toggle-tip"` scope 登録
  （`headless::MAPPING_TABLE` の trigger→toggle 対応・`overlay::OverlayKind`・
  `position::PositionedKind` への追加。click → dispatch・Escape・外側
  クリック閉鎖・placement 計算の配線）
- 配線後の content `tabindex="-1"` 付与の再評価
- `docs/api/headless-ui-api.md` §4a.1 表の ToggleTip 行が
  `PositionedKind` 未登録の実装と整合していない点（hover-card と同状態）

スタイル済みの表示例は [Toggle Tip](../themes/toggle-tip.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、
`[data-scope="toggle-tip"][data-part="..."]` セレクタでスタイルを当てます。
author スタイルの `display` が UA の `[hidden] { display: none }` を
上書きして closed 状態が見えてしまわないよう、`positioner`/`content`
には `[hidden]` ガードを必ず含めます。

```css
[data-scope="toggle-tip"][data-part="root"] {
  position: relative;
  display: inline-block;
}

[data-scope="toggle-tip"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
}

[data-scope="toggle-tip"][data-part="positioner"][hidden],
[data-scope="toggle-tip"][data-part="content"][hidden] {
  display: none;
}

[data-scope="toggle-tip"][data-part="content"] {
  background: #fff;
  border: 1px solid rgb(0 0 0 / 0.1);
  border-radius: 6px;
  padding: 0.5rem 0.75rem;
  box-shadow: 0 0.25rem 1rem rgb(0 0 0 / 0.15);
}

[data-scope="toggle-tip"][data-part="trigger"][data-state="open"] {
  background: #eef2ff;
}

[data-scope="toggle-tip"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toggle-tip"][data-part="arrow"] {
  position: absolute;
  width: 0.5rem;
  height: 0.5rem;
}
```
