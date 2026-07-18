# 検証ゲート（`fw gate`）判定ルール設計（TASK-13.3a）

> **本書のステータスと前提**: 本書は TASK-13.3（親イシュー #138、成果物
> `cli/src/gate.rs`）の 4 分割サブタスク（TASK-13.3a 判定ルール設計・#139
> （本書）/ TASK-13.3b コマンド実装・#140 / TASK-13.3c `cargo-deny`・XSS
> 回帰テスト連携・#141 / TASK-13.3d テスト整備・#142）の先頭に位置づけられる
> 設計文書である。
>
> **重要**: `fw gate` の実装本体はすでに `origin/main` へマージ済みである
> （PR #261 `feat(global): TASK-13.3 検証ゲート（fw gate）の Rust 実装`、
> 追補 PR #262 負例回帰テスト、PR #263 `raw_html` カスタム clippy lint 化）。
> 親 #138 は CLOSED 済み。本書は「これから実装する設計」ではなく、**実装済み
> かつテスト済みのコード（`cli/src/gate.rs`、約 1370 行）が内包する判定ルールを、
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
| TASK-13.3a | #139（本書） | 判定ルール設計の正式化 | 本書が単一の情報源。`cli/src/gate.rs` の実装から逆引きして記述する |
| TASK-13.3b | #140 | `gate` サブコマンドの CLI 接続 | 実装済み（`cli/src/main.rs` の `gate::run_gate` ディスパッチ）。本書 §4 が CLI 契約を規定 |
| TASK-13.3c | #141 | `cargo-deny`・XSS 回帰テスト連携 | 実装済み（本書 §2 の `policy` チェック・`default_escape_check`） |
| TASK-13.3d | #142 | テスト整備 | 実装済み（`cli/tests/gate_integration.rs`・`cli/tests/negative_cases.rs`・`cli/tests/raw_html_lint_e2e.rs`） |

- **土台**: PoC-7 の `docs/spec/03-poc/ai-self-maintenance/tools/poc7_tool.py`
  `cmd_gate`（5 チェックの実行・集約という骨格）からの Rust 移植。PoC-7 の
  「マーカーコメント方式」による既定エスケープ検査は、コンパイラに検証されず
  偽装可能という脆弱性が判明したため（イシュー #157）、製品実装では
  `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]`
  属性方式へ全面的に置き換えている（§2.2・§5 参照）。
- **先行パターンの踏襲**: TASK-13.2a（#133、`docs/impact-analysis-design.md`）
  が確立した「設計文書 + 実装トレーサビリティ表」の構成に倣う。

## 2. 判定ルール本体

`fw gate` は `structure.toml`（[`crate::structure`]、TASK-13.1）を唯一の
情報源として宣言クレート一覧を求め、以下の 5 チェックを**すべて実行**する
（後述 §3 のとおり早期打ち切りはしない）。各チェックは `GateCheck { name,
passed, output }` として結果を持つ。

| # | `name`（JSON） | 目的 | 実行コマンド・判定内容 | 対応 REQ |
|---|----------------|------|------------------------|----------|
| 1 | `type_check` | 型チェック | `cargo check --locked -p <crate>...`（宣言クレートごとに `-p` を連ねる） | REQ-13 |
| 2 | `default_escape_check` | 既定エスケープ検査（保険層） | `role = "core"` 以外の宣言ディレクトリの `src/**/*.rs` を走査し、未レビューの `raw_html()` 呼び出し・ブランケット抑止属性を検出する純粋関数（外部コマンド起動なし） | REQ-1 |
| 3 | `lint` | lint（既定エスケープ検査の主防御を含む） | `cargo clippy --locked -p <crate>... -- -D warnings`。起動前に `clippy.toml` の `disallowed-methods` 設定健全性を検証する（§2.3） | REQ-1・REQ-13 |
| 4 | `test` | テスト | `cargo test --locked -p <crate>...` | REQ-13 |
| 5 | `policy` | 依存ポリシー | `deny.toml` の存在確認 → `cargo deny check bans licenses sources`（`advisories` はネットワーク前提のためオフラインゲート対象外、`docs/cargo-deny-advisories.md` 参照） | REQ-4 |

実行時の作業ディレクトリ（cwd）はいずれも `--project` で指定したプロジェクト
ルート（省略時はカレントディレクトリ）。`--locked` はチェック 1・3・4 に
共通して付与し、ロックファイル逸脱（依存すり替え）を検出する
（security.md A06）。

### 2.1 PASS/FAIL 判定条件

- チェック 1・3・4（`cargo` サブコマンド）: プロセスの終了ステータスが成功
  （exit code 0）であれば `passed = true`。起動失敗・非 0 終了はすべて
  `passed = false`。
- チェック 2（`default_escape_check`）: `violations` リストが空であれば
  `passed = true`。1 件でも違反があれば `passed = false`。
