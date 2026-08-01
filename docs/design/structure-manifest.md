# structure.toml スキーマ設計と `fw structure`（TASK-13.1）

> **本書のステータスと前提**: 本書は TASK-13.1（親イシュー #127）の 4h
> 分割サブタスク全体（TASK-13.1a スキーマ設計・#128 / TASK-13.1b パーサ実装・
> #129 / TASK-13.1c マニフェスト生成・#130 / TASK-13.1d テスト整備・#131）の
> 成果物であり、すべて実装済みです。スキーマ設計の経緯・判断根拠（§2）に
> 加え、実装の到達点（§4）・`fw structure` の使い方（§4 末尾）を記載します。
>
> TASK-13.1 は自動運転モードで実装されています。判断が必要な境界ケースは
> すべて**安全側（fail-closed・未知キーはエラー・依存追加なし）**に倒して
> 確定し、判断根拠を本書に明記しました。人間レビュー（PR）で判断ポイントを
> 確認し、緩和が必要な場合は後続 PR で個別に対応してください。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-13（AI 自己保守・改修のためのフック・ゲート機構、Must）が
  第 1 要素として要求する「機械可読なプロジェクト構造」（`docs/spec/04-requirements.md`
  168 行目以降）。
- **背景**: PoC-7（`docs/spec/03-poc/ai-self-maintenance/`）が
  `structure.toml` + `tools/poc7_tool.py structure` の Python プロトタイプで
  実証した。本タスクはこれを Rust CLI（`crates/cli/src/structure.rs`）として
  製品化する最初のサブタスクであり、**後続 #129〜#131 が依拠する単一の
  情報源**（スキーマ設計 + 型定義 + 参照マニフェスト）を確定させる。
- **親タスク**: TASK-13.1（#127、`docs/spec/05-tasks.md` 参照）。
- **サブタスク分割**:

| サブタスク | Issue | 内容 | 本書との関係 |
|-----------|-------|------|-------------|
| TASK-13.1a | #128（本書） | スキーマ設計 | 本書 + `crates/cli/src/structure.rs` の型定義 + ルート `structure.toml` |
| TASK-13.1b | #129 | TOML サブセットの手書きパーサ実装 | 本書 §2 が構文サブセット・エラー方針を規定 |
| TASK-13.1c | #130 | `cargo metadata` 連携・マニフェスト生成・`fandhe-frontend-router-v1` 抽出器実装 | 本書 §2.2/§2.3 がスキーマ・検証範囲の境界を規定 |
| TASK-13.1d | #131 | ルートの `structure.toml` をフィクスチャとした統合テスト | 本書のスキーマ・ルート `structure.toml` を直接使用 |

## 2. スキーマ v1

### 2.1 TOML サブセット制約

TASK-13.1b の手書きパーサ（`toml`/`serde` 等の外部クレートを使わない、
`coding-rust.md` の依存グラフ上限・`cli` 外部依存ゼロ方針に基づく）が
対応すべき構文を、あらかじめ狭いサブセットに絞る。

- **許可**: 標準テーブル `[a.b]`、`key = "文字列"`、整数、文字列配列
  （`["a", "b"]`）、コメント（`#`）、UTF-8。
- **禁止**: インラインテーブル、複数行文字列、dotted key、日時型、float。
- **未知キー・サブセット外構文はエラー**（寛容パースはしない）。
  `structure.toml` は「宣言の機械的検証」という性格上、タイプミス・
  スキーマ外フィールドを黙って無視すると検証自体の信頼性が損なわれるため。

### 2.2 スキーマ構造

```toml
[manifest]
version = 1                     # スキーマバージョン（整数・必須）

[directories.<name>]            # <name>: ^[a-z0-9_-]+$ のワークスペース相対ディレクトリ名（論理名）
role = "core"                   # 閉じた語彙（§2.2.1）
crate = "fandhe-frontend-core"               # 対応クレート名（キー省略可。空文字は不可）
description = "..."              # 役割の説明（必須）
depends_on = ["..."]              # 依存を許可する directories キー（既定 []）
allowed_dependents = ["..."]      # 被依存を許可する directories キー（既定 []）
path = "crates/core"             # 任意。実配置パス（§2.2.0a）。省略時は <name> がそのまま実体（フラット配置）

[routing]
definition_dir = "app"           # ルート定義を許すディレクトリ（directories キー参照）
extractor = "fandhe-frontend-router-v1"      # 組み込み抽出器 ID
```

#### 2.2.0 予約名 `root`（イシュー #353 で正式化）

