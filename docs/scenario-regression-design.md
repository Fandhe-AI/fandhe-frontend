# 改修シナリオ回帰テスト設計（TASK-13.4a）

> **本書のステータスと前提**: 本書は TASK-13.4（親イシュー #143、成果物
> `cli/tests/scenarios/`）の 4 分割サブタスク（TASK-13.4a シナリオ選定・
> 設計・#144（本書） / TASK-13.4b シナリオ 1（バグ修正）・#145 /
> TASK-13.4c シナリオ 2（UI 改善）・#146 / TASK-13.4d シナリオ 3（機能追加）・
> #147）の先頭に位置づけられる設計文書である。
>
> 本タスクは自動運転モードで実装されている。判断が必要な境界ケース
> （PoC-7 実測値と製品判定ルールの差分の解釈、環境差の吸収方針等）は
> すべて**安全側（fail-closed・環境ごとに最強のアサーションを常時実行・
> テストの弱体化禁止）**に倒して確定し、判断根拠を本書に明記した。
> 人間レビュー（PR）で判断ポイントを確認し、緩和が必要な場合は後続 PR で
> 個別に対応すること。
>
> 本サブタスクの成果物は**設計文書（本書）+ 共有テストハーネスのスケルトン**
> （`cli/tests/scenarios/common.rs` + `cli/tests/scenarios/main.rs` の
> ベースライン smoke test）である。シナリオ 1〜3 の回帰テスト本体は
> 後続 #145〜#147 の領分であり、本書 §8 でスコープ外として明示する
> （TASK-13.1a `docs/structure-manifest.md` / TASK-13.2a
> `docs/impact-analysis-design.md` / TASK-13.3a `docs/gate-design.md` と
> 同型の「設計文書 + スケルトン」構成）。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-13（AI 自己保守・改修のためのフック・ゲート機構、Must）。
  `fw structure`（TASK-13.1）・`fw impact`（TASK-13.2）・`fw gate`
  （TASK-13.3）の 3 サブコマンドが揃った製品 CLI に対し、PoC-7 が検証した
  「代表的な改修シナリオ 3 件（バグ修正・UI 改善・機能追加）」を
  `impact` → 変更適用 → `gate` の一連の流れとして再現し、CI で継続検証する。
- **親タスク**: TASK-13.4（#143、`docs/spec/05-tasks.md` TASK-13.4、
  成果物 `cli/tests/scenarios/`。前提タスク TASK-13.2・TASK-13.3 は
  完了済み）。
- **サブタスク分割**:

| サブタスク | Issue | 内容 | 本書との関係 |
|-----------|-------|------|-------------|
| TASK-13.4a | #144（本書） | シナリオ選定・共有ハーネス設計 | 本書 + `cli/tests/scenarios/{main.rs,common.rs}` の smoke test が単一の情報源 |
| TASK-13.4b | #145 | シナリオ 1（バグ修正: エスケープ回帰）の回帰テスト実装 | 本書 §4.1 行 1・§4.4 が実装契約を規定 |
| TASK-13.4c | #146 | シナリオ 2（UI 改善: 一覧件数サマリー）の回帰テスト実装 | 本書 §4.1 行 2・§4.4 が実装契約を規定 |
| TASK-13.4d | #147 | シナリオ 3（機能追加: タイトル部分一致検索）の回帰テスト実装 | 本書 §4.1 行 3・§4.4 が実装契約を規定 |

- **土台**: PoC-7 `docs/spec/03-poc/ai-self-maintenance/scenarios/`
  （`bugfix-escape-regression/` / `ui-item-count/` / `feature-search/`）に
  保存済みの実測 `impact.json` / `gate*.json`。本書 §4.1 でこれらの実測値と
  製品判定ルール（`docs/impact-analysis-design.md` §3.1〜3.5・
  `docs/gate-design.md` §2）との対応を確定する。
- **先行パターンの踏襲**: `cli/tests/negative_cases.rs`（TASK-13.5、#262
  マージ済み）が確立した「ヘルメチックなフィクスチャ生成
  （`ScratchProject` Drop ガード）・`cargo generate-lockfile --offline`・
  フィクスチャごとの `CARGO_TARGET_DIR` 分離・cargo-deny 有無の環境差吸収
  （`#[ignore]` なしで環境ごとに最強のアサーションを実行）」の方針を
  本書の共有ハーネス設計にそのまま踏襲する。

## 2. 現状（再利用可能な既存資産）

