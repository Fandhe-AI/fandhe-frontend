# 横断 a11y 自動検証（axe-core 相当）導入評価（イシュー #1076）

## 1. 背景とトレーサビリティ

- 親 #1058（WAI-ARIA 準拠のブラッシュアップ Phase）／祖 #1056。
- `crates/headless-ui/`（63 部品）の WAI-ARIA 検証は各部品の inline テスト
  （`src/*.rs` の `#[cfg(test)]`）と `tests/<module>.rs` に散在しており、「63 部品
  すべてが最低限の ARIA 契約を満たしていること」を横断で強制する仕組みが存在
  しない。部品を新設したとき、ARIA アサーションを書き忘れても CI は素通りする
  （機械検証可能性のギャップ）。イシューはこの横断検証を axe-core 相当のツール
  導入で埋めるべきかを評価し、採用 / 見送りを結論づけることを求めている。
- イシュー本文の受け入れ条件 3 件と、本書がどれを満たすかの対応:

| # | 受け入れ条件 | 対応節 |
|---|-------------|--------|
| 1 | 候補手段（axe-core + ブラウザハーネス / Rust 製静的検証の自前実装 / 契約テスト拡充のみ）の比較評価を文書化 | §4 |
| 2 | 依存追加を伴う案はユーザー承認必須である旨を明記し、結論を `docs/ci/` または `docs/policy/` へ記録 | §8（承認境界の明記）、本書自体が `docs/ci/` への記録 |
| 3 | 調査のみでコード変更を行わない | 本 PR は `crates/*/src/**`・`.github/workflows/**` を一切変更しない（`git diff --stat` で確認可能） |

- **記録先を `docs/ci/` とした根拠**: `docs/policy/intentional-non-adoption.md` に
  `cargo-semver-checks` への言及は存在しない（`grep -rn 'cargo-semver-checks'
  docs/policy/intentional-non-adoption.md` は 0 件）。同文書 §3 は製品・アーキ
  テクチャ上の非採用（仮想 DOM / signal / UI 部品）の記録場所であり、CI ツーリ
  ングの導入評価は `docs/ci/`（先例: `docs/ci/cargo-semver-checks-evaluation.md`、
  イシュー #656）に置くのが本リポジトリの確立した型である。イシュー本文が挙げる
  「`intentional-non-adoption.md` 型の評価軸」は同文書 §2 の 4 軸を**流用せよ**
  という意味であり、§3.x エントリを追加せよという意味ではないと解釈する。

## 2. 評価軸

`docs/policy/intentional-non-adoption.md` §2 の 4 軸を本件の文脈で解釈すると:

- **明示性**: 検査項目・却下理由が読み手（人間・AI）に一目で分かるか
- **決定性**: 同一入力に対し常に同一の合否が出るか（ヘッドレスブラウザの起動
  順・タイミング依存で揺れないか）
- **機械検証可能性**: CI で自動判定できるか（目視レビュー依存を減らせるか）
- **コンテキスト消費**: 導入・保守に要する追加の認知コスト（依存追随・バージョ
  ン pin 管理・失敗時のデバッグ手順）

本件固有に以下 2 軸を追加する:

- **依存グラフ上限**（REQ-3: 標準サーバー構成で依存パッケージ 60 件以内・深さ
  6 以内、`docs/policy/dependency-graph-policy.md`）
- **サプライチェーン**（OWASP A08、`.claude/rules/security.md`）: 本リポジトリ
  は CI ツールを「バージョン固定 + SHA256 チェックサム検証済みプリビルトバイ
  ナリ」パターンに統一している（`tools/ci/ensure-gate-tools.sh` が cargo-deny
  で先例）

## 3. 現状の集計

- **計測日**: 2026-07-27
- **base commit**: `a40ee04`（本ブランチの調査時点 HEAD。`main` 未マージの
  stacked commit を含み、`origin/main` の実コミットではない）

### 3.1 計測方法

