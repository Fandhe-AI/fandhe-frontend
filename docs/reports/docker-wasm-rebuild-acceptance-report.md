# Docker マルチステージビルド内 WASM 再ビルド 受け入れ基準検証レポート（TASK-10.3c）

> **注記（#433 改名）**: 本レポートは旧名称時代の実測記録です。crate 名 `rws-*`（`rws-dist-server` 等）は #441 で `fandhe-frontend-*` へ、クレート配置はルート直下から #442 で `crates/` 配下へ、環境変数 `RWS_BIND_ADDR` / `RWS_WASM_BUILD` は #437 で `FANDHE_FRONTEND_BIND_ADDR` / `FANDHE_FRONTEND_WASM_BUILD` へ改名され、リポジトリ名 `Fandhe-AI/frontend-framework` は #439 で `Fandhe-AI/fandhe-frontend` へ改名済みです（新旧対応は `docs/design/framework-naming.md` 参照）。以下の記録中のコマンド・パス・URL・値は当時のまま残しています。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-10 受け入れ基準 4 点目「Docker マルチステージビルド内で
  WASM ターゲットの再ビルドが行われ、CI 環境での再現性が担保されること」
  （`docs/spec/04-requirements.md` 132〜142 行目）。
- **親タスク**: TASK-10.3（#114、`docs/spec/05-tasks.md` 258〜263 行目）。
- **サブタスク構成**:
  - TASK-10.3a（#115・クローズ済み）: 設計、`docs/design/docker-wasm-build-stage.md`
  - TASK-10.3b（#116）: `Dockerfile` builder ステージへの WASM 再ビルド
    ステージ統合（実装）
  - TASK-10.3c（本レポート・#117）: コンテナ内 WASM 再ビルドの実ビルド検証
- 本レポートは `docs/design/docker-wasm-build-stage.md` §6 が引き継いだ 6 つの
  検証観点に対する実測結果を記録する。

## 2. 判定ステータス: 検証済み（PASS）

`docs/design/docker-wasm-build-stage.md` §6 の検証観点 1〜6 をいずれも実測し
Pass を確認した。観点 6（x86_64・aarch64 両ビルドホストでのアーキ分岐確認）は
当初（TASK-10.3c 時点）利用可能な環境が x86_64 のみだったため未実測（手動
確認手順の記載で代替）としていたが、イシュー #450 で aarch64 実機
（Apple Silicon macOS ホスト上の Docker Engine、`linux/arm64` ネイティブ）が
利用可能になったことを受けて実測を実施し、Pass を確認した（第 5a 節）。

## 3. 検証結果一覧

| # | 検証観点（§6 引用） | 検証手段 | 結果 |
|---|---------------------|---------|------|
| 1 | `RWS_WASM_BUILD=0` 削除後、`docker build` が成功し、最終イメージの `static/wasm/` 相当の URL パスから WASM 成果物が配信されること | `docker build` → `docker run` → `curl` 実機確認 | Pass（第 4.1 節） |
| 2 | `wasm-bindgen-cli` バージョン不一致時に、`build.rs` のフェイルクローズがビルダーステージのビルド失敗として正しく伝播すること | `WASM_BINDGEN_VERSION` を意図的に不一致値へ変更した負例ビルド＋`xtask/tests/wasm_bindgen_version_sync.rs` による固定値同期ドリフトの回帰検出 | Pass（第 4.2 節） |
| 3 | 最終イメージに `wasm32-unknown-unknown` ターゲット・`wasm-bindgen-cli` バイナリが含まれないこと | `docker export \| tar -t` によるファイル一覧の全数確認 | Pass（第 4.3 節） |
| 4 | `xtask check-image-size`（50MB 上限）が WASM 成果物込みでも PASS すること | `cargo run -p xtask -- check-image-size` | Pass（第 4.4 節） |
| 5 | `image-size.yml` の `paths` 追加後、`wasm-full/`・`wasm-thin/`・`interactive/` の変更が正しくワークフローをトリガーすること | `.github/workflows/image-size.yml` の `paths` 実測確認 | Pass（第 4.5 節、追加は TASK-9.3b 以降の先行変更で既に存在） |
| 6 | `docker build` を x86_64・aarch64 の両ビルドホストで実行し、アーキ分岐が両方で成功すること | x86_64: 実機確認（第 4.1 節）。aarch64: 実機確認（イシュー #450、第 5a 節） | x86_64 Pass / aarch64 Pass（第 5a 節） |

