# collapsible / accordion の `hidden` 属性から高さ CSS 制御への移行 設計評価

- イシュー: #2190（親 #2189。関連: #2191〔headless-ui 側の実装候補〕・#2192〔pre-styled-ui 側の実装候補〕）
- 出典: PR #2146 の対象外節、#2029 突合、#2026 / PR #2179（accordion 側の類似突合）
- 本文書の性質: **設計評価・決定記録**。コードは変更しない。採用案の確定は本文書末尾「ユーザー判断」節への記入をもって行う

## 1. 背景・トレーサビリティ

`fandhe-frontend-headless-ui` の disclosure 系パーツ（collapsible / accordion / bubble 等）は、閉状態のとき `hidden` 存在属性を content 要素へ付与する契約になっている。この契約は JS なしの SSR でも閉状態を表現できる一方、`hidden` は UA スタイルシートが `display: none` を強制するため、開閉の高さアニメーション（`height: 0 → height: auto` のような遷移）を CSS だけで実現できない。

この構造的な制約は `crates/pre-styled-ui/src/dialog.rs` の rustdoc（イシュー #1795、codex P1 レビュー確定）で既に一度言語化されている。同モジュールは `hidden` の付与・除去が「同一フレームで即時」に起こるため、開閉どちらの方向にもトランジションが発火しないこと、真に機能させるには (1) headless 層と協調した JS タイミング制御、(2) `@starting-style` + `transition-behavior: allow-discrete` を `SlotRecipe` へ追加する設計、のいずれかが必要であることを対象外事項として記録済みである。本評価はこの既存記録と矛盾しない形で候補を組み立てる。

`docs/policy/intentional-non-adoption.md` §3.25（ユーザー判断 2026-07-25）は headless-ui の責務境界を anatomy・アクセシビリティ・表示状態（`data-*`）までとし、参照元が primitives 層へ持ち込んでいる装飾・アニメーション・レイアウト計測の関心は headless へ持ち込まず、必要なら上層の pre-styled-ui または wasm-full 側の責務として設計する規則を定めている。本評価は高さ計測をこの規則に沿って `fandhe-frontend-wasm-full` の責務として割り当てる（§5）。

`docs/design/shadcn-reference-adoption-policy.md` §8 は headless 層を ark-ui 準拠に維持する方針を継続しており、本評価が挙げる候補案はいずれも ark-ui の Collapsible/Accordion 実装パターン（`hidden` またはアニメーション対応の `data-state`/CSS 変数）の範囲内にとどめる。

## 2. 現状契約の棚卸し

### 2.1 `hidden` を出しているパーツ（headless-ui、実装確認済み）

| モジュール | パート | 備考 |
|---|---|---|
| `crates/headless-ui/src/collapsible.rs`（`content()`、`CONTENT_RESERVED = ["data-state", "data-disabled", "id", "hidden"]`） | `content` | closed のとき `hidden` を付与。モジュール doc 「§参考サイトとの意図的な差分（イシュー #1637）」でアニメーション対応の `open`/`visible` 分離・CSS 変数出力をスコープ外と明記済み |
| `crates/headless-ui/src/accordion.rs` | `item_content` | collapsible と同型の契約 |
| `crates/headless-ui/src/bubble.rs` | `collapse_content` | collapsible の属性契約を再利用 |
| `crates/headless-ui/src/dialog.rs` | `positioner`/`backdrop`/`content` | `dialog.rs` rustdoc（#1795）で既に問題提起済み |
| `crates/headless-ui/src/drawer.rs` | `content` 相当 | dialog と同型 |
| `crates/headless-ui/src/popover.rs` | `content` | 同型 |
| `crates/headless-ui/src/tooltip.rs` | `content` | 同型 |
| `crates/headless-ui/src/floating_panel.rs` | `body` | 同型 |

`hidden` は collapsible 固有の契約ではなく、disclosure/overlay 系全体で共有される横断契約である。collapsible/accordion だけを変更すると、この家族内で表示状態の表現方法が分岐する（§4 の評価軸 6）。

### 2.2 下流で `hidden` に依存している箇所