```bash
# (0) 台帳の module 一覧（63 件）を抽出
grep -o 'module: "[a-z0-9_]*"' crates/docs-site/src/primitives_catalog.rs \
  | sed 's/module: "//;s/"//' | sort -u

# (1) src 側が role / aria-* を出力しているか（aria.rs ヘルパ経由・直書きの両方）
grep -lE 'aria::|"aria-|"role"' crates/headless-ui/src/<module>.rs

# (2-a) src 内の inline #[cfg(test)] 節が (1) の出力をアサートしているか
awk '/#\[cfg\(test\)\]/{f=1} f' crates/headless-ui/src/<module>.rs \
  | grep -cE 'aria-|"role"|role='

# (2-b) tests/<module>.rs が存在し、同アサーションを持つか
grep -cE 'aria-|role=' crates/headless-ui/tests/<module>.rs
```

計測は **per-module の突合**とした。`grep -rl` によるファイル件数集計は弱い
代理指標であり不採用とした（`src/*.rs` には inline `#[cfg(test)]` を持つモジュ
ールが多数あり、`tests/*.rs` と重複カウント・取りこぼしの双方が起きるため）。

### 3.2 部品別 ARIA 検証カバレッジ表（63 部品）

`yes`/`no` は src が role・aria-* を出力するか、数値は該当箇所のマッチ行数
（`NOFILE` は `tests/<module>.rs` 自体が存在しない）。

| module | src が role/aria-* を出力 | inline test アサーション行数 | tests/\<module\>.rs アサーション行数 |
|---|---|---|---|
| accordion | yes | 15 | 9 |
| action_bar | yes | 4 | NOFILE |
| angle_slider | yes | 7 | NOFILE |
| avatar | no | 0 | 0 |
| breadcrumb | yes | 9 | NOFILE |
| calendar | yes | 10 | 2 |
| carousel | yes | 17 | 4 |
| checkbox | yes | 7 | 0 |
| checkbox_group | yes | 10 | 3 |
| clipboard | no | 0 | NOFILE |
| collapsible | yes | 6 | 2 |
| color_picker | yes | 13 | NOFILE |
| combobox | yes | 30 | 41 |
| date_input | yes | 8 | 3 |
| date_picker | yes | 3 | 2 |
| dialog | yes | 19 | 6 |
| download_trigger | no | 0 | NOFILE |
| drawer | yes | 11 | 6 |
| editable | no | 0 | 0 |
| field | yes | 11 | 13 |
| fieldset | yes | 6 | 3 |
| file_upload | yes | 2 | 1 |
| floating_panel | yes | 8 | NOFILE |
| hover_card | yes | 6 | 5 |
| image_cropper | yes | 12 | NOFILE |
| json_tree_view | yes | 10 | NOFILE |
| link | yes | 2 | NOFILE |
| link_overlay | no | 0 | NOFILE |
| listbox | yes | 19 | 32 |
| menu | yes | 44 | 30 |
| menubar | yes | 34 | 7 |
| nav_list | yes | 7 | NOFILE |
| navigation_menu | yes | 19 | 8 |
| number_input | yes | 13 | 5 |
| pagination | yes | 6 | 3 |
| password_input | yes | 7 | 3 |
| pin_input | yes | 2 | 2 |
| popover | yes | 13 | 7 |
| progress | yes | 13 | 9 |
| qr_code | yes | 1 | 3 |
| radio_group | yes | 7 | 5 |
| rating_group | yes | 11 | 3 |
| scroll_area | yes | 3 | NOFILE |
| segment_group | yes | 9 | 6 |
| select | yes | 25 | 16 |
| signature_pad | yes | 1 | NOFILE |
| skip_nav | no | 0 | NOFILE |
| slider | yes | 10 | NOFILE |
| splitter | yes | 10 | NOFILE |
| steps | yes | 5 | NOFILE |
| switch | yes | 3 | 2 |
| tabs | yes | 36 | NOFILE |
| tags_input | yes | 14 | 2 |
| timer | no | 0 | NOFILE |
| toast | yes | 7 | 2 |
| toggle | yes | 3 | NOFILE |
| toggle_group | yes | 9 | NOFILE |
| toggle_tip | yes | 15 | 9 |
| toolbar | yes | 17 | 7 |
| tooltip | yes | 10 | 6 |
| tour | yes | 4 | 2 |
| tree_view | yes | 46 | NOFILE |
| visually_hidden | yes | 1 | NOFILE |

