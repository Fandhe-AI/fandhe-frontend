# NPM アセットビルドパイプライン利用ガイド（TASK-12.1b）

> **本書のステータス**: TASK-12.1（親イシュー #37）配下のサブイシュー
> TASK-12.1b（#39）の成果物です。TASK-12.1a（`install.sh` 実装、#38、
> PR #186 でマージ済み）・TASK-12.2a〜c（`check_static_only.py` 実装、
> #122〜#124）はいずれも完了済みで、本書はそれらをつなぐ**パイプライン
> 全体の利用ガイドと受け入れ検証レポート**です。判定ルールそのものの
> 設計は `docs/policy/npm-static-asset-rules.md` に委ね、本書では二重管理しません。
> イシュー #296（REQ-12 残課題）で `install.sh` に allowlist 自動連携と
> `npm audit` 統合を追加しました（§2〜§4 に反映済み）。
> イシュー #316（#296 の将来拡張の着手判断）で allowlist 半自動追記
> （`apply_exempt.py`、§3.4）と `templates/default/` への本パイプライン
> 同梱（§3.5）を追加しました。`npm audit` キャッシュ機構・cargo-deny 統合 /
> `xtask` 呼び出し導線は見送りました（理由は §6 参照）。

## 1. 目的とトレーサビリティ

REQ-12「NPM 互換（ビルド時静的アセット限定、実行時スコープ外）」
（`docs/spec/04-requirements.md`）の受け入れ基準は次の 3 点です。

| 受け入れ基準 | 担当成果物 | 状態 |
|-------------|-----------|------|
| 基準 1: ビルド時に取り込む NPM パッケージのインストールが `--ignore-scripts` を既定で使用すること | `tools/npm-asset-build/install.sh` | 実装済み（#38）+ 実 npm 検証（本書 §5 ケース A/B） |
| 基準 2: 取り込んだパッケージに実行可能コードを含まないことを機械的に検証する仕組みを持つこと | `tools/npm-asset-build/check_static_only.py` | 実装済み（#123）+ パイプライン連結検証（本書 §5 ケース C/D） |
| 基準 3: クライアント実行時（配布バイナリ・ブラウザ）に NPM パッケージのコード・Node ランタイムが一切含まれないこと | `dist-server/tests/no_npm_runtime_in_binary.rs` | 実装済み（TASK-12.3、#125） |

タスク・イシュー・PR の対応関係:

| タスク | イシュー | 内容 | PR |
|--------|---------|------|-----|
| TASK-12.1a | #38 | `--ignore-scripts` 既定の `install.sh` 実装 | #186 |
| TASK-12.1b | #39（本書） | パイプライン e2e 検証・利用ガイド整備 | #230 |
| TASK-12.2a | #122 | 静的アセット判定ルール設計 | #223 |
| TASK-12.2b | #123 | `check_static_only.py` 実装 | #226 |
| TASK-12.2c | #124 | fixture テスト・CI 統合 | #225 |
| TASK-12.3 | #125 | 配布バイナリ・Docker イメージへの NPM/Node 非混入の機械検証（`dist-server/tests/no_npm_runtime_in_binary.rs`） | #255 |

## 2. パイプライン全体像

```
NPM パッケージ指定
      │
      ▼
tools/npm-asset-build/install.sh   … 入口。--ignore-scripts を迂回不能に強制
      │  （npm install/ci --ignore-scripts --no-audit 実行 → node_modules/ を生成）
      ▼
npm audit --audit-level=<level>    … 既知 advisory の導入時検査（既定有効）
      │  （しきい値以上の advisory 検出時は非 0 で終了）
      ▼
tools/npm-asset-build/check_static_only.py … 後段ゲート。静的アセット限定を機械検証
      │  （install.sh が allowlist 自動連携付きで自動起動。exit 0 のときのみ後続へ）
      ▼
配布物（static/ 等）への取り込み

      … 違反検出時（exit 1）は --suggest-exempt が [[exempt]] 雛形を提案 …
              │
              ▼ （人間によるレビュー・reason 編集。ここは自動化しない）
      tools/npm-asset-build/apply_exempt.py --suggestions <reviewed.toml>
              │  （§3.4。レビュー済みエントリを allowlist.toml へ半自動追記）
              ▼
      allowlist.toml 更新 → check_static_only.py を再実行して確認
```

