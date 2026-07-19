# `fw new` 設計（TASK-13.4 相当、イシュー #350）

## 1. 目的とトレーサビリティ

- **関連 Issue**: #350「feat(cli): fw new — templates/default の決定的展開」
  （親イシュー #338「決定的スキャフォールド — fw new」の第 1 タスク）。
- **背景**: AI エージェントが `fw new` なしで毎回 boilerplate を生成すると
  プロジェクト構成がドリフトし、`fw gate` / `fw impact` / `structure.toml`
  が前提とする「全プロジェクトが同一構成」を維持できなくなる。`fw new` は
  `templates/default/` を決定的に展開することでこれを防ぐ。
- **受け入れ条件**:
  1. 同一引数での 2 回実行が同一出力（決定性）
  2. 既存ディレクトリへの上書きは fail-closed（明示フラグなしでは拒否）
  3. 終了コード契約（0/1/2）を他サブコマンド（`structure` / `gate` / `impact`）
     と統一
- **スコープ外**: テンプレートへの `structure.toml` 追加・「生成直後
  `fw gate` PASS」の e2e は兄弟イシュー #351 で扱う。本書・本実装には
  含まない。

## 2. CLI 契約

```
fw new <project-name> [--dir <parent-dir>] [--force]
```

| 要素 | 説明 |
|------|------|
| `<project-name>` | 必須の第 1 位置引数。§3 の検証規則を満たさない場合は使用法エラー（終了コード 2） |
| `--dir <parent-dir>` | 展開先の親ディレクトリ。省略時はカレントディレクトリ。ターゲットは `<parent-dir>/<project-name>` |
| `--force` | ターゲットが既存でも展開を許可する。テンプレート該当ファイルのみ上書きし、テンプレート外の既存ファイルは削除しない（`rm -rf` 相当の自動削除は行わない） |

### 終了コード規約

`main.rs` 冒頭の doc コメントが明文化する全サブコマンド共通規約に従う:

- **0**: 成功
- **1**: 検証違反・実行失敗（既存ターゲットへの `--force` なしアクセス・
  I/O エラー・テンプレート置換回数の不一致）
- **2**: 使用法エラー（引数欠落・不正なプロジェクト名・未知フラグ・
  `--dir` の値欠落）

### 成功時の標準出力

他サブコマンド同様、1 行の JSON を stdout へ出力する。すべての文字列値は
`json_out::quoted`（`escape_str` 経由）でエスケープする
（`json_out.rs` の既存契約、security.md A08 対策）。

```json
{"created":"<target-path>","files":["<rel-path-1>","<rel-path-2>", ...]}
```

