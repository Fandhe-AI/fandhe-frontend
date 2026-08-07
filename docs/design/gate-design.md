# 検証ゲート（`fw gate`）判定ルール設計（TASK-13.3a）

> **本書のステータスと前提**: 本書は TASK-13.3（親イシュー #138、成果物
> `crates/cli/src/gate.rs`）の 4 分割サブタスク（TASK-13.3a 判定ルール設計・#139
> （本書）/ TASK-13.3b コマンド実装・#140 / TASK-13.3c `cargo-deny`・XSS
> 回帰テスト連携・#141 / TASK-13.3d テスト整備・#142）の先頭に位置づけられる
> 設計文書である。
>
> **重要**: `fw gate` の実装本体はすでに `origin/main` へマージ済みである
> （PR #261 `feat(global): TASK-13.3 検証ゲート（fw gate）の Rust 実装`、
> 追補 PR #262 負例回帰テスト、PR #263 `raw_html` カスタム clippy lint 化）。
> 親 #138 は CLOSED 済み。本書は「これから実装する設計」ではなく、**実装済み
> かつテスト済みのコード（`crates/cli/src/gate.rs`、約 1370 行）が内包する判定ルールを、
> モジュール doc コメントに散在した状態から正式な設計文書として抽出・固定する」
> ことを目的とする。実装とのトレーサビリティは §6 を参照。乖離が見つかった
> 場合は実装（マージ済み・テスト済み）を正とし、本書側を合わせる方針を取った。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-13（AI 自己保守・改修のためのフック・ゲート機構、Must）の
  4 要素のうち「(4) 検証・制約の強制」を担う。REQ-4（`cargo-deny` 等の既定
  同梱）・REQ-1（既定エスケープによる出力安全性）とも接続する
  （`docs/spec/04-requirements.md` REQ-1/REQ-4/REQ-13）。
- **親タスク**: TASK-13.3（#138、`docs/spec/05-tasks.md` 355 行目、前提タスク
  TASK-13.1・TASK-4.1・TASK-1.2）。

| サブタスク | Issue | 内容 | 本書との関係 |
|-----------|-------|------|-------------|
| TASK-13.3a | #139（本書） | 判定ルール設計の正式化 | 本書が単一の情報源。`crates/cli/src/gate.rs` の実装から逆引きして記述する |
| TASK-13.3b | #140 | `gate` サブコマンドの CLI 接続 | 実装済み（`crates/cli/src/main.rs` の `gate::run_gate` ディスパッチ）。本書 §4 が CLI 契約を規定 |
| TASK-13.3c | #141 | `cargo-deny`・XSS 回帰テスト連携 | 実装済み（本書 §2 の `policy` チェック・`default_escape_check`）。連携の回帰固定は `crates/cli/tests/xss_regression_link.rs` と CI `test` ジョブへの cargo-deny 導入（`.github/workflows/ci.yml`）で行う（§6 参照） |
| TASK-13.3d | #142 | テスト整備 | 実装済み（`crates/cli/tests/gate_integration.rs`・`crates/cli/tests/negative_cases.rs`・`crates/cli/tests/raw_html_lint_e2e.rs`） |

- **土台**: PoC-7 の `docs/spec/03-poc/ai-self-maintenance/tools/poc7_tool.py`
  `cmd_gate`（5 チェックの実行・集約という骨格）からの Rust 移植。PoC-7 の
  「マーカーコメント方式」による既定エスケープ検査は、コンパイラに検証されず
  偽装可能という脆弱性が判明したため（イシュー #157）、製品実装では
  `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]`
  属性方式へ全面的に置き換えている（§2.2・§5 参照）。
- **先行パターンの踏襲**: TASK-13.2a（#133、`docs/design/impact-analysis-design.md`）
  が確立した「設計文書 + 実装トレーサビリティ表」の構成に倣う。

## 2. 判定ルール本体

`fw gate` は `structure.toml`（[`crate::structure`]、TASK-13.1）を唯一の
情報源として宣言クレート一覧を求め、以下の 7 チェックを**すべて実行**する
（後述 §3 のとおり早期打ち切りはしない）。各チェックは `GateCheck { name,
passed, output }` として結果を持つ。

| # | `name`（JSON） | 目的 | 実行コマンド・判定内容 | 対応 REQ |
|---|----------------|------|------------------------|----------|
| 1 | `type_check` | 型チェック | `cargo check --locked -p <crate>...`（宣言クレートごとに `-p` を連ねる） | REQ-13 |
| 2 | `default_escape_check` | 既定エスケープ検査（保険層） | `role = "core"` 以外の宣言ディレクトリの `src/**/*.rs` を走査し、未レビューの `raw_html()` 呼び出し・ブランケット抑止属性を検出する純粋関数（外部コマンド起動なし） | REQ-1 |
| 3 | `url_validation_check` | URL 属性検証の弱体化検出（保険層、イシュー #401） | `set_attribute` 系呼び出しの未ガード経路（U1）・`core` 役割の `URL_ATTRS`/許可スキーム緩和（U2）・ガード関数呼び出しの削除（U3）をテキスト走査で検出する純粋関数（外部コマンド起動なし）。詳細は §2.4 | REQ-1 |
| 4 | `lint` | lint（既定エスケープ検査の主防御を含む） | `cargo clippy --locked --all-targets -p <crate>... -- -D warnings`。`--all-targets` はテストターゲット内の未レビュー `raw_html()` 呼び出しも検出対象に含め、CI `clippy` ジョブ（イシュー #299）と検出範囲を一致させる（イシュー #315）。起動前に `clippy.toml` の `disallowed-methods` 設定健全性を検証する（§2.3） | REQ-1・REQ-13 |
| 5 | `lint_wasm32` | wasm32 target 向け lint（イシュー #1174） | `role = "client-entrypoint"` を宣言するクレートのみを対象に `cargo clippy --locked --all-targets --target wasm32-unknown-unknown -p <crate>... -- -D warnings` を実行する。CI `clippy-wasm32` ジョブ（イシュー #1160）と同一の検出範囲で、host target のみの `lint` チェックでは検知できない `#[cfg(target_arch = "wasm32")]` ゲート配下の警告（イシュー #1140/PR #1147 のすり抜けが動機）をローカル・AI 自己保守フックでも検知する。対象クレートが 0 件の場合は not-applicable PASS（§2.6） | REQ-1・REQ-13 |
| 6 | `test` | テスト | `cargo test --locked -p <crate>...` | REQ-13 |
| 7 | `policy` | 依存ポリシー | `deny.toml` の存在確認 → `cargo deny check bans licenses sources`（`advisories` はネットワーク前提のためオフラインゲート対象外、`docs/policy/cargo-deny-advisories.md` 参照） | REQ-4 |