| 項目 | 状態 |
|------|------|
| `fw structure` / `fw gate` | 実装・テスト済み（`cli/src/structure.rs` / `cli/src/gate.rs`）。CLI 配線済み（`cli/src/main.rs`） |
| `fw impact` | 実装・CLI 配線済み（`cli/src/impact.rs`、PR #277 マージ済み・`origin/main` ee61c6b 時点）。TASK-13.4b 以降がこれを前提にできる |
| `cli/tests/negative_cases.rs` | ヘルメチックなフィクスチャ生成・`fw gate` 起動ヘルパ・`check_passed` JSON 抽出ヘルパの実装先例 |
| `cli/tests/structure_integration.rs` | `fw structure` 起動ヘルパ・リポジトリ自身のワークスペースルートを対象にした smoke test の実装先例 |
| PoC-7 実測値 | `docs/spec/03-poc/ai-self-maintenance/scenarios/{bugfix-escape-regression,ui-item-count,feature-search}/` に `impact.json` / `gate*.json` 保存済み |

## 3. 成果物・対象ファイル（本サブタスク #144 分）

| パス | 変更 | 内容 |
|------|------|------|
| `docs/scenario-regression-design.md` | 新規（本書） | シナリオ回帰テスト設計文書一式 |
| `cli/tests/scenarios/main.rs` | 新規 | 統合テストターゲット `scenarios` のエントリ（`mod common;` + ベースライン smoke test 2 件） |
| `cli/tests/scenarios/common.rs` | 新規 | 共有ハーネス（フィクスチャ生成・`fw` 起動ヘルパ・JSON フィールド抽出ヘルパ） |

- `cli/tests/scenarios/scenario1_bugfix.rs` / `scenario2_ui.rs` /
  `scenario3_feature.rs`（TASK-13.4b/c/d の成果物）は本サブタスクでは
  **作成しない**。本書 §4.4 でファイル名・責務のみ確定する。
- `cli/Cargo.toml` の変更は不要（外部依存ゼロを維持。`[[test]]` 宣言なしで
  cargo の auto-discovery に任せる。`tests/scenarios/main.rs` は cargo が
  単一の統合テストターゲット `scenarios` として自動認識する）。

## 4. 設計内容（本タスクで確定する事項）

### 4.1 シナリオ選定（PoC-7 → 製品 CLI 対応表）

選定根拠: `docs/spec/05-tasks.md` TASK-13.4 が「PoC-7 のバグ修正・UI 改善・
機能追加の 3 シナリオ」を明示指定しており、選定は PoC-7 の 3 件を
そのまま採用する（新規シナリオの追加検討は行わない）。

| # | シナリオ | PoC-7 土台 | 対象シンボル | `fw impact` 期待値（製品判定ルール準拠） | `fw gate` 期待値 |
|---|---------|-----------|-------------|--------------------------------------|------------|
| 1 | バグ修正: エスケープ回帰（シングルクォート欠落）の混入 → 修正 | `scenarios/bugfix-escape-regression/` | `render`（core クレート由来） | `affected_crates` 2 件 + クライアント境界クレート波及 → `breaking_risk: high`・`requires_human_approval: true` | 混入時: `test` チェックが `passed:false` → `gate_result: BLOCKED` / 修正後: コア 4 チェック（`type_check`/`default_escape_check`/`lint`/`test`）が `passed:true` |
| 2 | UI 改善: 一覧画面への件数サマリー追加 | `scenarios/ui-item-count/` | `list_page`（app クレート由来） | `affected_crates` 2 件（`rws-server`・クライアント境界クレート） → `breaking_risk: high`・`affected_routes` 非空・`requires_human_approval: true` | 変更適用後: コア 4 チェックが `passed:true`（件数表示の新規アサーション込み） |
| 3 | 機能追加: タイトル部分一致検索（`search_page` + `GET /search`） | `scenarios/feature-search/` | `search_page`（新規追加、app クレート由来） | `affected_crates` 1 件（`rws-server`） → `breaking_risk: medium`・`affected_routes` 非空（新設 `/search` 含む）・`requires_human_approval: true` | 変更適用後: コア 4 チェックが `passed:true`（新規テスト込み） |

#### PoC-7 実測値と製品判定ルールの差分

PoC-7 の `impact.json`（`docs/spec/03-poc/ai-self-maintenance/scenarios/*/impact.json`）
は実測時点のスキーマで保存されており、製品スキーマ（`docs/impact-analysis-design.md`
§3.5）とは以下の点で異なる。シナリオ実装（#145〜#147）はこの差分を
踏まえ、**PoC-7 の JSON をそのまま突き合わせず、製品判定ルールに基づき
期待値を導出し直す**こと。