`files` は展開順（[`TEMPLATE_FILES`](#3-テンプレートの取得方式コンパイル時埋め込み--ドリフト検知テスト)
の固定配列順）で並ぶ。

## 3. テンプレートの取得方式: コンパイル時埋め込み + ドリフト検知テスト

`fw` は単一実行ファイル配布（Docker 想定）が目標のため、実行時に
`templates/default/` のファイルシステム配置へ依存させず、`include_str!`
によるコンパイル時埋め込みとする。

`cli/src/new_template.rs` に静的マニフェスト `TEMPLATE_FILES: &[TemplateFile]`
を定義する:

```rust
pub(crate) struct TemplateFile {
    pub(crate) rel_path: &'static str,   // 例: "src/main.rs"（コンパイル時定数のみ）
    pub(crate) contents: &'static str,   // include_str!("../../templates/default/...")
    pub(crate) executable: bool,         // git mode 100755 のファイルのみ true
}
```

正本は従来どおり `templates/default/`。埋め込みとの乖離は
`cli/tests/new_e2e.rs::embedded_template_matches_templates_default_on_disk`
（ドリフト検知テスト）が機械的に検出する。テンプレートにファイルが
増減・変更されたら CI で必ず落ちる仕組みとし、手動同期に頼らない
（`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と同じ運用方針）。

`templates/default/` の対象 12 ファイル（git mode 込み）:

- 100644: `.github/workflows/deny.yml` / `.github/workflows/npm-asset-gate.yml`
  / `Cargo.lock` / `Cargo.toml` / `clippy.toml` / `deny.toml` / `src/main.rs`
  / `tests/negative_type_error.rs` / `tools/npm-asset-build/allowlist.toml`
- 100755: `tools/npm-asset-build/apply_exempt.py` /
  `tools/npm-asset-build/check_static_only.py` /
  `tools/npm-asset-build/install.sh`

## 4. 変数置換: 明示的 allowlist + 置換回数の fail-closed 検証

置換対象は allowlist で固定する。置換 needle は `rws-template-default`
（`Cargo.toml` の `name = "rws-template-default"`、`Cargo.lock` の同キー）
で、対象ファイルと期待出現回数は以下のとおり:

| ファイル | 期待出現回数 |
|---------|-------------|
| `Cargo.toml` | 1 |
| `Cargo.lock` | 1 |

実装は `cli/src/new.rs::replace_exact(contents, needle, replacement,
expected_count) -> Result<String, String>` とし、**出現回数が期待値と
一致しない場合はエラー（終了コード 1）**にする（fail-closed。テンプレート
改変時の黙示的な置換漏れ・過剰置換を防ぐ）。

`tests/negative_type_error.rs` 内の `rws-template-default` への doc コメント
言及（テンプレート出自の説明）は**置換しない**（allowlist 最小化の方針、
かつコメント置換は意味的に不要）。

プロジェクト名の文字集合は §5 の検証で `[a-z][a-z0-9_-]*` に制限されるため、
TOML 文字列・ロックファイルへの構文注入は構造的に不可能。

## 5. プロジェクト名の検証規則

`cli/src/new.rs::validate_project_name`。すべて満たさない場合は使用法エラー
（終了コード 2）:

- 非空・64 文字以内
- 先頭は `[a-z]`、以降は `[a-z0-9_-]` のみ（cargo package name のサブセット）

パス区切り（`/` `\`）・`..`・先頭 `-` を構造的に排除する。ターゲットパスの
組み立て・テンプレート内文字列置換の双方でパストラバーサル・構文注入が
起こり得ない文字集合に限定している（security.md A01/A03）。

## 6. 決定性の保証

- 展開は `TEMPLATE_FILES` の配列順（固定）で実行する。内容はコンパイル時
  定数であり、プロジェクト名以外の入力を混ぜない。
- タイムスタンプ・乱数・環境変数由来の値を出力ファイルへ一切書き込まない。
- パーミッション: `executable: true` のファイルへ Unix では 0o755 を明示
  設定する（`std::os::unix::fs::PermissionsExt`、`#[cfg(unix)]`）。
  **非 Unix プラットフォームではパーミッションモデルが異なるため設定を
  スキップする**（`#[cfg(not(unix))]` の no-op 実装、`cli/src/new.rs::set_permissions`）。
- 書き込み途中の失敗は該当パス付きで stderr へ報告して終了コード 1 とする
  （部分生成物は削除しない = 成功と誤認させないことのみ保証する。
  `--force` でも削除系操作は一切行わない）。

## 7. セキュリティ考慮（OWASP Top 10 観点）

- **A01/A03 パストラバーサル・インジェクション**: プロジェクト名を
  `[a-z][a-z0-9_-]*`（64 文字以内）へ厳格検証してからパス連結・置換に使う。
  `/` `\` `..` 先頭 `-` を構造的に排除。テンプレート内相対パスは
  コンパイル時定数のみで、ユーザー入力からパスを組み立てない。
- **A03 OS コマンド注入**: 外部プロセス起動なし。`std::fs` のみで完結する
  （シェル文字列連結ゼロ）。
- **A05 fail-closed**: 既存ターゲットは明示 `--force` なしで拒否する。
  置換回数不一致・書き込み失敗は黙示的成功へ倒さず終了コード 1 とする
  （`main.rs` 冒頭の「黙示的成功を返さない」契約を継承）。`--force` でも
  削除系操作は行わない（ユーザーデータの巻き込み削除防止）。
- **A08 ソフトウェア・データ整合性**: テンプレートはコンパイル時埋め込みで
  ネットワーク・実行時ファイル配置に非依存。正本 `templates/default/` との
  整合はドリフト検知テストが CI で強制する。生成物の `deny.yml` /
  `npm-asset-gate.yml` / `install.sh`（`--ignore-scripts` 既定、REQ-12）は
  バイト単位で正本と同一。
- **A09 ログ・エラー出力**: エラーメッセージは対象パスと是正コマンドのみ。
  内部状態・環境変数を転記しない。
- **秘密情報**: テンプレート・テストフィクスチャにクレデンシャルなし
  （既存テンプレートを無改変で展開）。stdout JSON は `json_out::quoted` で
  必ずエスケープする（文字列手組み禁止の既存契約に従う）。
- **REQ-1/REQ-2/REQ-3**: HTML 生成なし（既定エスケープ非関与）・
  `forbid(unsafe_code)` 維持（`PermissionsExt` は safe API）・依存追加ゼロ
  （`cli` は外部依存ゼロを維持）。

## 8. テスト（`cli/tests/new_e2e.rs`）

実バイナリ（`CARGO_BIN_EXE_fw`）を起動する e2e テスト:

1. **決定性**: 同一引数で 2 つの親ディレクトリへ実行し、再帰走査した
   相対パス集合・各ファイルのバイト列・Unix パーミッション（実行ビット）が
   完全一致することを確認する。
2. **fail-closed**: 1 回目成功 → 同一ターゲットへの 2 回目は失敗（既存内容
   が不変）→ `--force` 付きで成功、の順に確認する。
3. **終了コード契約**: 引数なし・不正名・未知フラグは 2、成功は 0。
4. **置換検証**: 生成後の `Cargo.toml` / `Cargo.lock` に
   `name = "<project-name>"` があり `rws-template-default` が残らないこと。
   置換対象外ファイルはテンプレートとバイト一致すること。
5. **ドリフト検知**: `templates/default/` を再帰走査し、埋め込み
   マニフェストと相対パス集合・内容バイト列（`Cargo.toml`/`Cargo.lock` を
   除く）・実行ビットが 1:1 対応することを確認する。

## 9. 非目標（Non-goals）

- 複数テンプレート選択（`fw new --template embed` 等）は本イシューの範囲外。
- 非 Unix でのパーミッション再現（ACL 相当の代替設定等）は行わない。
- 生成直後の `fw gate` PASS 保証・`structure.toml` の同梱は #351 で扱う。
