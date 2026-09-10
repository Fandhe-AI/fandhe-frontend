# Questionnaire

単一選択・複数選択・自由記述・スキップ可の質問を多段で提示する shadcn/ui Questionnaire 相当の部品です（参照軸はイシュー #2001）。`fandhe-frontend-headless-ui` の `questionnaire` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、Root / Progress / Question / Prompt / Description / Options / Freeform / Actions / Back / Next / Skip の 11 anatomy パーツと、`count`（全質問数）・`step`（現在位置）から質問の表示状態を導出する状態機械 `Questionnaire` を持ちます。

**責務境界**

回答値の保持・検証（必須判定）・分岐（次にどの質問へ進むか）・送信はアプリケーション側の責務であり、本部品は持ちません。部品が担うのは「現在位置に応じて質問の表示状態を切り替え、前へ / 次へ / スキップのトリガーを出す」までです。`next` / `skip` の `data-disabled` は、必須判定の**結果（真偽）だけ**をアプリから引数として受け取ります。

**状態モデル**

`question` の `index`（0-origin）と `step` の比較で 3 状態（`data-state`）を導出します。

| 条件 | `data-state` |
|---|---|
| `index < step` | `completed` |
| `index == step` | `active` |
| `index > step` | `upcoming` |

`active` 以外の質問には `hidden` 属性が付き、タブ操作・支援技術の双方から除外されます。`step == count` は「全質問完了」を表し、`root` に `data-complete` が付き、`next` / `skip` は無条件で無効化されます。

`Skip` は状態遷移としては `Next` と同一（`min(step + 1, count)` へ進む）です。「どの質問をスキップしたか」は本部品に保持せず、`QuestionProps::skipped` として呼び出し側が渡します。

**QuestionProps**

`question(index, props, attrs, children)` の `props`（`QuestionProps`）は 4 つの存在属性フラグを持ち、いずれも既定 `false` です。

| フィールド | 出力 |
|---|---|
| `answered` | `data-answered` |
| `skipped` | `data-skipped` |
| `required` | `data-required` |
| `invalid` | `data-invalid` + `aria-invalid="true"`（`invalid` が `true` のときのみ） |

**Options / Freeform スロット**

`options` / `freeform` は純スロットです。選択肢は `options` へ [Radio Group](./radio-group.md) / Checkbox Group のパーツを、自由記述は `freeform` へ [Field](./field.md) の `textarea` パーツを入れ子にする契約とします。両者の `data-scope` は questionnaire scope と独立して残ります（回答ロジック自体は入れ子にした部品・アプリ側が担います）。

**Progress**

`progress(label, attrs, children)` は `role="progressbar"` を持ち、`aria-valuemin` / `aria-valuemax` / `aria-valuenow` / `aria-valuetext`（`{percent}% complete`）を出力します。`label` が空文字でないときのみ `aria-label` を付与します。

**アクセシビリティ**

- `question` は `fieldset` 要素です。`prompt`（`legend`）を先頭子に置く契約とし、ネイティブなグループ名が id 配管なしで付きます。
- `back` / `next` / `skip` はネイティブ `button`（`type="button"`）です。境界（`step == 0` / `step == count`）または呼び出し側の判定結果で `disabled` + `data-disabled` を出力します。
- キーボード操作は Tab / Shift+Tab（フォーカス移動）・Enter / Space（ボタン押下）のみで、矢印キー等の独自ハンドリングは持ちません。options / freeform のキー操作は入れ子にした部品のものを継承します。

back / next / skip の click から dispatch（`"prev"` / `"next"` / `"skip"`）への実際の DOM 配線（`fandhe-frontend-wasm-full`）は本部品のスコープ外です。本部品は SSR 静的マークアップと dispatch 契約のみを提供します。

**参考サイトとの差分**

shadcn/ui の `Questionnaire` は回答状態管理・バリデーション・キーボードショートカット・失敗時のフォーカス移動を内包しますが、本部品はこれらを持ちません。

- `QuestionnaireError` / `QuestionnaireSubmit`: 非採用です。検証結果の表示は呼び出し側が Field のエラー表示パーツを question 内へ入れ子にし、送信ボタンは `actions` スロットへ通常の `button` を置きます。
- `QuestionnaireChoice` / `QuestionnaireInput` / 英数字キーによるショートカット / `required` / `multiple` の回答ロジック: 非採用です。選択肢は Radio Group / Checkbox Group、自由記述は Field の `textarea` の再利用で賄います。
- `items` 配列からの一括描画・回答状態管理・バリデーション・失敗時のフォーカス移動: 非採用です（アプリケーションロジック・クライアント DOM 操作の関心）。

スタイル済みの表示例（recipe・golden テスト・Themes ページ）は現時点では未実装です（後続イシュー #2119）。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

```css
[data-scope="questionnaire"][data-part="question"] {
  display: block;
}
[data-scope="questionnaire"][data-part="question"][hidden] {
  display: none;
}
[data-scope="questionnaire"][data-part="question"][data-invalid] {
  border-left: 2px solid #b91c1c;
  padding-left: 0.75rem;
}
[data-scope="questionnaire"][data-part="next"][data-disabled],
[data-scope="questionnaire"][data-part="skip"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}
[data-scope="questionnaire"][data-part="progress"] {
  height: 0.25rem;
  background: #e5e7eb;
}
[data-scope="questionnaire"][data-part="progress"][data-complete] {
  background: #16a34a;
}
```
