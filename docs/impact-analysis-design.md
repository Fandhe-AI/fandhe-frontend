# 変更影響範囲解析（`fw impact`）設計（TASK-13.2a）

> **本書のステータスと前提**: 本書は TASK-13.2（親イシュー #132）の 5
> 分割サブタスク（TASK-13.2a 設計・#133（本書） / TASK-13.2b 依存グラフ構築・
> #134 / TASK-13.2c コマンド実装・#135 / TASK-13.2d 出力フォーマット・#136 /
> TASK-13.2e テスト整備・#137）の先頭であり、本サブタスクの成果物は
> **設計文書（本書）+ 型定義・判定ロジックのスケルトン**（`cli/src/impact.rs`）
> です。ソース走査・CLI 接続・JSON 出力・統合テストは後続 #134〜#137 の
> 領分であり、本書 §8 でスコープ外として明示します。
>
> 本タスクは自動運転モードで実装されています。判断が必要な境界ケースは
> すべて**安全側（fail-closed・過検知容認・多重定義は人間承認強制）**に
> 倒して確定し、判断根拠を本書に明記しました。人間レビュー（PR）で
> 判断ポイントを確認し、緩和が必要な場合は後続 PR で個別に対応してください。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-13（AI 自己保守・改修のためのフック・ゲート機構、Must）。
  `fw structure`（TASK-13.1、#127 実装済み）・`fw gate`（TASK-13.3、#138）に
  続く第 3 のサブコマンド `fw impact` の設計を確定する。
- **親タスク**: TASK-13.2（#132、成果物 `cli/src/impact.rs`。前提タスク
  TASK-13.1 は完了済み、`docs/spec/05-tasks.md` 348 行目以降）。
- **サブタスク分割**:

| サブタスク | Issue | 内容 | 本書との関係 |
|-----------|-------|------|-------------|
| TASK-13.2a | #133（本書） | アルゴリズム設計・型定義・判定ロジック | 本書 + `cli/src/impact.rs` の型定義・純粋関数が単一の情報源 |
| TASK-13.2b | #134 | 依存グラフ構築・定義元特定・使用箇所走査の実装 | 本書 §3.1/§3.3/§6 が走査アルゴリズム・パストラバーサル対策を規定 |
| TASK-13.2c | #135 | `fw impact <symbol>` の CLI 接続 | 本書 §3.5 が CLI 仕様・終了コード規約を規定 |
| TASK-13.2d | #136 | JSON 出力フォーマット実装 | 本書 §3.5 の JSON スキーマを規定 |
| TASK-13.2e | #137 | 判定境界の統合テスト整備 | 本書 §3.4 の判定境界一覧をテスト観点として使用 |

- **土台**: PoC-7 の `docs/spec/03-poc/ai-self-maintenance/tools/poc7_tool.py`
  `cmd_impact`（定義元特定 → 使用箇所走査 → 影響ルート突き合わせ →
  `breaking_risk` / `requires_human_approval` 判定）。`docs/spec/05-tasks.md`
  TASK-13.2 の指示に従い、ファイル単位の粗粒度ヒューリスティックの限界を
  維持し、AST 解析ベースの精密化は将来スコープとして本書 §7 に明記する。
- **先行パターンの踏襲**: TASK-13.1a（#128、`docs/structure-manifest.md`）は
  「設計文書 + 型定義スケルトン（`cli/src/structure.rs`）」を成果物とした。
  本タスクも同型で進める。

## 2. 現状（再利用可能な既存資産）

`cli/`（`rws-cli`、bin 名 `fw`）は外部依存ゼロ（`cli/src/toml.rs` /
`cli/src/json.rs` の手書きパーサを保有）。TASK-13.2b〜e が再利用すべき
既存資産:

- `cli/src/metadata.rs`: `cargo metadata` 連携（`WorkspaceMetadata` /
  `MemberPackage::normal_workspace_deps`）。#134 の依存グラフ構築が
  `normal_workspace_deps` から逆依存閉包を構築する土台になる。
- `cli/src/routes.rs`: ルート抽出（`extract_routes`）+ パストラバーサル対策
  （`resolve_within_root` / `scan_root` / `list_rs_files`、シンボリック
  リンクを辿らない設計）。#134 のソース走査はこれらを再利用し、
  独自のファイル列挙ロジックを重複実装しない。