実行時の作業ディレクトリ（cwd）はいずれも `--project` で指定したプロジェクト
ルート（省略時はカレントディレクトリ）。`--locked` はチェック 1・3・4・5 に
共通して付与し、ロックファイル逸脱（依存すり替え）を検出する
（security.md A06）。

### 2.1 PASS/FAIL 判定条件

- チェック 1・4・5・6（`cargo` サブコマンド）: プロセスの終了ステータスが
  成功（exit code 0）であれば `passed = true`。起動失敗・非 0 終了はすべて
  `passed = false`。
- チェック 2（`default_escape_check`）: `violations` リストが空であれば
  `passed = true`。1 件でも違反があれば `passed = false`。
- チェック 3（`url_validation_check`）: `violations` リストが空であれば
  `passed = true`。1 件でも違反があれば `passed = false`（§2.4）。
- チェック 7（`policy`）: `deny.toml` が存在し、かつ `cargo deny check bans
  licenses sources` が成功終了であれば `passed = true`。

### 2.2 既定エスケープ検査の 3 層体制

REQ-1 の唯一の許容迂回経路である `raw_html()` の呼び出しを検出するため、
3 層の独立した検出を組み合わせる（[`default_escape_check`] モジュール doc
コメント、`docs/design/raw-html-lint-design.md` に詳細な脅威モデル・方式比較を持つ）。

1. **主防御（`lint` チェック）**: `cargo clippy` + workspace ルート
   `clippy.toml` の `disallowed-methods` エントリ（`fandhe_frontend_core::raw_html`）。
   コンパイラのパス解決（HIR）に基づくため、コメントでは偽装できない
   （リネームインポート・複数行呼び出しも検出、`raw_html_lint_e2e.rs` で
   実 clippy 起動による検証を持つ）。
2. **保険層（`default_escape_check`）**: テキスト走査による独立検出。受理
   条件は「呼び出し開始行自体、または呼び出し直前に隙間なく連なる属性
   グループ列のいずれか 1 つが `#[expect(clippy::disallowed_methods,
   reason = "ESCAPE-REVIEWED: ...")]` を含むこと」（[`reviewed_attribute_covers_call`]・
   [`collect_attribute_groups`]、イシュー #1116）。属性グループは `#[` から
   対応する `]` までの括弧バランス（文字列リテラル内は除外）で判定するため、
   rustfmt が `reason = "..."` を複数行へ折り返した属性や `#[rustfmt::skip]`
   等の重ね掛けも `#[rustfmt::skip]` を追加せず受理される（マーカー文字列の
   比較は空白除去後に行うため、トークン間の改行・インデント挿入で分断され
   ない）。属性と呼び出しの間に空行・コメント行・無関係なコード行を挟んだ
   場合は連鎖が途切れ受理しない（保険層の偽陰性ゼロ方針、主防御である
   `lint` チェックは無変更）。旧方式（`// ESCAPE-REVIEWED:` コメント単体での
   受理）はイシュー #157 の指摘を受けて廃止済みで、コメントのみのマーカーは
   違反として検出される（`scan_file_rejects_comment_only_marker_as_spoofable`
   が回帰を固定）。詳細は `docs/policy/raw-html-review-gate.md` §1。
3. **ブランケット抑止監査**: `#![allow(clippy::disallowed_methods)]` /
   `#![expect(clippy::disallowed_methods)]` によるファイル・モジュール
   一括無効化を、呼び出し個別のレビュー宣言とは独立に一律違反として検出する
   （[`scan_file_for_violations`]）。行コメント・ドキュメンテーションコメント・
   文字列リテラル内の言及は [`line_has_real_blanket_attribute`] /
   [`position_is_inside_string_literal`] の簡易判定で除外する（PR #263 の
   Bugbot 指摘対応）。

`default_escape_check` は `role = "core"` のディレクトリを走査対象から除外
する（core は `raw_html()` 自体を提供する側であり、検査対象ではないため）。
走査は `tests/` ディレクトリを対象外とし、テストコード内の `raw_html()`
利用は TASK-13.5 以降の負例回帰テストの対象とする限界を持つ。

### 2.2a コード文脈限定の走査（イシュー #372）

`fw gate --project .` を本リポジトリ自身へ適用すると、保険層（`default_escape_check`）
のテキスト走査がコード以外の文脈に現れる `raw_html(` 文字列まで違反として
検出する自己参照誤検知が生じていた（PR #365/#366/#367 の申し送り）。確定した
誤検知の実態は 3 分類:

- **コメント内の言及**（doc コメントでの `raw_html()` API 説明、`gate.rs`
  自身のモジュール doc コメント等）
- **文字列リテラル内**（エラーメッセージ文言・テストフィクスチャ文字列）
- **識別子サフィックス**（`fn ..._detects_unreviewed_raw_html()` のような
  別識別子の一部としての `raw_html`。直後の `(` は関数定義のパラメータリスト
  であり呼び出しではない）

これらは tooling（`cli`）に限らず `interactive` / `wasm-*` 系クレートの doc
コメントにも及ぶため、「特定 role の除外」や「ファイル単位 allowlist」では
解決にならない（allowlist は実呼び出しの死角を作る）。そこで
[`find_raw_html_call_positions`] の走査を「コード文脈限定」へ精密化した:

1. [`code_context_mask`] が Rust ソース全体を状態機械（`Code` /
   `LineComment` / `BlockComment`（ネスト対応） / `Str` / `RawStr`）で走査し、
   各バイト位置がコード文脈か否かのマスクを構築する
2. 出現位置がコード文脈であることに加え、直前バイトが識別子構成文字
   （`[A-Za-z0-9_]`）でないこと（左境界チェック）を要求する。`fandhe_frontend_core::raw_html(`
   （直前 `:`）・`.raw_html(`（メソッド形）は直前バイトが識別子構成文字では
   ないため引き続き検出する

**REQ-1 非弱体化の根拠**: コメント・文字列リテラル・別識別子は Rust の字句
規則上 `raw_html()` の呼び出しになり得ない。除外は誤検知（偽陽性）のみを
削り、偽陰性を生まない。主防御は従来どおり §2.2 の 3 層体制（`lint` チェック、
HIR パス解決）であり無変更。

**字句判定の近似と限界**: `code_context_mask` は完全な Rust 字句解析器ではない。
文字リテラルとライフタイム（`'a`）の判別は「`'` の直後がエスケープ列または
任意 1 文字＋閉じ `'`」の近似で行い、判別不能（ライフタイムの可能性を
排除できない `'`）な場合は **コード文脈（Code）側に倒す**（fail-closed、
security.md A05: 保険層の偽陰性ゼロを優先し、偽陽性の残存は許容する）。
字句判定を狂わせる敵対的パターン（文字リテラル `'"'` で文字列状態を狂わせる・
文字列内の `/*` でコメント状態を狂わせる等）は
`find_raw_html_call_positions_still_detects_call_after_char_literal_confusion_attempt`
等の敵対的回帰テスト（`crates/cli/src/gate.rs`）で固定する。また
`raw_html /* comment */ (x)` のようにコメントを呼び出し内に挟む記法は保険層の
検出漏れとして残る（従来からの限界。主防御 `lint` チェックがこの経路も検出する）。

