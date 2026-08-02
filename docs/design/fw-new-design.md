# `fw new` 設計（TASK-13.4 相当、イシュー #350／複数テンプレート選択、
イシュー #378）

## 1. 目的とトレーサビリティ

- **関連 Issue**: #350「feat(cli): fw new — templates/default の決定的展開」
  （親イシュー #338「決定的スキャフォールド — fw new」の第 1 タスク）。
- **背景**: AI エージェントが `fw new` なしで毎回 boilerplate を生成すると
  プロジェクト構成がドリフトし、`fw gate` / `fw impact` / `structure.toml`
  が前提とする「全プロジェクトが同一構成」を維持できなくなる。`fw new` は
  `templates/<name>/` を決定的に展開することでこれを防ぐ。
- **受け入れ条件**:
  1. 同一引数での 2 回実行が同一出力（決定性）
  2. 既存ディレクトリへの上書きは fail-closed（明示フラグなしでは拒否）
  3. 終了コード契約（0/1/2）を他サブコマンド（`structure` / `gate` / `impact`）
     と統一
- **関連 Issue（追補）**: #351「test(cli): fw new 生成直後に fw gate が
  PASS する構成保証」（兄弟イシュー）でテンプレートへの `structure.toml`
  追加・「生成直後 `fw gate` PASS」の e2e（`crates/cli/tests/new_gate_e2e.rs`）が
  実装済み。§3・§4・§8 参照。
- **関連 Issue（追補 2）**: #378「feat(cli): fw new の複数テンプレート選択と
  テンプレート骨格の拡充」で `--template` 選択 UI と、fandhe-frontend-core / fandhe-frontend-app
  依存の拡充テンプレート `app`（Loader・束縛点 API・`fandhe_frontend_core::render` の
  実体サンプル）を追加。§2・§3・§3a・§9 参照。

## 2. CLI 契約

```
fw new <project-name> [--template <template>] [--dir <parent-dir>] [--force]
```

| 要素 | 説明 |
|------|------|
| `<project-name>` | 必須の第 1 位置引数。§5 の検証規則を満たさない場合は使用法エラー（終了コード 2） |
| `--template <template>` | 使用するテンプレート名（イシュー #378）。省略時は `default`。未知の名前は使用法エラー（終了コード 2）で、stderr に利用可能テンプレート一覧を出す。allowlist は `crates/cli/src/new_template.rs::TEMPLATES` |
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
{"created":"<target-path>","template":"<template-name>","files":["<rel-path-1>","<rel-path-2>", ...]}
```

`template` フィールド（イシュー #378 追加、既存フィールドへの追加的変更）は
使用したテンプレート名（`default`/`app`）を表す。`files` は展開順
（テンプレートの `files` 固定配列順、§3）で並ぶ。

## 3. テンプレートの取得方式: コンパイル時埋め込み + レジストリ化 + ドリフト検知テスト

`fw` は単一実行ファイル配布（Docker 想定）が目標のため、実行時に
`templates/<name>/` のファイルシステム配置へ依存させず、`include_str!`
によるコンパイル時埋め込みとする。

`crates/cli/src/new_template.rs` に静的マニフェスト `Template`/`TemplateFile` を
定義する（イシュー #378 で単一 `TEMPLATE_FILES` 配列から一般化）:

```rust
pub(crate) struct TemplateFile {
    pub(crate) rel_path: &'static str,   // 例: "src/main.rs"（コンパイル時定数のみ）
    pub(crate) contents: &'static str,   // include_str!("../../templates/<name>/...")
    pub(crate) executable: bool,         // git mode 100755 のファイルのみ true
}

pub(crate) struct Template {
    pub(crate) name: &'static str,                    // "--template <name>" の照合値
    pub(crate) files: &'static [TemplateFile],
    pub(crate) needle: &'static str,                   // パッケージ名置換対象
    pub(crate) substituted_files: &'static [&'static str],
}

pub(crate) const TEMPLATES: &[Template] = &[/* "default", "app", "embed" */];
```

正本は従来どおり `templates/<name>/`。埋め込みとの乖離は
`crates/cli/tests/new_e2e.rs::embedded_template_matches_templates_on_disk`
（全テンプレートをパラメタ化したドリフト検知テスト）が機械的に検出する。
テンプレートにファイルが増減・変更されたら CI で必ず落ちる仕組みとし、
手動同期に頼らない（`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と
同じ運用方針）。