`[directories.<name>]` の `<name>` に予約名 `root`（`crates/cli/src/structure.rs::ROOT_DIR_KEY`）
を使うと、そのディレクトリはワークスペースルート**自身**（クレートが
プロジェクトルート直下 `<project>/src` に直接配置される構成）を表す。
現行スキーマの命名規則（`^[a-z0-9_-]+$`）にはワークスペースルート自身を
指す記号（`.` 等）が無いため、通常のディレクトリ名と同じ命名規則の範囲内で
`root` を予約語として扱う（TOML 形式・既存フィールドは不変。スキーマ v1 の
**意味論の明確化**であり `[manifest] version` の bump は不要）。

`fw new`（イシュー #350/#351）が生成するプロジェクトは常にこの慣習を使う
（`templates/default/structure.toml` の `[directories.root]`）。

ディレクトリ名 → 実ファイルシステムパスの解決は `crates/cli/src/structure.rs::dir_fs_path`
を**単一の情報源**とし、`root` は `<project>` 自身へ、それ以外は従来どおり
`<project>/<name>` へ写像する。以下の消費箇所はいずれもこのヘルパー
（または `routes::resolve_within_root`／`routes::scan_root` に一般化した
同じ写像）を経由し、`root` 慣習の特例を個別に再実装しない:

| 消費箇所 | 用途 |
|---------|------|
| `main.rs::run_structure` | ディレクトリ実在確認 |
| `gate.rs::escape_check_src_dir` | `default_escape_check`（保険層）の走査対象解決 |
| `routes.rs::resolve_within_root` / `scan_root` | ルート抽出・コンポーネント境界抽出・`fw impact` 走査の走査起点解決 |
| `impact.rs::member_dir_name` | `cargo metadata` の member（`manifest_dir == workspace_root`）から `root` への逆変換 |

`root` 慣習下でも走査はワークスペースルート「全体」ではなく `<project>/src`
に限定する（`target/` 等の混入・過検知防止、`routes.rs::scan_root` の
既存方針をそのまま適用）。

#### 2.2.0a `path` キー（任意の実配置パス、イシュー #436）

`[directories.<name>]` の `<name>`（論理名）は `depends_on` / `allowed_dependents` /
`[routing] definition_dir` が参照する**依存宣言の語彙**であり、実ファイルシステム上の
配置とは独立させたい場合がある（例: 本リポジトリ自身の `crates/` 配下移設で、
論理名 `core` を維持したまま実体を `<workspace_root>/crates/core` へ動かす）。

この目的のため、任意キー `path`（ワークスペースルート相対の実配置パス文字列）を
追加できる。省略時は従来どおり `<name>` がそのまま実体（フラット配置、`fw new` が
生成するユーザープロジェクトの既定）。

制約（`validate()` が fail-closed で検証、OWASP A01/A05 対策）:

- `/` 区切りで 1〜3 セグメント（`crates/cli/src/structure.rs::MAX_PATH_SEGMENTS`）
- 各セグメントは `^[a-z0-9_-]+$`（[`is_valid_directory_name`] と同じ文字集合）
- 絶対パス（先頭 `/`）・`..`・空セグメント（連続 `/`・末尾 `/`）は拒否
- 予約名 `root`（§2.2.0）のエントリに `path` を指定することは拒否
  （`root` は常にワークスペースルート自身へ写像する意味論のため）
- 複数エントリが同一の解決先へ写像することは拒否（`path` 省略時の
  `<name>` 自身との衝突も含む）

ディレクトリ → 実ファイルシステムパスの解決は `crates/cli/src/structure.rs::
dir_fs_path_for_entry`（`path` があればそれを、無ければ `dir_fs_path` の
従来ロジックへフォールバック）を単一の情報源とする。走査系ヘルパー
（`routes::resolve_within_root`/`scan_root`）は `dir_name` 引数として
`/` 区切りの多段パスを受け付けるよう一般化されており、`StructureManifest::
resolved_dir_path(name)` がディレクトリキー名から実配置パス文字列への
変換窓口を担う（`main.rs`・`impact.rs::member_dir_name` も同じ制約
（段数上限・文字集合検証）を共有する）。

#### 2.2.1 `role` の閉じた語彙

PoC-7 は `role` を自由記述文字列としていたが、本スキーマでは機械的に
判定できるよう固定語彙にする（`crates/cli/src/structure.rs` の `Role` enum が
唯一の定義。パーサ側で語彙を再定義しない）。

