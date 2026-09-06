# headless-ui / wasm-full の部品別 feature gating によるサイズ削減の導入評価

## 1. 背景・トレーサビリティ

- REQ-11（`docs/spec/04-requirements.md`）: 「最小インタラクティブコンポーネント」の
  WASM バンドル gzip 合計を 200,000 B 以内とする受け入れ基準。計測経路は
  `crates/wasm-full/tests/bundle_size.rs`（CI `bundle-size` ジョブ）で、
  `crates/dist-server/build.rs` のネストビルドと同一構成（同テストのモジュール
  doc「計測経路と製品ビルドとの契約」節参照）。
- 親 #1953（REQ-11 余裕確保）は 3 本のレバーを持つ: (1) 警告閾値の追加
  （#1968、完了。`REQ11_BUNDLE_SIZE_WARN_BYTES = 190_000`〔上限の 95%〕が
  `bundle_size.rs` に導入済み）、(2) dist-server 経路への wasm-opt 実測・導入
  （#1969 / #1970、open・ユーザー承認待ち）、(3) **本イシュー = 部品別 feature
  gating の評価**。
- 実測記録: CI main 193,731 B（run 34025276417、2026-09 時点、[記録値]）、
  ローカル（#1647 記録、2026-09-05）199,579 B。既採用: wasm32 限定
  `opt-level="s"`（`.cargo/config.toml`、#1647）・6 クレートの
  `codegen-units=1`（ルート `Cargo.toml`）。見送り済み: `opt-level="z"`・
  アロケータ差し替え（`docs/ci/wasm-opt-adoption-evaluation.md` /
  `docs/ci/wasm-allocator-adoption-evaluation.md`）。
- 本文書は #1969/#1970（wasm-opt 導入）の領域とは独立し、相互参照のみ行う
  （§9 参照）。**本イシューは評価文書の新設のみを成果物とし、コードは
  変更しない。**

## 2. 計測経路の確認と前提の是正

`bundle_size.rs` が計測するコマンド列は次のとおり:

```
cargo build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown --release
  → wasm-bindgen --target web --no-typescript --out-dir <dir>
  → 出力ファイル（*_bg.wasm・*.js）を個別に gzip -9 した合算
```

`fandhe-frontend-wasm-full` は `crate-type = ["cdylib", "rlib"]` の cdylib
としてビルドされ、既定 feature `wasm-bindgen-exports`（`crates/wasm-full/
Cargo.toml` の `default = ["wasm-bindgen-exports"]`）が有効なとき
`src/entry.rs` が `AppState` 上の `mount` / `hydrate` / `start_router` の
3 関数のみを `#[wasm_bindgen]` エクスポートする（`entry.rs:73`/`89`/`132`）。
`crates/wasm-full/src/` 内の他の `#[wasm_bindgen]` は `nav.rs:247` の
`startViewTransition` extern バインディング（ブラウザ API 呼び出し側で
あり、エクスポートではない）のみで、他モジュールに追加のエクスポートは
ない。実際に §3 の手順でビルドした `fandhe_frontend_wasm_full.js`
（`--target web`）の `export function` 行も `hydrate` / `mount` /
`start_router` の 3 件のみであることを確認済み。cdylib の wasm-ld は
エクスポートから到達不能な関数を GC するため、**最終バイナリに載るのは
この 3 エクスポートからの到達閉包のみ**である。

イシューの立脚点「wasm-full は 63 部品分の headless wiring をすべて
リンクしている」は、この意味では正確ではない。`fandhe-frontend-headless-ui`
（71 モジュール、約 8 万行）のうち wasm-full が実際に import して製品コード
（`crates/wasm-full/src/*.rs` の `mod` 定義群、テストを除く）から参照するのは
`avatar` / `clipboard` / `timer` / `angle_slider` / `signature_pad` /
`file_upload` / `checkbox` / `number_input` 等の一部モジュールに限られる。
**削減レバーの実体は headless-ui 側の feature 分割ではなく、wasm-full 側の
`Runtime::mount`/`hydrate` が行う一括配線（wiring）である**。

