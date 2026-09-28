# examples/vercel-ssr

## 概要

`fandhe-frontend` フレームワークの Vercel Container Images（Beta）上で
リクエスト時 SSR を行う正本サンプルです（イシュー #3289）。
`docs/design/vercel-deployment-strategy.md` が確定した案 d（Container
Images）の実演であり、静的配置（案 c、
[`examples/vercel-ssg`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/examples/vercel-ssg/README.md)）
では賄えないリクエストごとの描画を扱います。crates.io へ公開済みの
`fandhe-frontend-dist-server`（v0.3.4。`PORT` 対応〔イシュー #3336〕・
graceful shutdown 対応〔イシュー #3337〕の両方を含む）を通常の外部依存として
使います。

## 学べること

- Vercel Container Images が要求する 2 点（`PORT` 環境変数でポートを受け取り
  全インターフェースで listen する・スケールイン時の `SIGTERM` を受けて
  自身で終了処理する）を満たす bind 先解決 + graceful shutdown の実装
  （`fandhe-frontend-dist-server` v0.3.4 の `src/main.rs` からの移植）
- bind 先アドレスの優先順位: `FANDHE_FRONTEND_BIND_ADDR` > `PORT`
  （`0.0.0.0:$PORT`） > 既定 `127.0.0.1:3100`。`PORT` の値は数値
  `1..=65535` 以外だと fail-closed に起動失敗し、実際の不正値はエラー
  メッセージへ含めません（機微情報を露出しない、`security.md` A09）
- graceful shutdown: `SIGTERM`/`SIGINT` 受信後、accept ループを抜けて
  listener を drop（以後の新規接続は即座に拒否）し、処理中の接続を最大
  25 秒（Vercel の 30 秒猶予より短い）待って `exit 0` する
- `Dockerfile.vercel` が `FANDHE_FRONTEND_BIND_ADDR` を**設定しない**設計判断
  （下記「Vercel 設定」参照）

## 制約

- crates.io からの外部依存利用時、`fandhe-frontend-dist-server` の
  `assets::lookup`／`build.rs` はこのプロジェクト直下の `static/` を
  解決しないため、本サンプルは静的アセットの自前配信を一切持たず、
  `/static/*` は常に 404 になります（
  [`examples/dist-server-docker`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/examples/dist-server-docker/README.md)
  の実測と同じ制約）
- 同じ理由で WASM（ハイドレーション・クライアント側の対話部品）も一切
  出荷されません。クライアント側の対話部品が必要な場合は
  `fandhe-frontend-wasm-full` を自アプリの直接依存として追加してください
  （[`examples/interactive-view-transitions`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/examples/interactive-view-transitions/README.md)
  参照）。本サンプルは SSR のみに絞った最小構成です
- ビルド時に内容が確定する静的ページ（ブログ・ドキュメント等）で十分な
  場合は、Beta 依存のない
  [`examples/vercel-ssg`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/examples/vercel-ssg/README.md)（案 c）を推奨します

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への到達性
  （依存解決に使用します）
- `fw gate --project examples/vercel-ssr` を実行する場合は clippy
  component / cargo-deny が必要です（
  `tools/ci/ensure-gate-tools.sh` で導入できます）
- Docker イメージのビルド・起動を試す場合は Docker（`docker build`/`docker run`）
- Vercel へデプロイする場合は Vercel CLI（`vercel`）とアカウント。
  **Container Images は Beta 機能であり、チームで有効化が必要な場合があります**
  （公式ドキュメント参照、下記「Vercel へのデプロイ」）

## Vercel 設定（必須）

Vercel プロジェクトの環境変数 `PORT` に **1024 以上**（例 `3100`）を
設定してください。

理由は次の 2 点です。

1. `Dockerfile.vercel` はイメージを非 root（`USER 65532:65532`）で実行します。
   非 root プロセスは 1024 未満のポート（Vercel Container Images の既定
   `80`）に bind できません
2. Vercel Container Images の Port resolution は「既定ポートは `80`。
   プロジェクト設定で `PORT` 環境変数を設定すれば、その値へ上書きできる」
   契約であり、Vercel 側は上書き後の `PORT` 値へ接続します（
   `docs/design/vercel-deployment-strategy.md` §2 で実測・引用確認済み、
   2026-09-27 時点の公式ドキュメント
   <https://vercel.com/docs/functions/container-images> `last_updated:
   2026-07-07`）

`Dockerfile.vercel` が `FANDHE_FRONTEND_BIND_ADDR` を設定していないのは、
設定すると `PORT` より優先されてしまい、Vercel 側でどれだけ `PORT` を
上書きしても効かなくなるためです（`src/main.rs` の bind 先優先順位、
`Dockerfile.vercel` のコメント参照）。

## Vercel へのデプロイ

```bash
# 0. サンプルディレクトリへ移動する
cd examples/vercel-ssr

# 1. プロジェクトを Vercel と紐付ける
vercel link