### 3.1 `default`（TASK-4.4 負例検出テスト土台、変更なし）

`templates/default/` の対象 13 ファイル（git mode 込み）:

- 100644: `.github/workflows/deny.yml` / `.github/workflows/npm-asset-gate.yml`
  / `Cargo.lock` / `Cargo.toml` / `clippy.toml` / `deny.toml` / `src/main.rs`
  / `structure.toml` / `tests/negative_type_error.rs` /
  `tools/npm-asset-build/allowlist.toml`
- 100755: `tools/npm-asset-build/apply_exempt.py` /
  `tools/npm-asset-build/check_static_only.py` /
  `tools/npm-asset-build/install.sh`

イシュー #378 以前と完全後方互換（`--template` 省略時は `default`、同一
バイト出力）。

### 3.2 `app`（イシュー #378 新設。fandhe-frontend-core / fandhe-frontend-app 依存の拡充テンプレート）

`templates/default/`（fandhe-frontend-core 非依存の最小骨格）に対し、フレームワークの
実 API（`Loader` trait 実装・束縛点 API・`fandhe_frontend_core::render`）を使う出発点を
提供する。`templates/app/` の対象 35 ファイル（イシュー #378 で 22 ファイル、
イシュー #411 で CSR wasm ビルド込み完全実体を追加し 35 ファイルへ拡張）:

- プロジェクト骨格: `Cargo.toml`（`fandhe-frontend-core`/`fandhe-frontend-app` へ vendor path 依存）・
  `Cargo.lock`・`structure.toml`（`crate = "fandhe-frontend-template-app"`）・
  `src/main.rs`（`DemoItemsLoader`/`DemoItemDetailLoader` → `list_page`/
  `detail_page` → `render` で `dist/` へ書き出す SSG 的最小 SSR。
  `bind_text`/`keyed_list` の束縛点 API 使用サンプルも含む）・
  `tests/escape_regression.rs`（XSS 回帰テスト）
- 共有ファイル（`templates/default/` とバイト単位で同一。
  `crates/cli/tests/template_vendor_drift.rs` が検証）: `.github/workflows/deny.yml`
  / `.github/workflows/npm-asset-gate.yml` / `clippy.toml` / `deny.toml` /
  `tools/npm-asset-build/*`（4 ファイル）
- `static/embed.html`: `templates/embed/embed.html`（既存の CSR マウント
  骨格）を同梱したもの。`tools/wasm/build.sh` 実行後は
  `static/wasm/fandhe_frontend_wasm_client.js`/`fandhe_frontend_wasm_client_bg.wasm` を参照して
  実際に動作する（§3b）
- `vendor/fandhe-frontend-core/`・`vendor/fandhe-frontend-app/`・`vendor/fandhe-frontend-interactive/`・
  `vendor/fandhe-frontend-wasm-client/`: fandhe-frontend-core / fandhe-frontend-app / fandhe-frontend-interactive /
  fandhe-frontend-wasm-client のソース vendor 同梱（§3a・§3b）
- `wasm/`・`tools/wasm/build.sh`: CSR wasm ビルド用の独立ワークスペース
  （glue クレート `app-csr-wasm`）とビルド手順（§3b、イシュー #411）

#### 3a. vendor 同梱の選定根拠

fandhe-frontend-core / fandhe-frontend-app は `publish = false`（crates.io 未公開）のため、生成
プロジェクトが依存する方式には次の選択肢があった:

| 方式 | 判定 |
|------|------|
| git 依存 | 却下: ビルド時ネットワーク依存（`security.md` サプライチェーン対策・オフライン決定性と矛盾） |
| フレームワークリポへの path 依存 | 却下: 生成プロジェクトが配布先で独立して成立しなくなる |
| **vendor 同梱（採用）** | fandhe-frontend-core / fandhe-frontend-app とも外部依存ゼロのため自己完結・オフライン・決定的。既存の「正本 + ドリフト検知テスト」運用にそのまま乗る |