- `crates/pre-styled-ui/src/collapsible.rs`: base 規則で `display` を宣言しない契約（テスト `content_base_does_not_declare_display`）。同モジュールは高さアニメーション非採用の理由を「構造的ブロッカーは headless の `hidden`」として既に記録している。
- `crates/wasm-full/src/focus_trap.rs`（`is_tabbable` 判定、`hidden` 属性を持つ要素を tabbable から除外）。`hidden` を外すと closed content 内の要素が Tab 順に残ってしまう。
- docs-site の原稿: `crates/docs-site/src/component_specs_overlay.rs`（COLLAPSIBLE.features）、`site/themes/collapsible.md`、`crates/pre-styled-ui/src/accordion.rs` rustdoc。
- `docs/guides/no-js-ssg.md`: JS ゼロ SSG 構成では `data-state` はビルド時固定であり、閉状態で SSR したものは閲覧時も閉じたままになる旨を明記している。

### 2.3 `hidden` が担っている 3 つの役割

移行案の比較は、`hidden` が実際に担っている次の 3 役割をどう代替するかで評価する。

1. **視覚的非表示**（UA スタイルシートの `[hidden] { display: none }` による、CSS 未読込でも成立する非表示）
2. **アクセシビリティツリー・Tab 順からの除外**（`focus_trap.rs` の tabbable 判定、スクリーンリーダーの読み上げ除外）
3. **JS ゼロ SSG での閉状態表現**（`no-js-ssg.md` が前提とする、ビルド時固定の `data-state` に対応する表示状態）

## 3. 候補案

### 案 A: 非移行（現状維持）

`hidden` 契約を変更しない。アニメーションは実現しない。

### 案 B: `hidden` 廃止・常時レンダリング

headless-ui が `hidden` の付与をやめ、content を常時 DOM に存在させる。pre-styled-ui は `content[data-state="closed"]` へ `height: 0; overflow: hidden; visibility: hidden` を宣言し、視覚的非表示・a11y 除外を CSS 側（または `visibility`/`inert` の別途付与）に委ねる。高さは wasm-full が実測して CSS 変数へ書き込む（§5）。

### 案 B′: 案 B + JS 稼働マーカー

案 B に加え、折りたたみ CSS の適用範囲を「wasm-full が hydration/mount 時に付与するマーカー（例: ルート要素への `data-*`）」配下に限定する。JS が動作していない環境では折りたたみ用 CSS が一切効かず、content は常時可視（progressive enhancement）になる。初期描画から hydration 完了までの一瞬のあいだ閉じているはずの content が見えてしまう「フラッシュ」が起こり得る点、その回避が `<head>` インラインスニペットや REQ-11 の「素の JS 最小化」方針と整合するかが論点になる。

### 案 C: `hidden` 維持 + CSS ネイティブの離散遷移（`allow-discrete` / `@starting-style`）

headless-ui は不変（`hidden` 契約を維持）。pre-styled-ui の `content` へ `transition: height <duration-token>, display <duration-token> allow-discrete;` を宣言し、`@starting-style` ブロックで遷移前スタイル（`height: 0` 等）を与える。`transition-behavior: allow-discrete` により、閉じる際は `display: none` の実適用が遷移完了まで遅延され、開く際は `@starting-style` が遷移前フレームの見た目を提供する。これは `dialog.rs` rustdoc（#1795）が既に記録している代替案 (2)「`@starting-style` + `transition-behavior: allow-discrete` を `SlotRecipe` へ追加する設計」の具体化であり、`crate::recipe::StateCondition`/`SlotRecipe` への DSL 拡張を前提とする。

### 参考（採用対象外として言及）

- `hidden="until-found"`: find-in-page（ページ内検索）対応の値だが、離散遷移の制御手段は提供しない。
- `interpolate-size: allow-keywords` / `calc-size()`: `height: auto` を含むキーワード値への遷移をブラウザネイティブに実現でき、成立すれば wasm-full の高さ計測が不要になる。本評価時点ではブラウザ対応が限定的であり主要案には採らないが、§8 の再評価トリガーとする。

## 4. 評価軸と比較表

