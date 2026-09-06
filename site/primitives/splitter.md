# Splitter

リサイズ可能なパネル分割レイアウトです。`fandhe-frontend-headless-ui` の `splitter` mod は Root / Panel / ResizeTrigger / ResizeTriggerIndicator の 4 anatomy パーツと、`role="separator"` + `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`aria-orientation`/`aria-controls` を提供する決定的なパネルサイズ状態機械です。Themes 版と異なりドラッグ操作の視覚表現を持たず、構造・ARIA 属性・数値正規化ロジックのみを担います。

スタイル済みの表示例は [Splitter](../themes/splitter.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Arrow（軸別、Horizontal: Left/Right、Vertical: Up/Down） | ResizeTrigger | `SplitterAction::Increment`/`Decrement`（ステップ 1%）として状態遷移し、`aria-valuenow` が追随します。DOM keydown 配線はイシュー #1074 で `fandhe-frontend-wasm-full` の `splitter` モジュールが実装済みです。 |
| Home / End | ResizeTrigger | `SplitterAction::SetToMin`/`SetToMax`（先行パネルを `min`/`max` へ設定）として状態遷移します。実装済みです。 |
| Shift+Arrow | ResizeTrigger | `SplitterAction::IncrementLarge`/`DecrementLarge`（ステップ 10%、zag.js `keyboardResizeBy` 既定値相当）として状態機械のみ提供します。**`fandhe-frontend-wasm-full` の Shift 修飾キー配線は現状存在せず未実装です**（イシュー #1664 時点で確認済み、別 Issue の起票対象）。 |
| Enter | ResizeTrigger | collapse/expand トグル相当の対応はありません（意図的非採用）。 |
| F6 | Root | トリガー間フォーカス循環の対応はありません（意図的非採用、DOM フォーカス管理は `fandhe-frontend-wasm-full` の責務）。 |

**参考サイトとの差分**

イシュー #1664 で ark-ui docs・zag.js（`packages/machines/splitter/src/splitter.connect.ts`）・WAI-ARIA APG Window Splitter パターンと突合しました（Radix Primitives に Splitter 相当は存在しません）。

- **panel の `data-index`/`data-id` 欠落を是正**: ark-ui docs の `data-*` 表に従い、`panel` へパネル序数 `index: usize`（`data-index`）と `id` の写し（`data-id`）を追加しました（破壊的変更、`panel` の引数順変更）。
- **resize-trigger の `data-id` 欠落・`aria-controls` が先行パネルのみだったのを是正**: `aria-controls` を zag.js の実出力（隣接 2 パネルの id を空白区切りで列挙）に合わせて拡張し、`data-id` を `"<leading>:<trailing>"` 形式で追加しました。`resize_trigger` の `controls: &str` 引数を `leading_id: &str, trailing_id: &str` へ置換しています（破壊的変更）。
- **Shift+Arrow（`keyboardResizeBy` 既定 ×10）の状態機械を追加**: `SplitterAction::IncrementLarge`/`DecrementLarge`（dispatch 名 `"increment_large"`/`"decrement_large"`）を追加しました。DOM 配線は上記キーボード操作表のとおり未実装です。
- **予約キーなりすまし除去を追加**: 呼び出し側 `attrs` が `role`/`aria-*`/`tabindex`/`data-*`/`id` を偽装・重複出力できないよう `drop_reserved` を導入しました（`slider`/`toolbar` 等と同型のパターン）。`aria-label`/`aria-labelledby` は zag.js が固定付与しない拡張点のため予約対象に含めません。

一方で以下は意図的に参考サイトと合わせていません。

- **`data-focus`/`data-dragging`（root/panel/resize-trigger）**: focus・pointer ドラッグの DOM ローカル状態です。headless 層は pointer 配線を持たないため実データがなく、`docs/policy/intentional-non-adoption.md` §3.25 規則 2（装飾・ポインタ計測は headless 層へ持ち込まない）に従い非採用です。
- **resize-trigger-indicator への `data-orientation` 追加**: ark-ui docs の Anatomy にこの行が無いため追加しません。
- **`aria-orientation` の向き**: WAI-ARIA APG Window Splitter パターンの記述（左右分割のセパレータは vertical）に従いパネルレイアウトと逆向きを維持します。zag.js の実出力（非反転）とは非同値のままです。
- **disabled 時の `tabindex="-1"` + `aria-disabled`**: zag.js は disabled 時にこれらを出力しませんが、本実装はリポジトリ横断規約（disabled でも属性で明示する）の superset として維持します。
- **Enter（collapse/expand）・F6（フォーカス循環）**: 上記キーボード操作表のとおり非採用です。
- **`dir`**: 本リポジトリ横断で未採用です。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="splitter"][data-part="..."]` セレクタと `data-orientation`/`data-disabled` で見た目を組み立てます。`resize-trigger` にはアクセシブルネームが自動付与されないため、呼び出し側から `aria-label`（`attrs` 経由）を渡してください。

```css
[data-scope="splitter"][data-part="root"] {
  display: flex;
}

[data-scope="splitter"][data-part="root"][data-orientation="vertical"] {
  flex-direction: column;
}

[data-scope="splitter"][data-part="panel"] {
  flex: 1 1 0;
  overflow: hidden;
}

[data-scope="splitter"][data-part="resize-trigger"] {
  flex: 0 0 4px;
  cursor: col-resize;
}

[data-scope="splitter"][data-part="root"][data-orientation="vertical"] [data-part="resize-trigger"] {
  cursor: row-resize;
}

[data-scope="splitter"] [data-disabled] {
  opacity: 0.5;
}

[data-scope="splitter"] :focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}
```