vendored `Cargo.toml`（`templates/app/vendor/{fandhe-frontend-core,fandhe-frontend-app}/Cargo.toml`）
は正本から 1 点のみ変換する: `fandhe-frontend-app` の `fandhe-frontend-core` path 依存の参照先を
`../core` → `../fandhe-frontend-core`（vendor 配下の実ディレクトリ名に合わせる）。

**重要**: vendor 側 `Cargo.toml` に `[workspace]` を追加してはならない。
生成プロジェクトの `Cargo.toml`（`fandhe-frontend-template-app`）は `[workspace]
members = ["."]` を明示することで、path 依存先（`vendor/fandhe-frontend-core`/
`vendor/fandhe-frontend-app`）が workspace member として自動編入されるのを防いでいる。
この状態で vendor 側にも独立した `[workspace]` を持たせると、cargo が
"multiple workspace roots found in the same workspace" で拒否する
（実装時に実測で確認済み。`templates/app/vendor/*/Cargo.toml` のコメント
参照）。

生成プロジェクトの依存グラフは vendored 2 crate のみ・外部クレートゼロ
（REQ-3 の 60 件 / 深さ 6 に対し余裕。依存クレートの新規追加なし）。

vendor 同梱は「`publish = false`（crates.io 未公開）である間」の暫定措置
であり、公開後はバージョン依存へ切り替えるべきものである。切替の
トリガー条件と切替手順は `docs/design/template-vendor-to-version-switch.md`
（イシュー #412）に定めた。イシュー #493 において全 9 クレートが
v0.1.0 で crates.io へ公開されたことを受け、本手順に従い切替を実施した
（`templates/app/vendor/` 削除、`templates/app/Cargo.toml` の crates.io
バージョン依存化、`crates/cli/tests/template_vendor_drift.rs` の テスト
更新）。`templates/app/Cargo.toml` は現在 `fandhe-frontend-core = "0.1.0"`・
`fandhe-frontend-app = "0.1.0"` でバージョン依存を宣言し、生成プロジェクト
のビルド時に crates.io から依存を取得する。

`structure.toml` は `vendor/fandhe-frontend-core`/`vendor/fandhe-frontend-app` を宣言しない
（`[directories.*]` 宣言外）。`fw gate` の `default_escape_check`・
`fw structure`・`fw impact` はいずれも宣言済みディレクトリのみを走査・
解決対象とするため、vendor 配下（正本の写しであり生成プロジェクトの
記述対象ではない）は意図的に走査対象から除外される。`lint` チェック
（`cargo clippy --all-targets -p fandhe-frontend-template-app`）はクレート境界で
検査するため、vendored crate 内部の `raw_html` 定義自体は違反にならない
（既存 gate 仕様どおり）。

`structure.toml`（イシュー #351 で追加）は `fw gate`（`crates/cli/src/gate.rs`）が
検証対象クレートを決定する唯一の情報源であり、これを同梱しない限り生成
直後のプロジェクトは `fw gate` が即 BLOCKED になる（宣言クレート不在の
fail-closed）。クレートはプロジェクトルート直下（`src/`）に置かれるため、
`[directories.root]` という慣習名で宣言する。`root` はスキーマ v1 の正式な
予約名（`crates/cli/src/structure.rs::ROOT_DIR_KEY`、イシュー #353 で正式化）で
あり、ディレクトリ名 → 実ファイルシステムパスの解決は
`structure::dir_fs_path` を単一の情報源とする（`root` は `<project>` 自身へ
写像する）。`fw gate` の `default_escape_check`（保険層）・`fw structure` の
ディレクトリ実在確認・`fw impact` の member 解決はいずれもこの単一情報源
（または `routes::resolve_within_root`/`scan_root` に一般化した同じ写像）を
経由するため、`<project>/root/...`（実在しないパス）を参照してのスキップ・
解析不能は発生しない。主防御である `lint` チェック（`cargo clippy
--all-targets -p <crate>` 経由の `disallowed-methods`）もクレートの配置
ディレクトリに依存せず全ソースを検査するため、REQ-1 の検出保証は二重に
保たれる（詳細は `docs/design/structure-manifest.md` §2.2 を参照）。

#### 3b. CSR wasm ビルド込み完全実体（イシュー #411）