## 4. 実測詳細

実行環境: Linux（ワークツリー環境、`uname -m` = `x86_64`）、Docker 29.5.3。
対象コミット: 本レポートと同一 PR の実装コミット（`Dockerfile`・
`.github/workflows/image-size.yml` の変更を含む）。

### 4.1 観点 1: WASM 資産の配信確認

```
$ docker build -t rws-dist-server:local .
（省略。builder ステージで rustup target add wasm32-unknown-unknown・
  wasm-bindgen-cli 導入・cargo build -p rws-dist-server が成功）

$ docker run -d --name rws-local-114 -p <host-port>:3100 rws-dist-server:local
$ curl -fsS -o /dev/null -w '%{http_code} %{content_type}\n' \
    http://127.0.0.1:<host-port>/static/wasm/rws_wasm_full.js
200 text/javascript; charset=utf-8

$ curl -fsS http://127.0.0.1:<host-port>/static/wasm/rws_wasm_full_bg.wasm | head -c 4 | od -c
0000000  \0   a   s   m

$ curl -s -o /dev/null -w '%{http_code}\n' \
    http://127.0.0.1:<host-port>/static/wasm/does-not-exist.wasm
404

$ docker logs rws-local-114
rws-dist-server: listening on 0.0.0.0:3100
rws-dist-server: assets=embedded
```

`assets=embedded` のログ行は `dist-server/build.rs` の WASM ビルドステージ
（`wasm_assets_embedded` cfg）が実際に発火し、コンテナ内で生成された WASM
資産が埋め込みテーブルへ合流したことを示す（ホスト側事前ビルド成果物には
依存しない）。

### 4.2 観点 2: バージョン不一致時のフェイルクローズ

`Dockerfile` の `WASM_BINDGEN_VERSION` を意図的に `0.2.126` から
`0.2.100`（存在しない/チェックサム不一致のバージョン）へ変更し、
`docker build --no-cache` を実行:

```
+ sha256sum -c -
/tmp/wasm-bindgen-0.2.100-x86_64-unknown-linux-musl.tar.gz: FAILED
ERROR: failed to build: failed to solve: process "..." did not complete
successfully: exit code: 1
```

チェックサム検証の時点でビルドが失敗し、ビルダーステージのビルド失敗として
正しく伝播することを確認した（SHA256 固定値を意図的なバージョンに追随
させていないため `wasm-bindgen-cli` 導入ステップで検出されたケース）。
`dist-server/build.rs::expected_wasm_bindgen_version` とのバージョン
突き合わせ自体も同様にフェイルクローズであることは TASK-10.2b（#110）の
実装・テストで担保済み（`dist-server/build.rs` 219〜227 行目）。検証後、
`Dockerfile` は元の値（`0.2.126`、`x86_64`/`aarch64` 双方の正しい
チェックサム）へ復元済み。

上記は単発の負例ビルド確認に留まっていたため、`WASM_BINDGEN_VERSION` /
`WASM_BINDGEN_SHA256` の固定値が `Dockerfile`・`.github/workflows/ci.yml`・
`Cargo.lock` の間でサイレントにドリフトする事態（ビルド実行まで検出が
遅延し原因特定コストが高い）を `cargo test` 時点で前倒し検出する回帰テスト
`xtask/tests/wasm_bindgen_version_sync.rs` を TASK-10.3c で追加した。
Cargo.lock の解決バージョンとの不一致・Dockerfile/ci.yml 間の SHA256
不一致を意図的に作った負例でいずれも fail-closed であることを確認済み。

