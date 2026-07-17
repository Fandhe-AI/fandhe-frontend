# NPM アセットビルドパイプライン利用ガイド（TASK-12.1b）

> **本書のステータス**: TASK-12.1（親イシュー #37）配下のサブイシュー
> TASK-12.1b（#39）の成果物です。TASK-12.1a（`install.sh` 実装、#38、
> PR #186 でマージ済み）・TASK-12.2a〜c（`check_static_only.py` 実装、
> #122〜#124）はいずれも完了済みで、本書はそれらをつなぐ**パイプライン
> 全体の利用ガイドと受け入れ検証レポート**です。判定ルールそのものの
> 設計は `docs/npm-static-asset-rules.md` に委ね、本書では二重管理しません。

## 1. 目的とトレーサビリティ

REQ-12「NPM 互換（ビルド時静的アセット限定、実行時スコープ外）」
（`docs/spec/04-requirements.md`）の受け入れ基準は次の 2 点です。

| 受け入れ基準 | 担当成果物 | 状態 |
|-------------|-----------|------|
| 基準 1: ビルド時に取り込む NPM パッケージのインストールが `--ignore-scripts` を既定で使用すること | `tools/npm-asset-build/install.sh` | 実装済み（#38）+ 実 npm 検証（本書 §5 ケース A/B） |
| 基準 2: 取り込んだパッケージに実行可能コードを含まないことを機械的に検証する仕組みを持つこと | `tools/npm-asset-build/check_static_only.py` | 実装済み（#123）+ パイプライン連結検証（本書 §5 ケース C/D） |

タスク・イシュー・PR の対応関係:

| タスク | イシュー | 内容 | PR |
|--------|---------|------|-----|
| TASK-12.1a | #38 | `--ignore-scripts` 既定の `install.sh` 実装 | #186 |
| TASK-12.1b | #39（本書） | パイプライン e2e 検証・利用ガイド整備 | 本 PR |
| TASK-12.2a | #122 | 静的アセット判定ルール設計 | #223 |
| TASK-12.2b | #123 | `check_static_only.py` 実装 | #226 |
| TASK-12.2c | #124 | fixture テスト・CI 統合 | #225 |

## 2. パイプライン全体像

```
NPM パッケージ指定
      │
      ▼
tools/npm-asset-build/install.sh   … 入口。--ignore-scripts を迂回不能に強制
      │  （node_modules/ を生成）
      ▼
tools/npm-asset-build/check_static_only.py … 後段ゲート。静的アセット限定を機械検証
      │  （exit 0 のときのみ後続へ進む）
      ▼
配布物（static/ 等）への取り込み
```

- **`install.sh`（入口・受け入れ基準 1）**: `npm install` / `npm ci` を
  必ず `--ignore-scripts` 付きで実行するラッパーです。フラグと
  `npm_config_ignore_scripts=true` 環境変数の二重で強制し、
  `--ignore-scripts=false` 等の迂回フラグ・未知フラグはすべて拒否します。
  これにより `preinstall` / `install` / `postinstall` の暗黙実行を防ぎます。
- **`check_static_only.py`（後段ゲート・受け入れ基準 2）**: `install.sh` が
  生成した `node_modules/` を走査し、allowlist 方式（既定拒否）で
  実行可能コード（`.js`/`.mjs`/`.cjs`/`.node`/`.wasm` 等）・
  実行ビット・shebang・`package.json` の lifecycle スクリプト等の混入を
  検出します。判定ルールの詳細は `docs/npm-static-asset-rules.md` を
  参照してください。
- **両者は独立した多層防御**です。`install.sh` 単体ではパッケージ内の
  明示的な `require()` やビルドプラグイン実行までは防げないため（PoC-6、
  §4 参照）、`check_static_only.py` がその隙間を埋めます。

## 3. 使い方

### 3.1 `install.sh`

```
install.sh --dir <project-dir> [<package-spec>...]
```

- `--dir <project-dir>`: 必須。`package.json` が存在するディレクトリ
- `<package-spec>...`: 省略時は `<project-dir>` の依存を解決してインストール
  - `package-lock.json` が存在する場合: `npm ci --ignore-scripts`
    （ロックファイルとの完全性検証込み）
  - 存在しない場合: `npm install --ignore-scripts`