ブランケット抑止監査（§2.2 の 3 番目、[`line_has_real_blanket_attribute`]）は
行単位判定のまま変更していない（既存実装・既存テストで機能しており本イシュー
の障害要因ではないため、差分最小化を優先）。

自己適用回帰は `default_escape_check_passes_on_this_repository_itself`
（`crates/cli/src/gate.rs`）が固定する。本リポジトリ自身の `structure.toml` を実際に
読み込み `default_escape_check`（純粋関数・外部コマンド起動なし）を適用し、
`passed` であることを検証する。

### 2.3 `lint` チェックの起動前ガード（clippy ポリシー健全性）

`lint` チェックは `cargo clippy` を起動する前に、workspace ルート
`clippy.toml` に `disallowed-methods` の `fandhe_frontend_core::raw_html` エントリが
存在するかをテキストで検証する（[`clippy_policy_is_configured`]）。判定は
各行の `#` 以降（TOML コメント）を [`crate::toml::strip_comment`] で除去した
うえで `disallowed-methods` と `fandhe_frontend_core::raw_html` の両文字列を含むかで行う
（コメントアウトされたエントリを「設定済み」と誤判定しないための対策、
PR #263 の Bugbot 指摘対応）。`clippy.toml` の欠落・エントリ欠落は `cargo
clippy` を起動せず即座に `lint` チェックを failed とする（§3 の fail-closed
原則）。

### 2.3a 環境エラーのプリフライト検出（clippy component / cargo-deny の有無、イシュー #292）

CI runner（旧 self-hosted プール・現 GitHub ホステッドランナーのいずれも、
`.claude/rules/ci.md`「Runner 方針」節）はインスタンス（ジョブ）ごとに
clippy component / cargo-deny の導入状態が異なり得る。`lint`／`policy` チェックがこれらツールの不在で
failed になった場合、コード内容起因の FAIL（clippy 違反・deny.toml ポリシー
違反）と区別が付かず、「当たった runner 次第で BLOCKED になる」間欠failure
として現れる（イシュー #292）。

これに対応するため、それぞれの本実行の直前に軽量な疎通確認を行う。

- `lint`: `clippy_policy_check`（§2.3）の後、本実行の前に
  [`clippy_environment_preflight`] が `cargo clippy --version` を起動する。
- `policy`: `deny.toml` 存在確認の後、本実行の前に
  [`cargo_deny_environment_preflight`] が `cargo deny --version` を起動する。

いずれも疎通確認が失敗した場合のみ、`output` 先頭に固定プレフィックス
`ENVIRONMENT_ERROR_PREFIX`（`"environment error: "`）を付与した決定的な
メッセージで該当チェックを failed とする。**SKIP や黙示的 PASS には倒さない**
（fail-closed 維持、§3・security.md A05）。

- **是正案内の実在導線化（イシュー #1116）**: 是正コマンドは汎用コマンド
  （`lint`: `rustup component add clippy` / `policy`: `cargo install
  cargo-deny --locked`）を常に提示し、`<project_dir>/tools/ci/
  ensure-gate-tools.sh` は [`gate_tools_script_exists`] で実在確認できた
  場合のみ追記する。`templates/default/` 生成プロジェクトにはこのスクリプトが
  同梱されないため、無条件案内は「存在しないファイルへの案内」という DX
  阻害を招いていた（本イシューの動機の 1 つ）。
- **`GateCheck.environment_error`（イシュー #1116）**: 従来は `output` の
  先頭プレフィックス文字列でのみ区別していたが、[`clippy_environment_preflight`]・
  [`cargo_deny_environment_preflight`] が構造化フィールド
  `environment_error: bool` を `true` に設定するようになった。[`aggregate`]
  はこのフィールドを見て `gate_result` を `ERROR`/`BLOCKED` に振り分ける
  （§4）。`output` 先頭のプレフィックス文字列は後方互換のため引き続き残す。

ツールの自動インストールはここでは行わない（検証ゲートは検証のみに専念し、
ネットワーク非依存・サプライチェーン面の不拡大を維持する）。常設化・導入は
`tools/ci/ensure-gate-tools.sh`（§6）の責務とする。

### 2.3b 実行コマンドの可視化・`test` 要約・`--verbose`（イシュー #1116）

- **`GateCheck.command`**: 外部コマンド系 4 チェック（`type_check`/`lint`/
  `test`/`policy`）は、本実行に到達した場合のみ完全なコマンドライン文字列
  （例: `cargo clippy --locked --all-targets -p app -- -D warnings`）を
  `command` フィールドへ保持する（[`finish_command_check`]）。プリフライト
  失敗・`deny.toml` 欠落等、本実行に到達しなかった場合は `command: None`
  のまま（JSON では当該キーを省略）。可読性のため `output` 先頭にも
  `$ <command>\n` を前置する。
- **`test` チェック PASS 時の output 要約**: [`summarize_test_output`] が
  `running N tests`／`test result:`／`Doc-tests` ヘッダ行のみを残す決定的な
  フィルタを適用し、`--verbose` 未指定時のみ [`run_gate`] から呼ばれる
  （[`summarize_passing_test_output`]）。`test result:` 行が 1 件も抽出でき
  ない想定外の出力形式では要約せず全文（丸めあり）へフォールバックする
  （情報を隠さない fail-safe）。**失敗時（`passed = false`）は要約しない**
  （違反の詳細を削らない）。
- **`fw gate [--project <dir>] [--verbose]`**: `--verbose` 指定時は上記要約を
  無効化し、常に全文（`OUTPUT_TRUNCATE_CHARS` 丸めのみ）を出力する。

### 2.4 URL 属性検証の弱体化検出（`url_validation_check`、イシュー #401）

イシュー #373（PR #386）は `crates/core/src/url.rs` に URL スキーム検証
（`is_safe_url`/`is_safe_srcset`/`is_url_attr`/`is_event_handler_attr`、
正本 allowlist `URL_ATTRS`）を導入し、SSR（`render_into`）・CSR 実 DOM
（`wasm-client::binding_dom::apply_one`/`keyed_dom::build_element`）の
3 経路へ適用した。しかしこの保証はレビュー・テストのみに依存しており、
`fw gate` は以下 3 種の弱体化を機械検出できていなかった:

1. 検証関数を経由せず URL 属性を設定する新規経路の追加
2. allowlist の緩和（`URL_ATTRS` からの属性削除・許可スキームの追加）
3. 既存 3 経路からのガード呼び出しの削除

`url_validation_check` は `default_escape_check` と同型の「外部コマンド
起動なしの純粋関数走査」（保険層）として、以下 3 ルール（U1〜U3）を実装
する。走査基盤は §2.2a の[`code_context_mask`]・左境界チェックを再利用
する（`find_code_context_call_positions`、`find_raw_html_call_positions`
を needle 引数化した共通実装）。

**U1（非 core ディレクトリの未検証 DOM 属性設定経路の検出）**: `role !=
"core"` の宣言ディレクトリの `src/**/*.rs`（`tests/` は対象外、symlink 非
追従）を走査し、`set_attribute`/`set_attribute_ns`/`set_attribute_node`
呼び出しを含むファイルが、同一ファイル内にガード 4 種
（`is_url_attr`/`is_safe_url`/`is_safe_srcset`/`is_event_handler_attr`）の
呼び出しをすべて持たない場合、呼び出しごとに違反とする。**既知の限界**:
ファイル単位の共起判定のため「同一ファイル内にガード済み呼び出しと未
ガード呼び出しが併存」は見逃す。保険層としての限界であり、行動保証の
本体は `test` チェック（XSS 回帰テスト）が担う。

**U2（core ディレクトリの allowlist 非緩和、ピン検査）**: `role = "core"`
の宣言ディレクトリの `src/**/*.rs` から `const URL_ATTRS` 定義ファイルを
特定し（単なる `URL_ATTRS` 識別子ではなく `const URL_ATTRS` を要求し、
`pub use url::{..., URL_ATTRS};` のような再エクスポート言及を定義と誤認
しない）、以下をすべて満たすことを要求する（fail-closed。いずれか不成立
で違反）:

- `URL_ATTRS` 定義ファイルが core role の src 内に存在すること
- 定義ブロック（`const URL_ATTRS` 出現位置から直近の `];` まで）内の文字列
  リテラル集合が、gate 側にピンした 12 属性（href/src/action/formaction/
  xlink:href/poster/cite/data/background/ping/dynsrc/lowsrc）をすべて
  含むこと（削除＝緩和を検出。追加は強化のため許容）
- 同ファイル内のコード文脈における `eq_ignore_ascii_case("<literal>")` の
  スキーム比較リテラル集合が、ピン集合 `{http, https, mailto, tel}` の
  部分集合であること（スキーム追加＝緩和を検出）
- ガード関数 4 種（`is_safe_url`/`is_safe_srcset`/`is_url_attr`/
  `is_event_handler_attr`）の定義が core role の src 内（`URL_ATTRS` 定義
  ファイルに限定しない）に存在すること（削除検出）

**U3（core ディレクトリのガード呼び出し実在、適用経路の削除検出）**:
`role = "core"` の宣言ディレクトリの src 内で、ガード 4 種それぞれについて
「`fn` 定義行を除いたコード文脈の呼び出し」（[`is_fn_definition_call`] で
定義行を除外）が 1 箇所以上存在することを要求する。**既知の限界**:
`is_safe_url` は `is_safe_srcset` 内部呼び出しで自明に成立するため、実効的
な検出対象は `is_url_attr`/`is_event_handler_attr`/`is_safe_srcset` の
削除。

**core role 非宣言プロジェクトの扱い**: `structure.toml` に `role = "core"`
のディレクトリが 1 つも宣言されていないプロジェクト（`fw new` 生成物・
既存テストフィクスチャの多く）では U2/U3 は対象なしで素通しする。U1 は
宣言済み非 core ディレクトリのみに適用されるため、core role 非宣言でも
機能する。

自己適用回帰は `url_validation_check_passes_on_this_repository_itself`
（`crates/cli/src/gate.rs`）が固定する。本リポジトリ自身の `structure.toml` を
実際に読み込み `url_validation_check`（純粋関数・外部コマンド起動なし）を
適用し、`passed` であることを検証する。

**追加スコープ外（PR 本文で切り出し提案。§7 参照）**: clippy
`disallowed-methods` による `web_sys::Element::set_attribute` の主防御化
（`raw_html` と同じ 3 層体制への昇格）・`web_sys` の型付きセッター
（`set_href`/`set_src` 等）や `insert_adjacent_html` 経路の検出・
`fw new` テンプレートの crates.io バージョン依存整合性検出
（`crates/cli/tests/template_vendor_drift.rs` が `templates_app_cargo_toml_declares_version_dependency_matching_source_crates` 等で担保）は
本イシューのスコープ外。

### 2.5 静的専用（asset-only）プロジェクトの判定（イシュー #410）

`fw new --template embed` が生成する「静的単一ファイルの部分埋め込み構成」
（REQ-7）は cargo パッケージを一切持たない。この構成では cargo 系 5 チェック
（`type_check`/`lint`/`lint_wasm32`/`test`/`policy`）は検証対象クレートが
存在せず、「検証不能」と「検証したが違反なし」を区別できないまま §3 の
宣言クレート 0 件 fail-closed（[`no_declared_crates_message`]）に落ちてしまう。

`fw gate`（[`is_asset_only_project`]）は以下の条件を**すべて**満たす場合に
限り、これを静的専用プロジェクトの明示的オプトインと認識する:

- 宣言クレートが 0 件（どの `[directories.*]` も `crate = "..."` を持たない）
- 宣言ディレクトリが 1 件以上存在し、**全件**が `role = "asset"` である

両方を満たす場合、`type_check`/`lint`/`lint_wasm32`/`test`/`policy` の
5 チェックは cargo を一切起動せず、`passed: true` と決定的な not-applicable
文言（`static-only project (all directories declare role = "asset" with no
crate): cargo-based check not applicable`）で PASS 化する
（[`not_applicable_check`]）。`lint_wasm32` は §2.6 の client-entrypoint
クレート 0 件判定（同じく not-applicable PASS）と経路は異なるが、asset-only
プロジェクトは `role = "client-entrypoint"` を宣言し得ないため
（[`is_asset_only_project`] の「全件 `role = \"asset\"`」条件と両立しない）、
`run_all_checks` は asset-only 分岐へ直接 `not_applicable_check("lint_wasm32")`
を組み込む（§2.6 の判定関数を経由しない）。