| 評価軸 | 案 A | 案 B | 案 B′ | 案 C |
|---|---|---|---|---|
| 1. headless 変更・semver | 変更なし | headless マイナーバンプ、bubble/accordion 連鎖 | B と同じ + マーカー機構追加 | 変更なし（pre-styled-ui のみ） |
| 2. JS 無効時の表示（役割 iii） | 現状どおり閉固定（`no-js-ssg.md` と整合） | CSS 不在（Primitives 層のみ利用時）だと `aria-expanded="false"` なのに content が全文可視という状態不整合が起こり得る | JS 無効時は常時開いて見える（唯一「閲覧可能」を文字どおり満たす） | 現状どおり閉固定 |
| 3. SSR 決定性 | 不変 | 高さ実測値は実行時のみで SSR 出力に含めない | 同左（マーカーも SSR には出さない） | 不変。`hidden` の有無はビルド時に決定 |
| 4. `aria-expanded` 整合（役割 ii） | `hidden` が担うため整合 | `visibility`/`inert` を CSS または wasm-full に依存させると、CSS 未読込時に `aria-expanded=false` と実 DOM の可視・操作可能状態が食い違う（`drop_reserved` が防ぐ A05 型の状態偽装と同種のリスク） | B と同じ | `hidden` が担い続けるため整合 |
| 5. a11y ツリー・Tab 順 | `focus_trap.rs` の `hidden` 判定がそのまま機能 | `inert` 属性の追加付与判定が必要になる | B と同じ | 変更不要 |
| 6. 家族内一貫性 | 一貫（disclosure/overlay 系すべて `hidden`） | collapsible/accordion だけ変えると dialog/drawer/popover/tooltip/floating_panel と契約が分岐する。分岐の根拠が要る | B と同じ | 一貫を維持 |
| 7. アニメーション成立条件 | 成立しない | 高さ実測（wasm-full）が必要（`height: auto` は遷移不可のため） | B と同じ | 高さ実測（wasm-full）が必要な点は同じ |
| 8. reduced-motion | 該当なし | `Theme::to_css` は `prefers-reduced-motion: reduce` で `--fandhe-motion-duration-*` を `0ms` に一括上書きする（`transition: none` ではない）。duration 0ms でも `transitionend` は CSS Transitions 仕様上発火しないため、JS 側で `transitionend` を待つ実装を作らないことが設計制約になる | B と同じ | `allow-discrete` は duration 0ms でも即時適用に切り替わるだけで破綻しない。`data-state` 駆動の CSS 側で完結し、JS が `transitionend` を待つ実装を持たない設計が必須 |
| 9. ブラウザ対応 | 該当なし | 該当なし（表示制御は素の CSS プロパティのみ） | 同左 | `transition-behavior: allow-discrete` と `@starting-style` に依存する。本文書作成時点でのブラウザ対応状況は §11 に執筆時点の一次情報（出典・確認日付）を記載する運用とし、未対応ブラウザでは離散遷移が働かず「遷移なしで即時開閉」へ自然劣化することを実装側で確認する |
| 10. `SlotRecipe` DSL ギャップ | なし | `inert` 属性付与の責務を headless か wasm-full のどちらが持つか未決定 | B と同じ | `@starting-style`/`transition-behavior` の出力機構が `crate::recipe::StateCondition`/`SlotRecipe` に存在しない。新規追加が必要（横断判断、`tests/recipe_css.rs` への影響を伴う） |
| 11. 既存テスト・文書への影響 | なし | headless: `content_closed_has_hidden_attr_open_does_not` 等の契約変更、`crates/headless-ui/tests/collapsible.rs`・`tests/accordion.rs`。pre-styled: `content_base_does_not_declare_display`（golden `tests/collapsible_css.rs`/`accordion_css.rs`、更新手順は `docs/internal/pre-styled-ui-golden-test-update-guide.md`）。wasm-full: `focus_trap.rs`。docs-site: `component_specs_overlay.rs`・`site/themes/collapsible.md`。`docs/guides/no-js-ssg.md`・`docs/api/headless-ui-api.md` | B と同じ + マーカー関連のテスト追加 | headless-ui 側のテストは不変。pre-styled 側の golden 更新・`recipe.rs` の新規テストが必要 |
| 12. `intentional-non-adoption.md` §2 の 4 軸（明示性・決定性・機械検証可能性・コンテキスト消費） | 現状維持のため中立 | headless の責務変更は §3.25 の適用範囲拡大にあたり、再評価トリガーの充足確認が要る | B と同じ | headless は不変のまま pre-styled-ui の表現力を拡張するのみであり、§3.25 の役割分担（headless: 構造・a11y／pre-styled: 表示）に対する影響が最も小さい |

