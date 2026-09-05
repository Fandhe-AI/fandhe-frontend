# Combobox

テキスト入力しながら候補一覧を絞り込む部品です。フォーカスを保持する `input` パーツ（`role="combobox"`）が `aria-activedescendant` の配線先を担い、`role="listbox"` の候補一覧とセットで動作します。候補データの取得・整形は利用者側の責務であり、本部品は anatomy と ARIA 配線のみを提供します。候補件数の変化は `live_region` パーツ（`aria-live="polite"` 固定）が支援技術へ通知します（テキスト更新の実配線は wasm-full の後続責務）。

`ComboboxProps`（`disabled`/`readonly`/`invalid`/`required`）を root/label/control/input/trigger/clear_trigger へ一律付与します。`data-required` は label のみに付与され、`input` は対応するネイティブ `disabled`/`readonly`/`required` 存在属性と、invalid 時のみ `aria-invalid="true"` を追加で持ちます。`data-scope`/`data-part`/`data-state`/`data-disabled`/`data-invalid`/`data-highlighted` 等の属性セレクタで自前 CSS を当てられます（詳細は Themes 版ページの Examples を参照）。

`fandhe-frontend-headless-ui` の `combobox` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

スタイル済みの表示例は [Combobox](../themes/combobox.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