**明示宣言によるオプトインであり黙示的 PASS ではない**（security.md
A05）。`crate` キーの削除し忘れ等の設定不備で非 asset ロールが 1 件でも
混在していれば `is_asset_only_project` は `false` を返し、従来どおり
[`no_declared_crates_message`] による fail-closed（BLOCKED）が働く。

**テキスト走査ベースの保険層は継続実行**: `default_escape_check`・
`url_validation_check` は cargo パッケージの有無に依存しない純粋関数
走査であり、静的専用モードでも通常どおり実行する。`role = "asset"` の
`root` 慣習ディレクトリ配下（プロジェクトルート直下 `src/`）に Rust
コードが混入し、そこに未レビューの `raw_html()` 呼び出しが含まれる場合は
`default_escape_check` が検出し `BLOCKED` にする（`crates/cli/tests/new_gate_e2e.rs::
fw_new_embed_template_gate_detects_injected_rust_violation` が回帰固定）。
静的専用モードは「cargo 系チェックの対象が存在しない」ことの明示化に
限定され、検証の全面停止ではない。

**JSON 契約への影響なし**: `checks[].name`/`passed`/`output` の形状・
7 チェックの名前と順序は不変。既存クライアント（AI 自己保守フック・CI）は
`not_applicable_check` の `passed: true` を通常の PASS と同様に扱える。