`crates/wasm-full/src/lib.rs` の `Runtime::mount`（948 行）・`Runtime::hydrate`
（1045 行）はいずれも次を同一順序で呼ぶ（`lib.rs:995-1024` および
`lib.rs:1100-1123`）:

1. `events::wire_events`（`data-action` 委譲。全構成で必須）
2. `keynav::wire_keynav`（`lib.rs:996`/`1101`。7,813 行、`match scope { "tabs" |
   "accordion" | "menu" | "select" | "radio" | "menubar" | "combobox" |
   "listbox" | "navigation-menu-trigger" | "navigation-menu-link" | ... }`
   の scope 別キーボード操作分岐、`keynav.rs:7552` 付近）
3. `focus_visible::wire_focus_visible`
4. `Self::wire_avatar` → `headless_avatar::wire_avatar_events`
5. `Self::wire_clipboard` → `headless_clipboard::wire_clipboard_events`
6. `Self::wire_timer` → `headless_timer::wire_timer_events`
7. `Self::wire_angle_slider`
8. `Self::wire_splitter`
9. `Self::wire_signature_pad` → `headless::wire_headless_component`
   （`MAPPING_TABLE` 経由の click → action dispatch。§7 参照）
10. `Self::wire_number_input`

一方、`crates/wasm-full/src/` は `overlay.rs` / `tooltip.rs` / `position.rs`
/ `focus_trap.rs` / `headless_file_upload.rs` / `headless_select.rs` を
`pub mod` として公開しているが、`Runtime::mount`/`hydrate` を含む
wasm-full の製品コード（非テスト）内にこれら 6 モジュールへの実呼び出しは
存在しない（`grep -rn "overlay::\|tooltip::\|position::\|focus_trap::"
crates/wasm-full/src/*.rs` の唯一のヒットは
`headless_clipboard.rs:397` のコメント中の言及のみ。`headless_file_upload::`
/ `headless_select::` は自ファイル外に呼び出し 0 件）。これらは
`Runtime::mount`/`hydrate` から到達しないアプリ側直接利用 API であり
（`examples/interactive-view-transitions/wasm/src/lib.rs` が実際に呼ぶ）、
`fandhe-frontend-wasm-full` を単体で `--release` ビルドしたとき wasm-ld の
到達性解析により**既に**最終バイナリから除外されている（§5 の変種 (e) は
このモジュール群を含まないため、この既存の tree-shaking の効果を
それ自体としては測っていない点に注意。§5 冒頭の注記参照）。

`--no-default-features` でのビルドは `wasm-bindgen-exports` を含む唯一の
feature を落とすため `entry.rs` のエクスポート自体が消え、
`wasm-bindgen --target web` の出力が事実上空同然になる。これは「最小構成の
計測」として無意味であり、`bundle_size.rs` のモジュール doc が定める
「dist-server が実際に生成するものと同一構成を計測する」契約とも矛盾する。
実装者は本評価が扱う「最小構成」を `--no-default-features` と誤読しない
こと。

## 3. 計測環境・再現手順

| 項目 | 値 |
|---|---|
| OS | macOS（Darwin 25.6.0） |
| rustc / cargo | 1.96.0（stable、`rust-toolchain.toml` 準拠） |
| wasm-bindgen-cli | 0.2.128 |
| twiggy | 未導入（§4 参照、減算ビルドで代替） |
| HEAD（`origin/main`） | `6ec7bcf8` |
| 計測日 | 2026-09-06 |

**注意**: ローカルツールチェーンは CI と一部バージョンが異なりうるため、
本文書中の実測値は絶対値ではなく変種間の**相対比較**として読む
（`docs/ci/wasm-opt-adoption-evaluation.md` と同じ注記）。

再現手順（scratch worktree 内、`--target-dir` を scratchpad 配下へ隔離）:

```bash
cargo build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown \
  --release --locked --target-dir <scratch>/target-X
wasm-bindgen --target web --no-typescript \
  --out-dir <scratch>/out-X \
  <scratch>/target-X/wasm32-unknown-unknown/release/fandhe_frontend_wasm_full.wasm
gzip -9 -c <scratch>/out-X/fandhe_frontend_wasm_full_bg.wasm | wc -c
gzip -9 -c <scratch>/out-X/fandhe_frontend_wasm_full.js | wc -c
```

