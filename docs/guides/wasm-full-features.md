# wasm-full feature 選択ガイド

本ドキュメントはイシュー #2330 を契機に作成しました。`fandhe-frontend-wasm-full`
（イシュー #2326/#2327）が持つ 2 軸の Cargo feature（配線群別 17 件・
scope 別 16 件、いずれも既定 on）と、`fandhe-frontend-dist-server`
（イシュー #2329）が配布する最小構成を、利用者向けに一箇所へ集約します。
機械可読な一次情報（対応表そのもの）は `crates/wasm-full/src/lib.rs`
クレート doc・`crates/wasm-full/Cargo.toml` の `[features]` 直前コメント
であり、本書はそれを利用者の目的別に読みやすく再構成したものです。

## 1. 対象読者

- `fandhe-frontend-wasm-full` を直接依存に持つアプリ（`examples/interactive-view-transitions`
  のように `default-features = false` で必要な feature だけを選びたい利用者）
- `fandhe-frontend-dist-server` を利用し、配布 WASM に含まれる部品を把握
  したい利用者（`examples/dist-server-docker` 相当）

## 2. feature の 2 軸と既定 on

`fandhe-frontend-wasm-full` の feature は独立した 2 軸に分かれます。

- **配線群別 feature**（イシュー #2326）: `Runtime::mount`/`hydrate` が呼ぶ
  `wire_*` 呼び出し 1 件を feature 1 件でゲートします。off にすると、その
  配線自体が `mount`/`hydrate` から呼ばれなくなります。
- **scope feature**（イシュー #2327）: `keynav::wire_keynav` 内部の
  `match scope` 分岐（部品ごとの 1 arm）と `headless::MAPPING_TABLE` の
  該当行を feature 1 件でゲートします。off にすると、`keynav::wire_keynav`
  自体は呼ばれ続けますが、その部品のキーボード操作・クリック配線のみが
  失われます。

両軸は独立しているため、部品ごとに次の組み合わせを選べます。

- **クリック操作のみで良い** = 当該 scope feature のみを有効にする
  （`keynav` は off のままで良い）
- **キーボード操作も使う** = `keynav` feature + 当該 scope feature の両方
  を有効にする

いずれも既定はすべて on（`default-features = false` を指定しない限り、
既存の挙動は変わりません）。

## 3. 配線群別 feature 対応表（イシュー #2326、0.19.0 で追加）

| 配線 | feature |
|---|---|
| `events::wire_events` | ゲートしない（`data-action` 委譲、全構成必須） |
| `keynav::wire_readonly_click_guard` | ゲートしない（readonly RadioGroup の click capture 保護） |
| `Runtime::wire_headless` | ゲートしない（`headless::MAPPING_TABLE` 全行のクリック dispatch） |
| `keynav::wire_keynav` | `keynav` |
| `focus_visible::wire_focus_visible` | `focus-visible` |
| `Runtime::wire_avatar` | `avatar` |
| `Runtime::wire_clipboard` | `clipboard` |
| `Runtime::wire_timer` | `timer` |
| `Runtime::wire_angle_slider` | `angle-slider` |
| `Runtime::wire_splitter` | `splitter` |
| `Runtime::wire_signature_pad` | `signature-pad` |
| `Runtime::wire_number_input` | `number-input` |
| `Runtime::wire_command` | `command` |
| `Runtime::wire_sidebar` | `sidebar` |
| `Runtime::wire_chart` | `chart` |
| `Runtime::wire_chart_range` | `chart-range` |
| `Runtime::wire_questionnaire` | `questionnaire` |
| `Runtime::wire_message_scroller` | `message-scroller` |
| `Runtime::wire_data_table` | `data-table` |
| `Runtime::wire_in_view` | `in-view` |

`position` feature（0.20.1 で追加）はこの表とは別枠です。
`headless::wire_headless_component` 内の自動 positioning 呼び出し
（`ensure_global_controller`・配線時/dispatch 後の `reposition_within`）
のみをゲートし、`position` モジュール自体・公開 API（`PositionController`
等）はゲート対象外です。

`stagger` feature（0.21.0 で追加、イシュー #2397）も同じく別枠です。
`Runtime::apply_update_for_dirty` の keyed list 構造反映（`Insert`/
`Move`）直後の `stagger_index::sync_stagger_index` 呼び出しのみをゲート
し、`stagger_index` モジュール自体・公開関数（`stagger_index_value`）は
ゲート対象外です。