## 5. 高さ計測の責務割り当て（`docs/policy/intentional-non-adoption.md` §3.25 規則 2 に基づく）

- **headless-ui**: 計測・CSS 変数・アニメーション語彙を一切持たない。既存の `navigation_menu` における `no_part_outputs_data_motion` 系テストと同型の「headless はアニメーション関連の出力を持たない」ことを固定するテストを、案 B/B′/C いずれを採る場合も維持する。
- **pre-styled-ui**: CSS 変数の**消費**のみを担う（例: `height: var(--fandhe-collapsible-content-height, auto);`）。変数未設定時にも破綻しないフォールバック値を必ず伴わせる。
- **wasm-full**: `crates/wasm-full/src/headless.rs` の `wire_headless_component` が担う dispatch 後の `on_update` 経路（`apply_dirty_if_any`/`apply_update_for_dirty`）で、content 要素の `scroll_height` を実測し CSS 変数へ書き込む配線を追加する。書き込み手段は `crates/wasm-full/src/position.rs`（`resolve_position` が返す `style: String` を `set_attribute("style", ...)` で書き込む、`crates/wasm-full/src/focus_trap.rs::set_dom_attribute` 等と同一の先例パターン）に倣う。**考慮事項**: `style` 属性への直接書き込みは CSP の `style-src` が `unsafe-inline` を許可しない配布環境では拒否され得る。本リポジトリには CSP 方針を定めた文書が現存しないため実装を阻害する要因ではないが、将来 CSP 方針が導入される場合は `CssStyleDeclaration::set_property`（CSSOM 経由）への切り替えを検討する旨を設計メモとして残す。
- 計測値は実行時のみで SSR 出力へは含めない（決定性、評価軸 3）。CSS 変数の値は数値 + `px` の固定書式のみとし、文字列連結で任意値が CSS へ流れる経路を新設しない（§10 参照）。

### 5.1 実装記録（イシュー #2191、親トラッキング #2189）

- **CSS 変数名は `--fandhe-content-height` に確定**した（§5 冒頭の例示名
  `--fandhe-collapsible-content-height` は accordion にも共通で使う
  部品非依存の 1 変数へ統合したため読み替える。姉妹イシュー #2192 と
  共有する唯一のリテラルは `crates/wasm-full/src/content_height.rs::
  CONTENT_HEIGHT_VAR`）。
- **書き込み手段は CSSOM（`CssStyleDeclaration::set_property`/
  `remove_property`）を採用**し、§5 が示した「将来 CSP 方針導入時に
  検討」という位置づけから前倒しで確定した。`set_attribute("style",
  ...)` 直書きは content 要素の既存インライン宣言を破壊する副作用が
  あり、CSSOM ならプロパティ単位の更新で済むため（`crates/wasm-full/
  src/chart.rs::set_tooltip_position` に同一クレート内の先例あり）。
  これにより本節が挙げていた CSP `style-src` の懸念は解消済み。
- 統合先は `wire_headless_component`（`apply_dirty_if_any`/
  `apply_update_for_dirty` 経路ではない。dispatch 成功時の `on_update`
  直後・および配線時点の 2 箇所で `content_height::sync_content_height`
  を呼ぶ）。`Runtime::apply_dirty_if_any` 経路への統合は #2191 のスコープ
  外として整理した（詳細は `docs/design/wasm-full-architecture.md`
  §28.8）。
- **遷移成立条件の実測結果**: `Element::scroll_height()` はスタイル
  再計算を同期的に強制するため、`hidden` 解除直後の最初のスタイル
  計算時点で変数が未設定だと `@starting-style` 方式の `0 → auto` 遷移は
  補間不能。本ヘルパー（ステートレス）は同一要素に前回値が残る
  in-place 開閉の 2 回目以降でのみオープン方向の遷移を成立させられる。
  詳細・実ブラウザ確認範囲は `docs/design/wasm-full-architecture.md`
  §28.6 参照。

### 5.2 実装記録（イシュー #2192）

- **閉状態のキーは `StateCondition::Attr("hidden")` に確定**した
  （`AttrEq("data-state", "closed")` ではなく）。`hidden` は headless の
  disclosure 系全部品で保証された契約であり、bubble 等への横展開（#2282）
  時に部品ごとの `data-state` 出力有無を確認する必要がなくなる。
  collapsible `content`・accordion `item-content` はいずれも `data-state`
  と `hidden` の両方を出力するためどちらでも成立するが、契約の一般性を
  理由に `hidden` を採用した。