| 値 | 意味 | 本リポジトリでの対応例 |
|----|------|------------------------|
| `core` | 外部依存ゼロの描画コア | `core` |
| `state` | 状態管理層 | `interactive` |
| `component` | モード非依存の共通コンポーネント | `app` |
| `server-entrypoint` | SSR/SSG/ルーティングのサーバーエントリ | `server` |
| `client-entrypoint` | CSR/ハイドレーションのクライアントエントリ | `wasm-full` / `wasm-thin` |
| `distribution` | 単一バイナリ配布層 | `dist-server` |
| `asset` | 静的アセット（対応クレートなし） | `static` |
| `tooling` | 開発者・CI 用ツール | `xtask` / `cli` |

**`fw gate` 静的専用（asset-only）モードとの関係（イシュー #410）**:
宣言クレートが 0 件、かつ宣言ディレクトリ全件が `role = "asset"` である
マニフェスト（`fw new --template embed` が生成する `templates/embed/
structure.toml` が代表例）は、`fw gate`（`crates/cli/src/gate.rs::
is_asset_only_project`）にとって「cargo パッケージを持たない静的専用
プロジェクトである」ことの明示的オプトイン宣言として機能する。この場合
cargo 系 4 チェック（`type_check`/`lint`/`test`/`policy`）は not-applicable
PASS 化され、テキスト走査ベースの保険層（`default_escape_check`/
`url_validation_check`）のみが通常どおり実行される（詳細判定ルールは
`docs/design/gate-design.md` §2.5 を参照。本ファイルはスキーマの語彙定義
までを担い、`fw gate` 側の解釈規則は同文書を単一の情報源とする）。

#### 2.2.2 PoC-7 からの主な変更点と理由

| 変更 | 理由 |
|------|------|
| `[manifest] version` を新設 | 前方互換の判定基盤。TASK-13.1b 以降がバージョン分岐できるようにする |
| `crate = ""`（空文字）を廃止し「キー省略」に統一 | 空文字と未設定の二重表現を避ける。パーサ側の分岐を単純化 |
| `role` を自由記述からクローズドな語彙へ変更 | `validate()` が機械的に判定できるようにする（§2.2.1） |
| `[routing] handler_pattern`（ユーザー定義正規表現）を廃止し、組み込み抽出器 ID の選択式（`extractor`）に変更 | `crates/server/src/router.rs` は正規表現・バックトラックを排した設計（DoS 耐性を狙う実装判断）。マニフェスト経由で任意正規表現をツール側に実行させる経路は、宣言ファイルがコード実行相当の振る舞い（ReDoS・意図しないパターンマッチによる誤爆）を注入できる面を増やすため、v1 スキーマから排除した。抽出器自体の実装は TASK-13.1c（#130）のスコープ |
| `[routing] definition_file_pattern`（glob 文字列）を `definition_dir`（`directories` キー参照）に変更 | 「ルート定義を許すディレクトリ」を独自の glob 文字列ではなく既存の `directories` 宣言に統一し、二重管理・書式の不一致を避ける |

#### 2.2.3 宣言単位は cargo パッケージであり、ターゲット（bin/lib）ではない（イシュー #1115）

`[directories.<name>]` の宣言単位は **cargo パッケージ（クレート）** であり、
そのパッケージが持つターゲット（`src/main.rs` の bin ターゲット・
`src/lib.rs` の lib ターゲット等）ではない。`fw gate`
（`crates/cli/src/gate.rs::run_gate`）は宣言済みディレクトリの `crate`
（パッケージ名）へ `cargo check`/`cargo clippy`/`cargo test` を `-p <crate>`
指定で実行する。cargo の `-p` はパッケージ単位の指定でありパッケージが持つ
全ターゲット（bin/lib/tests 等）を包含するため、`fw new --template app` が
生成する bin のみのプロジェクトへ利用者が `src/lib.rs` を追加して
bin + lib 構成に拡張しても、`structure.toml`（本ファイルが定義するスキーマの
実インスタンス）の更新は不要であり、`fw gate` は無編集のまま lib ターゲットも
検証対象に含めて PASS する。

この挙動は `fw gate` 側の実装最適化ではなく、本スキーマが「クレート単位の
宣言」として設計されていること（§2.2 冒頭・`[directories.<name>].crate` が
パッケージ名を表す）から構造的に導かれる。テンプレート同梱の
`templates/app/structure.toml` にも同旨のコメントを明記している
（`fw new --template app` 生成直後のプロジェクトで利用者が直接参照できる
ようにするため）。

### 2.3 整合性検証ルール（`StructureManifest::validate()`）