- `cli/src/component_boundary.rs`: `^pub (fn|struct|enum|const) <name>` の
  トップレベル公開シンボル走査。定義元特定（本タスクの走査対象と同じ
  正規表現相当のパターン）に流用できる。
- `cli/src/json_out.rs`: JSON 出力レンダリング（`escape_str` による
  JSON インジェクション対策）。#136 が同じ方針で `impact` 用の出力を書く。
- `cli/src/main.rs`: サブコマンドディスパッチ（`structure` / `gate`、
  終了コード規約: 正常 0 / 検証違反 1 / 使用法エラー 2、
  `parse_project_arg` の共有）。#135 がここに `impact` を追加する。

## 3. 設計内容（本タスクで確定する事項）

### 3.1 アルゴリズム（PoC-7 準拠 + 製品ワークスペースへの適応）

1. **定義元特定**: 全ワークスペースメンバーの `src/**/*.rs` を走査し、
   行頭 `pub (fn|struct|enum|const) <symbol>` を検出する
   （`component_boundary.rs` と同粒度の文字列走査。正規表現クレートは
   使わない）。
2. **使用箇所列挙**: 定義ファイル以外の全 `.rs` から、識別子境界
   （前後が `[A-Za-z0-9_]` でない）を手書き判定して `<symbol>` の
   出現行を列挙し `affected_files`（file + 行番号一覧）/ `affected_crates`
   を構築する。
3. **影響ルート突き合わせ**: `routes::extract_routes` の結果と
   `affected_files` をファイル単位で突き合わせ、`affected_routes` を
   列挙する（過検知側に倒す。PoC-7 の既知の限界をそのまま維持する）。
4. **判定**: `breaking_risk = high`（影響クレート 3 件以上、または
   クライアント境界クレートへの波及）/ `medium`（1 件以上）/
   `low`（0 件）。`requires_human_approval = breaking_risk ∈
   {high, medium} または affected_routes 非空 または ambiguous`。

### 3.2 製品適応で確定した設計判断（すべて安全側 = fail-closed / 過検知側）

| 判断事項 | PoC-7 | 製品仕様（本書で確定） | 根拠 |
|---------|-------|----------------------|------|
| クライアント境界クレート | `rws-wasm-client` 単独 | `rws-wasm-client` / `rws-wasm-full` / `rws-wasm-thin` の 3 クレート（[`impact::CLIENT_BOUNDARY_CRATES`]）。いずれかへの波及で `high` | 製品ワークスペースは WASM 配布クレートが 3 つに分割されている（`docs/spec/06-roadmap.md` MS-3〜MS-4）。1 つでも見逃すと過小評価になるため全て対象にする |
| 定義元が見つからない場合 | `defined_in_crate: None` で成功扱い | エラー終了（終了コード 1）。黙って `defined_in: null` で成功させない | `fw structure` の「黙示的成功を返さない」方針（security.md A05）と統一する |
| 定義元が複数の場合 | 未考慮（`break` で最初の 1 件のみ採用） | 全候補を列挙し `ambiguous: true`、`requires_human_approval` を強制 `true` | 「シンボル 1 つに定義は 1 つ」という走査の前提が崩れている状態であり、他の判定材料の信頼性も下がるため人間承認へ倒す |
| シンボル入力の検証 | 未検証（任意文字列を正規表現に埋め込む） | Rust 識別子（`^[A-Za-z_][A-Za-z0-9_]*$`）以外は使用法エラー（終了コード 2） | 任意文字列を走査パターンに使わない（本書 §6 A03 対策）。正規表現クレート自体を使わないため ReDoS 面はないが、識別子境界判定の入力として不正な文字種を拒否する |
| コメント・文字列内のヒット | 除外しない | 除外しない（据え置き） | PoC-7 の過検知容認方針をそのまま踏襲。AST 解析なしに正確な除外は不可能であり、除外を試みてかえって見逃す（過小評価）方が危険 |
| AST 精密化・行単位のルート特定 | スコープ外 | 将来スコープとして本書 §7 に明記 | `docs/spec/05-tasks.md` が明示的に将来スコープ指定済み。新規 Issue 起票はユーザー承認が必要な行為であり自動運転では行わず、設計文書と PR 本文への記載に留める（`out-of-scope-tracking.md`） |