「ARIA 出力があるのに一切アサートされていない module」は **0 件**である
（src が role/aria-* を出力する 56 モジュールすべてで inline test アサーショ
ン行数が 1 以上）。src が role/aria-* を出力しない 7 モジュール（avatar /
clipboard / download_trigger / editable / link_overlay / skip_nav / timer）は
モジュール doc コメントで「ark-ui / Zag.js も追加の `role`/`aria-*` を付与しな
い」「本モジュールも追加の `role`/`aria-*` を付与しない」等と明記されており、
native semantics（`<button>` / `<a>` 等）で足りるとする意図的な判断であって
検証漏れではない。

この計測結果は「ARIA アサーションの書き忘れが横断で放置されている」という
イシューの懸念仮説を、少なくとも現時点の 63 部品については裏付けない。ただし
これは**将来の新設部品でも同様であることを機械的に保証しない**（今回の実測は
既存 63 部品の目視相当の突合であり、CI ゲートではない）。この「保証の不在」こそ
が本イシューが埋めるべき真のギャップである（§3.4）。

### 3.3 既に横断で強制されている機械的契約の一覧

- `crates/docs-site/tests/primitive_showcase.rs`: Anatomy 網羅・scope 一致・
  headless-ui ソース全走査によるパート網羅の双方向 fail-closed 検証
- `crates/docs-site/tests/primitives_catalog.rs`: 台帳（63 部品）と headless-ui
  ソースのドリフト検知
- `crates/docs-site/tests/wrap_state.rs`（PR #1096、イシュー #1064）: Primitives
  63 部品 / Themes 107 部品のラップ状態（層をまたぐ対応関係）を 4 バケット分割
  で機械可視化し、`docs/policy/intentional-non-adoption.md` §3.25 に反する
  「本来 headless をラップすべき部品の独自実装」がレビューをすり抜けるのを防ぐ
- `crates/headless-ui/tests/menu_menubar_aria_contract.rs`（PR #1091、イシュー
  #1068 相当）: menubar と menu の ARIA 語彙一致を横断契約テストで固定
- `crates/headless-ui/tests/xss_escape.rs`: 既定エスケープ（REQ-1）の回帰検証
- `crates/wasm-full/tests/keynav_browser.rs` / `focus_trap_browser.rs` 等:
  キーボード操作・フォーカス管理の実ブラウザテスト

### 3.4 横断で強制されていない検査項目（真のギャップ）

- 新設部品が role/aria-* を出力するのにテスト（inline / `tests/<module>.rs`）
  でアサートしなかった場合に **CI が機械的に検知する仕組み**が存在しない
  （§3.2 の集計は本評価作成時点の手動突合であり、継続的なゲートではない）
- 同一ツリー内の `id` ↔ `aria-controls` / `aria-labelledby` /
  `aria-activedescendant` の参照整合性を横断で検証する仕組みがない（個別部品
  ごとの契約テストに委ねられている）
- 必須 ARIA 属性の欠落（role に対する ARIA in HTML 仕様上必須の属性が揃って
  いるか）を横断で検証する仕組みがない

## 4. 候補手段の比較評価

### 4.1 案 A: axe-core + ブラウザハーネス（3 変種）