`fandhe-frontend-wasm-client`（正本 `crates/wasm-client/`）は `wasm-bindgen`/`web-sys` という
外部依存を持つため、§3a のソース vendor 方式（外部依存ゼロが前提）を
そのまま適用できない。当初はハイブリッド方式（`fandhe-frontend-interactive`/`fandhe-frontend-wasm-client`
をソース vendor、`wasm-bindgen`/`web-sys` のみ `wasm/Cargo.lock` でバージョン依存）を採択
していたが、イシュー #493（§3a 参照）の crates.io バージョン依存への切替に伴い、
以下の現在の構成へ移行した:

**構成（§3a 切替後）**:
- `wasm/Cargo.toml`（glue クレート `app-csr-wasm`、cdylib）: `fandhe-frontend-wasm-client`
  の `hydrate`/`mount_csr`（`#[wasm_bindgen]` エクスポート）を crates.io バージョン
  依存（`fandhe-frontend-wasm-client = "0.1.0"`）で再エクスポートするのみ。
  HTML 組み立て・DOM 直接操作・`raw_html()` を持たない。
- `wasm/Cargo.lock`: wasm-bindgen / web-sys をリポジトリ本体 `Cargo.lock` と同一
  バージョンへピン。バージョン一致は `crates/cli/tests/template_vendor_drift.rs`
  が機械的に検証する（手動同期に頼らない）。
- `tools/wasm/build.sh`（実行ビット 100755）: (a) rustup target・wasm-bindgen-cli
  の存在チェック、(b) `wasm/Cargo.lock` から読んだ wasm-bindgen バージョンと
  `wasm-bindgen --version` の完全一致検証（`crates/dist-server/build.rs::expected_wasm_bindgen_version`
  と同一の fail-closed 契約）、(c) `cargo build --manifest-path wasm/Cargo.toml
  --target wasm32-unknown-unknown --release`、(d) `wasm-bindgen --target web
  --out-dir static/wasm --out-name fandhe_frontend_wasm_client` を実行する固定コマンド列。

**独立ワークスペースへの隔離**: `wasm/` は
`templates/app`（root、`fandhe-frontend-template-app`）の `[workspace] members = ["."]`
に含まれない別の `[workspace]`（`templates/app/wasm/Cargo.toml`）として
切り離す。wasm-bindgen の取得にはビルド時ネットワークが必要である。
`structure.toml` は `wasm/` を宣言しない（§3a の除外と同一根拠）ため `fw gate` の
検証対象クレート決定にも影響しない。

**REQ-3（60 件/深さ 6）への影響**: root（`fandhe-frontend-template-app`）の依存グラフ・
`xtask check-deps` の計測対象は変わらない（§3a のバージョン依存化後）。`wasm/` は
「標準サーバー構成」の外にあるオプトイン CSR 成果物であり REQ-3 の計測
基準へ影響しない。新規外部クレートの追加はゼロ。

**CI 回帰検証**: `.github/workflows/ci.yml` の `template-app-wasm-smoke`
ジョブが `fw new --template app` → `tools/wasm/build.sh` →
`static/wasm/fandhe_frontend_wasm_client.js`/`fandhe_frontend_wasm_client_bg.wasm` の生成と
`mount_csr`/`hydrate` エクスポートの存在を e2e 検証する。

**スコープ外**（`.claude/rules/out-of-scope-tracking.md`）: crates.io 公開後の
vendor → バージョン依存への切替はイシュー #412 で追跡する（本方式はその
移行を阻害しない）。

### 3.3 `embed`（イシュー #410 新設。静的単一ファイルの部分埋め込み構成）

`app` §9（旧非目標）が申し送っていた「静的単一ファイルの `embed` テンプ
レート」を製品化する。`templates/embed/` の対象 2 ファイル:

- `embed.html`: `templates/embed/embed.html`（TASK-7.1a・#52 の正本）を
  バイト無変更で流用（`crates/xtask/tests/template_embed_html.rs`・
  `crates/cli/tests/template_vendor_drift.rs` が参照する正本と同一であることが
  前提のため、本テンプレート追加時も一切変更しない）
- `structure.toml`: `fw gate` が唯一の情報源として読む静的専用
  （asset-only）マニフェスト。`[directories.root]` は `role = "asset"` の
  みを宣言し `crate` キーを持たない（cargo パッケージが存在しないため）