`view-transition-name` feature（0.22.0 で追加、イシュー #2515）も別枠
ですが、`position`/`stagger` とはゲート対象が異なります: `Runtime` 内部の
呼び出し箇所ではなく、`view_transition_name::set_view_transition_name`
（`Runtime` を経由しないアプリ直接利用 API）という公開関数**そのもの**
の存在をゲートします。`view_transition_name` モジュール自体・純粋関数
（`is_valid_view_transition_name`）はゲート対象外です。off にすると
`set_view_transition_name` が使えなくなります。

## 4. scope feature 対応表（イシュー #2327、0.20.0 で追加）

feature 名は `headless::MAPPING_TABLE` の `scope` 文字列と一致します
（keynav の match arm リテラルが異なる場合のみ併記）。

| feature | MAPPING_TABLE 行数 | keynav の match arm |
|---|---|---|
| `accordion` | 1 | `"accordion"` |
| `calendar` | 3 | `"calendar"` |
| `collapsible` | 1 | なし |
| `combobox` | 3 | `"combobox"` |
| `dialog` | 2 | なし |
| `listbox` | 0（keynav 専用） | `"listbox"` |
| `menu` | 4 | `"menu"` |
| `menubar` | 3 | `"menubar"` |
| `navigation-menu` | 1 | `"navigation-menu-trigger"`・`"navigation-menu-link"` |
| `popover` | 2 | なし |
| `radio-group` | 1 | `"radio"`・`change` リスナー |
| `select` | 3 | `"select"` |
| `tabs` | 1 | `"tabs"`・bubble click |
| `toggle-group` | 1 | `"toggle-group"` |
| `tooltip` | 1 | なし |
| `tree-view` | 2 | `"tree-view"`・tree 復元 |

配線群別 feature である `sidebar`（MAPPING_TABLE 2 行）・`signature-pad`
（同 1 行）も、それぞれ同名 feature で MAPPING_TABLE 行を追加ゲートします
（上表とは別枠）。

readonly RadioGroup の click capture 保護（`keynav::wire_readonly_click_guard`）
はいずれの scope feature にも依存しない常時配線です。`radio-group` を off
にしても、この保護は失われません。

## 5. feature に関わらず常時有効な配線

- `events::wire_events`（`data-action` 委譲）
- `keynav::wire_readonly_click_guard`（readonly RadioGroup の click capture 保護）
- `Runtime::wire_headless`（`headless::MAPPING_TABLE` 全行のクリック dispatch）

## 6. gating 対象外の直接利用 API

`overlay` / `tooltip` / `position` / `focus_trap` / `headless_file_upload` /
`headless_select` モジュール（`Runtime` を経由せずアプリが直接呼び出す
公開 API）は、いずれの feature でもゲートされません。

**名前の衝突に注意してください**: `tooltip` / `select` / `menu` 等の
feature 名は、上記モジュール名と同じ文字列ですが、feature が実際に
ゲートするのは `headless::MAPPING_TABLE` 行・`keynav` の match arm のみで、
`tooltip` モジュール自体や `headless_select` モジュール自体の公開 API を
無効化するものではありません。

## 7. `default-features = false` 利用者の移行手順

`fandhe-frontend-wasm-full` は以下の版数で feature を追加しました
（いずれも 0.x の破壊的変更として minor バンプ）。

| 版数 | 追加内容 |
|---|---|
| 0.19.0 | 配線群別 feature 14 件（イシュー #2326） |
| 0.20.0 | scope feature 16 件（イシュー #2327） |
| 0.20.1 | `position` feature（イシュー #2209/#2332） |
| 0.20.5 | `message-scroller` feature（イシュー #2122） |
| 0.20.8 | `data-table` feature（イシュー #2126） |
| 0.21.0 | `stagger` feature（イシュー #2397）・`in-view` feature（イシュー #2396） |
| 0.22.0 | `view-transition-name` feature（イシュー #2515） |

**0.19.0 以降へアップグレードし `default-features = false` を使っている
場合**、上記の配線・MAPPING_TABLE 行・keynav 分岐が既定では失われます。
従来どおりの挙動を維持するには、`Cargo.toml` の依存指定へ `default` 配列
と同じ 37 件を明示してください（`entry` 機能を使わないアプリは
`wasm-bindgen-exports` を省略できます）。

```toml
[dependencies.fandhe-frontend-wasm-full]
version = "0.22.0"
default-features = false
features = [
  "wasm-bindgen-exports",
  "keynav",
  "focus-visible",
  "avatar",
  "clipboard",
  "timer",
  "angle-slider",
  "splitter",
  "signature-pad",
  "number-input",
  "command",
  "sidebar",
  "chart",
  "chart-range",
  "questionnaire",
  "message-scroller",
  "data-table",
  "in-view",
  "position",
  "stagger",
  "view-transition-name",
  "accordion",
  "calendar",
  "collapsible",
  "combobox",
  "dialog",
  "listbox",
  "menu",
  "menubar",
  "navigation-menu",
  "popover",
  "radio-group",
  "select",
  "tabs",
  "toggle-group",
  "tooltip",
  "tree-view",
]
```