- `<package-spec>` を 1 つ以上指定した場合: `npm install --ignore-scripts -- <package-spec>...`
- `--ignore-scripts=false` / `--no-ignore-scripts` / `--foreground-scripts`
  等、スクリプト実行を再有効化しようとするフラグは拒否され、非 0 で終了します
  （npm は一切呼び出されません）
- `--dir`・パッケージ指定以外の未知フラグも既定で拒否されます

### 3.2 `check_static_only.py`

```
check_static_only.py (--node-modules <path> | --dir <project-dir>) [--allowlist <allowlist.toml>]
```

- `--node-modules` または `--dir`（`<dir>/node_modules` を対象にする）の
  いずれか一方が必須です
- `--allowlist`: 免除エントリを記載した `allowlist.toml` へのパス
  （省略可・暗黙探索はしません）
- **終了コード契約**: `0` = 全合格 / `1` = 違反あり / `2` = 実行エラー
  （パス不在・allowlist 不正等、fail-closed）

### 3.3 `allowlist.toml`

パッケージ + ルール単位（`R2-ext` のみ拡張子 or 個別ファイルパス単位が必須）
の免除機構です。ワイルドカード不可・`reason` 必須という fail-closed な
設計であり、雛形とスキーマは `tools/npm-asset-build/allowlist.toml` の
コメントと `docs/npm-static-asset-rules.md` §3.4 を参照してください。
本書では詳細を二重管理しません。

## 4. セキュリティモデルと限界

本パイプラインは PoC-6（`docs/spec/03-poc/npm-compat-feasibility/README.md`）
の実証結果に基づく多層防御であり、それぞれの層が防ぐ範囲・防がない範囲を
正確に理解して利用する必要があります。

| 層 | 防ぐもの | 防がないもの |
|----|---------|-------------|
| `install.sh`（`--ignore-scripts`） | `preinstall` / `install` / `postinstall` の**インストール時暗黙実行** | パッケージ内の明示的な `require()` 呼び出し・ビルドプラグインとしての実行コード呼び出し |
| `check_static_only.py`（静的アセット限定検証） | 実行可能コード（`.js`/`.mjs`/`.cjs`/`.node`/`.wasm`・実行ビット・shebang・SVG 内スクリプト等）の node_modules への混入 | 検証対象拡張子に該当しない未知の実行経路（allowlist は既定拒否だが、判定ルール自体が想定していない新種の攻撃手法までは保証しない） |

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

既存の回帰テストもあわせて全件 PASS を確認済みです。

| テスト | 結果 |
|--------|------|
| `bash tools/npm-asset-build/tests/test_install.sh` | 9 passed, 0 failed |
| `python3 tools/npm-asset-build/tests/test_check_static_only.py -v` | 42 tests, OK |
| `bash tools/npm-asset-build/tests/test_pipeline_e2e.sh` | 4 passed, 0 failed |

CI（`.github/workflows/ci.yml` の `npm-asset-build` ジョブ）は上記 3 つの
テストをすべて fail-closed（緩和用 input・`continue-on-error` なし）で
実行します。`test_pipeline_e2e.sh` はローカルで `npm`/`node`/`python3` が
見つからない場合のみ notice を出して skip しますが、CI 環境
（`ubuntu-latest`）には常にプリインストールされているため、CI 上では
実質的に必ず実行されます。

## 6. スコープ外事項

以下は本タスク（TASK-12.1b）のスコープ外として記録し、別途 Issue 化を
検討します（`.claude/rules/out-of-scope-tracking.md` 準拠）。

- `ci.yml` の `check_static_only.py` 存在ガード撤去（fail-closed 化）:
  TASK-12.2b（#123）マージ済みによりガードは実質無効化されていますが、
  ガードコード自体の撤去は別途フォローアップとして扱います
- `install.sh` 本体の機能拡張（allowlist 連携の自動起動・`npm audit`
  統合等）: TASK-12.1 のスコープ（インストール入口の `--ignore-scripts`
  強制）を超えるため対象外です
