# Date Picker

トリガー起点で開閉する日付選択オーバーレイの unstyled 部品です。開閉・配置の基盤は `crate::popover` と共通で、`content` の内部に Calendar のパーツ群を合成して月表示・選択 UI を組み立てる想定です。

`DatePickerProps`（`disabled`/`readonly`/`invalid`/`required`）を root/label/control/input/trigger/clear_trigger へ一律付与します。`data-required` は label のみに付与され、`input` は対応するネイティブ `disabled`/`readonly`/`required` 存在属性と、invalid 時のみ `aria-invalid="true"` を追加で持ちます。`label` は `for_` 引数（ark `htmlFor` 準拠）でネイティブ `label[for]` 関連付けも成立させます。`data-scope`/`data-part`/`data-state`/`data-disabled`/`data-invalid`/`data-readonly`/`data-required` 等の属性セレクタで自前 CSS を当てられます（例: `[data-scope="date-picker"][data-part="input"][data-invalid]`）。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Date Picker](../themes/date-picker.md) を参照してください。

フォーカストラップ・Escape での閉鎖・外側クリックでの閉鎖・実際の日付整形・フォーム送信処理は本部品の責務外であり、クライアントランタイムまたは利用側が担います。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