`crates/cli/src/structure.rs` の `validate()` が実装する、マニフェスト**内部**の
宣言整合性検証。ファイルシステム・`cargo metadata` へは一切アクセスしない
純粋関数（実体との突き合わせは TASK-13.1c のスコープ、§4 参照）。

1. `directories` は 1 件以上（`ValidationError::NoDirectories`）。
2. ディレクトリ名は `^[a-z0-9_-]+$`
   （`ValidationError::InvalidDirectoryName`）。絶対パス・`..`・パス
   区切り文字を含む名前を拒否することで、TASK-13.1c 以降のファイル
   走査がワークスペース外へ出るパストラバーサル面を仕様段階で塞ぐ
   （OWASP A01 破損アクセス制御 / A05 セキュリティ設定ミス対策）。
3. `directories` 内に同名のキーが複数存在してはならない
   （`ValidationError::DuplicateDirectoryName`）。名前は参照解決の
   過程で `HashSet<&str>` に集約されるため、重複を許すと 2 件目以降が
   握りつぶされ、後続の参照検証・対称性検証が `find()` で最初に
   一致した要素にのみ束縛される（内部一貫性が実際には保証されない）。
4. `depends_on` / `allowed_dependents` の各要素は宣言済み `directories`
   キーを参照する（`ValidationError::UnknownReference`）。自己参照
   （`ValidationError::SelfReference`）・重複
   （`ValidationError::DuplicateReference`）は拒否する。
5. `role = "core"` のエントリは `depends_on = []` を強制する
   （`ValidationError::CoreRoleHasDependencies`）。REQ-3 の core 外部
   依存ゼロ規約をマニフェスト宣言側にも反映する。
6. 対称性: A の `depends_on` に B が含まれるなら、B の
   `allowed_dependents` に A が含まれること。逆方向（B の
   `allowed_dependents` に A が含まれるなら、A の `depends_on` に B が
   含まれること）も同様に検証する（`ValidationError::AsymmetricDependency`）。
   両フィールドは宣言の双方向であるため、どちらの片落ちも見逃さない。
7. `[routing] definition_dir` は宣言済み `directories` キーを参照する
   （`ValidationError::UnknownRoutingDefinitionDir`）。
8. `validate()` は検出した違反を 1 件で打ち切らず、可能な限りすべて
   収集して返す（`Result<(), Vec<ValidationError>>`）。エラーメッセージは
   `japanese-style.md` の方針（ユーザー向け文字列は英語）に従い英語。

## 3. ルートの `structure.toml`（参照マニフェスト）

リポジトリルートの `structure.toml` は、本リポジトリ自身の構成
（`Cargo.toml` の workspace members + `static/` + `crates/xtask/` + `crates/cli/`）を
現行のスキーマ v1 で宣言した参照マニフェストであり、以下を兼ねる:

- 2.2 節スキーマの正例
- TASK-13.1d（#131）の統合テストフィクスチャ（予定）

構成上の留意点:

- `server`（`fandhe-frontend-server`）は TASK-6.1c で `ssr.rs`/`ssg.rs` が `fandhe-frontend-app` の
  ページ関数を呼ぶようになったため、`fandhe-frontend-core`/`fandhe-frontend-app` を
  `crates/server/Cargo.toml` の**通常依存**（`[dependencies]`、path 依存のみ・
  外部クレート追加なし）に昇格済みである。`directories.server.depends_on`
  はこれを反映して `["core", "app"]` を宣言する（TASK-13.1c の
  `cargo metadata` 実体突き合わせが、この宣言と実際の path 依存の一致を
  検証する）。
- `wasm-full`（`fandhe-frontend-wasm-full`）は TASK-CSR-loader（#349）で `csr` モジュール
  （CSR 経路の loader 解決層）が `fandhe_frontend_app::Loader`/`assemble_list_page`/
  `assemble_detail_page` を呼ぶようになったため、`fandhe-frontend-app` を
  `crates/wasm-full/Cargo.toml` の**通常依存**（`[dependencies]`、path 依存のみ・
  外部クレート追加なし）に追加済みである。`directories.wasm-full.depends_on`
  はこれを反映して `["core", "interactive", "app"]` を、
  `directories.app.allowed_dependents` は `["server", "dist-server",
  "wasm-full"]` を宣言する（対称性、本節冒頭 §2.3 検証 6）。