各変種は `crates/wasm-full/src/lib.rs`（`Runtime::mount`/`hydrate` 内の
`wire_*` 呼び出し行）への一時パッチを適用してビルドし、計測後
`git checkout -- crates/wasm-full/src/lib.rs` で復元する。

## 4. 現状実測（全部品有効、HEAD、ベースライン (a)）

| ファイル | raw | gzip |
|---|---:|---:|
| `fandhe_frontend_wasm_full_bg.wasm` | 541,425 B | 190,630 B |
| `fandhe_frontend_wasm_full.js` | 48,100 B | 9,096 B |
| **合計** | 589,525 B | **199,726 B**（[再計測]） |

CI main 記録値 193,731 B（[記録値]）とは約 6,000 B の差があるが、これは
ローカルとホストランナーのツールチェーン差（rustc パッチバージョン・
wasm-bindgen-cli の生成コード差分）に由来する差と考えられ、
`docs/ci/wasm-opt-adoption-evaluation.md` が同様に注記する「オーダーの
一致で計測対象として妥当と判断する」方針に従い、変種間の相対比較の
基準として本値（199,726 B）を用いる。

twiggy は本評価では未導入とした（バージョン固定の追加ツール導入コストに
対し、§5 の減算ビルド〔`wire_*` 呼び出しを直接コメントアウトしてビルド
差分を測る〕で目的の情報（配線群別の寄与度）が十分な精度で得られたため。
導入を要する場合は `cargo install twiggy --version <pin> --locked` で
バージョン固定すること）。

## 5. 最小構成実測（使い捨てパッチによる減算ビルド）

各変種は `Runtime::mount`/`Runtime::hydrate` 双方の該当呼び出し行を
同時にコメントアウト/削除し、`hydrate`/`start_router` のエクスポート自体
（`entry.rs` の 3 関数）は変えていない（ルート到達性は変えず、ルートから
到達する配線コードのみを減らしている）。

| 変種 | 内容 | wasm gzip | js gzip | 合計 | 対 (a) 削減率 |
|---|---|---:|---:|---:|---:|
| (a) ベースライン | HEAD（全 `wire_*` 有効） | 190,630 | 9,096 | 199,726 | — |
| (b) keynav 除去 | `keynav::wire_keynav` 呼び出しのみ削除 | 162,383 | 8,892 | 171,275 | **14.2%** |
| (d) button/input/dialog 相当 | `events::wire_events` + `Self::wire_signature_pad`（`MAPPING_TABLE` 経由の click → action dispatch、dialog/collapsible/popover/tooltip/menu 等の trigger toggle を含む）のみを残し、`keynav`/`focus_visible`/`wire_avatar`/`wire_clipboard`/`wire_timer`/`wire_angle_slider`/`wire_splitter`/`wire_number_input` を除去 | 135,305 | 7,798 | 143,103 | **28.4%** |
| (e) 理論下限 | `events::wire_events` のみ残し、`keynav`/`focus_visible`/`wire_avatar`/`wire_clipboard`/`wire_timer`/`wire_angle_slider`/`wire_splitter`/`wire_signature_pad`/`wire_number_input` を全除去 | 128,170 | 7,491 | 135,661 | **32.1%** |

（いずれも [再計測]。`wasm-opt` は適用していない — これは REQ-11 の実際の
計測経路〔`bundle_size.rs`〕そのままの構成であり、#1969/#1970 が扱う
`wasm-opt -Os` 適用列はここでは意図的に測っていない副次論点として扱う。)

**読み取り**: `keynav::wire_keynav` 単体で 28,451 B（14.2%）を占める。これは
`keynav.rs` が 7,813 行に及ぶ scope 別（tabs/accordion/menu/select/radio/
menubar/combobox/listbox/navigation-menu-* 等、10 種類超）キーボード操作
分岐を 1 つの関数に集約しているためで、gating の効果が最も大きい単一
箇所である。(d)（button/input/dialog 相当。`MAPPING_TABLE` 経由の
click dispatch のみ残す構成）は (b) からさらに `focus_visible`/
`wire_avatar`/`wire_clipboard`/`wire_timer`/`wire_angle_slider`/
`wire_splitter`/`wire_number_input` を除いても追加で 28,172 B
（14.1pt）しか縮まず、(e)（理論下限、`wire_signature_pad` も除去）との
差は 7,442 B（3.7pt）に留まる。すなわち keynav 除去が支配的なレバーで
あり、残りの 8 関数は個々には小さいが積算すると無視できない規模になる。