- **`@starting-style` を利用する `SlotRecipe` の公開 DSL** として
  `starting_style`/`starting_style_state`（Hover 系条件は fail-closed に
  除外）を追加し、`transition_declarations_allow_discrete`
  （`transition_declarations` + `transition-behavior: allow-discrete`）と
  組み合わせて `content_height_transition` preset へ統合した
  （`docs/api/pre-styled-recipe-api.md` §2 参照）。base 2 個目ブロック・
  `[hidden]` state・`@starting-style` の 3 規則を 1 呼び出しで登録する。
- **`box-sizing: border-box` を含める**。#2191 の実測値は `scrollHeight`
  （padding 込み）のため、content-box のまま `height: <px>` を当てると
  開いた定常状態で padding 分だけ箱が伸びてしまう。
- **`padding-block` も閉状態・`@starting-style`・`transition-property` に
  含める**。`height: 0` 単独だと閉じる途中で padding 分の高さが残ったまま
  `display: none` に落ちる見た目のジャンプが生じるため。代償として、
  変数未設定の初回オープン（劣化経路）では `height` が `auto` へ即時
  スナップする一方 `padding-block` は `0 → P` を補間できてしまい、200ms
  かけて padding だけが広がる見た目になり得るが、閉じ切る直前の padding
  ジャンプを消す利点（サポート経路の見た目）を優先した。
- **既知の限界（定常状態のクリップ）**: 開いた定常状態で `height: <px>`
  固定 + `overflow: hidden` のため、ウィンドウ幅変化で本文が伸びた場合や
  内包要素の後発的な高さ変化（#2191 の同期タイミング外）でクリップされ
  得る。#2191 が記録する「縮んだ場合に前回値が残る」限界（§5.1）とは
  逆方向の限界であり、対策（`interpolate-size: allow-keywords` の
  progressive enhancement・resize 時の再同期）は本イシューのスコープ外
  として別イシュー提案の対象とする。
- **`scrollHeight` は border を含まない**ため、border-box で
  `height: <scrollHeight>px` を当てると collapsible（1px border）では
  content 領域が上下計 2px 短くなる（`overflow: hidden` の切り取り境界は
  padding box のため本文は切れず、下 padding が実質 14px に見えるだけ）。
  accordion は border なしで完全一致する。修正不要の「意図した限界」と
  して記録する。
- **`@starting-style` ブロックの出力位置**は states の後・
  `@media (hover: hover)` の前に 1 個だけ集約する（hover ブロックと同じ
  「常に末尾に 1 個」契約を壊さないため）。`@starting-style` を利用しない
  既存部品の golden は差分ゼロ（純追加）。
- **動作確認の範囲**: docs-site は JS ハイドレーションを行わないため、
  Themes ページ上で開閉トランジションそのものを観察することは構造的に
  できない。`make docs` の生成 CSS に新ブロックが載ること・静的 Demo
  （open 固定）の見た目が `box-sizing: border-box` + `auto` フォール
  バックで従来と同寸であることの確認に留める。実ブラウザでの遷移確認は
  `crates/wasm-full/tests/content_height_browser.rs` の in-place 再開閉
  ケースが担う（wasm-full は pre-styled-ui に依存しないため、pre-styled-ui
  側の CSS を実際に注入した browser テストの新規追加は本イシューのスコープ
  外）。

## 6. JS 無効時の表示方針（親 #2189 の受け入れ条件「JS 無効時に content が閲覧可能」への回答）

- **(a) 原則維持**: `docs/guides/no-js-ssg.md` の原則どおり、JS ゼロ構成では `data-state` をビルド時に固定する。「JS 無効でも読ませたい content」は呼び出し側が `OpenState::Open` で SSR するか、`<details>`/`<summary>` を使う運用をガイドへ明記する（案 A・案 C はこの読み替えで受け入れ条件を満たす）。
- **(b) 案 B′ のマーカー方式**: JS 無効時は常に開いて見える。受け入れ条件を文字どおり満たす唯一の選択肢だが、headless-ui のマイナーバンプ・マーカー機構の新設・hydration 完了までのフラッシュ対策を要する。
- **(c) headless の `Default` を Open にする**: 既存の `collapsible_default_is_closed` 契約や dialog 等の他パーツとの整合性を崩す度合いが大きく、非推奨。