| 変種 | 内容 | 却下根拠（一次ソース） |
|------|------|------|
| A-1 | Node.js + Playwright/puppeteer で axe-core を実行 | `docs/guides/browser-testing.md:38-39`「代替案（Playwright ベース E2E）は Node.js 依存・依存面拡大が大きく、仕様の第一指名が `wasm-pack test` であるため v1 では不採用」。本件は**既存決定の継承**であり、新たな判断の発明ではない |
| A-2 | `tools/npm-asset-build/install.sh` 経由で axe-core を npm から取り込む | `tools/npm-asset-build/check_static_only.py:83`（`HARD_DENY_EXTS = {".js", ".mjs", ".cjs", ".node", ".wasm"}`）・`:165`・`:424-427`（`.min.js` を `.js` として明示ハード拒否）。**サンクションされた npm 取り込み経路は axe-core（実行コード）を構造的に受け付けない**（allowlist 方式・既定拒否、REQ-12） |
| A-3 | `axe.min.js` をリポジトリへ vendor し、既存 `browser-test` 系ジョブから注入・実行 | (a) `docs/guides/browser-testing.md:49,99`「第三者製 action の新規追加なし」「実行時ダウンロードを封じる」姿勢と、第三者製ミニファイ JS を CI 内で実行する行為が非整合（OWASP A08）。(b) vendor した JS の CVE・WAI-ARIA 仕様追随が人手依存になり、コンテキスト消費軸で不利。(c) cargo-deny / `cargo audit` の監査対象外となる（Rust 依存ではないため） |

### 4.2 案 B: Rust 製静的検証の自前実装

**適合点**: `fandhe_frontend_core::Node`（`crates/core/src/lib.rs:123-143`）は
`Element { tag, attrs, children }` / `Text(String)` / `RawHtml(String)` の
公開 enum であり、**HTML パーサを持ち込まずにノード木を直接走査できる**。外部
依存ゼロ（REQ-3・core 依存ゼロ方針と無矛盾）で、決定性・機械検証可能性の軸で
最良。検査対象コーパスは `crates/docs-site/src/primitive_showcase/`
（Primitives 63 部品の Demo）と `src/showcase.rs`（Themes 107 部品）が既存で
全部品を描画済みであり、新規に用意する必要がない。

#### 4.2.x 案 B で到達できない検査カテゴリ

- コントラスト比（CSS 計算値が必要）
- フォーカス順序・タブ順（レイアウト・DOM 実体が必要）
- accessible name computation の完全実装（`aria-labelledby` の連鎖・`title`
  フォールバック・CSS 生成コンテンツ）
- ランタイム状態遷移後の ARIA（`data-state` 変化に伴う `aria-expanded` 追随。
  これは既存の `crates/wasm-full/tests/*_browser.rs` 群が担う領域）

到達可能な範囲は role 出現・`aria-*` 語彙の妥当性・同一ツリー内の `id` ↔
`aria-controls` / `aria-labelledby` / `aria-activedescendant` の参照整合性・
必須 ARIA 属性の欠落に限られる。**案 B は案 A の代替ではなく、静的に判定可能な
部分集合の機械化である**（過大主張を避ける）。

### 4.3 案 C: 契約テスト拡充のみ

親 #1058 配下で本イシュー着手時点までに以下が既にマージ済みであり、C は
新規提案ではなく**マージ済みベースラインからの delta**として記述する。

| PR | 内容 |
|----|------|
| #1097 | combobox の `aria-controls` / `aria-activedescendant` opt-in 欠落を契約で防ぐ |
| #1091 | menubar のロール・状態出力が menu と同語彙であることの横断契約テスト（`menu_menubar_aria_contract.rs`） |
| #1092 | combobox / tags_input への `aria-live` 追加 |
| #1096 | `crates/docs-site/tests/wrap_state.rs` — headless 63 / pre-styled 107 のラップ状態を機械可視化する契約テスト |

これらに加える delta として残る作業は、§3.4 が示す「新設部品の ARIA アサー
ション書き忘れを CI が機械検知する仕組み」であり、これは案 B（の縮小版、
xtask サブコマンドまたは `crates/docs-site/tests/` への新規テスト）でのみ
埋められる。契約テストの手動拡充だけでは §3.4 の恒常的なギャップは解消しない。

### 4.4 評価軸マトリクス