### `wire_signature_pad_component` を `Runtime` 経由せず直接呼ぶ利用者への注意

`headless_signature_pad::wire_signature_pad_component` を自前のマウント
処理から直接呼んでいる利用者（`Runtime::wire_headless` 経由ではない構成）
は、既定の `signature-pad` feature 有効化だけでは救済されません。上記の
`features = [...]` 一覧に加えて、`headless::wire_headless_component` を
同じ `root`/`component` へ明示的に呼び出してください
（`Runtime::wire_headless` の実装と同型の呼び出しで足ります）。

## 8. dist-server 最小構成

`fandhe-frontend-dist-server`（配布 WASM）は「最小インタラクティブ
コンポーネント」として次の 6 feature のみを有効にして配布します
（イシュー #2329、単一定義は `crates/dist-server/src/wasm_dist_features.rs`
の `WASM_DIST_FEATURES`）。

```
wasm-bindgen-exports, collapsible, dialog, popover, tooltip, position
```

判断根拠:

- REQ-11 が定義するワークロード（カウンター・フォーム入力・動的リスト
  更新相当）は常時配線の `events::wire_events` と束縛点更新のみで成立し、
  scope feature を 1 つも要求しません（理論下限は `wasm-bindgen-exports`
  のみ）。
- 「button / input / dialog 系」に、クリック操作のみで機能が完結する
  disclosure / overlay 部品（collapsible / dialog / popover / tooltip）を
  加えます。これら 4 scope は `keynav.rs` に cfg 分岐を持たず、`keynav`
  off でも WAI-ARIA 上のキーボード操作が欠けません。
- `keynav`/`focus-visible` は REQ-11 の実測でサイズへの影響が大きい
  支配的なレバーのため除外します。他の scope feature（accordion /
  calendar / combobox / listbox / menu / menubar / navigation-menu /
  radio-group / select / tabs / toggle-group / tree-view）は keynav の
  match arm を持つため、「click 行だけを配布してキーボード操作を欠いた
  部品」を出荷しないよう除外します。
- `position` は popover / tooltip の表示位置決めに必要なため含めます。
- 配線群別 feature（avatar / clipboard / timer / angle-slider / splitter /
  signature-pad / number-input / command / sidebar / chart / chart-range /
  questionnaire / message-scroller / data-table / in-view）は対象外です。

実測（ローカル、wasm-opt 適用済み）: `bundle-size: total_gzip_bytes=120618/200000
files=2 result=PASS`（上限余裕 79,382 B）。CI は wasm-opt 未導入のため
構成が異なりますが、実測比較（[wasm-opt 導入評価](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/ci/wasm-opt-adoption-evaluation.md)
「#1972 の結論」節）から CI 側の値はむしろ小さくなる方向であり、上記
余裕を侵食する懸念はありません。

### 最小構成に含まれない部品を使う場合

配布 WASM の最小構成に含まれない部品（accordion / menu / select 等）を
利用したい場合の選択肢は 2 つです。

1. `fandhe-frontend-wasm-full` を自アプリの直接依存として追加し、上記
   §7 の要領で必要な feature を選ぶ（`examples/interactive-view-transitions`
   参照）。
2. `fandhe-frontend-dist-server` を fork/vendor し、
   `crates/dist-server/src/wasm_dist_features.rs` の `WASM_DIST_FEATURES`
   を変更する（`crates/wasm-full/tests/bundle_size.rs` の実測値を必ず
   併せて確認する）。

## 9. 検証方法

`-p fandhe-frontend-wasm-full` を単体で指定し、対象 target を明示して
`cargo check` してください。

```bash
cargo check -p fandhe-frontend-wasm-full \
  --no-default-features \
  --features wasm-bindgen-exports,collapsible,dialog,popover,tooltip,position \
  --target wasm32-unknown-unknown --locked
```

`--workspace` を付けて `--no-default-features`/`--features` を指定した
場合、Cargo の解決対象がワークスペース全体に広がるため、意図した
縮小構成の検証にはなりません。縮小構成の検証は必ず `-p
fandhe-frontend-wasm-full` 単体で行ってください。

## 10. 関連ドキュメント

