# structure.toml スキーマ設計（TASK-13.1a）

> **本書のステータスと前提**: 本書は TASK-13.1（親イシュー #127）の 4h
> 分割サブタスクのうち **スキーマ設計**（TASK-13.1a・本イシュー #128）の
> 成果物です。兄弟サブタスクは TASK-13.1b（パーサ実装、#129）・
> TASK-13.1c（マニフェスト生成、#130）・TASK-13.1d（テスト整備、#131）
> であり、TOML テキストの実パース・`cargo metadata` との突き合わせ・
> CI 組み込みはいずれも本書のスコープ外です（`docs/npm-static-asset-rules.md`
> と同型の設計契約ドキュメント）。
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
  実証した。本タスクはこれを Rust CLI（`cli/src/structure.rs`）として
  製品化する最初のサブタスクであり、**後続 #129〜#131 が依拠する単一の
  情報源**（スキーマ設計 + 型定義 + 参照マニフェスト）を確定させる。
- **親タスク**: TASK-13.1（#127、`docs/spec/05-tasks.md` 参照）。
- **サブタスク分割**:

| サブタスク | Issue | 内容 | 本書との関係 |
|-----------|-------|------|-------------|
| TASK-13.1a | #128（本書） | スキーマ設計 | 本書 + `cli/src/structure.rs` の型定義 + ルート `structure.toml` |
| TASK-13.1b | #129 | TOML サブセットの手書きパーサ実装 | 本書 §2 が構文サブセット・エラー方針を規定 |
| TASK-13.1c | #130 | `cargo metadata` 連携・マニフェスト生成・`rws-router-v1` 抽出器実装 | 本書 §2.2/§2.3 がスキーマ・検証範囲の境界を規定 |
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

[directories.<name>]            # <name>: ^[a-z0-9_-]+$ のワークスペース相対ディレクトリ名
role = "core"                   # 閉じた語彙（§2.2.1）
crate = "rws-core"               # 対応クレート名（キー省略可。空文字は不可）
description = "..."              # 役割の説明（必須）
depends_on = ["..."]              # 依存を許可する directories キー（既定 []）
allowed_dependents = ["..."]      # 被依存を許可する directories キー（既定 []）