| 軸 | A-1 (Playwright) | A-2 (npm 取込) | A-3 (vendor 注入) | B（自前静的検証） | C（契約テスト拡充のみ） |
|---|---|---|---|---|---|
| 明示性 | 中 | 低（allowlist 拒否で不可視化） | 低 | 高 | 高 |
| 決定性 | 中（ブラウザ起動依存） | N/A（導入不可） | 中 | 高 | 高 |
| 機械検証可能性 | 高（ただし到達範囲が広すぎる） | N/A | 高 | 中（§4.2.x の範囲に限定） | 低（手動拡充依存） |
| コンテキスト消費 | 高（Node.js・依存追随） | N/A | 高（vendor 更新） | 中（新規実装コスト） | 低 |
| 依存グラフ上限 | 抵触しうる（Rust 側は不変だが CI 面が拡大） | N/A | 抵触せず（Rust 依存に非ず） | 抵触せず | 抵触せず |
| サプライチェーン | 既決不採用と非整合 | 構造的に不可 | OWASP A08 抵触 | 最良（依存ゼロ） | 最良（依存ゼロ） |

## 5. 結論

**axe-core 相当の外部 a11y 検証ツールの導入は、現時点では見送る。**

根拠:

1. サンクションされた npm 取り込み経路（`tools/npm-asset-build/`）は実行コー
   ド拡張子を allowlist 方式で構造的に拒否しており、axe-core の取り込みには
   経路自体の変更（REQ-12 の既定拒否原則の緩和）が必要になる
2. Playwright 系ハーネス（A-1）は `docs/guides/browser-testing.md` で既に v1
   不採用と決定済みであり、本評価はその決定を a11y ツールへ継承するに留まる
3. vendor 注入（A-3）は依存グラフ上限（REQ-3）には直接抵触しないものの、
   CI ツールの「バージョン固定 + SHA256 検証済みプリビルトバイナリ」統一パター
   ンおよび「第三者製 action・実行時ダウンロードの新規追加なし」方針と非整合
   （OWASP A08）
4. 動機である「ARIA アサーション書き忘れの横断検知」の主要部分（role/aria-*
   語彙の妥当性・id 参照整合性・必須属性欠落）は、外部依存ゼロの静的検証
   （案 B）と既存契約テスト（案 C）の組み合わせで到達可能である
5. §3.2 の実測は、現時点の 63 部品について「ARIA 出力があるのに一切アサート
   されていない」モジュールが 0 件であることを示しており、懸念されていた
   重大なカバレッジ欠落は現状では確認されなかった

当面の担保は案 C（§4.3 の delta、既にマージ済みの契約テスト群を継続拡充）
とし、将来的に案 B（xtask サブコマンドまたは docs-site テストとしての Rust
製静的検証、外部依存を伴わない）を条件付きで検討する。

## 6. 再評価トリガー

- ARIA 契約の欠落に起因する不具合が、リリース済みバージョンで 1 件でも報告
  された場合
- §3.2 の表でゼロアサーション部品（ARIA 出力があるのに一切アサートされてい
  ない module）が新規部品追加により発生し、かつ恒常的に増加傾向（目安: 四半
  期で 5 部品超）となった場合
- コントラスト比・フォーカス順序など**案 B が構造的に到達できない検査カテゴ
  リ**（§4.2.x）に起因する不具合が発生した場合（この場合のみ案 A の再評価に
  価値が生じる）
- axe-core 相当ツールに、Node.js ランタイムを要さず checksum 検証済みプリビ
  ルトバイナリで配布される実装が現れた場合

## 7. 導入する場合の実装方式メモ（将来の再評価用参考）

案 B を将来実装する場合の設計メモ（**本イシューでは実装しない**）:

- `fandhe_frontend_core::Node` 木を**読み取るのみ**の走査関数として実装し、
  HTML 文字列の組み立てや `raw_html()` の使用を伴わない（既定エスケープ
  REQ-1 の迂回経路を新設しない不変条件）
- 検査対象コーパスは既存の `crates/docs-site/src/primitive_showcase/`
  （Primitives）・`src/showcase.rs`（Themes）の Demo 出力を再利用し、新規
  コーパスを持たない