- **`install.sh`（入口・受け入れ基準 1）**: `npm install` / `npm ci` を
  必ず `--ignore-scripts` 付きで実行するラッパーです。フラグと
  `npm_config_ignore_scripts=true` 環境変数の二重で強制し、
  `--ignore-scripts=false` 等の迂回フラグ・未知フラグはすべて拒否します。
  これにより `preinstall` / `install` / `postinstall` の暗黙実行を防ぎます。
- **`npm audit` 統合（REQ-12 残課題、イシュー #296）**: install/ci 成功後に
  既定で `npm audit --audit-level=high`（既定しきい値。`--audit-level` で
  変更可能）を実行し、既知 advisory を導入時に検出します。npm install/ci
  自身の簡易 audit は `--no-audit` で抑止し、検査を本ステップへ一本化します。
  オフライン環境向けに `--no-audit` で明示オプトアウトできますが、使用時は
  警告を出力します。
- **`check_static_only.py`（後段ゲート・受け入れ基準 2）**: `install.sh` が
  生成した `node_modules/` を走査し、allowlist 方式（既定拒否）で
  実行可能コード（`.js`/`.mjs`/`.cjs`/`.node`/`.wasm` 等）・
  実行ビット・shebang・`package.json` の lifecycle スクリプト等の混入を
  検出します。判定ルールの詳細は `docs/policy/npm-static-asset-rules.md` を
  参照してください。**イシュー #296 で `install.sh` からの自動連携**が
  加わり、install/ci・audit の成功後に allowlist 解決（§3.1 参照）付きで
  自動起動されます（既定有効。`--no-check` で明示オプトアウト可）。
- **両者は独立した多層防御**です。`install.sh` 単体ではパッケージ内の
  明示的な `require()` やビルドプラグイン実行までは防げないため（PoC-6、
  §4 参照）、`check_static_only.py` がその隙間を埋めます。

## 3. 使い方

### 3.1 `install.sh`

```
install.sh --dir <project-dir> [<package-spec>...]
           [--no-audit] [--audit-level <low|moderate|high|critical>]
           [--no-check] [--allowlist <path>]
```

- `--dir <project-dir>`: 必須。`package.json` が存在するディレクトリ
- `<package-spec>...`: 省略時は `<project-dir>` の依存を解決してインストール
  - `package-lock.json` が存在する場合: `npm ci --ignore-scripts --no-audit`
    （ロックファイルとの完全性検証込み）
  - 存在しない場合: `npm install --ignore-scripts --no-audit`
- `<package-spec>` を 1 つ以上指定した場合: `npm install --ignore-scripts --no-audit -- <package-spec>...`
- `--ignore-scripts=false` / `--no-ignore-scripts` / `--foreground-scripts`
  等、スクリプト実行を再有効化しようとするフラグは拒否され、非 0 で終了します
  （npm は一切呼び出されません）
- `--dir`・パッケージ指定・以下の 4 フラグ以外の未知フラグは既定で拒否されます
- `--no-audit`: install/ci 成功後の `npm audit` ステップを明示的にスキップします
  （警告を stderr に出力。オフライン/エアギャップビルド向け）
- `--audit-level <level>`: `npm audit` のしきい値（`low`/`moderate`/`high`/
  `critical` の完全一致のみ受理・既定 `high`。それ以外の値は npm を呼び出さず
  拒否します）
- `--no-check`: 自動連携された `check_static_only.py` をスキップします
  （警告を stderr に出力。CI の独立ゲートは別途残るため多層防御は維持されます）
- `--allowlist <path>`: `check_static_only.py` に渡す allowlist.toml を明示指定します
  （既定の解決順より優先。存在しないパスはエラー）

**allowlist 解決順**（`--no-check` 時は評価しません）:

1. `--allowlist <path>` が明示された場合はそれを使う
2. `<project-dir>/allowlist.toml` が存在すればそれを使う
3. 標準雛形 `tools/npm-asset-build/allowlist.toml` を使う

探索はこの 2 段のみで、それ以上の暗黙探索はしません。また
`node_modules` が生成されない（依存 0 件の）プロジェクトでは、
`check_static_only.py` の自動起動を notice 付きでスキップします
（fail-closed の対象は「存在すべきものの不正・不在」であり、依存ゼロは
正常系として扱います）。

### 3.2 `check_static_only.py`