### 4.3 観点 3: 最終イメージの非汚染確認

```
$ cid=$(docker create rws-dist-server:local)
$ docker export "$cid" | tar -t
.dockerenv
dev/
dev/console
dev/pts/
dev/shm/
dist-server
etc/
etc/hostname
etc/hosts
etc/mtab
etc/resolv.conf
proc/
sys/
$ docker rm "$cid"
```

最終イメージの内容は `/dist-server` バイナリ 1 つ（+ `scratch` ランタイムが
必然的に持つ `/dev` `/proc` `/sys` 等のメタエントリ）のみであり、builder
ステージへ追加した `curl`・`wasm32-unknown-unknown` ターゲット・
`wasm-bindgen-cli` バイナリはいずれも含まれない。この非汚染検証は
`.github/workflows/image-size.yml` の「Verify final image does not leak
the build toolchain」ステップとして CI に組み込み済み（fail-closed）。

### 4.4 観点 4: イメージサイズ上限（50MB）

```
$ cargo run --locked -p xtask -- check-image-size --image rws-dist-server:local
image-size: image=rws-dist-server:local size_bytes=571227/50000000 size_mb=0.57 result=PASS
```

WASM 資産込みでも 50MB 上限に対し十分なマージン（約 0.57MB）で PASS。

### 4.5 観点 5: `image-size.yml` の paths トリガー

`.github/workflows/image-size.yml` の `paths` を確認したところ、本レポート
執筆時点で既に `wasm-full/**`・`wasm-thin/**`・`interactive/**` が含まれて
いた（TASK-9.3b（#103）以降の別変更で先行して追加済み。
`docs/design/docker-wasm-build-stage.md` §2.2 参照）。TASK-10.3b・10.3c では
追加の変更は不要であることを確認した。

### 4.5b main マージ後の CI 実測（TASK-10.3c 完了時点）

第 4.5 節までの実測はワークツリー環境での単発実行に留まっており、REQ-10
「CI 環境での再現性が担保されること」の直接証跡としては、`main` ブランチ
マージ後の `image-size.yml` 継続実行結果を別途記録する必要がある。
TASK-10.3c 完了時点（本節追記時）で `main` 上の直近 5 回の
`image-size.yml` 実行を確認し、いずれも success（本レポートが検証対象と
する 2 ステップ「Verify WASM assets are served」「Verify final image does
not leak the build toolchain」を含む）であることを確認した。

| run ID | headSha | conclusion | URL |
|--------|---------|------------|-----|
| 29599984697 | 0461c6c2 | success | https://github.com/Fandhe-AI/frontend-framework/actions/runs/29599984697 |
| 29599400275 | 77cabefd | success | https://github.com/Fandhe-AI/frontend-framework/actions/runs/29599400275 |
| 29598763674 | 6d5cc693 | success | https://github.com/Fandhe-AI/frontend-framework/actions/runs/29598763674 |
| 29598461396 | 0fbfe1fe | success | https://github.com/Fandhe-AI/frontend-framework/actions/runs/29598461396 |
| 29597370943 | bf5ec748 | success | https://github.com/Fandhe-AI/frontend-framework/actions/runs/29597370943 |

最新 run（29599984697）についてステップ単位でも確認し、以下 2 ステップが
`conclusion=success` であることを確認済み:

- `Verify WASM assets are served (TASK-10.3c, issue #117)`
- `Verify final image does not leak the build toolchain (TASK-10.3c, issue #117)`

これにより、観点 1・3 は単発実測（第 4.1・4.3 節）に加え `main` 上での
継続的な再現性が担保されていることを確認できる。

### 4.6 観点 6: x86_64 / aarch64 両ビルドホストでのアーキ分岐確認

