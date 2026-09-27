# examples/vercel-ssg

## 概要

`fandhe-frontend` フレームワークの Vercel 向け正本サンプルです
（イシュー #3290）。親ツリー #3282 の Phase 1（#3284〜#3286、#3288）で、
Vercel 上での既定方式は SSG → Vercel Build Output API → `vercel deploy
--prebuilt`（案 c）に確定しました（`vercel_runtime` 1.x は起動時に panic、
2.x は依存グラフ上限〔60 件/深さ 6〕を超えるため不採用。詳細な決定記録は
本リポジトリの `docs/design/vercel-deployment-strategy.md` を参照してくだ
さい）。本サンプルはこの唯一の既定方式を動く形で示します。**Vercel 側に
Rust ツールチェーンは不要**で、ローカルまたは CI でビルドした静的出力を
配置するだけです。

`examples/ssg-blog`（イシュー #501）と同じく crates.io へ公開済みの
`fandhe-frontend-core` / `fandhe-frontend-server`（v0.4.3/v0.2.6）を
バージョン依存として実際に使う「正本」です。

## 学べること

- `fandhe_frontend_server::ssg::generate_pages` による Build Output API
  `static/` ディレクトリ（HTML ページ群）の生成
- `fandhe_frontend_server::ssg::generate_assets` による `config.json`
  （Build Output API v3）と `404.html` の生成
- `config.json` の `routes` で「ファイルシステムに一致しなければ 404」を
  表現する方法（`{"handle": "filesystem"}` の後続ルート）
- 古いビルド成果物を残したまま `--prebuilt` すると意図しない内容が
  公開されてしまうため、生成前に固定リテラルの出力先（`.vercel/output`）
  だけを削除する fail-closed な取り扱い（`.vercel/project.json` 等の
  `vercel link` 由来ファイルは削除しません）
- 既定エスケープ（REQ-1）: ページ本文はすべて `text()` 経由でノード木へ
  載せ、`raw_html()` や HTML 文字列の直接組み立ては使いません

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への
  到達性（依存解決に使用します）
- `fw gate --project examples/vercel-ssg` を実行する場合は clippy
  component / cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh` で
  導入できます）
- デプロイを試す場合のみ: [Vercel CLI](https://vercel.com/docs/cli) と
  Vercel アカウント

## 動かし方

```bash
# .vercel/output/ へ Build Output API 形式の静的サイトを生成
cargo run --release

# 生成結果をローカルで確認（任意）
python3 -m http.server -d .vercel/output/static 8000
# ローカルの簡易サーバーでは config.json の routes（404 フォールバック等）は
# 効きません。routes の動作確認は Vercel 上のデプロイ（下記）で行ってください。

# テスト（既定エスケープ回帰・fail-closed 回帰・古い出力の削除を含む）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/vercel-ssg
```

`cargo run --release` の実行後、次のツリーが生成されます。

```
.vercel/output/
├── config.json              # generate_assets（Build Output API v3）
└── static/
    ├── index.html           # generate_pages
    ├── pages/
    │   ├── about/index.html
    │   └── default-escaping/index.html
    └── 404.html             # generate_assets
```

## Vercel へのデプロイ（`--prebuilt`）

1. Vercel CLI をインストールします（グローバル導入になるため、`npm i -g`
   の実行前に導入元のスクリプトを信頼できるか確認してください）。

   ```bash
   npm i -g vercel
   ```

2. プロジェクトを Vercel と紐付けます（`.vercel/project.json` が作られ、
   本ディレクトリの `.gitignore` で除外済みです）。

   ```bash
   vercel link
   ```

3. 静的出力を生成します（Vercel 側では実行されないため、ローカルまたは
   CI で事前に実行しておく必要があります）。

   ```bash
   cargo run --release
   ```

4. ビルド済み出力をそのままデプロイします（Vercel は `.vercel/output/`
   をアップロードするだけで、Rust のビルドは行いません）。

   ```bash
   # プレビューデプロイ
   vercel deploy --prebuilt

   # 本番デプロイ
   vercel deploy --prebuilt --prod
   ```

Basic 認証によるアクセス制御（fail-closed な設定手順）は
[デプロイガイド](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/guides/deployment.md)を参照してください。

## Deployment Protection（SSO）の注意

新規に作成した Vercel プロジェクトでは、既定で Vercel Authentication
（SSO によるデプロイ保護）が有効になっています。有効なままだと、未認証の
アクセスはすべて Vercel のログイン画面へ 302 リダイレクトされ、本サンプル
の `config.json` が定義する 404 フォールバックへは到達しません。

- デプロイ内容を一般公開したい場合は、Vercel の Project Settings →
  Deployment Protection で無効化してください。
- 非公開のまま検証だけしたい場合は、有効のままにしておいて構いません。
- バイパス用トークン等の秘密情報は、リポジトリにも README にも書かない
  でください。

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | crates.io バージョン依存 2 件のみ（`fandhe-frontend-core` / `-server`）。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | Build Output API 生成エントリ（`layout` / `build_pages` / `not_found_page` / `clean_output_dir` + `generate_pages`/`generate_assets` 呼び出し） |
| `tests/build_output.rs` | 既定エスケープ・fail-closed・古い出力削除・シンボリックリンク拒否の回帰と CLI ブラックボックステスト |

## 関連ガイド

- [`docs/guides/quickstart.md`](../../docs/guides/quickstart.md)
- [`docs/guides/examples.md`](../../docs/guides/examples.md)
- [`docs/api/server-api.md`](../../docs/api/server-api.md)
- [`examples/ssg-blog/README.md`](../ssg-blog/README.md)

デプロイ方式の決定根拠は `docs/design/vercel-deployment-strategy.md`
（docs サイト非掲載のため上記はリンクにせずプレーンテキストで参照して
います）を参照してください。