(c)（`wire_avatar`〜`wire_number_input` を個別に 1 つずつ外す 8 変種）は、
(b)・(d)・(e) の 3 点で「keynav 除去が支配的」「残りは積算で 14pt 程度」の
傾向が既に明確であり、判定ルール（§11）に照らした結論を変える追加情報を
持たないと判断し、本評価では省略した（採用判断後、実装 issue（§13）側で
配線群別の feature 単位を確定する際に必要なら再測定する）。

## 6. feature 分割の粒度案

| 案 | 単位 | 削減効果（§5 実測との対応） | 機械検証コスト | 利用者の指定しやすさ |
|---|---|---|---|---|
| (i) headless-ui 63 部品別 | `crates/headless-ui/` の部品単位（Avatar/Checkbox/... 63 種） | 薄い: wasm-full が実際に import するのは 8〜10 モジュールのみで、大半の部品は既に到達閉包外（§2）。63 feature を作っても大部分は wasm-full 側で何も削れない | 高い: 63 通りの feature 組合せは CI matrix で現実的に全数検証できない | 低い: 利用者は「使う部品」単位で選びたいが、実際の削減は wasm-full 側の配線（keynav 等）に依存するため、部品単位の選択と削減量が一致しない |
| (ii) wasm-full の配線群別（`wire_*` 単位 + keynav の scope 単位、≈10〜20 個） | `Runtime::mount`/`hydrate` の `wire_*` 呼び出し・`keynav::wire_keynav` 内の scope 分岐 | 高い: §5 の実測がそのまま対応する。keynav 単体で 14.2%、全体で 32.1%（理論下限） | 中: 10〜20 個の feature は組合せ数が現実的（`--no-default-features` + 全 feature + 代表 2〜3 通りで足りる） | 中: 「tabs は使うが combobox は使わない」等、利用者が UI パターン単位で判断しやすい |
| (iii) カテゴリ別（forms / overlay / navigation / input 系等） | (ii) を意味カテゴリでグルーピング | 中: (ii) の粒度を粗くした形。個別最適はできないが大枠の削減は可能 | 低: feature 数が少なく matrix も小さい | 高: カテゴリ名は利用者にとって直感的 |

§2 で是正した前提（wasm-full が実際にリンクするのは headless-ui の一部
モジュールのみ）から、(i) は「削減効果」列が示すとおり効果と対応関係が
薄い。自然な単位は (ii) または (iii) であり、§5 の実測（keynav が単独で
最大の削減源）はこれを裏付ける。

## 7. `MAPPING_TABLE` / keynav の条件コンパイル方法

`crates/wasm-full/src/headless.rs:84` の `const MAPPING_TABLE: &[MappingRow]
= &[ ... ]`（`MappingRow` は `scope`/`part`/`action`/`requires_value` の
4 フィールド、`headless.rs:69`）の各要素へ `#[cfg(feature = "...")]` を
付与する方式は、最小コンパイル確認（`cargo check -p fandhe-frontend-wasm-full
--target wasm32-unknown-unknown` に一時 feature
`measure-cfg-test`〔`crates/wasm-full/Cargo.toml`〕を追加し、1 行目の
`MappingRow` エントリへ `#[cfg(feature = "measure-cfg-test")]` を付けて
実施）で **stable Rust で問題なく通ることを確認した**（配列リテラル要素へ
の `#[cfg]` はスタンダードな stable 機能）。確認後は変更を破棄済み
（`git checkout -- crates/wasm-full/src/headless.rs crates/wasm-full/
Cargo.toml`、コミットに含めない）。

`keynav.rs:7552` の `match scope { "tabs" => ..., "accordion" => ..., ... }`
の各 arm へ同様に `#[cfg(feature = "...")]` を付ける場合も同じ機構が
使える（match arm への `#[cfg]` も stable）。ただし arm を cfg で落とすと
`scope` の型（`&str`）に対する match は既定で非網羅にならない（フォール
スルー先の `_` arm が別に存在する設計であれば安全）。