親 #2189 の受け入れ条件は、案 B′ を選ばない限り「SSR 時の初期状態の選び方」で満たす読み替えになる。この読み替えを採るかどうかは §7 のユーザー判断に委ねる。

## 7. 採否判定（推奨）と「ユーザー判断」節

### 推奨

現時点の調査結果では **案 C（`hidden` 維持 + `allow-discrete`/`@starting-style` + wasm-full 高さ計測）** を第一推奨とする。理由は次の 4 点をすべて維持できる唯一の案であるため。

1. `docs/policy/intentional-non-adoption.md` §3.25 の責務境界（headless 不変）
2. §4 評価軸 6 の家族内一貫性（dialog/drawer/popover/tooltip/floating_panel と契約を分岐させない）
3. `docs/guides/no-js-ssg.md` の原則（JS ゼロ構成の閉状態表現）
4. `aria-expanded` と実 DOM 状態の整合（`hidden` が role/state の整合性を保証し続ける）

案 B′ は「JS 無効時に閲覧可能」を文字どおり満たす唯一の案だが、headless-ui のマイナーバンプ・マーカー機構・フラッシュ対策のコストを伴う。案 B 単体は役割 (ii)(iii) を CSS/JS 側の追加実装なしには失うため非推奨。案 A は現状維持であり、#2192（pre-styled-ui のトランジション実装）の前提が成立しない。

### ユーザー判断（イシュー #2190、2026-09-10 確定）

- [x] 採用案（A / B / B′ / C）：**案 C**（`hidden` 維持 + `allow-discrete`/`@starting-style` + wasm-full 高さ計測）
- [x] JS 無効時方針（§6 (a) / (b) / (c)）：**§6 (a) 原則維持**
- [x] collapsible に加え accordion / bubble を同時対象にするか：**collapsible + accordion**。bubble（`collapse_content`）は対象外（別イシューで再評価）
- [x] `SlotRecipe` へ `@starting-style` / `transition-behavior` サポートを追加する（案 C を採る場合の前提）ことの承認：**承認**
- [x] #2191 / #2192 の再スコープ要否：**要**。§9 の案 C 対応どおり

- [x] 実装方針の追加指示（2026-09-10）：**機構は部品非依存の共通実装とする**。`SlotRecipe` の `@starting-style` / `transition-behavior` は部品横断の DSL 機能として `crate::recipe` に置き、wasm-full の高さ実測・CSS 変数書き込みも部品非依存の共通ヘルパーとして実装する。collapsible / accordion はその適用側にとどめ、bubble 等への後続適用は同じ機構の適用イシュー（#2001 配下に起票）で扱う

2026-09-10 に確定済み。#2191 / #2192 は §9 の案 C 対応へ再スコープのうえ着手可。

## 8. 再評価トリガー

- `interpolate-size: allow-keywords` / `calc-size()` が主要 3 エンジン（Chromium / Firefox / WebKit）で利用可能になった時点。成立すれば wasm-full の高さ実測配線が不要になり、案の優先順位が変わり得る。
- headless 層全体（dialog/drawer/popover/tooltip/floating_panel を含む）で `hidden` を別機構へ置き換える横断判断が発生した時点。
- Primitives 層（`fandhe-frontend-headless-ui`）のみを利用する開発者から「JS 無効時に content を開いて見せたい」という具体的要望が生じた時点（案 B′ の再評価）。

## 9. 実装 issue 分割案（既存 #2191 / #2192 との対応）

（2026-09-10 案 C 確定。#2191 / #2192 の本文は同日に再スコープ済み）

採用案が確定した後、以下の対応関係を目安に #2191 / #2192 を更新する（更新自体はユーザー承認事項であり、`update-issue-tree` で行う）。