```
check_static_only.py (--node-modules <path> | --dir <project-dir>)
                      [--allowlist <allowlist.toml>] [--suggest-exempt]
```

- `--node-modules` または `--dir`（`<dir>/node_modules` を対象にする）の
  いずれか一方が必須です
- `--allowlist`: 免除エントリを記載した `allowlist.toml` へのパス
  （省略可・暗黙探索はしません）
- `--suggest-exempt`（イシュー #296）: 違反検出時に、対応する
  `allowlist.toml` の `[[exempt]]` 雛形（`reason = "TODO: ..."` 付き）を
  stdout へ出力します。**allowlist.toml への自動書き込みは一切行いません**
  （提案のみ・免除の追加は必ず人間のレビューを経る）。ハード拒否対象
  （`.js`/`.mjs`/`.cjs`/`.node`/`.wasm`）の違反には雛形の代わりに
  「免除不可」の注記を出力します
- **終了コード契約**: `0` = 全合格 / `1` = 違反あり / `2` = 実行エラー
  （パス不在・allowlist 不正等、fail-closed）

### 3.3 `allowlist.toml`

パッケージ + ルール単位（`R2-ext` のみ拡張子 or 個別ファイルパス単位が必須）
の免除機構です。ワイルドカード不可・`reason` 必須という fail-closed な
設計であり、雛形とスキーマは `tools/npm-asset-build/allowlist.toml` の
コメントと `docs/policy/npm-static-asset-rules.md` §3.4 を参照してください。
本書では詳細を二重管理しません。

### 3.4 `apply_exempt.py`（allowlist 半自動追記、イシュー #316）

```
apply_exempt.py --suggestions <reviewed.toml> --allowlist <allowlist.toml> [--dry-run]
```

`--suggest-exempt`（§3.2）が出力する `[[exempt]]` 雛形を、**人間がレビュー・
編集して保存したファイル**に対して allowlist.toml へ半自動で追記するコマンドです。
`check_static_only.py` 本体・`install.sh` からは呼び出されません（自動連携
しない）。「チェッカー・install.sh からの allowlist 自動書き込みなし」という
既存方針（イシュー #296・A08 観点）は維持したまま、人間レビュー後の適用
手作業（コピー & ペースト）に伴う転記ミス・エスケープ崩れのリスクだけを
減らすことが目的です。

- **入力契約**: `--suggestions` はファイル全体が有効な TOML であることを
  要求します。`--suggest-exempt` の生出力（`VIOLATION ...` 行が混在する）を
  そのまま渡すとパースエラーで拒否されます。人間が VIOLATION 行を取り除き、
  `reason = "TODO: ..."` を実際の理由へ書き換えて保存したファイルだけを
  受理する意図的なゲートです
- **検証**: `check_static_only.py` の `validate_exempt_entries()` を再利用し、
  allowlist の検証規則を二重実装しません。`reason` が空・`TODO:` で始まる
  雛形のままのエントリは拒否します（人間レビューの強制）。ハード拒否拡張子
  （`.js`/`.mjs`/`.cjs`/`.node`/`.wasm`）への免除は適用側でも拒否されます
- **書き込み**: 既存 allowlist + 追記エントリのマージ結果を書き込み前に
  再検証し、合格した場合のみ同一ディレクトリの一時ファイル経由で
  `os.replace()` によりアトミックに置換します。検証に失敗した場合、対象
  ファイルは一切変更されません。既存 allowlist に同一エントリ（package,
  rule, ext/file）が既にあれば `SKIPPED` として重複追記しません（冪等）
- **`--dry-run`**: 検証・適用予定の内容を出力するのみで、ファイルは変更しません
- **終了コード契約**: `0` = 適用完了（適用 0 件の冪等成功を含む）/
  `1` = エントリ検証で拒否あり（提案ファイルに修正が必要）/
  `2` = 実行エラー（ファイル不在・TOML 構文エラー等）

### 3.5 `templates/default/` への同梱（イシュー #316）

`templates/default/tools/npm-asset-build/` に正本 4 ファイル
（`install.sh` / `check_static_only.py` / `apply_exempt.py` /
`allowlist.toml`）をバイト同一のままコピー同梱しています
（`deny.toml` + `deny.yml` の同梱前例、TASK-4.1/4.2 に倣う）。
テンプレート利用者向けの CI ワークフロー
`templates/default/.github/workflows/npm-asset-gate.yml` が
`package.json` / `package-lock.json` / `allowlist.toml` /
`tools/npm-asset-build/**` の変更時に `install.sh --dir .` を実行します
（`deny.yml` と同様 `ubuntu-latest` を使用。本リポジトリ CI の
self-hosted 既定は適用外）。