**テスト側への影響**: `crates/wasm-full/tests/headless_wiring.rs` /
`keynav_native.rs`（ドリフト検知テスト、両ファイルとも本評価では未変更・
未確認の詳細レベルまでは踏み込んでいない）は、feature 集合を絞った
ビルド構成でも同じ対象を検証し続けられるよう、`required-features` 指定
または `#[cfg(feature = "...")]` によるテスト自体の条件コンパイルで
feature 集合に追随させる必要がある。これは採用時の実装 issue（§13-2）の
スコープとする。

## 8. REQ-11 計測対象との整合

gating を採用しても、`crates/dist-server/build.rs` のネストビルドが同じ
縮小 feature 集合を選ばない限り**配布物の実サイズは減らない**。「計測
だけ縮小して配布物は据え置き」は REQ-11 ゲートの形骸化であり、採用する
場合は避けなければならない。選択肢は 2 つ:

- **(A)** dist-server 経路の feature 集合を「最小インタラクティブ
  コンポーネント」の定義に合わせて縮小し、`bundle_size.rs` の計測構成も
  同じ feature 集合に更新する。実配布物と計測値が一致し続ける。
- **(B)** 計測（`bundle_size.rs`）は現行どおり全部品有効のまま維持し、
  gating は wasm-full を直接利用する下流アプリ（例:
  `examples/interactive-view-transitions/wasm`）側の payload 削減にのみ
  効かせる。REQ-11 ゲート自体には寄与しない。

上限値（200,000 B）自体は spec（`docs/spec/04-requirements.md`）所有で
あり本リポジトリでは変更しない（親 #1953 の既定方針）。自動運転のため
本評価では (A)/(B) いずれかへの決定は行わず、推奨（§11）のみ記す。

## 9. `intentional-non-adoption.md` §2 の 4 軸評価

（`docs/policy/intentional-non-adoption.md` の評価軸に基づく。対象は
「(ii) wasm-full の配線群別 feature 分割」案。）

- **明示性**: Cargo feature は `Cargo.toml` に宣言的に現れ、有効/無効が
  ビルドコマンドから読み取れる点で明示的。一方、gating 済み scope への
  操作（例: `keynav` feature を無効化した状態でキーボード操作を行う）は
  `MAPPING_TABLE`/keynav の既存 fail-closed 契約（未知の (scope, part) は
  `None`、`data-disabled`/`data-readonly` は no-op）と同じ経路で黙って
  no-op になる。「feature を切ったら何が起きなくなるか」は Cargo.toml の
  コメントに頼ることになり、実行時エラーでは検出できない DX コストが
  ある。
- **決定性**: feature unification（`cargo build -p` 単体と `cargo test
  --workspace` で有効 feature 集合が変わりうる Cargo の既知挙動）により、
  ワークスペース全体のテストでは常に全 feature が事実上有効化される
  リスクがある。バンドルサイズの安定性は `dist-server` の `-p` 単体ネスト
  ビルド（`crates/dist-server/build.rs`）にのみ依存し、これは既存の
  `bundle_size.rs` 契約と同型のため、決定性の悪化は限定的。
- **機械検証可能性**: `--no-default-features` ビルド・feature 別ビルド・
  `--all-features` ビルドを CI matrix 化すれば検証可能。§7 のとおり
  `MAPPING_TABLE`/keynav の cfg 化自体は stable で機械的に扱える。ただし
  `clippy-wasm32` ジョブ（`.claude/rules/ci.md`）を matrix 化する追加コスト、
  および `headless_wiring.rs`/`keynav_native.rs` のドリフト検知テストの
  feature 追随（§7）が新規の保守対象になる。
- **コンテキスト消費**: 粒度 (ii)（10〜20 個）は 63 部品別 (i) や
  カテゴリ別 (iii) と比べ、AI エージェントが「このアプリで何を有効化
  すべきか」を判断する際に読む必要のある feature 数として中程度。
  `Runtime::mount`/`hydrate` の呼び出し列（§2 の 10 項目）と 1 対 1 対応
  させれば、コード読解と feature 一覧の対応が機械的に取れる利点がある。