# 2. デプロイする（`vercel deploy` の build step が Dockerfile.vercel を
#    自動検出してコンテナイメージをビルドし、Vercel Container Registry
#    （VCR）へ格納してからデプロイする。ビルドはこの build step 内で
#    行われ、事前にローカルで `docker build`/`docker push` する必要は
#    ない。プロジェクト設定で PORT を 1024 以上に設定していることを
#    確認してから実行する）
vercel deploy
# 本番デプロイの場合は --prod を付ける
vercel deploy --prod
```

`--prebuilt` は付けません（`.vercel/output/` の事前ビルド成果物を使う
Build Output API 経路〔案 c、`examples/vercel-ssg` 参照〕向けのオプションで、
`Dockerfile.vercel` の build step とは無関係です）。

Vercel CLI の Container Images 固有の挙動（デプロイ経路・イメージの
ビルド主体等）は公式ドキュメント
<https://vercel.com/docs/functions/container-images>・
<https://vercel.com/docs/container-registry> に従ってください
（2026-09-28 時点で `last_updated: 2026-07-07`・`2026-09-04`、Beta のため
今後変わり得ます）。

### Deployment Protection（既定で有効）

Vercel のデプロイは既定で Deployment Protection（Vercel Authentication）が
有効で、未認証アクセスは SSO 認証ページへの `302` リダイレクトになります。
無効化・Basic 認証（Protection Bypass for Automation）による迂回の手順は
[`docs/guides/deployment.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/guides/deployment.md)
の「Deployment Protection（既定で有効）」節を参照してください。

Protection Bypass のシークレット・トークンはリポジトリへコミットしない
でください（`.gitignore` の `/.env`、`security.md`「秘密情報の混入防止」）。

## ローカル確認

```bash
# ネイティブ起動（既定ループバック bind、PORT を渡すと 0.0.0.0:$PORT で listen する）
cargo run
# 別シェルで:
curl -sS http://127.0.0.1:3100/
curl -sSI http://127.0.0.1:3100/no-such-page   # 404

# PORT 経由の起動（Vercel Container Images と同じ経路）
PORT=3100 cargo run
# こちらは 0.0.0.0:3100 で listen する点に注意（外部到達可能なアドレス）

# テスト（実プロセス起動 + GET / ・404・不正 PORT の fail-closed・
# SIGTERM による graceful shutdown の検証）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/vercel-ssr
```

Docker イメージのビルド・起動（受け入れ条件。CI での自動検証は本サンプルの
スコープ外、PR 本文の後続 Issue 提案を参照）:

```bash
docker build -f Dockerfile.vercel -t vercel-ssr-example .
docker run --rm -e PORT=3100 -p 3100:3100 vercel-ssr-example
# 別シェルで:
curl -sS http://127.0.0.1:3100/
curl -sSI http://127.0.0.1:3100/no-such-page   # 404

# 停止すると SIGTERM を受けて graceful shutdown する（stderr に drain の
# ログが出て、正常終了コードで終わる）
docker stop <container-id>
```

## 未検証事項

- Vercel 実機でのデプロイ確認は本サンプル追加時には行っていません
  （人手チェックリストは後続 Issue #3339 を参照）
- `docs/design/vercel-deployment-strategy.md` §「未確定事項」（イメージ
  サイズ上限〔250MB〕・実行時アーキテクチャ・読み取り専用ファイル
  システムの有無・ヘルスチェック契約・コールドスタート特性）は本サンプル
  追加時点でも未確定のままです。詳細は同文書を参照してください

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | `fandhe-frontend-dist-server` + hyper 系クレートへの crates.io バージョン依存。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | 薄い hyper トランスポート層 + bind 先解決 + graceful shutdown |
| `tests/boot.rs` | 実プロセス起動検証（GET / ・404・不正 PORT の fail-closed） |
| `tests/graceful_shutdown.rs` | SIGTERM による graceful shutdown の実プロセス検証（`#[cfg(unix)]`） |
| `Dockerfile.vercel` | musl 静的リンク → `FROM scratch` のマルチステージビルド（Vercel Container Images 向け、`FANDHE_FRONTEND_BIND_ADDR` を設定しない） |
| `.dockerignore` | ビルドコンテキストからの除外（`examples/dist-server-docker` 版を流用） |

## 採らなかった案

`Dockerfile.vercel` 内で `cargo install fandhe-frontend-dist-server@0.3.4` を
実行し、公開バイナリをそのまま動かす方式も検討しましたが、採用しませんでした。
理由は次の 2 点です。

1. コード重複は無くなりますが、`examples/vercel-ssr` の `src/main.rs`（学習
   目的で読まれる実体）とデプロイされる実体が別物になり、学習用サンプル
   として紛らわしくなります
2. `fw gate` の `test` チェックで起動・PORT 検証・シグナル挙動を検証できなく
   なります（`cargo install` されたバイナリはプロジェクト内テストの対象に
   なりません）

## 関連ガイド

- [`docs/guides/quickstart.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/guides/quickstart.md)
- [`docs/guides/deployment.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/guides/deployment.md)
- [`docs/design/vercel-deployment-strategy.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/vercel-deployment-strategy.md)
- [`docs/design/dist-server-design.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/dist-server-design.md)
- [`examples/vercel-ssg/README.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/examples/vercel-ssg/README.md)（案 c、静的配置の既定方式）
- [`examples/dist-server-docker/README.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/examples/dist-server-docker/README.md)（Vercel 以外のコンテナ基盤向け単一バイナリ配布）