`default`/`app` と異なり **cargo パッケージを持たない**ため、
`Template::substituted_files` は空配列（`&[]`）とする。`needle`
（`fandhe-frontend-template-embed`）はどのファイルにも出現しないダミー文字列であり、
置換ループは素通りする。生成物はテンプレート正本と全ファイルバイト一致
になり、`crates/cli/tests/new_e2e.rs::embed_template_output_is_byte_identical_to_template_and_contains_no_needle`
がこれを固定する。

cargo パッケージを持たない構成のまま `fw gate` PASS を保証するには、
`fw gate`（`crates/cli/src/gate.rs::is_asset_only_project`）側に静的専用プロジェ
クトの明示的オプトインモードが必要だった（`docs/design/gate-design.md`
§2.5 参照）。判定条件は「宣言クレートが 0 件、かつ宣言ディレクトリ全件が
`role = "asset"`」で、満たす場合のみ cargo 系 4 チェック
（`type_check`/`lint`/`test`/`policy`）を not-applicable PASS 化する。
`default_escape_check`・`url_validation_check`（テキスト走査ベースの保険層）
は cargo パッケージの有無に依存しないため、静的専用モードでも通常どおり
実行され、`root` 慣習ディレクトリ配下（プロジェクトルート直下 `src/`）へ
Rust コードが混入した場合の回帰を検出する。

## 4. 変数置換: 明示的 allowlist + 置換回数の fail-closed 検証

置換対象は allowlist で固定する。置換 needle はテンプレートごとに異なる
仮パッケージ名（`Template::needle`、イシュー #378 でテンプレートごとに
一般化）: `default` は `fandhe-frontend-template-default`、`app` は `fandhe-frontend-template-app`
（`Cargo.toml` の `name = "..."`、`Cargo.lock` の同キー）。対象ファイルと
期待出現回数は両テンプレート共通で以下のとおり:

| ファイル | 期待出現回数 |
|---------|-------------|
| `Cargo.toml` | 1 |
| `Cargo.lock` | 1 |
| `structure.toml` | 1（`[directories.root]` の `crate` 値。イシュー #351） |

`app` テンプレートの `structure.toml` はコメント中に置換 needle と同じ
部分文字列を含めないよう配慮する必要がある（`replace_exact` の
出現回数 fail-closed 検証により、意図しない 2 箇所目のマッチはエラーに
なる。実装時に実際に検出・修正した経緯があり、テンプレート改稿時の注意点
として明記する）。

実装は `crates/cli/src/new.rs::replace_exact(contents, needle, replacement,
expected_count) -> Result<String, String>` とし、**出現回数が期待値と
一致しない場合はエラー（終了コード 1）**にする（fail-closed。テンプレート
改変時の黙示的な置換漏れ・過剰置換を防ぐ）。

`tests/negative_type_error.rs` 内の `fandhe-frontend-template-default` への doc コメント
言及（テンプレート出自の説明）は**置換しない**（allowlist 最小化の方針、
かつコメント置換は意味的に不要）。

プロジェクト名の文字集合は §5 の検証で `[a-z][a-z0-9_-]*` に制限されるため、
TOML 文字列・ロックファイルへの構文注入は構造的に不可能。

## 5. プロジェクト名の検証規則

`crates/cli/src/new.rs::validate_project_name`。すべて満たさない場合は使用法エラー
（終了コード 2）:

- 非空・64 文字以内
- 先頭は `[a-z]`、以降は `[a-z0-9_-]` のみ（cargo package name のサブセット）

パス区切り（`/` `\`）・`..`・先頭 `-` を構造的に排除する。ターゲットパスの
組み立て・テンプレート内文字列置換の双方でパストラバーサル・構文注入が
起こり得ない文字集合に限定している（security.md A01/A03）。

## 6. 決定性の保証

- 展開は選択した `Template::files` の配列順（固定）で実行する。内容は
  コンパイル時定数であり、プロジェクト名以外の入力を混ぜない。
- タイムスタンプ・乱数・環境変数由来の値を出力ファイルへ一切書き込まない。
- パーミッション: `executable: true` のファイルへ Unix では 0o755 を明示
  設定する（`std::os::unix::fs::PermissionsExt`、`#[cfg(unix)]`）。
  **非 Unix プラットフォームではパーミッションモデルが異なるため設定を
  スキップする**（`#[cfg(not(unix))]` の no-op 実装、`crates/cli/src/new.rs::set_permissions`）。
