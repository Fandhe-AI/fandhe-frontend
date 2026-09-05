# Tags Input

`fandhe-frontend-headless-ui` の `tags_input` mod が提供するタグ配列入力の unstyled 部品です。Root / Label / Control / Input / Item / ItemPreview / ItemText / ItemInput / ItemDeleteTrigger / ClearTrigger / HiddenInput / LiveRegion の 12 anatomy パーツを持ち、タグ文字列の可変長リスト + 重複拒否 + 上限 + 編集中/キーボード強調インデックスを表現する値状態機械を備えます。重複・カンマ含有・空文字列のタグはすべての入口で拒否されます。LiveRegion はタグ数の変化を `aria-live="polite"` で支援技術へ通知します（テキスト更新の実配線は wasm-full の後続責務）。

`/themes/` 側の `TagsInput`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`TagsInputProps`（`disabled`/`readonly`/`invalid`/`required`）は root/label/control/input/clear-trigger/hidden-input へ `data-disabled`/`data-invalid`/`data-readonly` を、`label` にはさらに `data-required` を反映します。`invalid: true` のとき `input` へ `aria-invalid="true"`（valid のときは属性自体を省略）、`readonly: true` のとき `input` へネイティブ `readonly` 存在属性を付与します。タグ 1 個分の状態は `TagItem`（`value`/`disabled`/`editing`/`highlighted`）で表し、`item`/`item_preview` に `data-value`（タグ文字列）、`item_preview`/`item_text`/`item_delete_trigger` に `data-highlighted` を反映します。編集モードでは `item_preview` が `hidden`、`item_input` が表示されます。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| ArrowLeft | Control | キーボード強調（highlight）を 1 つ前へ移動します（dispatch: `"highlight-prev"`）。強調なしなら末尾タグへ、先頭タグでそれ以上前進しません。 |
| ArrowRight | Control | 強調を 1 つ後ろへ移動します（dispatch: `"highlight-next"`）。末尾タグ強調中は強調解除（入力欄へ戻る）。強調なしは no-op です。 |
| Backspace | Input/Control | 強調中ならそのタグを削除、なければ末尾タグを削除します（dispatch: `"backspace"`）。 |
| Delete | Control | 強調中のタグを削除します（dispatch: `"delete-highlighted"`）。 |
| Enter | Control/ItemPreview | 強調中なら編集開始（`"edit-start"`）、入力欄にフォーカスがあれば入力中の値を追加（`"add"`）します。振り分けはクライアント側の責務です。 |
| Escape | Control | 強調を解除します（dispatch: `"highlight-clear"`）、または編集中なら編集を破棄します（`"edit-cancel"`）。 |

**上記キーの実際の DOM 配線（`keydown`/`paste` イベントハンドラ、`fandhe-frontend-wasm-full`）は本部品のスコープ外です。本部品は SSR 静的マークアップと dispatch 契約のみを提供し、キー入力を検出して自動的に dispatch する配線は未実装です。**

**参考サイトとの差分**

Zag.js `tags-input.connect.ts`（ark-ui 基盤）・ark-ui 公式 Data Attributes / Keyboard Support 表と突合し、`data-*`/`TagsInputProps`/`TagItem`・上記のキーボード操作の dispatch 契約へ揃えました。一方、以下は意図的に合わせていません。

- **`role="listbox"`（control）/`role="option"`（item_preview）は撤去しました**。zag/ark の control・item-preview はいずれも `role` を持たず、`role="listbox"` の許容子は `option`/`group` のみだが `control` は `<input type="text">` を内包しており content model 違反だったため是正しました。アクセシブルネームは `label` の `for` 関連付けが担います。
- `data-focus`（root/control、DOM フォーカスという ephemeral な状態）は SSR では決定できないため不採用です。
- `data-empty`（root/input のタグ 0 件）は自由関数 `root` がタグ列を保持しないため未実装です（別 issue 候補）。
- `readonly` の DOM 表現は zag（`control` の `tabindex="0"` 維持 + `input` へネイティブ `disabled`）と異なり、`input` へネイティブ `readonly` を採用します（`pin-input`/`password-input`/`file-upload` の直近精査と同じ規約へのリポ内一貫性を優先した判断）。
- `maxLength`（タグ文字数上限）・`validate` コールバック・`delimiter`/`addOnPaste`/`blurBehavior`/`allowOverflow`/`autoFocus`/`dir` はアプリロジック・クライアント配線の関心のため対象外です（`docs/policy/intentional-non-adoption.md` §3.25）。

スタイル済みの表示例は [Tags Input](../themes/tags-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="tags-input"][data-part="..."]` セレクタでスタイルを当てます。以下は control の枠線・折り返し、強調タグの outline、invalid 時の枠線、readonly 時の背景の最小例です。

```css
[data-scope="tags-input"][data-part="control"] {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  border: 1px solid #ccc;
  padding: 0.25rem;
}

[data-scope="tags-input"][data-part="item-preview"][data-highlighted] {
  outline: 2px solid #2563eb;
}

[data-scope="tags-input"][data-part="input"][data-invalid] {
  border-color: #dc2626;
}

[data-scope="tags-input"][data-part="control"][data-readonly] {
  background: #f3f4f6;
}
```
