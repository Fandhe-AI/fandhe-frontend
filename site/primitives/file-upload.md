# File Upload

ドロップゾーン・ボタン起点のファイル選択・受理済みファイル一覧を持つアップロード部品です。ファイルメタデータ（ファイル名・サイズ・MIME タイプ）のみを扱い、`File` オブジェクト自体や実際のアップロード処理は保持しません（accept/max-files 等の検証は決定的な純粋関数で行いますが、実送信・保存は利用者側の責務です）。

`fandhe-frontend-headless-ui` の `file_upload` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

`FileUploadProps`（`disabled`/`readonly`/`invalid`/`required`）が Root/Label/Dropzone/Trigger/ClearTrigger/HiddenInput へ `data-disabled`/`data-readonly`/`data-invalid` を一律付与し、Label のみ追加で `data-required` を付与します。Dropzone は disabled または readonly のとき `tabindex="-1"` + `aria-disabled="true"`、それ以外は `tabindex="0"` になり、呼び出し側 `attrs` に `aria-label`/`aria-labelledby` が無ければ既定 `aria-label="dropzone"` を付与します。Trigger/ItemDeleteTrigger/ClearTrigger/HiddenInput は readonly でもネイティブ `disabled` を付与します（zag `disabled: disabled || readOnly` と同値の判断）。HiddenInput には `tabindex="-1"`・`aria-hidden="true"`・`required`（`props.required`）を追加します。ClearTrigger は `hidden` 引数（受理済みファイル 0 件を表す）で `hidden` 属性を出力します。ItemGroup/Item/ItemName/ItemSizeText/ItemDeleteTrigger には `ItemType`（`Accepted`/`Rejected`）固定語彙による `data-type` を付与します。Item への `data-invalid` は参照側（zag/ark）も出さないため付与しません（呼び出し側 `attrs` 経由でのみ有効化できます）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Space / Enter | Trigger / ItemDeleteTrigger / ClearTrigger | いずれもネイティブ `<button>` であるため、ブラウザ標準の活性化操作が働きます |
| Space / Enter | Dropzone | 参照実装（zag）ではこのキー操作でファイル選択ダイアログを起動します。本部品は `role="button"` + `tabindex="0"`（disabled/readonly 時は `tabindex="-1"`）の SSR マークアップのみを提供し、`fandhe-frontend-wasm-full` 側の keydown 配線は未実装です（スコープ外、フォローアップ Issue 提案）。キーボード専用利用者は Trigger（ネイティブ `<button>`）で操作できるため a11y 上のブロッカーではありません |

ドラッグ&ドロップの実際の DOM 配線（`fandhe-frontend-wasm-full`）は disabled/readonly いずれの状態でも新規ファイルの追加操作を無視します。

スタイル済みの表示例は [File Upload](../themes/file-upload.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="file-upload"][data-part="..."]` セレクタでスタイルを当てます。以下はドラッグ中の強調表示・disabled trigger・rejected item・required label の最小例です。

```css
[data-scope="file-upload"][data-part="dropzone"][data-dragging] {
  border-color: dodgerblue;
}

[data-scope="file-upload"][data-part="trigger"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="file-upload"][data-part="item"][data-type="rejected"] {
  color: crimson;
}

[data-scope="file-upload"][data-part="label"][data-required]::after {
  content: " *";
}
```
