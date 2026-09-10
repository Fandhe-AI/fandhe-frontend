# Forms 家族の `label[data-required]` 視覚化と `control`/`clear-trigger` の `data-disabled` 横断規則 決定記録

- **イシュー**: [#2195](https://github.com/Fandhe-AI/fandhe-frontend/issues/2195)
- **対象**: `crates/pre-styled-ui/src/{date_picker,combobox,select,color_picker,number_input,rating_group,date_input}.rs`
- **関連**: [#1470](https://github.com/Fandhe-AI/fandhe-frontend/issues/1470)（date-picker chakra-ui/ark-ui 基準調整の親、Themes 側判断を本イシューへ委ねた積み残し元）・[#1471](https://github.com/Fandhe-AI/fandhe-frontend/issues/1471)（分割 1/3、`control`/`clear-trigger` への `data-disabled` 視覚追加を「headless 非出力」を理由に見送った時点の記録）・[#2013](https://github.com/Fandhe-AI/fandhe-frontend/issues/2013)（PR #2177、date-picker の shadcn/ui 突合。`label` の `data-required` 視覚化・`control`/`clear-trigger` の `data-disabled` を再度対象外としてスコープを本イシューへ委ねた）・`docs/design/pre-styled-ui-interaction-visual-language.md`（hover/disabled/transition の共通ビジュアル言語、§5 手順 3 へ本イシューが補足を追加）

## 1. 背景・目的

date-picker の shadcn/ui 突合（#2013 / PR #2177）で、Themes 側の 2 点が「Forms 家族横断の軸判断が必要」として対象外に送られた。

1. `label` に headless が出す `data-required` を Themes でどう見せるか
2. `control` / `clear-trigger` に headless が出す `data-disabled` を Themes で消費するか

(2) は #1471（親 #1470）が「headless が当時 `data-disabled` を出さない」ことを理由に見送り、その後 headless-ui 0.41.0（#1627）で出力されるようになったまま Themes 側が未消費の積み残しだった。本イシューはこの 2 軸を Forms 家族横断の規則として確定し、規則に従って Themes recipe を是正する。

## 2. 実装前に確認した事実

- headless は `control` / `clear-trigger` の `data-disabled` を以下で出力済み: date-picker / combobox / select / color-picker / number-input / date-input（いずれも `state_attrs(props)` または `data_disabled(flags.disabled)` 経由）。rating-group も `control` へ出力済み
- headless が出さない 2 箇所（意図的な設計、本イシューでは変更しない）: `pin_input::control`（rustdoc「状態を持たない最小主義」）・`editable::control`（`data-state` のみ）
- `label` の `data-required` は date-picker / combobox / select / color-picker / number-input / rating-group（および `field`/`checkbox`/`switch`/`radio_group` 系）で既に出力済み。date-input は `label` に `data-required` を出さない
- Themes 側の消費状況（着手前）: `control[data-disabled]` を消費するのは tags-input / password-input / signature-pad のみ、`clear-trigger[data-disabled]` は tags-input / file-upload / signature-pad のみ。`label[data-required]` を CSS 消費する recipe は 0 件
- `crates/pre-styled-ui/src/combobox.rs` の「headless は `control`/`clear-trigger` へ `data-disabled` を出さない」コメントは陳腐化していた（headless-ui 0.41.0 以降は出力する）

## 3. 決定事項

### R1: `data-disabled` の「opacity 単一階層」規則（Forms 家族共通）

`disabled_declarations()`（opacity 0.5 + cursor）は部品ごとに **ちょうど 1 階層の slot（opacity 所有 slot）** にのみ適用する。他階層は `cursor: not-allowed` のみとする。既存の tags-input（`control`/`item`）・file-upload・signature-pad の判断の一般化である。

| 所有型 | 該当部品 | opacity 所有 slot | 追加した `control`/`clear-trigger` |
|---|---|---|---|
| コンテナ所有型 | number-input・date-input | `root` | `control`: cursor のみ |
| 葉所有型 | date-picker・combobox・select | `input`/`trigger` | `control`: cursor のみ、`clear-trigger`: `disabled_declarations()` |
| 葉所有型（trigger 単独） | color-picker | `trigger` | `control`: cursor のみ（`clear-trigger` パーツなし） |
| 葉所有型（item） | rating-group | `item` | `control`: cursor のみ |
| コンテナ所有型（既存） | tags-input・signature-pad | `root` | 変更なし（既存規則の確認のみ） |
| 葉所有型（trigger 単独、既存） | password-input | `control`（自身が opacity 所有） | 変更なし |

`clear-trigger` は `trigger` と同格の単独クリック可能な `<button>`（葉）であるため、葉所有型の部品では `disabled_declarations()` を適用する。`control` は常にレイアウトのみのコンテナであるため `cursor: not-allowed` のみとする。

### R2: `label[data-required]` は Themes で CSS 消費しない

必須の視覚マーカーは `field::required_indicator`（headless が `aria-hidden="true"` と `hidden` フリップを出す独立パーツ）を label の children へ合成して表現する。各 Forms 部品の `label` へ `*` 等の CSS 生成コンテンツを付けない。

根拠（3 者主基準 §8 に基づく部品ごと判断）:

- chakra-ui は `Field.RequiredIndicator` という**明示要素**で `*` を出す（`Field` コンポーネントの合成要素であり `Label` 自体が自動生成しない）
- shadcn/ui の `Label`/`FieldLabel` は自動マーカーを持たない（既存 `field` で充足、`docs/design/shadcn-inventory.md` 記載）
- Radix Themes に `Field`/必須マーカー相当は無い
- `SlotRecipe`/`StateCondition` は疑似要素（`::after`）を表現できない（`splitter.rs` rustdoc で既に記録済みの制約）
- CSS 生成コンテンツは `aria-hidden` にできず、ネイティブ `required` の読み上げと重複する
- 同クレート `field.rs` の既存設計（`data-required` への CSS は持たない）と整合する

### R3: headless が出さない属性へ規則を書かない（headless は変更しない）

pin-input `control` / editable `control` への `data-disabled` 追加、date-input への `label[data-required]` 追加は本イシューでは行わない。R2 により `data-required` の消費者が存在せず、`control` の 2 件は headless 側の明示的な設計判断（rustdoc）であるため、追加すると headless-ui の semver バンプと依存 3 クレート（pre-styled-ui / wasm-full / xtask）の追随が必要になり本イシューの目的（Themes 側の横断判断）に対して過大である。

結果として **変更クレートは `fandhe-frontend-pre-styled-ui` のみ**（+ docs-site の showcase・本設計文書）。

## 4. 対応表（部品 × パーツ × 属性、本イシュー適用後の Themes 規則）

| 部品 | `label[data-required]` headless 出力 | Themes CSS | `control[data-disabled]` headless | Themes（適用後） | `clear-trigger[data-disabled]` headless | Themes（適用後） | 所有型 |
|---|---|---|---|---|---|---|---|
| date-picker | あり | なし（R2） | あり | **`cursor` のみ（新規）** | あり | **`disabled_declarations()`（新規）** | 葉（input/trigger） |
| combobox | あり | なし | あり | **`cursor` のみ（新規）** | あり | **`disabled_declarations()`（新規）** | 葉（input/trigger） |
| select | あり | なし | あり | **`cursor` のみ（新規）** | あり | **`disabled_declarations()`（新規）** | 葉（trigger） |
| color-picker | あり | なし | あり | **`cursor` のみ（新規）** | （パーツなし） | — | 葉（trigger/channel-input） |
| number-input | あり | なし | あり | **`cursor` のみ（新規）** | （パーツなし） | — | コンテナ（root） |
| rating-group | あり | なし | あり | **`cursor` のみ（新規）** | （パーツなし） | — | 葉（item が opacity を所有。root へは追加しない） |
| date-input | **なし** | — | あり | **`cursor` のみ（新規）** | （パーツなし） | — | コンテナ（root） |
| tags-input | あり | なし | あり | `cursor` のみ（既存） | あり | `cursor` のみ（既存） | コンテナ（root） |
| file-upload | あり | なし | （`dropzone` が相当・既存 `cursor`） | — | あり | `cursor` のみ（既存） | コンテナ（root） |
| signature-pad | **なし** | — | あり | `cursor` + `touch-action`（既存） | あり | `cursor` のみ（既存） | コンテナ（root） |
| password-input | あり | なし | あり | `disabled_declarations()`（既存、control が所有） | （パーツなし） | — | コンテナ（control） |
| pin-input | あり | なし | **なし（R3）** | 規則なし | （パーツなし） | — | コンテナ（root） |
| editable | あり | なし | **なし（R3）** | 規則なし | （パーツなし） | — | コンテナ（root） |
| field | あり（全パーツ） | なし（`required-indicator` パーツで表現） | （`control` なし） | — | — | — |

rating-group の注記: headless は `root`/`control`/`item` すべてに `data-disabled` を出すが、Themes は `item` にのみ `disabled_declarations()` を持つ（葉所有型）。R1 に従い `control` は `cursor` のみとし、`root` に opacity を足すと item と二重になるため追加しない。

## 5. 適用結果

- `crates/pre-styled-ui/src/{date_picker,combobox,select,color_picker,number_input,rating_group,date_input}.rs` の recipe へ、上記対応表に従い `control`/`clear-trigger` の `[data-disabled]` state を純追加した（既存 state の並び替え・削除なし）
- 各ファイルのインライン `#[cfg(test)]` へ opacity 所有の有無を検証するブロック抽出テストを追加した
- golden CSS テスト（`crates/pre-styled-ui/tests/*_css.rs`）は state ブロックの純追加のみで再生成した
- 横断契約テスト `crates/pre-styled-ui/tests/forms_state_matrix.rs` を新設し、R1（opacity 単一階層）・R2（`[data-required]` 非出現）・R3（headless 非出力の固定）を出力ベースで機械検証する
- `crates/pre-styled-ui/tests/data_attr_vocabulary.rs` へ date-picker の `label[data-required]` が headless-sourced（自前出力なし）であることの固定テストを追加した
- `crates/pre-styled-ui/tests/xss_escape_styled.rs` へ `date_picker::root`（styled ラッパー）のエスケープ回帰を補完した（新しい動的値経路の追加ではなく、既存部品の未登録の補完）
- `fandhe-frontend-pre-styled-ui` を minor バンプした（CSS 出力の純追加、Rust API は不変だが直近の同型先例に倣う）

## 6. 再評価トリガー

- ark-ui / zag.js が pin-input `control` または editable `control` へ `data-disabled` を出す仕様変更を行った場合、R3 の「headless 非出力」前提が崩れるため Themes 側の規則追加を再検討する
- `fandhe-frontend-wasm-full` 側で control 単位の操作抑止（個別 slot の pointer-events 制御等）が必要になった場合、R1 の cursor-only 判断を再検討する
- `SlotRecipe`/`StateCondition` が疑似要素（`::after`）を表現できるよう拡張された場合、R2（label への CSS 生成コンテンツ非採用）の前提が変わるため再評価する

## 7. スコープ外（`.claude/rules/out-of-scope-tracking.md`）

- pin-input `control` / editable `control` への headless `data-disabled` 追加（R3）
- date-input / signature-pad / slider / angle-slider / checkbox-group `label` への `data-required` 追加（R2 により Themes 消費者が無いため実益なし）
- number-input / pin-input 等「コンテナ所有型」で `label` に `data-disabled` の減光を付けるか（現状 `field.rs` のみが `label[data-disabled]` を消費。本イシューの 2 軸〔required/control・clear-trigger〕の外）
- Forms 家族の variant 軸（`outline`/`subtle` 相当）追加（#1470 系で継続見送り中の別軸）

## 8. 参照競合の判定（`docs/design/shadcn-reference-adoption-policy.md` §8）

- 参照競合の判定: Forms 家族の label 必須マーカーは chakra-ui の値（明示要素 `Field.RequiredIndicator` による表現）を採る。理由: shadcn-ui/Radix Themes は自動マーカーを持たず、CSS 生成コンテンツは `aria-hidden` にできずネイティブ `required` の読み上げと重複する。既存 `field.rs` の設計とも整合し、`SlotRecipe` は疑似要素を表現できない。
- 参照競合の判定: Forms 家族の control/clear-trigger の disabled 視覚は chakra-ui/shadcn-ui 共通の値（disabled は opacity 0.5 + cursor not-allowed、コンテナと葉で opacity を重ねない）を採る。理由: 既存 tags-input/file-upload の二重 opacity 回避判断の一般化であり 3 者間に競合なし。