| 差分項目 | PoC-7 実測値 | 製品スキーマ（本書が正とする） | 根拠 |
|---------|-------------|-------------------------------|------|
| `affected_routes` の型 | オブジェクト配列 `{path, handler, defined_in}` | 文字列配列 `["/items/:id", ...]`（`cli/src/impact.rs` `ImpactReport::affected_routes: Vec<String>`） | `docs/impact-analysis-design.md` §3.5 で確定済み。シナリオテストはパス文字列のみを検証する |
| `verdict` の文言 | 日本語 2 値（「要人間承認（…）」/「自動適用可（…）」） | 英語 2 値（`docs/impact-analysis-design.md` 判断 D1） | `.claude/rules/japanese-style.md`「ユーザー向け文字列は英語」規約と統一 |
| `ambiguous` フィールド | なし | あり（定義元が複数の場合 `true`）。本書のシナリオ 1〜3 はいずれも単一定義のため常に `false` | 製品追加フィールド、`docs/impact-analysis-design.md` §3.2 |
| クライアント境界クレートの扱い | `rws-wasm-client` 単独 | `rws-wasm-client` / `rws-wasm-full` / `rws-wasm-thin` の 3 クレート（[`impact::CLIENT_BOUNDARY_CRATES`]）のいずれかで `high` | `docs/impact-analysis-design.md` §3.2。フィクスチャがクライアント境界クレートを模す場合、3 クレートのうちどれを採用するかはシナリオ実装側（#145/#146）が決めてよいが、名前は `CLIENT_BOUNDARY_CRATES` のいずれかと厳密一致させること |
| `gate.rs` のチェック名表記 | `"type_check(cargo check)"` 等（括弧付き） | `"type_check"` / `"default_escape_check"` / `"lint"` / `"test"` / `"policy"`（括弧なし、`cli/tests/negative_cases.rs` の `check_passed` 実測と一致） | 実装済みコード（`cli/src/gate.rs`）を正とする。PoC-7 の表記はツール（`poc7_tool.py`）独自のものであり、製品出力とは一致しない |

シナリオ 1 の `breaking_risk: high` 判定根拠: `affected_crates` が
`rws-app`・`rws-wasm-client` の 2 件で、うち `rws-wasm-client` が
クライアント境界クレートに該当するため、`docs/impact-analysis-design.md`
§3.4 判定境界表の「1 以上（3 未満含む）・クライアント境界クレートを
含む → high」に該当する（PoC-7 実測の `affected_crates` 件数 2 件・
`breaking_risk: high` と整合）。

シナリオ 3 の `breaking_risk: medium` 判定根拠: `affected_crates` が
`rws-server` の 1 件のみで、クライアント境界クレートを含まないため、
判定境界表の「1〜2 件・クライアント境界クレートを含まない → medium」に
該当する（PoC-7 実測値と一致）。

### 4.2 フィクスチャ設計（ヘルメチック・ミニワークスペース）

`negative_cases.rs` の `write_case_project` パターンを共有ハーネス
（`common.rs`）へ抽出・一般化し、PoC-7 `target-project` 相当の最小
ワークスペースを一時ディレクトリに生成する。

- **構成**: 仮想 workspace + `app`（`negative_cases.rs` と同じ
  `role = "component"`・依存ゼロクレート）。付帯ファイルは
  `structure.toml`（`[manifest] version = 1` + `[directories.app]`）・
  `clippy.toml`（`disallowed-methods` の `rws_core::raw_html` エントリ、
  `templates/default/clippy.toml` 同等）・`deny.toml`（bans/licenses/sources
  最小版）を配布する。`cargo generate-lockfile --offline` により決定的な
  `Cargo.lock` を生成する（`fw gate` は `--locked` で `cargo` を起動するため
  必須、`negative_cases.rs` の実測済み知見）。
- **13.4a（本サブタスク）でのフィクスチャ用途**: ベースライン検証
  （§4.3 のベースライン smoke test）に限定し、この単一クレート構成のまま
  `fw structure` と `fw gate` を通す。
- **13.4b/c 拡張時の指針**: シナリオ 1（バグ修正）・シナリオ 2（UI 改善）は
  `affected_crates` に複数クレート・クライアント境界クレードの波及が
  必要なため、`common.rs` のフィクスチャ生成関数を拡張し、`app` に加えて
  `server`（`rws-server` 役 = role `"server-entrypoint"`）・クライアント
  境界クレート（`rws-wasm-client` 等、role `"client-entrypoint"`）を
  ワークスペースメンバーに追加できるようにする。ルート抽出
  （`fw impact` の `affected_routes`）を伴うシナリオ 2・3 では、
  `structure.toml` に `[routing] definition_dir = "server"` /
  `extractor = "rws-router-v1"` を追加し、`server/src/main.rs` に
  `.route("<path>", get(<handler>))` 形式のルート定義を文字列で
  含める（`cli/src/routes.rs` の抽出器が対象とする構文、axum 等の
  実依存は不要でスタブ文字列のみで足りる。依存ゼロを維持）。