### 2.6 wasm32 target 向け lint の適用範囲（イシュー #1174）

`lint_wasm32` チェックは `role = "client-entrypoint"` を宣言する
クレートのみを対象とする（[`declared_client_entrypoint_crate_names`]）。
CI `clippy-wasm32` ジョブ（イシュー #1160、`.github/workflows/ci.yml`）が
wasm-full / wasm-client / wasm-thin の 3 クレートを固定列挙するのに対し、
`fw gate` は §2 冒頭の原則（`structure.toml` を唯一の情報源とする）を
踏襲し、`role` 宣言から動的に検出対象を求める。

対象クレートが 0 件の場合（`fw new --template default`/`--template app`
等、多くのユーザープロジェクトが該当）は「wasm クレートを持たないため
対象外」を not-applicable PASS（`no client-entrypoint crate declared in
structure.toml: wasm32 lint not applicable`）で明示する。`structure.toml`
上の `role` 宣言に基づく明示的な条件であり黙示的 PASS ではない
（security.md A05、§2.5 の asset-only 判定と同じ思想）。

対象クレートが 1 件以上ある場合は、`lint` チェック（§2.3）と同じ
`clippy_policy_check`（`clippy.toml` の `disallowed-methods` 健全性）・
`clippy_environment_preflight`（`cargo clippy --version` 疎通）を前置した
うえで、wasm32 target 自体の導入状態を確認する専用プリフライト
（[`wasm32_target_environment_preflight`]、`rustup target list --installed`
の出力に `wasm32-unknown-unknown` が行完全一致で含まれるかを判定。
`wasm32-wasip1` 等の前方一致の他 target を誤って「導入済み」と判定しない）
を実行し、いずれも通過した場合のみ `cargo clippy --locked --all-targets
--target wasm32-unknown-unknown -p <crate>... -- -D warnings` を起動する
（§3・§4 の fail-closed・環境エラー分類は §2.3a と同型）。

## 3. fail-closed 原則

`fw gate` は「検証できないこと」を暗黙の PASS として扱わない
（security.md A05）。以下の条件はすべて明示的に failed / BLOCKED として
扱われる。

| 条件 | 扱い |
|------|------|
| `structure.toml` の読み込み・パース失敗 | ゲート全体を即座に `BLOCKED`（他チェックを実行しない。宣言クレート一覧が定まらず以降のチェックが無意味になるため） |
| `structure.toml` のセマンティック検証（[`StructureManifest::validate`]）失敗 | ゲート全体を即座に `BLOCKED` |
| 宣言クレートが 0 件（`structure.toml` にどのディレクトリも `crate = "..."` を持たない） | `type_check`/`lint`/`test` を `-p` なしのワークスペース全体検証へフォールバックせず、各チェックを個別に failed とする（[`no_declared_crates_message`]。「検証対象なし＝ PASS」でも「範囲不明な全体検証」でもなく、設定不備として明示する）。**例外**: 宣言ディレクトリ全件が `role = "asset"` である場合のみ静的専用プロジェクトの明示的オプトインとみなし、`type_check`/`lint`/`lint_wasm32`/`test`/`policy` を not-applicable PASS 化する（§2.5、イシュー #410） |
| `role = "client-entrypoint"` を宣言するクレートが 0 件（wasm クレートを持たないプロジェクト） | `type_check` 等とは異なり fail-closed ではなく not-applicable PASS とする（`lint_wasm32` のみの例外。§2.6、イシュー #1174） |
| `deny.toml` が存在しない | `cargo deny` を起動せず `policy` チェックを failed とする（`<project>/deny.toml` を唯一の情報源とする。本リポジトリ自身への自己適用時もこの契約は変更せず、リポジトリ直下へ `templates/default/deny.toml` と同一強度のポリシーを配置することで解決する。イシュー #372、workspace 参照解決方式は gate の fail-closed 契約を複雑化させるため不採用） |
| `clippy.toml` の欠落・`disallowed-methods` エントリ欠落 | `cargo clippy` を起動せず `lint`/`lint_wasm32` チェックを failed とする（§2.3・§2.6） |
| clippy component が runner に未導入（`cargo clippy --version` 疎通確認失敗） | `cargo clippy` 本実行を起動せず `lint`/`lint_wasm32` チェックを `environment error:` 付き・`environment_error: true` で failed とする（§2.3a・§2.6、イシュー #292/#1116/#1174） |
| `wasm32-unknown-unknown` rustup target が runner に未導入（`rustup target list --installed` に不在） | `cargo clippy --target wasm32-unknown-unknown` 本実行を起動せず `lint_wasm32` チェックを `environment error:` 付き・`environment_error: true` で failed とする（§2.6、イシュー #1174） |
| cargo-deny が runner に未導入（`cargo deny --version` 疎通確認失敗） | `cargo deny check ...` 本実行を起動せず `policy` チェックを `environment error:` 付き・`environment_error: true` で failed とする（§2.3a、イシュー #292/#1116） |
| 外部コマンド（`cargo` 系）の起動自体に失敗（バイナリ不在等） | 該当チェックを failed とする（[`CommandRunner::run`] が起動失敗を `Ok((false, ...))` として返し、呼び出し元は `Err` 分岐を用意せず fail-closed 集約する） |

## 4. 集約規則と CLI 契約

- **集約**: 7 チェックすべてを実行し、早期打ち切りはしない（AI エージェントが
  一括修正できるよう全違反を報告する PoC-7 の方針を踏襲、[`run_all_checks`]）。
  集約規則は 3 値（イシュー #1116 で 2 値から拡張、[`aggregate`]）:
  - 全チェック `passed = true` → `gate_result = "PASS"`
  - 不合格が 1 件以上あり、**不合格の全件**が `environment_error: true`
    （実行環境にツールが無いだけ、§2.3a） → `gate_result = "ERROR"`
  - 不合格にコード起因（`environment_error: false`）が 1 件でも含まれる
    → `gate_result = "BLOCKED"`（環境エラーと混在してもコード起因を優先し、
    fail-closed を弱めない）