[routing]
definition_dir = "server"        # ルート定義を許すディレクトリ（directories キー参照）
extractor = "rws-router-v1"      # 組み込み抽出器 ID
```

#### 2.2.1 `role` の閉じた語彙

PoC-7 は `role` を自由記述文字列としていたが、本スキーマでは機械的に
判定できるよう固定語彙にする（`cli/src/structure.rs` の `Role` enum が
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

#### 2.2.2 PoC-7 からの主な変更点と理由

| 変更 | 理由 |
|------|------|
| `[manifest] version` を新設 | 前方互換の判定基盤。TASK-13.1b 以降がバージョン分岐できるようにする |
| `crate = ""`（空文字）を廃止し「キー省略」に統一 | 空文字と未設定の二重表現を避ける。パーサ側の分岐を単純化 |
| `role` を自由記述からクローズドな語彙へ変更 | `validate()` が機械的に判定できるようにする（§2.2.1） |
| `[routing] handler_pattern`（ユーザー定義正規表現）を廃止し、組み込み抽出器 ID の選択式（`extractor`）に変更 | `server/src/router.rs` は正規表現・バックトラックを排した設計（DoS 耐性を狙う実装判断）。マニフェスト経由で任意正規表現をツール側に実行させる経路は、宣言ファイルがコード実行相当の振る舞い（ReDoS・意図しないパターンマッチによる誤爆）を注入できる面を増やすため、v1 スキーマから排除した。抽出器自体の実装は TASK-13.1c（#130）のスコープ |
| `[routing] definition_file_pattern`（glob 文字列）を `definition_dir`（`directories` キー参照）に変更 | 「ルート定義を許すディレクトリ」を独自の glob 文字列ではなく既存の `directories` 宣言に統一し、二重管理・書式の不一致を避ける |

### 2.3 整合性検証ルール（`StructureManifest::validate()`）

`cli/src/structure.rs` の `validate()` が実装する、マニフェスト**内部**の
宣言整合性検証。ファイルシステム・`cargo metadata` へは一切アクセスしない
純粋関数（実体との突き合わせは TASK-13.1c のスコープ、§4 参照）。

1. `directories` は 1 件以上（`ValidationError::NoDirectories`）。
2. ディレクトリ名は `^[a-z0-9_-]+$`
   （`ValidationError::InvalidDirectoryName`）。絶対パス・`..`・パス
   区切り文字を含む名前を拒否することで、TASK-13.1c 以降のファイル
   走査がワークスペース外へ出るパストラバーサル面を仕様段階で塞ぐ
   （OWASP A01 破損アクセス制御 / A05 セキュリティ設定ミス対策）。
3. `depends_on` / `allowed_dependents` の各要素は宣言済み `directories`
   キーを参照する（`ValidationError::UnknownReference`）。自己参照
   （`ValidationError::SelfReference`）・重複
   （`ValidationError::DuplicateReference`）は拒否する。
4. `role = "core"` のエントリは `depends_on = []` を強制する
   （`ValidationError::CoreRoleHasDependencies`）。REQ-3 の core 外部
   依存ゼロ規約をマニフェスト宣言側にも反映する。
5. 対称性: A の `depends_on` に B が含まれるなら、B の
   `allowed_dependents` に A が含まれること
   （`ValidationError::AsymmetricDependency`）。宣言の片落ちを検出する。
6. `[routing] definition_dir` は宣言済み `directories` キーを参照する
   （`ValidationError::UnknownRoutingDefinitionDir`）。
7. `validate()` は検出した違反を 1 件で打ち切らず、可能な限りすべて
   収集して返す（`Result<(), Vec<ValidationError>>`）。エラーメッセージは
   `japanese-style.md` の方針（ユーザー向け文字列は英語）に従い英語。

## 3. ルートの `structure.toml`（参照マニフェスト）

リポジトリルートの `structure.toml` は、本リポジトリ自身の構成
（`Cargo.toml` の workspace members + `static/` + `xtask/` + `cli/`）を
現行のスキーマ v1 で宣言した参照マニフェストであり、以下を兼ねる:

- 2.2 節スキーマの正例
- TASK-13.1d（#131）の統合テストフィクスチャ（予定）

構成上の留意点:

- `server` は本リポジトリの現行実装では `rws-core`/`rws-app` を
  **テストコードでのみ**使用し、本番コード（`server/src/router.rs`）は
  外部依存ゼロの自前実装である（`server/Cargo.toml` の
  `[dependencies]` は空、`rws-core`/`rws-app` は `[dev-dependencies]`
  のみ）。そのため `directories.server.depends_on` は宣言していない。
  `core`/`app` への実質的な依存を束ねるのは `dist-server`
  （`role = "distribution"`）である。
- `[routing] definition_dir = "server"` は「ルートは `server/src/router.rs`
  の `Router::route(...)` 呼び出しに定義される」という規約を宣言する。
  実際の抽出処理（`rws-router-v1` 抽出器）は TASK-13.1c（#130）で実装する。

## 4. スコープの境界（TASK-13.1b/c との責務分担）

- 本書・TASK-13.1a のスコープ: スキーマの型定義（`cli/src/structure.rs`）と
  マニフェスト**内部**の宣言整合性検証（`validate()`、§2.3）。
- TASK-13.1b（#129）: `structure.toml` の TOML サブセット（§2.1）をパースし
  `StructureManifest` を構築する。未知キー・サブセット外構文はエラーと
  する。
- TASK-13.1c（#130）: `cargo metadata` との連携、実ディレクトリ・実クレート
  との突き合わせ（宣言と実体の差分検出）、`rws-router-v1` 抽出器の実装、
  JSON 出力（`xtask/src/json.rs` の手書き JSON 方式を踏襲）。
- TASK-13.1d（#131）: ルートの `structure.toml` をフィクスチャとした
  統合テスト・負例テストの整備。

## 5. スコープ外事項（本タスクでは対応しない）

- **impact/gate 用フィールド**（`breaking_risk` 等の付加）: 既存
  イシュー（#132 / #138）で追跡する。
- **影響範囲判定の AST 精密化**: REQ-13 自体が現時点でスコープ外と
  している制約（`docs/spec/04-requirements.md`）。
- **抽出器の複数対応・正規表現ベース抽出の一般化**: 必要になった時点で
  frontend-framework-spec 側の仕様検討を提案する。
- 仕様（`docs/spec/`）自体の変更は本リポジトリでは行わない
  （PoC-7 マニフェストとの差分理由は本書側に記載した）。