- **変更適用方式**: 「ベースライン文字列に対する一意な部分文字列置換」
  （`negative_cases.rs` の `replace_unique` を踏襲）で、シナリオごとに
  before/after を明確化する。
- **`CARGO_TARGET_DIR` 分離**: フィクスチャごとに専用 `target/` を明示
  指定し、ビルドキャッシュ衝突による偽陰性を防止する
  （`negative_cases.rs` の実測済み知見をそのまま踏襲）。

### 4.3 アサーション設計（環境差吸収を含む）

- `fw impact <symbol> --project <dir>` の JSON
  （`docs/impact-analysis-design.md` §3.5 スキーマ）に対し
  `breaking_risk` / `requires_human_approval` / `affected_routes` /
  `affected_crates` をフィールド単位で検証する。`common.rs` に
  `json_string_field` / `json_bool_field` / `json_string_array_field`
  抽出ヘルパを用意し、`check_passed`（`negative_cases.rs` 由来）と
  同じ「文字列走査による軽量抽出、専用 JSON パーサ依存を持ち込まない」
  方針を踏襲する（`cli` の外部依存ゼロを維持）。
- `fw gate` は総合終了コードではなく**チェック単位**
  （`"name":"<check>","passed":true/false`）で検証する。`policy`
  （cargo-deny）はリポジトリ CI に未導入のため `cargo_deny_available()`
  判定で環境ごとに最強のアサーションを常時実行する（環境依存の
  `#[ignore]`・スキップは行わない — `coding-rust.md`「テストの
  `#[ignore]` 追加でごまかさない」準拠）。
- シナリオ 1 の BLOCKED 判定は「`test` チェックが `passed:false` で
  あること」まで特定して検証し、無関係な失敗理由（型エラー・lint 違反等）
  との混同を防ぐ（`negative_cases.rs` の「ブロック理由の特定性」方針と
  同じ）。

本サブタスク（13.4a）が `cli/tests/scenarios/main.rs` に実装する
ベースライン smoke test 2 件（シナリオ実装の前提健全性を確認する
対照群であり、シナリオ 1〜3 固有のアサーションは含まない）:

1. `baseline_fixture_passes_fw_structure`: フィクスチャが `fw structure`
   を終了コード 0 で通過すること（`cli/tests/structure_integration.rs`
   の smoke test パターンを流用）。
2. `baseline_fixture_passes_gate_core_checks`: フィクスチャが
   `fw gate` のコア 4 チェック（`type_check`/`default_escape_check`/
   `lint`/`test`）すべてで `passed:true` を返すこと（`policy` は
   `cargo_deny_available()` で環境ごとに最強のアサーションを行う、
   `negative_cases.rs` の `baseline_fixture_passes_core_checks` と
   同じ方針）。

### 4.4 後続（13.4b/c/d）への引き継ぎと前提

- **ファイル分担**: `scenario1_bugfix.rs`（#145）/ `scenario2_ui.rs`
  （#146）/ `scenario3_feature.rs`（#147）。いずれも
  `cli/tests/scenarios/` 配下に新設し、`main.rs` の `mod common;` に
  加えて各ファイル冒頭で `#[path = "common.rs"] mod common;` 等の
  形で共有ハーネスを参照する（cargo の統合テストはターゲット単位で
  独立コンパイルされるため、`main.rs` と `scenario1_bugfix.rs` は
  別ターゲットになる点に注意。ターゲットを分けたい場合は各ファイルが
  個別に `common` を include する。ターゲットを分けず単一バイナリに
  収めたい場合は `main.rs` に `mod scenario1_bugfix;` 等で吸収する
  実装判断を #145 以降に委ねる。いずれの構成を選んでも `common.rs` の
  実装自体は複製・分岐させない）。
- **共有ハーネスの拡張が必要な場合の変更指針**: フィクスチャ生成関数
  （§4.2）・JSON フィールド抽出ヘルパ（§4.3）はシナリオ横断で再利用する
  前提のため、シナリオ固有の特殊化（特定シナリオでのみ必要なファイル
  構成等）は `common.rs` に持ち込まず、呼び出し側（各 `scenarioN_*.rs`）
  でパラメータとして渡す設計とする（`common.rs` をシナリオ数だけ
  分岐させない）。