正本とコピーのドリフトは `tools/npm-asset-build/tests/test_template_sync.sh`
が CI（`.github/workflows/ci.yml` の `npm-asset-build` ジョブ）で機械検証
します。正本を変更したら `templates/default/tools/npm-asset-build/` へも
同じ内容を反映してください（同期を忘れると本テストが fail-closed で検知します）。

## 4. セキュリティモデルと限界

本パイプラインは PoC-6（`docs/spec/03-poc/npm-compat-feasibility/README.md`）
の実証結果に基づく多層防御であり、それぞれの層が防ぐ範囲・防がない範囲を
正確に理解して利用する必要があります。

| 層 | 防ぐもの | 防がないもの |
|----|---------|-------------|
| `install.sh`（`--ignore-scripts`） | `preinstall` / `install` / `postinstall` の**インストール時暗黙実行** | パッケージ内の明示的な `require()` 呼び出し・ビルドプラグインとしての実行コード呼び出し |
| `npm audit` 統合（イシュー #296） | 依存パッケージの**既知**advisory（npm レジストリに報告済みの脆弱性）をしきい値付きで導入時検出 | **未知・未報告**の悪意あるパッケージ・0-day・レジストリに未登録の脆弱性は検出できない（過大に主張しない。PoC-6 準拠） |
| `check_static_only.py`（静的アセット限定検証） | 実行可能コード（`.js`/`.mjs`/`.cjs`/`.node`/`.wasm`・実行ビット・shebang・SVG 内スクリプト等）の node_modules への混入 | 検証対象拡張子に該当しない未知の実行経路（allowlist は既定拒否だが、判定ルール自体が想定していない新種の攻撃手法までは保証しない） |

`npm audit` はレジストリへ依存メタデータ（パッケージ名・バージョン等）を
送信します。エアギャップ/オフラインビルドでは `--no-audit` で明示的に
迂回できますが、この場合は既知 advisory の検出自体が行われません
（警告出力あり。黙って skip はしません）。

**過大に主張しないこと**: このパイプラインは「静的アセット限定のパッケージを
安全に取り込める」ことを保証するものであり、「任意の NPM パッケージが
本フレームワークの実行時に安全に動作する」ことは保証しません（REQ-12 は
実行時スコープ外）。また、ビルド時（開発機・CI 上でこのパイプラインを実行する
プロセス自体）のリスク（例: 悪意あるパッケージが `check_static_only.py` の
判定前に何らかの手段でホスト環境に影響を与える可能性）は、`--ignore-scripts`
と静的アセット限定検証の 2 層防御でリスクを大幅に低減しますが、ゼロにする
ものではありません。

## 5. 検証レポート（e2e テスト結果）

`tools/npm-asset-build/tests/test_pipeline_e2e.sh` により、実 npm を用いて
パイプライン全体を end-to-end で検証しました（完全オフライン・
`npm pack` で生成したローカル tarball のみ使用、レジストリアクセスなし）。

| ケース | 内容 | PoC-6 との対応 | 結果 |
|--------|------|----------------|------|
| A | `postinstall` でマーカーファイルを書き込む evil-pkg 相当 fixture を `install.sh` 経由でインストール → インストール自体は成功しつつマーカーファイルが生成されない（ライフサイクルスクリプトがブロックされた） | PoC-6 の evil-pkg 実証を製品成果物 `install.sh` で再現 | PASS |
| B（対照実験） | 同じ fixture を `install.sh` を経由しない素の `npm install`（`npm_config_ignore_scripts` 未設定）でインストール → マーカーファイルが生成される | ケース A が偽陰性でないことの担保 | PASS |
| C | 静的アセット限定 fixture（CSS のみ）を `install.sh` でインストール後 `check_static_only.py` を実行 → exit 0 | 基準 2・パイプライン連結の正常系 | PASS |
| D | JS 実行エントリ（`main` が `.js`）を持つ fixture を同パイプラインで流す → `check_static_only.py` が exit 1 で違反報告 | 基準 2・パイプライン連結の異常系（違反検出） | PASS |
| E | ケース D と同じ fixture を `install.sh`（`--no-check` なし・自動連携有効）に流す → `install.sh` 自身が非 0 終了 + `[[exempt]]` 提案を出力 | イシュー #296: allowlist 自動連携の統合検証（正常な fail-closed 動作） | PASS |
| F | ケース E と同じ fixture・`--no-check` 付き → `install.sh` は成功し、check スキップの警告を出力 | イシュー #296: 明示オプトアウトの動作確認 | PASS |
| G | R1-bin 単独違反 fixture を `install.sh` でインストール → `--suggest-exempt` の提案出力 → VIOLATION 行除去・reason 編集（人間レビュー相当）→ `apply_exempt.py` で allowlist へ適用 → 同じ allowlist で再チェックすると `EXEMPTED` + exit 0 | イシュー #316: 半自動追記の往復（提案 → レビュー → 適用 → 再検証） | PASS |