- [コンポーネント記述ガイド](./component-authoring.md)
- [`examples/interactive-view-transitions/README.md`](../../examples/interactive-view-transitions/README.md)
- [`examples/dist-server-docker/README.md`](../../examples/dist-server-docker/README.md)
- [wasm-full feature gating 評価](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/wasm-full-feature-gating-evaluation.md)
- [wasm-full アーキテクチャ設計](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/wasm-full-architecture.md)

## 11. 新規 feature 追加チェックリスト（イシュー #2395）

Phase 4 以降（イシュー #2396 以降、`fandhe-frontend-animation` 由来の
feature を含む）で `fandhe-frontend-wasm-full` へ feature を追加する際の
定型チェックリストです。feature 集合の記述は複数箇所へ分散しており、
更新漏れが起きやすいため、追加時は以下を順に確認してください。

### 11.1 共通チェックリスト（配線群別 feature・scope feature 共通）

1. `crates/wasm-full/Cargo.toml` の `[features]` へ `<name> = []` を追加
   し、`default` 配列にも追加する（配線群か scope かで見出しコメントの
   表を選ぶ）。
2. `crates/wasm-full/src/lib.rs` クレート doc の該当対応表（配線群別
   feature 表／scope feature 対応表）と、直前の件数記述（「配線群別
   feature n 件 + scope feature n 件」等）を更新する。
3. `CLAUDE.md` の `crates/wasm-full/` 行（「配線群別 feature n 件 + scope
   feature n 件」）の件数を更新する。
4. 既存 CI ジョブ（配線群 feature なら `wasm-full-feature-matrix-wiring`、
   scope feature なら `wasm-full-feature-matrix-scope`）へ、既存 feature
   と同型の `name:`/`run:` 1 行を追加する。**新規ジョブは作らない**
   （ruleset PUT が必要になるため、`.claude/rules/ci.md` の「wasm-full の
   feature matrix ジョブ」節参照）。
5. 本ガイド（§3 or §4 の対応表、§7 の `default-features = false` 移行
   手順の feature 列挙・版数表）へ反映する。
6. `cargo test -p fandhe-frontend-wasm-full --test bundle_size` を実行し、
   `bundle-size:` 1 行サマリの実測値を PR 本文に記録する
   （`.claude/rules/ci.md` 既定要件）。
7. `crates/dist-server/src/wasm_dist_features.rs` の `WASM_DIST_FEATURES`
   （6 feature の最小インタラクティブ構成）へ新規 feature を**追加しな
   い**ことを確認する（既定は非追加。追加が必要な場合は同ファイル冒頭
   コメントへ判断根拠を追記する重い変更になる）。

### 11.2 3 層構成（`fandhe-frontend-animation` 由来 feature、#2417）固有の追加項目

- `fandhe-frontend-animation` を `crates/wasm-full/Cargo.toml` の
  optional 依存として追加し、新設 feature が `dep:fandhe-frontend-animation`
  で有効化する形にする。
- `wasm-full` 側にアニメーション演算ロジックを書かない。
  `fandhe-animation`/`fandhe-frontend-animation` が計算した結果を
  DOM/Web Animations API へ配線する呼び出しのみを `wasm-full` に置く
  （`docs/design/animation-core-architecture.md` の層責務を踏襲）。

### 11.3 検証チェックリスト

- ローカルで以下を実行し、単体構成が成立することを CI 追加前に確認する。

  ```bash
  cargo check -p fandhe-frontend-wasm-full \
    --no-default-features \
    --features wasm-bindgen-exports,<new-feature> \
    --target wasm32-unknown-unknown --locked
  ```

- scope feature の場合は `crates/wasm-full/tests/feature_gating_contract.rs`
  （`MAPPING_TABLE`/`keynav` の `cfg` ゲート・`default` 配列列挙の一貫性を
  機械検証）を実行する。既存テストが追加漏れを検知するため、新規テスト
  作成は不要。
- CI ジョブへの 1 行追加は `crates/xtask/tests/workflow_wasm_full_feature_matrix.rs`
  （ci.yml の `run:` 行から抽出した feature 集合と `Cargo.toml`
  `[features]` の過不足なき一致を検証）を実行する。追加漏れ・記載ミスは
  fail-closed に検知される。
- 新規 CI ジョブを作らない方針は ruleset の `required_status_checks`
  個別列挙契約（`.claude/rules/ci.md`「`ci-complete` 集約ジョブと ruleset
  必須チェック」節）に追随更新（マニフェスト更新＋ruleset PUT）を要する
  運用コストが根拠であり、既存 2 ジョブへの行追加で完結することを確認
  する。