### 3.3 定義元特定・使用箇所走査のパストラバーサル対策（#134 実装時の必須事項）

- ソース走査は `routes::resolve_within_root` / `scan_root` /
  `list_rs_files` を再利用し、ワークスペースルート外への脱出を拒否する
  （シンボリックリンクは辿らず一律スキップ、`routes.rs` の既存設計）。
- 独自のファイル列挙・パス解決ロジックを新設しない（二重管理・
  防御漏れの回避）。

### 3.4 判定境界（#137 のテスト観点として使用）

`cli/src/impact.rs` の `judge_breaking_risk` / `requires_human_approval`
（純粋関数、副作用なし）が確定した判定境界:

| affected_crates 件数 | クライアント境界クレートを含むか | breaking_risk |
|---------------------|--------------------------------|---------------|
| 0 | いいえ | low |
| 1〜2 | いいえ | medium |
| 3 以上 | 問わない | high |
| 1 以上（3 未満含む） | はい | high |

`requires_human_approval` は次のいずれかで `true`:

- `breaking_risk` が `high` または `medium`
- `affected_routes` が非空
- `ambiguous`（定義元が複数）

テスト観点一覧（#137 が統合テストとして実装する想定）:

1. クレート数 0 / 1 / 2 / 3 の境界値
2. `rws-wasm-client` / `rws-wasm-full` / `rws-wasm-thin` 各々を単独で含む場合の `high` 判定
3. ルート有無（低リスクでもルート影響ありなら要承認）
4. 多重定義（`ambiguous`）時の承認強制
5. 識別子検証の正常系・異常系（空文字・数字始まり・記号混入・パス区切り混入）

### 3.5 CLI 仕様・JSON スキーマ（#135 / #136 の実装契約）

- **CLI**: `fw impact <symbol> [--project <dir>]`。`<symbol>` は
  `impact::validate_symbol` を経由し、不正な場合は使用法エラー
  （終了コード 2）。定義元が見つからない場合は検証違反（終了コード 1）。
  `--project` の解決は `main.rs` の既存 `parse_project_arg` を再利用する
  （`structure` / `gate` と実装を共有し重複実装しない）。
- **JSON スキーマ**（PoC-7 互換フィールド + 製品追加分）:

```json
{
  "symbol": "string",
  "defined_in_crate": "string | null（複数なら先頭要素、ambiguous 参照）",
  "defined_in_file": "string | null",
  "ambiguous": "boolean（製品追加。定義元が複数の場合 true）",
  "affected_files": [{ "file": "string", "lines": [1, 2] }],
  "affected_crates": ["string", "..."],
  "affected_routes": ["string", "..."],
  "breaking_risk": "high | medium | low",
  "requires_human_approval": "boolean",
  "verdict": "string（人間可読な要約、PoC-7 互換）"
}
```

  `cli/src/impact.rs` の `ImpactReport` 型がこのスキーマに対応する
  （`defined_in_crates: Vec<String>` / `defined_in_files: Vec<String>` を
  複数形で保持し、`ambiguous` が `false` の場合に限り JSON では先頭要素を
  単数のスカラーとして出力する変換を #136 が実装する契約）。

## 4. 対象ファイル・変更箇所（本サブタスク #133 分）

| パス | 変更 | 内容 |
|------|------|------|
| `docs/impact-analysis-design.md` | 新規（本書） | 設計文書一式 |
| `cli/src/impact.rs` | 新規 | 型定義（`BreakingRisk` / `AffectedFile` / `ImpactReport` / `ImpactError`）+ 判定純粋関数（`judge_breaking_risk` / `requires_human_approval` / `validate_symbol` / `contains_symbol_at_boundary`）+ 単体テスト。走査・CLI 接続は実装しない |
| `cli/src/main.rs` | 編集 | `mod impact;` の追加のみ |

- `docs/spec/` は編集禁止（サブモジュール）。
- 新規依存クレートは追加していない（`rws-cli` 外部依存ゼロ方針・REQ-3）。