- **終了コード規約**（`main.rs` 冒頭 doc コメントと同一契約。イシュー #1116 で
  `3` を追加）:
  - `0`: `gate_result = "PASS"`
  - `1`: `gate_result = "BLOCKED"`（`structure.toml` 読み込み・検証失敗を含む）
  - `2`: 使用法エラー（`--project` 引数の解析失敗・未知フラグ）
  - `3`: `gate_result = "ERROR"`（実行環境にツールが無いだけの不合格。§2.3a）
- **JSON 出力契約**（PoC-7 互換の形状 + 後方互換拡張キー、[`render_report`]）:

```json
{
  "checks": [
    { "name": "type_check", "passed": true, "output": "...", "environment_error": false },
    { "name": "default_escape_check", "passed": true, "output": "...", "environment_error": false },
    { "name": "url_validation_check", "passed": true, "output": "...", "environment_error": false },
    { "name": "lint", "passed": true, "output": "...", "environment_error": false, "command": "cargo clippy --locked --all-targets -p app -- -D warnings" },
    { "name": "lint_wasm32", "passed": true, "output": "...", "environment_error": false, "command": "cargo clippy --locked --all-targets --target wasm32-unknown-unknown -p wasm-full -- -D warnings" },
    { "name": "test", "passed": true, "output": "...", "environment_error": false, "command": "cargo test --locked -p app" },
    { "name": "policy", "passed": true, "output": "...", "environment_error": false, "command": "cargo deny check bans licenses sources" }
  ],
  "gate_result": "PASS",
  "action": "all checks passed; changes may proceed"
}
```

  - `checks[].output` は末尾 `OUTPUT_TRUNCATE_CHARS = 4000` 文字に丸められる
    （§5 参照）。`test` チェック PASS 時（`--verbose` 未指定）は丸めの前に
    [`summarize_test_output`] による要約が適用される（§2.3b）。外部コマンド系
    チェック（[`finish_command_check`]、§2.3b）はこの 4000 文字丸めの**後**に
    可読性のための `$ <command>\n` 前置行を追加するため、`output` 全体の
    長さは `4000 + "$ ".len() + command.len() + "\n".len()` を上限とする
    （丸め対象はコマンド出力本体のみで、前置行自体は丸めない。肥大化防止の
    実務上の目的（A09）はコマンド出力本体の丸めで既に達成されており、コマンド
    文字列自体は数十〜百数十文字程度で肥大化の主因にならないため許容する）。
  - `checks[].environment_error`（イシュー #1116 で追加）: 全チェックで常時
    出力する `bool`。`true` はツール未導入等の環境要因を表す。
  - `checks[].command`（イシュー #1116 で追加）: 外部コマンド系チェックが
    本実行に到達した場合のみ出力する（`Some` の場合のみキーを出力、`None`
    はキー自体を省略）。
  - `action` は `gate_result` に応じた固定文言（`PASS`: `"all checks passed;
    changes may proceed"` / `BLOCKED`: `"fix the reported failing checks and
    re-run \`fw gate\`"` / `ERROR`: `"fix the runner environment (see
    environment-error checks) and re-run \`fw gate\`"`。`structure.toml`
    段階の即時 `BLOCKED` は専用の `"fix structure.toml and re-run \`fw
    gate\`"`）。
- **利用契約**: AI 自己保守フック・CI は本サブコマンドの終了コードと JSON の
  `gate_result` を照合し、変更適用の可否を判断する。`gate_result = "ERROR"`
  も `"PASS"` 以外である以上、無条件で変更を適用してはならない（TASK-13.6
  #151、`docs/policy/ai-self-maintenance-policy.md` を参照。ERROR は「是正
  対象がランナー環境である」点のみ BLOCKED と異なる）。

## 5. セキュリティ不変条件

- **A03（インジェクション）**: 外部コマンドは [`std::process::Command`] に
  引数配列で渡し、シェル文字列連結を行わない（`RealCommandRunner::run`）。
- **A05（セキュリティ設定ミス、fail-closed）**: §3 に列挙した全条件。
- **A06（脆弱な依存）**: `type_check`/`lint`/`test` すべてに `--locked` を
  付与し、ロックファイル逸脱（依存すり替え）を検出する。
- **A08（ソフトウェア・データ整合性の失敗）**: `default_escape_check` の
  3 層検出（§2.2）が REQ-1 の唯一の迂回経路である `raw_html()` の呼び出しを
  明示レビュー済み宣言に限定する契約を担保する。詳細な脅威モデル・方式比較は
  `docs/design/raw-html-lint-design.md`、レビュー運用は `docs/policy/raw-html-review-gate.md`
  を参照。
- **A09（ログ・情報露出）**: `checks[].output` は `OUTPUT_TRUNCATE_CHARS =
  4000` 文字（末尾優先）に丸め、コマンド出力の肥大化・秘密情報の意図しない
  大量転記を防止する（[`truncate_output`]）。JSON へ格納する全文字列は
  [`crate::json_out::quoted`] を経由し、`"`・改行・制御文字によるレポート
  構造の破壊を防ぐ。
- **A01（アクセス制御の不備、パストラバーサル・DoS）**:
  `default_escape_check` のファイル走査（[`scan_dir_for_violations`]）は
  シンボリックリンク（ディレクトリ・ファイルいずれも）を辿らない。
  `DirEntry::file_type().is_symlink()` による明示チェックにより、自己参照
  リンクによる無限再帰（fail-closed の実行自体を阻害する DoS）と、
  プロジェクト外を指すリンクを辿ってのパストラバーサルの双方を防ぐ
  （`crates/cli/src/routes.rs` の `list_rs_files_inner` と同一方針）。

## 6. 実装トレーサビリティ