- **前提の明記**: シナリオテストの `fw impact` 呼び出しは
  TASK-13.2c（#135 / PR #277）のマージが前提だったが、**本サブタスク
  着手時点（`origin/main` ee61c6b）で既にマージ済み**であることを
  確認した。13.4b 以降は待ち合わせなしに `fw impact` を呼び出してよい。

## 5. 実装トレーサビリティ

| 設計節 | 対応する実装・テスト |
|-------|----------------------|
| §4.1 シナリオ選定・差分表 | `docs/spec/03-poc/ai-self-maintenance/scenarios/*/impact.json`・`gate*.json`（PoC-7 実測値、参照のみ）/ `cli/src/impact.rs`・`cli/src/gate.rs`（製品判定ロジック、参照のみ） |
| §4.2 フィクスチャ設計 | `cli/tests/scenarios/common.rs`（本サブタスクで新規実装） |
| §4.3 アサーション設計・ベースライン smoke test | `cli/tests/scenarios/main.rs`（本サブタスクで新規実装） |
| §4.4 後続引き継ぎ | #145 / #146 / #147（未実装、本書が実装契約を規定） |

## 6. セキュリティ考慮事項（OWASP Top 10 観点）

- **A03 インジェクション / XSS（REQ-1）**: シナリオ 1 は既定エスケープ
  回帰の検出そのものであり、防御を強めるテストである。フィクスチャ・
  テストコード内で `raw_html` 系トークンを平文で持たない
  （リポジトリ自身の `default_escape_check`・clippy lint への誤検知
  防止。`negative_cases.rs` の「baseline に `raw_html` 文字列を一切
  含まない」方針を踏襲する）。`fw` 起動は `Command` の引数配列渡しの
  みでシェル経由を禁止する（`common.rs` の `run_fw` ヘルパで一元化）。
- **A01/A05 パストラバーサル・設定ミス**: フィクスチャ生成は
  `CARGO_TARGET_TMPDIR`（未設定環境では OS 標準の一時領域）配下に閉じ、
  シンボリックリンクを辿らない既存設計（`routes.rs`）を変更しない。
  fail-closed 原則（黙示的成功を返さない）をアサーション設計に反映する
  （`check_passed` は「チェック自体が JSON に現れない」ことと
  `passed:false` を区別し、`None` を安易に成功扱いしない）。
- **A06/A08 脆弱依存・サプライチェーン**: 新規依存クレート追加ゼロ
  （`cli` の外部依存ゼロを維持、依存グラフ上限 60 件/深さ 6 に影響なし）。
  フィクスチャは `--offline` でロックファイル生成しネットワークアクセス
  なし。禁止依存の実クレート（`openssl-sys` 等）は取得しない。
- **機微情報**: フィクスチャ・設計書にクレデンシャル・実在ホスト名を
  含めない（ダミー値のみ）。
- **テスト弱体化の禁止**: 環境差（`cargo-deny` 有無）を `#[ignore]` や
  スキップで吸収せず、環境ごとに最強のアサーションを常時実行する設計を
  §4.3 で固定する（XSS 回帰テストの削除・弱体化禁止の規約準拠）。

## 7. 既知の限界と将来スコープ

- 影響ルート判定の AST 精密化は `docs/impact-analysis-design.md` §7 で
  将来スコープとして文書化済み（本書での新規起票は不要）。シナリオ
  テストはこの粗粒度ヒューリスティックの現行仕様を前提として期待値を
  導出する。
- シナリオ 1〜3 のフィクスチャはいずれも PoC-7 が検証した「単一シンボル・
  単一変更」の粒度に留める。複数シンボルにまたがる複合的な改修シナリオ
  （例: バグ修正と機能追加の同時実施）は本書のスコープ外であり、
  必要になった場合は新規 Issue として起票することをユーザーに提案する
  （`out-of-scope-tracking.md`）。

## 8. スコープ外（放置しない事項）

- シナリオ 1〜3 の回帰テスト本体の実装 → #145 / #146 / #147（既存
  イシューで追跡済み、新規起票不要）
- 影響ルート判定の AST 精密化 →
  `docs/impact-analysis-design.md` §7 で将来スコープとして文書化済み
  （新規起票不要）
- 複数シンボル・複合的改修シナリオの回帰テスト化（本書 §7）→
  現時点で必要性が確認されていないため起票しない。必要になった時点で
  ユーザー承認のうえ起票する
