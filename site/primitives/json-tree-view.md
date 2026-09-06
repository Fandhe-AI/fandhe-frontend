# JSON Tree View

JSON 風データ構造をツリー表示する部品です。`fandhe-frontend-headless-ui` の `json_tree_view` mod は `tree_view`（イシュー #753）の 12 anatomy パーツ・状態機械をそのまま再利用しつつ、`key`/`colon`/`value` の 3 パーツと決定的な変換ロジックのみを追加します。Themes 版と異なりインデント・展開アイコンのスタイルを持たず、構造とアクセシビリティ属性のみを担います。

スタイル済みの表示例は [JSON Tree View](../themes/json-tree-view.md) を参照してください。

**アクセシビリティ・参考サイトとの対応**

- ark-ui/zag（`@zag-js/json-tree-utils`）と突合した結果です（イシュー #1661）。chakra-ui・Radix には対応部品がありません。
- `key`（オブジェクトキー/配列 index）・`colon`（区切り、固定テキスト `": "`）・`value`（値、`data-kind` で型別に表示）は `branch-text`/`item-text` の内側へこの順で入れ子にします（ark-ui の `KeyNode`/`ValueNode` が `BranchText`/`ItemText` を包む構造に合わせています）。`colon` はキーを持つノードにのみ出力し、ルート自身には出ません。
- `value` の `data-kind` は `"null"`/`"boolean"`/`"number"`/`"string"`/`"array"`/`"object"` の 6 語彙です（イシュー #1661 で `"bool"` から `"boolean"` へ変更、破壊的変更）。
- キーボード操作は構造部（`tree-view` スコープ）が `fandhe-frontend-wasm-full` の TreeView 配線（イシュー #1072）をそのまま継承します（矢印キー・Home/End・Enter/Space・typeahead）。`*`（兄弟一括展開）は未実装の既知ギャップです。単一選択のみのため Shift+Arrow/Ctrl+A は対象外です。
- 参照実装が持つ `aria-label`（アクセシブル説明文）・`data-line`/`--line-length`（行番号・インデント計測）・`data-root`/`data-non-enumerable`/`quotesOnKeys`（表示オプション）はいずれも意図的に採用していません。展開可否・子要素数は既存の `aria-expanded`/`aria-setsize` で表現済みで、行番号・インデント計測は装飾・レイアウトの関心のため UI コンポーネント層へ持ち込みません。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