| 本書の章 | `crates/cli/src/gate.rs` の対応箇所 |
|---------|------------------------------|
| §2 表（7 チェック定義） | `run_all_checks`、`run_cargo_check`/`run_cargo_clippy`/`run_cargo_clippy_wasm32`/`run_cargo_test`/`policy_check`/`default_escape_check`/`url_validation_check` |
| §2.2（3 層体制） | モジュール doc コメント（1-35 行目）、`find_raw_html_call_positions`（420-440 行目）、`line_has_reviewed_expect_attribute`（521-528 行目）、`line_has_real_blanket_attribute`（487-497 行目）、`scan_file_for_violations`（607-667 行目） |
| §2.2a（コード文脈限定の走査、イシュー #372） | `find_raw_html_call_positions`（530 行目〜）、`code_context_mask`（578 行目〜）、`raw_string_hash_count`（742 行目〜）、`char_literal_end`（759 行目〜）、`utf8_char_len`（805 行目〜） |
| §2.3（clippy ポリシー健全性） | `clippy_policy_is_configured`（292-302 行目）、`clippy_policy_check`（315-329 行目） |
| §2.3a（環境エラーのプリフライト、イシュー #292） | `clippy_environment_preflight`・`cargo_deny_environment_preflight`（`ENVIRONMENT_ERROR_PREFIX` 定数とあわせて `run_cargo_clippy`/`policy_check` 直前で呼び出し） |
| §2.4（URL 属性検証の弱体化検出、イシュー #401） | `url_validation_check`（U1〜U3 集約）、`find_code_context_call_positions`（`find_raw_html_call_positions` の needle 引数化・共通化）、`is_fn_definition_call`（U3 の定義行除外）、`check_url_sink_guard_cooccurrence`（U1）、`check_core_url_validation_module`（U2）、`check_core_guard_calls_exist`（U3）、`walk_rs_files`（`scan_dir_for_violations` と共有する走査基盤） |
| §2.5（静的専用プロジェクトの判定、イシュー #410） | `is_asset_only_project`・`not_applicable_check`・`STATIC_ONLY_NOT_APPLICABLE_MESSAGE`（`run_all_checks` からの分岐呼び出し） |
| §3（fail-closed） | `run_gate`（97-153 行目、structure.toml 段階）、`no_declared_crates_message`（219-231 行目）、`run_locked_cargo_subcommand`（237-262 行目）、`run_cargo_clippy`（331-369 行目）、`policy_check`（383-405 行目）、`clippy_environment_preflight`/`cargo_deny_environment_preflight`（イシュー #292）、`check_core_url_validation_module`（core role 宣言時の URL_ATTRS モジュール欠落 fail-closed、イシュー #401） |
| §4（集約規則・CLI 契約） | `aggregate`（189-204 行目）、`render_report`（673-697 行目）、`main.rs` の終了コード規約（`main.rs` 33-35 行目） |
| §5（セキュリティ不変条件） | `RealCommandRunner::run`（74-85 行目）、`truncate_output`（207-217 行目）、`OUTPUT_TRUNCATE_CHARS`（44 行目）、`scan_dir_for_violations`（579-597 行目）、`walk_rs_files`（symlink 非追従の共有実装、イシュー #401） |
| §2.2（属性ブロック単位の受理、イシュー #1116） | `reviewed_attribute_covers_call`、`collect_attribute_groups`（行番号は割愛。上記行番号群は本イシュー実装時点で既に近似値であり、以後の変更で再度乖離する。本表の行番号は目安であり、単一の情報源は `crates/cli/src/gate.rs` 自体とする） |
| §2.3a/§2.3b（環境エラー種別・案内実在化・コマンド可視化・test 要約、イシュー #1116） | `gate_tools_script_exists`（是正案内の実在導線化）、`finish_command_check`・`command_line`（`command`/`$ <command>` 前置行）、`summarize_test_output`・`summarize_passing_test_output`（`test` PASS 時の要約、`run_gate` から `--verbose` 未指定時のみ呼ばれる） |
| §4（3 値集約・終了コード 3・`--verbose`、イシュー #1116） | `aggregate`（`GateCheck.environment_error` に基づく `PASS`/`ERROR`/`BLOCKED` 3 値化）、`run_gate`（`--verbose` 解析・終了コード `0/1/2/3`）、`render_report`（`environment_error`/`command` キー追加） |
| §2.6（`lint_wasm32` チェック、イシュー #1174） | `run_cargo_clippy_wasm32`・`declared_client_entrypoint_crate_names`・`wasm32_target_environment_preflight`（`clippy_policy_check`/`clippy_environment_preflight` は `check_name` パラメータ化して `lint`/`lint_wasm32` 双方から再利用） |

対応するテストは `crates/cli/tests/gate_integration.rs`（CLI 経由の統合テスト、
6 ケース）・`crates/cli/tests/negative_cases.rs`（型エラー・未エスケープ・禁止依存・
テストターゲット内 raw_html・自己参照様ソース（イシュー #372）・URL 属性
検証の弱体化（イシュー #401、U1 未ガード呼び出し・U2 スキーム緩和・U2/U3
正例基準線の 3 ケース）を含む負例群、TASK-13.5）・
`crates/cli/tests/raw_html_lint_e2e.rs`（実 clippy 起動による主防御の実証、
4 ケース）・`crates/cli/tests/scenarios/`（イシュー #401 対応: `bugfix_escape`
シナリオの `role = "core"` フィクスチャへ `url_validation_check` 充足専用の
未配線 `crates/core/src/url.rs` を追補、既存シナリオのアサーションは無変更）・
`gate.rs` 内 `#[cfg(test)] mod tests`（76 ユニットテスト。TASK-13.3d/#142
でポリシーチェックの単体テスト・外部コマンド起動引数契約・宣言クレート
0 件時の fail-closed・集約結果の `action` 文言・`run_all_checks` の
name/順序と PASS 経路・`render_report` の JSON ラウンドトリップ・
`truncate_output` のマルチバイト境界安全性を追補、イシュー #372 で
コード文脈限定の走査に関する誤検知解消・非弱体化の敵対的回帰・
`default_escape_check_passes_on_this_repository_itself` による自己適用回帰を
追補、イシュー #401 で `url_validation_check` の U1〜U3 単体テスト・
`url_validation_check_passes_on_this_repository_itself` による自己適用回帰を
追補、イシュー #410 で `is_asset_only_project` の判定境界（全 asset ロール／
宣言クレート存在／非 asset ロール混在の 3 分岐）・静的専用モードでの
`run_all_checks` 全 PASS かつ cargo 未起動・`default_escape_check` の非
バイパスを追補）。本書執筆時点でこれら全テストは `cargo test -p fandhe-frontend-cli`・
`cargo test --workspace` でグリーンであることを確認済み。

TASK-13.3c（#141、`policy`/`test` チェックの実連携固定）の対応:

- `crates/cli/tests/support/mod.rs`: `negative_cases.rs`・`xss_regression_link.rs`
  が共用するフィクスチャ書き出し・`fw gate` 起動・JSON レポート判定の
  共通ヘルパー（`negative_cases.rs` からの抽出）。
- `crates/cli/tests/xss_regression_link.rs`: `test` チェックが XSS 回帰テスト
  （TASK-1.2 相当、`crates/core/tests/xss_escape.rs` の代表ペイロードを移植）の
  合否をそのまま反映し、エスケープ実装の退行を `default_escape_check` とは
  独立に BLOCKED へ導くことを固定する正例・負例。
- `.github/workflows/ci.yml`（`test` ジョブ）: `Install cargo-deny
  (pinned + checksum-verified)` ステップで cargo-deny 本体をバージョン
  固定・SHA256 検証付きで導入し、続く `Gate linkage tests (TASK-13.3c:
  cargo-deny / XSS regression)` ステップで `negative_cases.rs`・
  `xss_regression_link.rs` を独立実行する。これにより
  `banned_dependency_blocks_gate_with_policy_failure` の strict 分岐
  （cargo-deny 実行・`banned`/`openssl-sys` ブロック理由の具体性検証）が
  CI 上で常時実行され、TASK-4.1 `deny.toml` ↔ `policy` チェックの連携が
  実証される（cargo-deny 未導入環境向けの fail-closed 分岐は
  `negative_cases.rs` 側の `cargo_deny_available()` 判定が環境非依存に
  担保する）。