- 書き込み途中の失敗は該当パス付きで stderr へ報告して終了コード 1 とする
  （部分生成物は削除しない = 成功と誤認させないことのみ保証する。
  `--force` でも削除系操作は一切行わない）。

### 6.1 非 Unix プラットフォームでのパーミッション挙動（イシュー #378 で明確化）

- `set_permissions` は非 Unix（`#[cfg(not(unix))]`）で no-op。実行ビットの
  設定自体が行われず、エラーにもならない（黙示的にスキップされる）。
- 決定性の保証（§6）は**バイト内容の同一性**が主であり、実行ビットは
  Unix のみで担保される副次的な性質と位置づける。非 Unix 環境での
  `same_args_produce_byte_identical_output_across_two_runs` 相当のテストは、
  `collect_tree`（`crates/cli/tests/new_e2e.rs`）が `#[cfg(not(unix))]` で
  `executable = false` を返すため、実行ビット差異を検知対象に含めない
  （プラットフォーム条件分岐込みで決定性の定義が閉じている）。
- `tools/npm-asset-build/*`（`executable: true` の 3 ファイル）は
  `.github/workflows/npm-asset-gate.yml` 等の CI ワークフローから
  インタープリタ経由（`bash install.sh` / `python3 check_static_only.py`
  等）で起動される契約であり、実行ビット欠落そのものが動作を妨げない
  （呼び出し側がシェバン実行に依存しない）。非 Unix でのローカル直接実行
  （ダブルクリック相当）は本フレームワークの配布形態（Docker/Linux 想定）
  のスコープ外とする。
- 各テンプレートの実行可能ファイル集合は `crates/cli/src/new_template.rs`
  （`executable_file_sets_match_expected_fixed_lists` テスト）が期待固定
  リストとの一致をプラットフォーム非依存に検証する（メタデータの記述内容
  のみを比較するため、どの OS でも実行できる）。
- **実機検証ハーネス（イシュー #413）**: 上記は設計上の主張であり、
  self-hosted Linux runner のみでの CI 実行では Windows 上での実挙動は
  未検証だった。イシュー #413 で `.github/workflows/fw-new-windows-verify.yml`
  （`workflow_dispatch` 専用）を確立し、Windows self-hosted runner 上で
  ビルド・`new_template`/`new_e2e` テスト・`fw new` 生成物のバイト決定性・
  fail-closed 契約・`executable: true` ファイルの no-op 生成を検証する。
  runner 調達要件は `docs/ci/ci-runner-requirements.md` §5、検証結果は
  `docs/reports/fw-new-windows-verification-report.md` に記録する。

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
- **テンプレート名の allowlist 照合（イシュー #378）**: `--template` は
  コンパイル時定数 `TEMPLATES` との完全一致照合のみで解決し、ユーザー
  入力から動的にパス・`include_str!` 対象を組み立てない（A01/A03）。
- **`app` テンプレート固有（イシュー #378）**: `raw_html()` は使用しない
  （REQ-1、`clippy.toml` の `disallowed-methods` が依存追加により初めて
  実効化される。`crates/cli/tests/new_gate_e2e.rs::fw_new_app_template_default_escape_check_detects_injected_violation`
  が実際に注入検出を固定）。`templates/app/tests/escape_regression.rs`
  （生成プロジェクト内 XSS 回帰テスト）が `fw gate` の `test` チェックで
  常時実行される。vendor 同梱（fandhe-frontend-core / fandhe-frontend-app）は正本とのドリフト検知
  （`crates/cli/tests/template_vendor_drift.rs`）で改ざん・陳腐化を検出する。

## 8. テスト（`crates/cli/tests/new_e2e.rs`）

実バイナリ（`CARGO_BIN_EXE_fw`）を起動する e2e テスト:

1. **決定性**: 同一引数で 2 つの親ディレクトリへ実行し、再帰走査した
   相対パス集合・各ファイルのバイト列・Unix パーミッション（実行ビット）が
   完全一致することを確認する。
2. **fail-closed**: 1 回目成功 → 同一ターゲットへの 2 回目は失敗（既存内容
   が不変）→ `--force` 付きで成功、の順に確認する。