## 10. CI・テスト・semver・利用者への影響

- 既定 on の feature 追加（`default = [...]` へ新 feature を追加しつつ
  既定で全部有効のままにする）は追加的変更であり、`fandhe-frontend-wasm-full`
  の patch/minor バンプで足りる（既存利用者のビルド結果は変わらない）。
- `examples/interactive-view-transitions/wasm`（`fandhe-frontend-wasm-full
  = "0.7.0"` に pin、`examples/interactive-view-transitions/wasm/Cargo.toml:37`）
  は既定 feature をそのまま使う限り非影響。§2 で確認したとおり、この
  example は `overlay`/`tooltip`/`position`/`focus_trap` を直接呼ぶ唯一の
  消費者であり（`examples/interactive-view-transitions/wasm/src/lib.rs:77`
  の `use fandhe_frontend_wasm_full::overlay::{...}`・`lib.rs:80` の
  `use fandhe_frontend_wasm_full::position::PositionController` で実際に
  import・使用していることを確認済み）、gating の feature 設計時にこれらのモジュールを
  「配線群」の対象に含めるかどうか（Runtime 側ではなく個別公開 API
  のため対象外が妥当）を明確にしておく必要がある。
- `crates/docs-site` は wasm-full に依存しない（JS ハイドレーションなし）。
  `templates/app/wasm` と `bench/csr/fandhe` は `fandhe-frontend-wasm-client`
  のみに依存し wasm-full を経由しない。したがって gating 導入時の影響範囲は
  `examples/interactive-view-transitions/wasm` と dist-server 経路（§8）に
  閉じる。
- `bundle-size` ジョブ（`.claude/rules/ci.md` の `ci-complete` 集約対象）は
  §8 の (A)/(B) いずれを選んでも設定変更を要する可能性がある。
  `clippy-wasm32` ジョブは feature matrix 化する場合コストが増える
  （§9 参照）。`forbid-unsafe` ジョブへの影響はない（wasm-full/wasm-client
  はいずれも `unsafe` 境界クレートであり feature 分割はこの境界を動かさ
  ない）。

## 11. 採否判定

**事前登録した判定ルール**: (d) 相当の最小構成（button/input/dialog 系の
み）が現状比 20% 以上、かつ配線群別分割で REQ-11 上限に対し 30 KB 以上の
余裕が確保できるなら採用候補。10% 未満なら見送り（#1969/#1970 の
wasm-opt を優先）。

**実測との照合**: 判定ルールが対象とする (d)（button/input/dialog 相当の
最小構成）は現状比 **28.4%** の削減（199,726 → 143,103 B）であり、判定
ルールの 20% を明確に上回る。理論下限 (e) は 32.1%、keynav 単体除去 (b)
でも 14.2% 削減する。REQ-11 上限（200,000 B）に対する余裕は、(d) 構成
なら 200,000 − 143,103 = 56,897 B、(e) 構成なら 200,000 − 135,661 =
64,339 B となり、いずれも判定ルールの「30 KB 以上」を満たす（keynav
単体除去 (b) のみだと 200,000 − 171,275 = 28,725 B で 30 KB をわずかに
下回るが、判定ルールが指すのは (d) であり (b) 単独ではない）。

**結論: 条件付き採用（レバー (ii) を推奨）**。以下を条件とする:

1. 分割粒度は §6 の (ii)（wasm-full の配線群別、10〜20 個）を採る。
   (i)（headless-ui 63 部品別）は§2/§6 で示したとおり効果が薄く、機械
   検証コストのみ増えるため不採用とする。
2. §8 の (A)/(B) のいずれを採るかはユーザー判断とし、本評価では推奨
   のみ記す: dist-server 配布物の実削減を伴わない (B) は REQ-11 の
   余裕確保という親 #1953 の目的に直接は寄与しないため、**(A)（dist-server
   経路の feature 集合を縮小し計測も同一構成に保つ）を推奨**する。
   ただし (A) は「最小インタラクティブコンポーネント」の定義（どの
   `wire_*` を含めるか）を利用者向けドキュメントとして明文化する追加
   コストを伴う。
