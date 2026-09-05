# Signature Pad

`fandhe-frontend-headless-ui` の `signature_pad` mod が提供する手書き署名入力の unstyled 部品です。Root / Label（`label` 要素）/ Control / Segment（`svg`）/ SegmentPath / Guide / ClearTrigger / HiddenInput の 8 anatomy パーツを持ち、canvas を一切使わず、ストローク（座標列）を SVG path 文字列へ変換する決定的な純粋関数のみで描画します。ポインタイベントの収集自体は本部品の責務外です。

`/themes/` 側の `SignaturePad`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`root` は `disabled`/`empty`（strokes が空か）に応じて `data-disabled`/`data-empty` を付与します。`label`/`guide` は `data-disabled` を反映します（label のタグはネイティブ `<label>` 要素です）。`control` は `role="application"`・`aria-roledescription="signature pad"` を常時付与し、`disabled` に応じて非 disabled 時のみ `tabindex="0"`、disabled 時のみ `aria-disabled="true"` を切り替えます。`control` のアクセシブルネーム（`aria-label`/`aria-labelledby`）は偽の説明文を捏造しない fail-closed 方針により本モジュールが自動生成することはなく、呼び出し側が `attrs` で渡す契約です。`clear-trigger` はネイティブ `disabled` + `data-disabled` を反映します。`SignaturePad::control` メソッド経由でのみ `data-readonly` が付与されます（自由関数 `control` にはこの属性はありません）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | Control | `tabindex="0"`（非 disabled 時のみ）によりフォーカスを移します。 |
| Enter / Space | ClearTrigger | フォーカスがネイティブ `<button>` 上にあるとき、ブラウザ既定の活性化操作で全ストロークを削除します。 |

参考サイト（ark-ui docs / zag.js `signature-pad.connect.ts`）に Keyboard Interactions 表・`onKeyDown` ハンドラは存在しません。描画自体はポインタ専用のままで、`undo`（直前ストロークの取り消し）を含むアクションに既定のキー割り当てはありません。

**参考サイトとの差分**

ark-ui docs（Anatomy / Data Attributes 表）と zag.js（`signature-pad.connect.ts`/`signature-pad.anatomy.ts`）を一次情報として突合し、`label` のタグ（`div` → `label`）・`data-disabled`（label/guide）・`control` の `role`/`aria-roledescription`/`tabindex`/`aria-disabled` を是正しました。一方、以下は意図的に合わせていません。

- `label` の `data-required`・`for`（hidden input への関連付け）: 本モジュールに「必須」概念が無く、`for` は呼び出し側 `attrs` に委ねます。
- `control` のアクセシブルネーム自動生成: 偽の説明文を捏造しない fail-closed 方針により、呼び出し側が `aria-label`/`aria-labelledby` を渡す契約のままとします。
- `clear-trigger` のネイティブ `hidden`（ストローク空 or 描画中）: author CSS（`display`）との干渉を避けるため持ち込まず、`root[data-empty]` を使う自前 CSS（下記参照）で代替します。
- `data-empty`（root）・`data-readonly`（control）・`segment` の `role="img"`: 本フレームワーク固有の拡張として維持します。
- hidden-input の `type="hidden"`（参照は `type="text" hidden readOnly required`）: フォーム送信用途として `type="hidden"` のまま維持します。

スタイル済みの表示例は [Signature Pad](../themes/signature-pad.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="signature-pad"][data-part="..."]` セレクタでスタイルを当てます。以下は control の枠線とフォーカスリング、segment-path の線描画、guide のベースライン、`root[data-empty]` による clear-trigger の非表示（参照の `hidden` 相当）の最小例です。

```css
[data-scope="signature-pad"][data-part="control"] {
  display: inline-block;
  border: 1px solid #ccc;
  border-radius: 6px;
}

[data-scope="signature-pad"][data-part="control"]:focus-visible {
  outline: 2px solid #06c;
  outline-offset: 2px;
}

[data-scope="signature-pad"][data-part="control"][data-disabled] {
  opacity: 0.5;
}

[data-scope="signature-pad"][data-part="segment-path"] {
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
}

[data-scope="signature-pad"][data-part="guide"] {
  border-bottom: 1px dashed #999;
}

[data-scope="signature-pad"][data-part="root"][data-empty] [data-part="clear-trigger"] {
  display: none;
}
```
