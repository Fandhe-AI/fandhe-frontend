# Docker マルチステージビルド内 WASM 再ビルド 受け入れ基準検証レポート（TASK-10.3c）

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

`docs/design/docker-wasm-build-stage.md` §6 の検証観点 1〜5 をワークツリー環境で
実測し、いずれも Pass を確認した。観点 6（aarch64 実ビルド）は本イシュー
実装時点で利用可能な CI ランナー・ローカル環境がいずれも x86_64 のため実測
不能であり、archive 実在・公式チェックサム照合済みである事実（TASK-10.3a
設計時点の確認）と、aarch64 実ホストでの手動確認手順（第 6 節）の記載を
もって代替する。

## 3. 検証結果一覧

| # | 検証観点（§6 引用） | 検証手段 | 結果 |
|---|---------------------|---------|------|
| 1 | `RWS_WASM_BUILD=0` 削除後、`docker build` が成功し、最終イメージの `static/wasm/` 相当の URL パスから WASM 成果物が配信されること | `docker build` → `docker run` → `curl` 実機確認 | Pass（第 4.1 節） |
| 2 | `wasm-bindgen-cli` バージョン不一致時に、`build.rs` のフェイルクローズがビルダーステージのビルド失敗として正しく伝播すること | `WASM_BINDGEN_VERSION` を意図的に不一致値へ変更した負例ビルド＋`xtask/tests/wasm_bindgen_version_sync.rs` による固定値同期ドリフトの回帰検出 | Pass（第 4.2 節） |
| 3 | 最終イメージに `wasm32-unknown-unknown` ターゲット・`wasm-bindgen-cli` バイナリが含まれないこと | `docker export \| tar -t` によるファイル一覧の全数確認 | Pass（第 4.3 節） |
| 4 | `xtask check-image-size`（50MB 上限）が WASM 成果物込みでも PASS すること | `cargo run -p xtask -- check-image-size` | Pass（第 4.4 節） |
| 5 | `image-size.yml` の `paths` 追加後、`wasm-full/`・`wasm-thin/`・`interactive/` の変更が正しくワークフローをトリガーすること | `.github/workflows/image-size.yml` の `paths` 実測確認 | Pass（第 4.5 節、追加は TASK-9.3b 以降の先行変更で既に存在） |
| 6 | `docker build` を x86_64・aarch64 の両ビルドホストで実行し、アーキ分岐が両方で成功すること | x86_64: 実機確認。aarch64: 実測不能、手動確認手順を記載 | x86_64 Pass / aarch64 未実測（第 6 節に手順記載） |

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
- **aarch64**: 本イシュー実装時点で利用可能な CI ランナー・ローカル環境が
  いずれも x86_64 のため実機ビルドは実測不能。TASK-10.3a（#115）設計時点で
  `wasm-bindgen-0.2.126-aarch64-unknown-linux-musl.tar.gz` の実在と
  チェックサム（`22451202...440a`）を GitHub Releases の公式
  `.sha256sum` と照合済みであることを踏まえ、以下を Apple Silicon 等の
  aarch64 実ホストでの手動確認手順として記載する（誇張しない: 本レポートは
  archive 実在の事実確認に留まり、aarch64 実ビルドの成功を主張しない）。

## 5. aarch64 実ホストでの手動確認手順（未実測・参考）

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

- **aarch64 実ホストでの `docker build` 実機ビルド**: 本レポート第 5 節に
  手動確認手順を記載。CI ランナーが x86_64 のみのため自動化は本タスクの
  スコープ外（新規 Issue 起票はユーザー承認事項のため提案に留める）。
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
  （TASK-10.3b・実装）・#117（本レポート・TASK-10.3c）