3. §7 の cfg 機構は stable コンパイル確認済みだが、§9 のテスト追随
   コスト（`headless_wiring.rs`/`keynav_native.rs`）は実装 issue（§13）
   側で必ず解消すること。

## 12. 再評価トリガー

- `bundle-size` が #1968 の警告しきい値（190,000 B）を恒常的に超える。
- wasm-opt 導入（#1970）後も REQ-11 上限への余裕が 10 KB 未満のまま。
- headless-ui への部品追加ペースが再加速し、wasm-full が新規にリンクする
  モジュールが増える（§2 の到達閉包が拡大する）。
- 利用者から具体的な feature 選択要望が寄せられる。

## 13. 採用時の実装 issue 分割案

条件付き採用（§11）に基づき、採用する場合の分割案:

1. `wasm-full` に配線群別 feature を追加（既定 on、`Runtime::mount`/
   `hydrate` の `wire_*` 呼び出しを cfg ゲート）。
2. `MAPPING_TABLE`/keynav の scope 分岐の cfg 化と、
   `headless_wiring.rs`/`keynav_native.rs` の `required-features` 追随。
3. CI feature matrix（`--no-default-features` / 各 feature / `--all-features`）
   の追加。`clippy-wasm32` ジョブへの反映要否を含めて検討する。
4. dist-server 経路の feature 集合決定（§8 (A)/(B) のユーザー判断）と
   `bundle_size.rs` 契約の更新。
5. docs（feature 一覧の利用者向けドキュメント化）・examples への反映。

見送りとなった場合はこれらの issue は起票しない。

## 14. セキュリティ考慮事項（OWASP Top 10 観点）

- **A06 脆弱で古いコンポーネント/サプライチェーン**: 本評価は製品依存を
  追加していない（REQ-3 不変）。twiggy は未導入（§4）。実測に使った一時
  feature（`measure-cfg-test`）・パッチはすべて `git checkout --` で
  破棄済みで、コミットに含めていない。ビルドは `--locked` で行い
  `Cargo.lock` の意図しない更新を防いだ。
- **A05 セキュリティ設定ミス**: 採用時の設計として、feature gating は
  `MAPPING_TABLE`/keynav の既存 fail-closed 契約（未知の (scope, part) は
  `None`、`data-disabled`/`data-readonly` は no-op）を弱めない。gating
  済み scope は「配線されない＝no-op」であり、エスケープ迂回や新たな
  dispatch 経路を作らない不変条件を維持すること（§9 明示性の項参照）。
- **A03 インジェクション/REQ-1**: 既定エスケープ経路
  （`fandhe_frontend_core::render`・`set_text_content`）に本評価で提案する
  設計は一切触れない。
- **A01 アクセス制御/パストラバーサル**: 実測の `--target-dir`/scratch
  worktree は scratchpad 配下の固定パスに限定し、`rm -rf` の広域削除や
  glob を使わなかった。
- **機微情報**: 環境表・ログにトークン・ホームディレクトリ絶対パス等は
  含めていない（`docs/reports/wasm-payload-baseline-1406.md` と同じ粒度）。
- **REQ-11 ゲートの形骸化防止**: 「計測構成だけを縮小して PASS させる」
  変更は本評価では行っておらず、§8・§11 で採用時も dist-server 配布物と
  計測の同一構成契約を維持する選択肢（(A)）を明示的に推奨した。

## 15. 参照

- `docs/spec/04-requirements.md`（REQ-11）
- `crates/wasm-full/tests/bundle_size.rs`（計測契約・警告しきい値）
- `docs/ci/wasm-opt-adoption-evaluation.md`（#1327、payload 削減の別レバー）
- `docs/ci/wasm-allocator-adoption-evaluation.md`（#1389、アロケータ差し替え見送り）
- `docs/policy/intentional-non-adoption.md`（4 軸評価の枠組み）
- `crates/wasm-full/src/lib.rs`（`Runtime::mount`/`hydrate` の配線列）
- `crates/wasm-full/src/headless.rs`（`MAPPING_TABLE`）
- `crates/wasm-full/src/keynav.rs`（scope 別キーボード操作分岐）