- チェック 5（`policy`）: `deny.toml` が存在し、かつ `cargo deny check bans
  licenses sources` が成功終了であれば `passed = true`。

### 2.2 既定エスケープ検査の 3 層体制

REQ-1 の唯一の許容迂回経路である `raw_html()` の呼び出しを検出するため、
3 層の独立した検出を組み合わせる（[`default_escape_check`] モジュール doc
コメント、`docs/raw-html-lint-design.md` に詳細な脅威モデル・方式比較を持つ）。

1. **主防御（`lint` チェック）**: `cargo clippy` + workspace ルート
   `clippy.toml` の `disallowed-methods` エントリ（`rws_core::raw_html`）。
   コンパイラのパス解決（HIR）に基づくため、コメントでは偽装できない
   （リネームインポート・複数行呼び出しも検出、`raw_html_lint_e2e.rs` で
   実 clippy 起動による検証を持つ）。
2. **保険層（`default_escape_check`）**: テキスト走査による独立検出。受理
   条件は「同一行または直前行に `#[expect(clippy::disallowed_methods,
   reason = "ESCAPE-REVIEWED: ...")]` 属性が存在すること」。旧方式
   （`// ESCAPE-REVIEWED:` コメント単体での受理）はイシュー #157 の指摘を
   受けて廃止済みで、コメントのみのマーカーは違反として検出される
   （`scan_file_rejects_comment_only_marker_as_spoofable` が回帰を固定）。
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

### 2.3 `lint` チェックの起動前ガード（clippy ポリシー健全性）

`lint` チェックは `cargo clippy` を起動する前に、workspace ルート
`clippy.toml` に `disallowed-methods` の `rws_core::raw_html` エントリが
存在するかをテキストで検証する（[`clippy_policy_is_configured`]）。判定は
各行の `#` 以降（TOML コメント）を [`crate::toml::strip_comment`] で除去した
うえで `disallowed-methods` と `rws_core::raw_html` の両文字列を含むかで行う
（コメントアウトされたエントリを「設定済み」と誤判定しないための対策、
PR #263 の Bugbot 指摘対応）。`clippy.toml` の欠落・エントリ欠落は `cargo
clippy` を起動せず即座に `lint` チェックを failed とする（§3 の fail-closed
原則）。

## 3. fail-closed 原則

`fw gate` は「検証できないこと」を暗黙の PASS として扱わない
（security.md A05）。以下の条件はすべて明示的に failed / BLOCKED として
扱われる。

| 条件 | 扱い |
|------|------|
| `structure.toml` の読み込み・パース失敗 | ゲート全体を即座に `BLOCKED`（他チェックを実行しない。宣言クレート一覧が定まらず以降のチェックが無意味になるため） |
| `structure.toml` のセマンティック検証（[`StructureManifest::validate`]）失敗 | ゲート全体を即座に `BLOCKED` |
| 宣言クレートが 0 件（`structure.toml` にどのディレクトリも `crate = "..."` を持たない） | `type_check`/`lint`/`test` を `-p` なしのワークスペース全体検証へフォールバックせず、各チェックを個別に failed とする（[`no_declared_crates_message`]。「検証対象なし＝ PASS」でも「範囲不明な全体検証」でもなく、設定不備として明示する） |
| `deny.toml` が存在しない | `cargo deny` を起動せず `policy` チェックを failed とする |
| `clippy.toml` の欠落・`disallowed-methods` エントリ欠落 | `cargo clippy` を起動せず `lint` チェックを failed とする（§2.3） |
| 外部コマンド（`cargo` 系）の起動自体に失敗（バイナリ不在等） | 該当チェックを failed とする（[`CommandRunner::run`] が起動失敗を `Ok((false, ...))` として返し、呼び出し元は `Err` 分岐を用意せず fail-closed 集約する） |

## 4. 集約規則と CLI 契約

- **集約**: 5 チェックすべてを実行し、早期打ち切りはしない（AI エージェントが
  一括修正できるよう全違反を報告する PoC-7 の方針を踏襲、[`run_all_checks`]）。
  全チェック `passed = true` であれば `gate_result = "PASS"`、1 件でも
  `passed = false` であれば `gate_result = "BLOCKED"`（[`aggregate`]）。
- **終了コード規約**（`main.rs` 冒頭 doc コメントと同一契約）:
  - `0`: `gate_result = "PASS"`
  - `1`: `gate_result = "BLOCKED"`（`structure.toml` 読み込み・検証失敗を含む）
  - `2`: 使用法エラー（`--project` 引数の解析失敗・未知フラグ）
- **JSON 出力契約**（PoC-7 互換の形状、[`render_report`]）:

```json
{
  "checks": [
    { "name": "type_check", "passed": true, "output": "..." },
    { "name": "default_escape_check", "passed": true, "output": "..." },
    { "name": "lint", "passed": true, "output": "..." },
    { "name": "test", "passed": true, "output": "..." },
    { "name": "policy", "passed": true, "output": "..." }
  ],
  "gate_result": "PASS",
  "action": "all checks passed; changes may proceed"
}
```

  - `checks[].output` は末尾 `OUTPUT_TRUNCATE_CHARS = 4000` 文字に丸められる
    （§5 参照）。
  - `action` は `gate_result` に応じた固定文言（`PASS`: `"all checks passed;
    changes may proceed"` / `BLOCKED`: `"fix the reported failing checks and
    re-run \`fw gate\`"`。`structure.toml` 段階の即時 `BLOCKED` は専用の
    `"fix structure.toml and re-run \`fw gate\`"`）。