3. **終了コード契約**: 引数なし・不正名・未知フラグは 2、成功は 0。
4. **置換検証**: 生成後の `Cargo.toml` / `Cargo.lock` に
   `name = "<project-name>"` があり `fandhe-frontend-template-default` が残らないこと。
   置換対象外ファイルはテンプレートとバイト一致すること。
5. **ドリフト検知**: `templates/default/` を再帰走査し、埋め込み
   マニフェストと相対パス集合・内容バイト列（`Cargo.toml`/`Cargo.lock`/
   `structure.toml` を除く）・実行ビットが 1:1 対応することを確認する。

さらに `crates/cli/tests/new_gate_e2e.rs`（イシュー #351／#378／#401／#410）が
`fw new` → `fw gate` の直列 e2e を実バイナリで実行し、生成直後のプロジェ
クトが無編集で `fw gate` の 6 チェック（type_check / default_escape_check /
url_validation_check / lint / test / policy）全 PASS になることをテンプレート
ごとに固定する。`default`/`app` は `policy`（cargo-deny 依存）のみ実行環境で
分岐するため、`crates/cli/tests/scenarios/bugfix_escape.rs::baseline_passes_gate`
と同一方針でスキップ・`#[ignore]` を使わず両分岐（PASS / 環境エラーによる
BLOCKED）を断定する。`.github/workflows/ci.yml` の test ジョブへ明示ステップ
として組み込み済み。`app` テンプレートの gate e2e は vendored 2 crate の
コンパイルを伴うため `default` より実行時間が長い。`embed`
（静的専用モード、§3.3）は cargo を一切起動しないため cargo-deny の
導入有無に依存せず、6 チェック全 PASS・`gate_result: "PASS"`・終了コード 0
を無条件に断定する。

`crates/cli/tests/template_vendor_drift.rs`（イシュー #378 新設、イシュー #411 で
`fandhe-frontend-interactive`/`fandhe-frontend-wasm-client` を追加）は vendor 同梱（fandhe-frontend-core /
fandhe-frontend-app / fandhe-frontend-interactive / fandhe-frontend-wasm-client）と正本 `crates/core/`/`crates/app/`/
`crates/interactive/`/`crates/wasm-client/` の乖離検知、`wasm/Cargo.lock` の
wasm-bindgen/web-sys バージョンとリポジトリ本体 `Cargo.lock` の一致検知、
および `templates/default/` と `templates/app/` の共有ファイル
（`.github/workflows/*`・`clippy.toml`・`deny.toml`・`tools/npm-asset-build/*`）
のバイト同一性を検証する。

## 9. 非目標（Non-goals）

- **静的単一ファイル `embed` テンプレート**は #378 の範囲外だったが、
  イシュー #410 で `fw new --template embed`（§3.3）として製品化済み。
- **wasm ビルドを含む CSR の完全実体**は本イシュー（#378）の範囲外だったが、
  イシュー #411 で同梱済み（§3b 参照）。その後イシュー #493 の crates.io
  バージョン依存への切替に伴い、§3b の構成も更新された。
- **crates.io 公開後の vendor → バージョン依存への切替**はイシュー #412 の
  チェックリストに従い、イシュー #493 で実施完了（§3a・§3b 参照）。
- **Windows 実機 CI での非 Unix 挙動の実測**: 本イシュー（#378）時点では
  self-hosted Linux runner のみのため未実施だったが、イシュー #413 で
  `.github/workflows/fw-new-windows-verify.yml`（`workflow_dispatch` 専用）
  として実機検証ハーネスを確立した（§6.1・`docs/ci/ci-runner-requirements.md`
  §5・`docs/reports/fw-new-windows-verification-report.md` 参照）。Windows
  self-hosted runner の調達（登録）完了までは実行待ちの状態。
- 非 Unix でのパーミッション再現（ACL 相当の代替設定等）は行わない。
- ルート直下クレートの `structure.toml` スキーマ上の正式化（`root` 慣習の
  一般化）と `fw structure`/`fw impact`/`default_escape_check` の当該盲点の
  一般対応はイシュー #353 で完了した（`docs/design/structure-manifest.md`
  §2.2 / `crates/cli/tests/impact_root_crate.rs` / `crates/cli/tests/new_gate_e2e.rs` 参照）。