既存の回帰テストもあわせて全件 PASS を確認済みです。

| テスト | 結果 |
|--------|------|
| `bash tools/npm-asset-build/tests/test_install.sh` | 18 passed, 0 failed |
| `python3 tools/npm-asset-build/tests/test_check_static_only.py -v` | 51 tests, OK |
| `bash tools/npm-asset-build/tests/test_pipeline_e2e.sh` | 7 passed, 0 failed（ケース G 含む） |
| `python3 tools/npm-asset-build/tests/test_apply_exempt.py -v` | 10 tests, OK |
| `bash tools/npm-asset-build/tests/test_template_sync.sh` | 4 passed, 0 failed |

CI（`.github/workflows/ci.yml` の `npm-asset-build` ジョブ）は上記 5 つの
テストをすべて fail-closed（緩和用 input・`continue-on-error` なし）で
実行します。`test_pipeline_e2e.sh` はローカルで `npm`/`node`/`python3` が
見つからない場合のみ notice を出して skip しますが、CI 環境
（self-hosted runner、`.claude/rules/ci.md` 準拠）には Node.js セットアップ
ステップがあるため、CI 上では実質的に必ず実行されます。

## 6. スコープ外事項

以下は本タスク（TASK-12.1b）のスコープ外として記録し、別途 Issue 化を
検討します（`.claude/rules/out-of-scope-tracking.md` 準拠）。

- `ci.yml` の `check_static_only.py` 存在ガード撤去: TASK-12.2（#121）で
  対応済みです。現行 `ci.yml` の `npm-asset-build` ジョブは存在ガードを
  介さず fail-closed で 3 テストを実行するため、本項目は解消済みです
- `install.sh` 本体の機能拡張（allowlist 連携の自動起動・`npm audit`
  統合等）: **イシュー #296 で対応済みです**（§2〜§5 参照）。allowlist.toml
  への自動書き込みは行わず提案出力（`--suggest-exempt`）にとどめる設計は
  A08（ソフトウェア・データ完全性）の観点から意図的な選択です

以下は #296 の実装時に新たにスコープ外と判断した事項で、いずれもイシュー
#316 で採否判断を行いました。

- allowlist の提案（`--suggest-exempt`）から `allowlist.toml` への半自動
  追記（人間レビュー後のワンコマンド適用等）: **イシュー #316 で採用・
  実装済みです**（`apply_exempt.py`、§3.4）
- `templates/default/` への本パイプライン同梱: **イシュー #316 で採用・
  実装済みです**（§3.5）
- `npm audit` 結果のキャッシュ機構: **イシュー #316 で見送りました**。
  audit の価値は既知 advisory の「鮮度」にあり、キャッシュは検出遅延
  （安全性低下）と引き換えに小さな CI コスト削減しか得られません。現行
  `npm-asset-build` ジョブ（timeout 15 分）で audit がボトルネックである
  実測もありません。要否が変わった場合はユーザー承認を得たうえで再度
  Issue 化を検討します
- cargo-deny 側との統合・`xtask` クレートからの呼び出し導線: **イシュー
  #316 で見送りました**。cargo-deny 統合は npm / cargo のエコシステム
  横断設計が必要で要件が未定義です。`xtask` 呼び出し導線は
  `templates/default/` へのテンプレート同梱（`install.sh` 直接呼び出しで
  十分）と価値が重複するため見送りました。ユーザーの承認なしに Issue は
  起票していません（`.claude/rules/out-of-scope-tracking.md` 準拠）