- **利用契約**: AI 自己保守フック・CI は本サブコマンドの終了コードと JSON の
  `gate_result` を照合し、変更適用の可否を判断する。TASK-13.6（#151、
  `docs/ai-self-maintenance-policy.md`）が「ゲート未通過は無条件拒否」という
  境界ルールを正式化する際、本書の終了コード規約・`gate_result` 値をそのまま
  前提として参照する。

## 5. セキュリティ不変条件

- **A03（インジェクション）**: 外部コマンドは [`std::process::Command`] に
  引数配列で渡し、シェル文字列連結を行わない（`RealCommandRunner::run`）。
- **A05（セキュリティ設定ミス、fail-closed）**: §3 に列挙した全条件。
- **A06（脆弱な依存）**: `type_check`/`lint`/`test` すべてに `--locked` を
  付与し、ロックファイル逸脱（依存すり替え）を検出する。
- **A08（ソフトウェア・データ整合性の失敗）**: `default_escape_check` の
  3 層検出（§2.2）が REQ-1 の唯一の迂回経路である `raw_html()` の呼び出しを
  明示レビュー済み宣言に限定する契約を担保する。詳細な脅威モデル・方式比較は
  `docs/raw-html-lint-design.md`、レビュー運用は `docs/raw-html-review-gate.md`
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
  （`cli/src/routes.rs` の `list_rs_files_inner` と同一方針）。

## 6. 実装トレーサビリティ

| 本書の章 | `cli/src/gate.rs` の対応箇所 |
|---------|------------------------------|
| §2 表（5 チェック定義） | `run_all_checks`（169-185 行目）、`run_cargo_check`/`run_cargo_clippy`/`run_cargo_test`/`policy_check`/`default_escape_check` |
| §2.2（3 層体制） | モジュール doc コメント（1-35 行目）、`find_raw_html_call_positions`（420-440 行目）、`line_has_reviewed_expect_attribute`（521-528 行目）、`line_has_real_blanket_attribute`（487-497 行目）、`scan_file_for_violations`（607-667 行目） |
| §2.3（clippy ポリシー健全性） | `clippy_policy_is_configured`（292-302 行目）、`clippy_policy_check`（315-329 行目） |
| §3（fail-closed） | `run_gate`（97-153 行目、structure.toml 段階）、`no_declared_crates_message`（219-231 行目）、`run_locked_cargo_subcommand`（237-262 行目）、`run_cargo_clippy`（331-369 行目）、`policy_check`（383-405 行目） |
| §4（集約規則・CLI 契約） | `aggregate`（189-204 行目）、`render_report`（673-697 行目）、`main.rs` の終了コード規約（`main.rs` 33-35 行目） |
| §5（セキュリティ不変条件） | `RealCommandRunner::run`（74-85 行目）、`truncate_output`（207-217 行目）、`OUTPUT_TRUNCATE_CHARS`（44 行目）、`scan_dir_for_violations`（579-597 行目） |

対応するテストは `cli/tests/gate_integration.rs`（CLI 経由の統合テスト、
6 ケース）・`cli/tests/negative_cases.rs`（3 負例: 型エラー・未エスケープ・
禁止依存、TASK-13.5）・`cli/tests/raw_html_lint_e2e.rs`（実 clippy 起動に
よる主防御の実証、4 ケース）・`gate.rs` 内 `#[cfg(test)] mod tests`（約 30
ユニットテスト）。本書執筆時点でこれら全テストは `cargo test -p rws-cli`
でグリーンであることを確認済み。

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
  `docs/ai-self-maintenance-policy.md`）の領分であり、本書では扱わない。
  本書は `fw gate` 単体の判定ルール（何を PASS/BLOCKED とするか）のみを
  対象とする。
- **`fw gate` の振る舞い変更・チェック追加**: 実装は #261/#262/#263 で
  完了済み。本書は現状の実装を正式化するものであり、振る舞いの変更提案は
  別 Issue・別 PR で扱う（out-of-scope-tracking.md、切り出し提案は本 PR の
  本文に記載する）。