- **x86_64**: 本レポートの実測環境（ワークツリー、`uname -m` = `x86_64`）で
  `docker build` が成功し、`wasm-bindgen-0.2.126-x86_64-unknown-linux-musl.tar.gz`
  （チェックサム `064948d5...869d`）の分岐が正しく選択されることを確認した
  （§4.1 参照）。
- **aarch64**: TASK-10.3c 完了時点では利用可能な CI ランナー・ローカル環境が
  いずれも x86_64 のため実機ビルドは実測不能だった。TASK-10.3a（#115）設計
  時点で `wasm-bindgen-0.2.126-aarch64-unknown-linux-musl.tar.gz` の実在と
  チェックサム（`22451202...440a`）を GitHub Releases の公式
  `.sha256sum` と照合済みであることを踏まえ、当時は第 5 節に Apple Silicon
  等の aarch64 実ホストでの手動確認手順を記載するに留めていた。
  **イシュー #450 で aarch64 実機環境（Apple Silicon macOS ホスト上の
  Docker Engine、`linux/arm64` ネイティブ）が利用可能になったため、第 5 節の
  手順に沿って実機ビルドを実測し、Pass を確認した（第 5a 節）**。

## 5. aarch64 実ホストでの手動確認手順（イシュー #450 で実測済み・第 5a 節参照）

> 本節はイシュー #450 実測（第 5a 節）を経て実測済みとなったが、以下の
> コマンド・パス例は TASK-10.3c 執筆当時（旧クレート名 `rws-*`）のまま残す
> （冒頭注記〔#433 改名〕の方針に合わせる。第 5a 節が現行クレート名・現行
> リポジトリ名での実測記録）。

Apple Silicon の Docker Desktop（既定で `linux/arm64` イメージを使用）等の
aarch64 実ホストで、以下を実行して確認する:

```bash
git clone https://github.com/Fandhe-AI/frontend-framework.git
cd frontend-framework
docker build -t rws-dist-server:aarch64-check .
docker run -d --name rws-aarch64-check -p 3100:3100 rws-dist-server:aarch64-check
curl -fsS -o /dev/null -w '%{http_code} %{content_type}\n' \
  http://127.0.0.1:3100/static/wasm/rws_wasm_full.js   # 期待値: 200 text/javascript
curl -fsS http://127.0.0.1:3100/static/wasm/rws_wasm_full_bg.wasm | head -c 4 | od -c  # 期待値: \0 a s m
docker rm -f rws-aarch64-check
```

`docker build` のログで `RUN set -eux; WASM_BINDGEN_VERSION=...` ステップの
`uname -m` 判定が `aarch64` 分岐（`wasm-bindgen-0.2.126-aarch64-unknown-linux-musl.tar.gz`）
を選択していることも合わせて確認する。

## 5a. aarch64 実機実測結果（イシュー #450 追記）

第 5 節の手動確認手順を、現行の `Fandhe-AI/fandhe-frontend` リポジトリ・
現行クレート名（`fandhe-frontend-*`）に沿って aarch64 実機で実行し、
以下のとおり Pass を確認した。

**環境**:

- ホスト: macOS 26.6（Apple Silicon、`uname -m` = `arm64`）
- Docker: Docker Engine 29.6.1、`docker version --format
  '{{.Server.Os}}/{{.Server.Arch}}'` = `linux/arm64`、`docker info` の
  `Architecture: aarch64`（QEMU エミュレーションではなく aarch64 ネイティブの
  Linux VM 上での実行）
- リソース: NCPU=16、Mem 約 15.6 GiB（`docker info` 実測）

**対象コミット**: `763fc153eb23750230cbdea923cb0e0e1be24b66`
（`Fandhe-AI/fandhe-frontend` main、TASK-10.3c 実測当時の旧名称
`Fandhe-AI/frontend-framework`・`rws-*` とは異なる現行名称・現行クレート構成
での実測である点に注意）