- **案 C を採る場合**: #2191 → 「wasm-full: content 高さ実測と CSS 変数書き込み配線（headless-ui は変更しない）」、#2192 → 「pre-styled-ui: `crate::recipe` へ `@starting-style` / `transition-behavior: allow-discrete` の出力を追加し、collapsible / accordion へ適用。golden テスト更新、`site/themes/collapsible.md` 等の文言更新」。実装順序は「`SlotRecipe` への DSL 拡張 → 各部品への適用 → wasm-full の計測配線」だが、後半 2 つは互いに独立して自然劣化する（計測配線が無くても遷移自体は `height: 0` ⇄ 固定値の範囲で機能する）ため並行実施も可能。
- **案 B / B′ を採る場合**: #2191 は現行本文どおり（headless-ui のマイナーバンプ、bubble/accordion への契約変更の連鎖、`focus_trap.rs` の `inert` 判定追加、`docs/guides/no-js-ssg.md` と `docs/api/headless-ui-api.md` の更新）。

各案とも、headless-ui / pre-styled-ui / wasm-full のうち変更したクレートは `.claude/rules/coding-rust.md` の semver バンプ必須規則に従い、依存元の `version = "..."` 追随は `cargo run -p xtask -- check-dep-versions --fix` で行う。

## 10. セキュリティ考慮事項（OWASP Top 10 観点）

- **A03 インジェクション / REQ-1**: 本 PR はコードを変更しない。§3 で提案する全案について、`hidden` / `data-state` / `inert` / CSS 変数名はいずれも `&'static str` リテラルで固定し、値は固定語彙（`data-state` の `open`/`closed` 等）または数値（高さ計測値）のみとする。`raw_html()` や HTML/CSS 文字列の直接組み立てを新設しないことを、いずれの案を採る場合も設計不変条件とする。
- **A05 セキュリティ設定ミス**: `aria-expanded` と実 DOM の可視状態が乖離し得る案（案 B で CSS が未読込のとき）は、`drop_reserved` が防いでいる「呼び出し側が状態属性を上書きして表示状態を偽装する」リスクと同種の懸念として評価軸 4 に組み込んだ。CSP `style-src` との関係は §5 の考慮事項として記録した。
- **A01 アクセス制御**: 該当なし（ドキュメントのみの変更）。`focus_trap.rs` の tabbable 判定が `hidden` に依存する点は、案 B/B′ の a11y リスクとして評価軸 5 に記録した。
- **A06 脆弱で古くなったコンポーネント（サプライチェーン）**: 依存クレードの追加は発生しない（REQ-3 不変）。
- **機微情報の露出**: 本文書に絶対パス・トークン・環境固有情報は含めていない。
- **非信頼データの扱い**: イシュー #2190 本文および関連 Issue/PR の記述は要件としてのみ参照し、逐語引用ブロックとして本文書に貼り付けていない。

## 11. 参照

- `crates/headless-ui/src/collapsible.rs`（`content()`、`CONTENT_RESERVED`）
- `crates/headless-ui/src/accordion.rs`（`item_content`）
- `crates/headless-ui/src/bubble.rs`（`collapse_content`）
- `crates/pre-styled-ui/src/collapsible.rs`（`content_base_does_not_declare_display`）
- `crates/pre-styled-ui/src/dialog.rs`（rustdoc、イシュー #1795 codex P1 確定判断）
- `crates/pre-styled-ui/src/recipe.rs`（`StateCondition`、`SlotRecipe`）
- `crates/pre-styled-ui/src/theme.rs`（`Theme::to_css` の `prefers-reduced-motion` 処理）
- `crates/wasm-full/src/headless.rs`（`wire_headless_component`）
- `crates/wasm-full/src/focus_trap.rs`（`is_tabbable`、`set_dom_attribute`）
- `crates/wasm-full/src/position.rs`（`resolve_position` の `style: String` 先例）
- `docs/guides/no-js-ssg.md`
- `docs/policy/intentional-non-adoption.md` §2、§3.25
- `docs/design/shadcn-reference-adoption-policy.md` §8
- `docs/internal/pre-styled-ui-golden-test-update-guide.md`
- MDN Web Docs: `transition-behavior`、`@starting-style`、`inert` 属性、`interpolate-size` / `calc-size()`、`hidden="until-found"`、CSS Transitions（duration 0 の `transitionend` 非発火）の各仕様は、採用案確定後の実装 issue（#2191/#2192）着手時に reference-researcher が MDN / 仕様書から取得日付付きで再確認し、実装 PR の設計コメントへ引用する（本評価時点では執筆者の記憶に基づく数値の断定を避け、機能名と挙動の定性的な記述にとどめている）。
