# wasm-full feature 選択ガイド

本ドキュメントはイシュー #2330 を契機に作成しました。`fandhe-frontend-wasm-full`
（イシュー #2326/#2327）が持つ 2 軸の Cargo feature（配線群別 15 件・
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

`position` feature（0.20.1 で追加）はこの表とは別枠です。
`headless::wire_headless_component` 内の自動 positioning 呼び出し
（`ensure_global_controller`・配線時/dispatch 後の `reposition_within`）
のみをゲートし、`position` モジュール自体・公開 API（`PositionController`
等）はゲート対象外です。

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

**0.19.0 以降へアップグレードし `default-features = false` を使っている
場合**、上記の配線・MAPPING_TABLE 行・keynav 分岐が既定では失われます。
従来どおりの挙動を維持するには、`Cargo.toml` の依存指定へ `default` 配列
と同じ 32 件を明示してください（`entry` 機能を使わないアプリは
`wasm-bindgen-exports` を省略できます）。

```toml
[dependencies.fandhe-frontend-wasm-full]
version = "0.20.1"
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
  "position",
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
  questionnaire）は対象外です。

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