**所要時間**: `docker build --no-cache -t fandhe-frontend-dist-server:aarch64-450 .`
の実行開始から完了まで実測約 22 秒（apt パッケージ導入・wasm-bindgen-cli
導入・`cargo build --release --locked --target aarch64-unknown-linux-musl
-p fandhe-frontend-dist-server` を含む全ステージ）。うち
`cargo build`（builder ステージ 10/10 ステップ）単体は
`Finished \`release\` profile [optimized] target(s) in 12.23s`（ビルドログ実測）。

**アーキ分岐の証跡**（ビルドログより抜粋）:

```
+ WASM_BINDGEN_ARCHIVE=wasm-bindgen-0.2.126-aarch64-unknown-linux-musl.tar.gz
+ WASM_BINDGEN_SHA256=2245120254a9f6c9a9adf3601f3d52bb31309219e9ceab7696e74e24885c440a
+ curl -sSfL -o /tmp/wasm-bindgen-0.2.126-aarch64-unknown-linux-musl.tar.gz ...
+ sha256sum -c -
/tmp/wasm-bindgen-0.2.126-aarch64-unknown-linux-musl.tar.gz: OK
```

`RUN case "$(uname -m)" in aarch64) echo aarch64-unknown-linux-musl > /musl_target ;;`
の musl ターゲット判定ステップも `aarch64-unknown-linux-musl` 分岐を選択し、
`cargo build --release --locked --target aarch64-unknown-linux-musl -p
fandhe-frontend-dist-server` が成功した。

**イメージサイズ**:

```
$ docker image inspect --format '{{.Size}}' fandhe-frontend-dist-server:aarch64-450
691605
$ cargo run --locked -p xtask -- check-image-size --image fandhe-frontend-dist-server:aarch64-450
image-size: image=fandhe-frontend-dist-server:aarch64-450 size_bytes=691605/50000000 size_mb=0.69 result=PASS
```

50MB 上限に対し十分なマージン（約 0.69MB）で PASS（x86_64 実測 0.57MB
〔TASK-10.3c 当時〕・第 4.4 節と同水準）。

**配信確認**（`docker run -d -p 127.0.0.1:<ephemeral>:3100` で起動し、現行の
WASM 資産名 `fandhe_frontend_wasm_full.js` / `fandhe_frontend_wasm_full_bg.wasm`
で確認、`rws_wasm_full.js` 等の旧名称〔#441 改名前〕ではない）:

```
$ curl -fsS -o /dev/null -w '%{http_code} %{content_type}\n' \
    http://127.0.0.1:<host-port>/static/wasm/fandhe_frontend_wasm_full.js
200 text/javascript; charset=utf-8

$ curl -fsS http://127.0.0.1:<host-port>/static/wasm/fandhe_frontend_wasm_full_bg.wasm | head -c 4 | od -c
0000000  \0   a   s   m

$ curl -s -o /dev/null -w '%{http_code}\n' \
    http://127.0.0.1:<host-port>/static/wasm/does-not-exist.wasm
404

$ docker logs ffds-450
fandhe-frontend-dist-server: listening on 0.0.0.0:3100
fandhe-frontend-dist-server: assets=embedded
```

**バイナリのアーキ確認**（アーキ分岐成功の直接証跡）:

```
$ cid=$(docker create fandhe-frontend-dist-server:aarch64-450)
$ docker cp "$cid":/dist-server ./dist-server-450 && docker rm "$cid"
$ file ./dist-server-450
./dist-server-450: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV),
statically linked, BuildID[sha1]=1a50ee692b850764c88be698b3dd61f04defa976, stripped
```

**判定**: 上記いずれの実測値も期待どおりであり、観点 6「x86_64・aarch64 の
両ビルドホストで `docker build` が成功しアーキ分岐が両方で成功すること」は
aarch64 側も Pass と判定する。検証後、コンテナ（`ffds-450`）・イメージ
（`fandhe-frontend-dist-server:aarch64-450`）・抽出バイナリはいずれも削除し、
scratchpad 外への残置はない。