- 実装形態は `crates/xtask/` への新規サブコマンド、または
  `crates/docs-site/tests/` への新規テストのいずれかとし、既存の
  `gate-design.md` §2.3a・`check_version_bump.rs` が確立した
  `environment error: ` プレフィックス規約（環境エラーとコード起因 FAIL の
  CI アノテーション区別）を踏襲する
- 検査対象部品の取りこぼしを PASS 扱いにしない、部品数の双方向 fail-closed
  突合（`primitives_catalog.rs` / `primitive_showcase.rs` と同型）を設ける
- 上記は実装を伴うため、着手には別イシューでの起票が必要になる。本評価では
  提案に留め、**イシューの起票は行わない**（`.claude/rules/out-of-scope-tracking.md`
  「ユーザーの承認なしに勝手に Issue を起票しない」）

## 8. 依存追加の承認境界（受け入れ条件 2 の明示）

本評価が扱ういずれの案も、外部依存（npm パッケージ・Rust クレート・第三者製
GitHub Action・追加のプリビルトバイナリ）の追加を伴う場合は、**着手前にユーザ
ー承認が必須**である（`CLAUDE.md`「依存クレート追加・Issue 起票は事前承認必
須」／`.claude/rules/coding-rust.md` REQ-3 節）。

案 B は外部依存を伴わないが、`crates/xtask/` への新規サブコマンド追加または
`crates/docs-site/tests/` への新規テスト追加という実装を伴うため、別イシュー
での実施が必要である（本イシューは調査のみ）。

## 9. セキュリティ考慮事項（OWASP Top 10 観点）

本評価文書自体は文書のみであり実行コードを含まないため直接の攻撃面はない。
以下は評価対象の各案が持つセキュリティ含意である。

- **A08（ソフトウェア・データ整合性の不備）**: axe-core 導入は第三者製ミニ
  ファイ JS を CI 実行面へ持ち込む行為であり、本リポジトリのサプライチェー
  ン対策（npm `--ignore-scripts` + allowlist、CI ツールのバージョン固定 +
  SHA256 検証済みプリビルトバイナリ統一、実行時ダウンロード禁止、第三者製
  action の新規追加なし）のいずれかに抵触する。案 B は外部依存を一切増やさ
  ないため、この軸で最も安全である
- **A05（セキュリティ設定ミス）**: 将来 a11y 検証ジョブを追加する場合は、
  `permissions:` の最小権限維持・シークレット参照なし・`run:` への
  `${{ }}` 直接展開の禁止（script injection 対策）・runner 方針（本評価時点は
  `runs-on: self-hosted` 既定だったが、2026-08-07 の反転〔#1220〕以降は
  GitHub ホステッドランナー既定、`.claude/rules/ci.md`「Runner 方針」節参照、
  イシュー #1238 注記）・一時領域は `RUNNER_TEMP` 配下（`.claude/rules/ci.md`）
  を要件とする。見送りの主根拠（npm 経路の REQ-12 allowlist 非適合）は
  runner 方針と独立のため §4 の結論は不変
- **A06（脆弱で古くなったコンポーネント）**: vendored `axe.min.js`（変種
  A-3）は cargo-deny / `cargo audit` の監査対象外となり、CVE 追随が人手依存
  になる
- **A03（インジェクション / XSS）**: 案 B の実装（将来）は
  `fandhe_frontend_core::Node` 木を読み取るのみであり、HTML 文字列の組み立
  てや `raw_html()` の使用を伴わない。既定エスケープ（REQ-1）の迂回経路を
  新設しないことを §7 の不変条件として明記した
- **A09（セキュリティログとモニタリングの失敗）**: 将来ジョブを追加する場合
  は既存の `environment error: ` プレフィックス規約を踏襲し、環境エラーと
  コード起因 FAIL を区別すること、検査対象部品の取りこぼしを PASS 扱いにし
  ないことを §7 に記載した
- 本 PR は依存を一切追加しない（`Cargo.toml`・`Cargo.lock`・`package.json`
  の変更なし）。コミット前に `git diff --cached` を確認し、トークン・
  `.env`・実クレデンシャルが含まれないことを確認した