イシュー #292（self-hosted runner の環境差による `fw gate` 間欠 BLOCKED）の
対応:

- `tools/ci/ensure-gate-tools.sh`: clippy component（`rustup component add
  clippy`）・cargo-deny（バージョン固定 + SHA256 チェックサム検証付き
  プリビルトバイナリ、atomic install）の存在チェック付きインストールを
  一元化するブートストラップスクリプト。`.github/workflows/ci.yml` の
  test ジョブから呼び出すほか、ローカル開発・AI 自己保守フックが
  `fw gate` 実行前に前置する運用手順として利用する
  （`.claude/rules/ci.md`「ツール前提の明示」節・
  `docs/policy/ai-self-maintenance-policy.md` 参照）。冪等（導入済みなら何もしない）。
- `crates/cli/src/gate.rs` の `clippy_environment_preflight` /
  `cargo_deny_environment_preflight`（§2.3a）: 上記スクリプトが前置されな
  かった場合の安全網。ツール不在を「環境エラー」として決定的に示し、
  コード起因の FAIL との区別を可能にする（自動インストールは行わない）。
- 真の常設化（runner イメージへの焼き込み）は先行イシュー #295
  （インフラ側管理）の領分として継続追跡し、本対応はその安全網として
  位置づける。

イシュー #400（#372/PR #382 で PASS 化した自己適用のリグレッション検出）の
対応:

- `.github/workflows/ci.yml`（`gate-self-apply` ジョブ）: `FANDHE_FRONTEND_WASM_BUILD=0`
  下で `tools/ci/ensure-gate-tools.sh` を前置したうえで
  `cargo run -p fandhe-frontend-cli --locked -- gate --project .` を PR ごと・main push
  ごとに実行し、`gate_result: "PASS"`（終了コード 0）を継続保証する。
  BLOCKED 時は JSON レポートの `checks[].output` 先頭の
  `"environment error: "`（§2.3a・`ENVIRONMENT_ERROR_PREFIX`）の有無で
  GitHub Actions アノテーション（`::error::`）を出し分け、runner 環境未整備と
  コード起因の FAIL（自己参照誤検知の再発・`deny.toml` 弱体化・
  `structure.toml` と実構成のドリフト等）を CI ログ上で判別可能にする。
  緩和経路（`continue-on-error`・スキップ input）は設けず fail-closed を維持する。

## 7. スコープ外

- **意味的脆弱性・ロジック誤りの検出**: `fw gate` が検出できるのは構文・
  ポリシー・エスケープ・テスト網羅の範囲に限定される。意味的な脆弱性・
  ロジック誤りの検出はスコープ外（`docs/spec/04-requirements.md` REQ-13
  「制約・宿題」節、PoC-2・PoC-7 の結論）。
- **AI エージェント本体の実装**: REQ-13 は機構（フック・ゲート）の提供に
  限定され、AI エージェント本体の実装は対象外。
- **自動適用と人間承認の境界ルール**: 「ゲート未通過は無条件拒否、
  `breaking_risk: low` かつ影響ルートなしは自動適用候補、それ以外は人間
  承認必須」という運用ルールの正式文書化は TASK-13.6（#151、
  `docs/policy/ai-self-maintenance-policy.md`）の領分であり、本書では扱わない。
  本書は `fw gate` 単体の判定ルール（何を PASS/BLOCKED とするか）のみを
  対象とする。
- **`fw gate` の振る舞い変更・チェック追加**: 実装は #261/#262/#263 で
  完了済み。本書は現状の実装を正式化するものであり、振る舞いの変更提案は
  別 Issue・別 PR で扱う（out-of-scope-tracking.md、切り出し提案は本 PR の
  本文に記載する）。**例外**: イシュー #401（`url_validation_check` の追加、
  §2.4）は「REQ-1 隣接の既存不変条件（イシュー #373 の URL スキーム検証）が
  レビュー・テストのみに依存し機械検出できていない」という明確な弱体化
  リスクへの対応であり、本原則が想定する「任意のチェック追加提案」の
  対象外として本書内で正式化した。
- **新 API（束縛点・keyed list・Loader）に対するチェック追加は非採用
  （イシュー #353 で判断）**: いずれもノード木 API 経由で HTML を構築し、
  REQ-1 は既存 3 層（`disallowed-methods` lint の主防御 + `default_escape_check`
  の保険層 + XSS 回帰テスト、§2.2）で担保される。`default_escape_check` は
  ソース走査ベースのため新 API の呼び出し自体には反応しないが誤検知もしない
  （束縛点属性文字列・`keyed_list(` 呼び出しは `raw_html()` 呼び出しの
  パターンに一致しない、単体テスト
  `gate::tests::scan_file_ignores_new_api_usage_but_still_detects_unreviewed_raw_html`
  で固定）。Loader の fail-closed 契約は `app`/`server`/`wasm-full` 各クレートの
  既存テスト（#348/#349 で追加済み）が担い、`test` チェック
  （`cargo test --locked -p <crate>`）経由でそのままカバーされる。束縛点更新の
  DOM 反映検証はブラウザテストの領分であり、`fw gate` はネイティブ高速
  チェックに限定する方針を維持する。5 チェック・JSON 契約（PoC-7 互換）は
  本イシューで変更していない。
  束縛点整合性（`data-bind-*` マーカーと状態フィールドの突き合わせ）は
  `fw gate` に追加せず、`wasm-client::binding` のテスト時検証ユーティリティ
  （イシュー #380、`collect_binding_specs`/`unresolved_binding_specs`）が担う。
  この検証は `test` チェック（`cargo test --locked -p <crate>`）経由で
  そのままカバーされる位置付けであり、上記の非採用判断と矛盾しない
  （詳細: `docs/design/dom-binding-update-design.md` #380 追補節）。
- **AI 開発評価軸（明示性・決定性・機械検証可能性・コンテキスト消費）の
  専用チェック追加は非採用（イシュー #381 で判断）**: 4 軸のうち機械判定
  可能な下位項目はすべて既存の機械的担保（`lint`/`test` チェック・
  `default_escape_check`・xtask 依存グラフ計測）で強制済みであり、
  残る「コンテキスト消費の直接計測」は決定的な PASS/FAIL 基準を設計でき
  ずヒューリスティック判定にならざるを得ないため、本書 §2.3a・§3 の
  決定的判定・環境エラー区別の原則と矛盾する。洗い出し表と判断根拠の
  詳細は `docs/policy/intentional-non-adoption.md` §3.12 を参照。5 チェック・
  JSON 契約（PoC-7 互換）は本イシューで変更していない。