## 5. インターフェース境界（後続サブタスクとの契約）

- **#134（依存グラフ構築）**: `metadata::WorkspaceMetadata` の
  `normal_workspace_deps` から逆依存閉包を構築し、`affected_crates` を
  保守的に拡張する（A に依存する B が影響を受けるなら、B にさらに依存する
  C も波及候補として保守的に含める）モジュールを実装する。走査結果は
  本書 §3.5 の型（`AffectedFile` / `Vec<String>`）で `impact.rs` の関数へ渡す。
- **#135（コマンド実装）**: `fw impact <symbol> [--project <dir>]` の
  CLI 仕様（本書 §3.5）を実装し、`main.rs` に接続する。
- **#136（出力フォーマット）**: 本書 §3.5 の JSON スキーマを
  `json_out.rs` の `escape_str` を用いて実装する。
- **#137（テスト整備）**: 本書 §3.4 の判定境界テスト観点を統合テスト
  （`cli/tests/`）として実装する。

## 6. セキュリティ考慮（OWASP Top 10 観点）

- **A03 インジェクション**: シンボル入力を Rust 識別子に制限して検証
  （`validate_symbol`）。正規表現クレート・シェル文字列組み立てを使わず、
  手書きの識別子境界判定（`contains_symbol_at_boundary`）のみを使う。
  HTML 出力は扱わないため既定エスケープ（REQ-1）には影響しない。
- **A01 破損アクセス制御 / パストラバーサル**: ソース走査は既存の
  `routes::resolve_within_root` / `scan_root` を再利用し、ワークスペース
  ルート外への脱出を拒否する契約を #134 実装時の必須事項として本書 §3.3
  に明記する。
- **A05 セキュリティ設定ミス（fail-closed）**: 定義元未発見はエラー終了
  （黙示的成功なし）、多重定義は `requires_human_approval` 強制、判定は
  過検知側に統一する。`fw gate` / `fw structure` と同じ終了コード規約で
  CI・エージェントの誤認を防ぐ。
- **A06 脆弱な依存 / サプライチェーン**: 新規依存ゼロを維持
  （REQ-3: 60 件 / 深さ 6 の上限に影響なし）。`build.rs` の追加もなし。
- **A09 ログ・情報露出**: 出力パスはワークスペース相対に限定し、
  絶対パス・環境情報を JSON 出力へ含めない方針を `AffectedFile::file` の
  rustdoc に明記した。
- **秘密情報**: 新規ファイルはドキュメントと純粋ロジックのみ。
  コミット前に staged 差分でシークレット混入がないことを確認済み。
- **`#![forbid(unsafe_code)]`**: `cli` は `main.rs` で宣言済み。
  `impact.rs` でも `unsafe` を使用していない。

## 7. 既知の限界と将来スコープ

- **ファイル単位の粗粒度ヒューリスティック**: コメント・文字列リテラル内の
  シンボル出現も「使用箇所」として数える（過検知）。行単位でのコメント/
  文字列除外は AST 解析なしには正確に行えないため、現時点では対応しない。
- **AST 解析ベースの精密化**: `docs/spec/05-tasks.md` TASK-13.2 が
  明示的に将来スコープと指定している。本タスクでは新規 Issue を起票せず
  （起票はユーザー承認が必要な行為、`out-of-scope-tracking.md`）、本書と
  PR 本文への記載に留める。
- **行単位のルート特定**: 現行設計はファイル単位でルート影響を突き合わせる
  （`affected_files` のファイルパスと `routes::extract_routes` の
  `defined_in` の一致のみ）。同一ファイル内の他ルートも影響ありと
  誤判定しうるが、過検知側であり安全側の設計として許容する。

## 8. スコープ外（後続サブタスクの領分・混入させない）

- ソース走査・逆依存グラフ構築の実装 → #134
- `fw impact` サブコマンドの CLI 接続 → #135
- JSON 出力レンダリングの実装 → #136
- 統合テスト → #137
- AST 解析ベースの精密化・行単位のルート特定 → 将来スコープ（本書 §7
  に明記。仕様 `docs/spec/05-tasks.md` が明示済みのため新規 Issue 起票は
  行わず、必要ならレビュー時にユーザーへ起票を提案する）