## 6. セキュリティ考慮事項（OWASP Top 10 観点）

`docs/design/docker-wasm-build-stage.md` §7 の設計方針を踏襲し、実装が同方針を
逸脱していないことを確認した。

- **A08 サプライチェーン**: `wasm-bindgen-cli` はバージョン固定
  （`0.2.126`、`Cargo.lock` の `wasm-bindgen` エントリと一致）+ GitHub
  Releases 固定 URL + SHA256 検証（x86_64/aarch64 双方、公式
  `.sha256sum` と照合済みの値）+ 対応外アーキのフェイルクローズ（`exit 1`）
  で導入されていることをコード上・実行結果（§4.2 の負例）の双方で確認。
  ベースイメージ digest 固定・`--locked` ビルド・明示 `COPY` 列挙は変更
  なし。`dist-server` の依存グラフは実測 `packages=21/60 depth=5/6
  result=PASS`（REQ-3 上限を弱めない）。
- **A05 セキュリティ設定ミス**: 最終イメージは `FROM scratch` + 非 root
  （UID 65532）+ バイナリ 1 つのみを維持（§4.3 で実機確認）。builder への
  `curl`・wasm ツールチェーン追加は最終イメージへ漏れないことを
  `docker export` で機械的に確認し、CI（`image-size.yml`）にも同等の
  検証ステップを追加した。
- **A03 XSS（REQ-1）**: 本タスクはビルド経路のみの変更であり、既定
  エスケープ・テキスト補間経路には関与しない。XSS 回帰テストの削除・
  弱体化は行っていない。
- **A01 パストラバーサル**: 未知パス（`/static/wasm/does-not-exist.wasm`）
  が 404 を返すことを実機確認した（§4.1）。コンパイル時埋め込みにより
  実行時 FS アクセスが発生しない性質は不変。
- **機微情報**: 本レポート・関連コミットにトークン・実在の内部 URL・
  シークレットを含まない。バージョン番号・チェックサムはいずれも一般公開
  されている OSS ツールのものである。

## 7. スコープ外事項（out-of-scope-tracking.md 準拠）

- **aarch64 実ホストでの `docker build` 実機ビルド**: イシュー #450 で実測
  済み（第 5a 節）。単発実測に留まり、CI ランナーが x86_64 のみのため
  aarch64 実機ビルドの CI 常設化は本イシューのスコープ外（新規 Issue 起票は
  ユーザー承認事項のため提案に留める）。この切り出しはイシュー #1216 で
  評価済みであり、`docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md`
  へ判断根拠・再評価トリガーを記録した（結論: 現時点では見送り）。
- **`build.rs` のキャッシュ・再ビルド制御の精緻化**: TASK-10.2c（#111、
  クローズ済み）のスコープ。
- **`wasm-bindgen --target web / nodejs` 出力使い分け DX**: 別イシュー
  （#161）のスコープ。
- `docs/spec/` は編集禁止（変更不要）。

## 8. 参照

- `docs/spec/04-requirements.md` REQ-10（132〜142 行目）
- `docs/spec/05-tasks.md` TASK-10.3（258〜263 行目）
- `docs/design/docker-wasm-build-stage.md`（TASK-10.3a 設計書、§3.1 実装方針・
  §6 検証観点の引き継ぎ元）
- `Dockerfile`（TASK-10.3b 実装）
- `.github/workflows/image-size.yml`（TASK-10.3c CI 検証ステップ追加）
- `dist-server/build.rs`（TASK-10.2b・#110、WASM ビルドステージ本体）
- `xtask/tests/wasm_bindgen_version_sync.rs`（TASK-10.3c 追加、固定
  バージョン・SHA256 同期ドリフトの回帰テスト）
- Issue #114（親・TASK-10.3）・#115（TASK-10.3a・設計）・#116
  （TASK-10.3b・実装）・#117（本レポート・TASK-10.3c）・#450（観点 6 の
  aarch64 実機実測、第 5a 節）