- `[routing] definition_dir = "app"` は「ルートは `crates/app/src/routes.rs` の
  `Router::route(...)` 呼び出しに定義される」という規約を宣言する
  （イシュー #407: server / client 単一定義からのルート生成（共有機構）
  採用に伴い、ルート表の正本を `server` から `app` へ移設した。
  `crates/server/src/ssr.rs`・`crates/wasm-full/src/nav.rs` はいずれも `fandhe_frontend_app::routes`
  経由で同一定義を参照する。詳細は `docs/design/route-definition-sharing.md`）。
  実際の抽出処理（`fandhe-frontend-router-v1` 抽出器、`crates/cli/src/routes.rs`）は
  `definition_dir` 配下の `src/`（Cargo の慣例に基づき `tests/` 等の
  integration test は対象外）を走査し、コメント・`#[cfg(test)]` 以降の
  内部テストも除外したうえで抽出する（TASK-13.1c 実装済み）。

## 4. スコープの境界（TASK-13.1b/c との責務分担・実装状況）

TASK-13.1（親 #127）の全サブタスクは実装済み。

- TASK-13.1a（#128）: スキーマの型定義（`crates/cli/src/structure.rs`）と
  マニフェスト**内部**の宣言整合性検証（`validate()`、§2.3）。
- TASK-13.1b（#129）: `crates/cli/src/toml.rs`（TOML サブセットパーサ）+
  `crates/cli/src/structure.rs` の `parse()`/`load()`（TOML → `StructureManifest`
  への変換とセマンティック検証）。未知キー・サブセット外構文・未知 `role` は
  すべてエラー（fail-closed）。
- TASK-13.1c（#130）: `crates/cli/src/metadata.rs`（`cargo metadata` 連携・
  workspace member と path 依存の抽出）、`crates/cli/src/routes.rs`
  （`fandhe-frontend-router-v1` 抽出器）、`crates/cli/src/component_boundary.rs`
  （コンポーネント境界抽出）、`crates/cli/src/json_out.rs`（4 要素の JSON 出力）。
  `main.rs` の `run_structure` がこれらを結線し、宣言と実体の差分
  （crate 実在・依存の宣言漏れ / 過剰宣言・ディレクトリ実在）を検出した
  場合は非 0 終了で列挙する。
- TASK-13.1d（#131）: `crates/cli/tests/structure_integration.rs`
  （ルートの `structure.toml` をフィクスチャとした `fw` バイナリ起動の
  統合テスト・負例テスト）+ `crates/cli/src/*.rs` 内の単体テスト。

`fw structure`（引数省略時はカレントディレクトリ、`--project <dir>` で
対象を指定）は、リポジトリルートで実行すると 4 要素
（`directories` / `routes` / `component_boundary` / `dependencies`）を
含む JSON を標準出力へ 1 行で出力し、終了コード 0 を返す:

```console
$ cargo run -p fandhe-frontend-cli --bin fw -- structure
{"directories":[...],"routes":[...],"component_boundary":[...],"dependencies":{...}}
```

パース失敗・宣言整合性違反・実体との不一致のいずれかがあれば、検出した
問題をすべて英語メッセージで標準エラーへ列挙し、終了コード 1 を返す
（黙示的成功を返さない、`main.rs` の契約）。

## 5. スコープ外事項（本タスクでは対応しない）

- **impact/gate 用フィールド**（`breaking_risk` 等の付加）: 既存
  イシュー（#132 / #138）で追跡する。
- **影響範囲判定の AST 精密化**: REQ-13 自体が現時点でスコープ外と
  している制約（`docs/spec/04-requirements.md`）。
- **抽出器の複数対応・正規表現ベース抽出の一般化**: 必要になった時点で
  fandhe-frontend-spec 側の仕様検討を提案する。
- 仕様（`docs/spec/`）自体の変更は本リポジトリでは行わない
  （PoC-7 マニフェストとの差分理由は本書側に記載した）。
- **`[loaders]` / 束縛点セクションの新設（イシュー #353 で非採用と判断）**:
  `app::Loader` の実装（`impl Loader for <Type>`）は通常の pub Rust シンボルと
  同じ扱いであり、`role = "component"` ディレクトリのコンポーネント境界抽出・
  `fw impact <symbol>` の走査（`affected_loaders`、§4 参照）で既にカバーされる。
  束縛点（`data-bind-text` 等）は HTML/ソース中の属性文字列として現れ
  `grep` 等で機械検証可能（`docs/design/dom-binding-update-design.md` §7.4 の
  明示性）。マニフェストへの重複宣言はコードとのドリフト（二重管理）を
  生みやすく、`[routing] handler_pattern` を v1 で廃止した理由（§2.2.2）と
  同じ判断で、新セクションは追加しない。
