# docs サイト生成器の外部リポジトリ利用ガイド

fandhe-frontend の docs サイト生成器（`docs-site`）は、fandhe-frontend 以外のリポジトリのドキュメントサイトを作るためにも使えます。本ガイドは、導入から `nav.toml` の書き方、ブランド表示のカスタマイズ、GitHub Pages への公開までの手順をまとめたものです。

足場の作成からデプロイ設定までを手作業ではなく自動化したい場合は、[setup-github-pages スキル](https://github.com/Fandhe-AI/agent-util-skills/tree/main/skills/setup-github-pages) を使えます。このスキルは、本ガイドの内容に沿った構成をリポジトリへ用意するためのものです。手順の中身を理解したい場合や、既存のサイトへ部分的に取り入れたい場合は、以下を順に読んでください。

設計判断の経緯は、[設計文書](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/docs-site-external-use.md) にあります。

## 導入

生成器は `crates/docs-site`（パッケージ名 `fandhe-frontend-docs-site`、バイナリ名 `docs-site`）です。crates.io には公開していないため、git から導入します。

```bash
cargo install --git https://github.com/Fandhe-AI/fandhe-frontend --rev <commit-sha> --locked fandhe-frontend-docs-site
```

- `--rev` には 40 桁の commit SHA を指定して固定してください。`main` などのブランチ名は動くため、再現性と供給網の観点から勧めません。
- `--locked` で依存の解決結果を固定します。
- 認証は不要です。private な submodule は取得されない設定になっています。
- 外部クレートへの依存はなく、必要なのは Rust stable の toolchain だけです。

コマンドラインは次のとおりです。

```text
docs-site --out <dir> [--root <dir>] [--no-page-sections] [--help]
```

- `--out`（必須）: 出力先ディレクトリ。
- `--root`: `site/nav.toml` を含むリポジトリルート。既定は `.` です。
- `--no-page-sections`: fandhe-frontend 本サイト専用の機能を止めます。
- `--help`（`-h`）: 使い方を標準出力へ出して、終了コード 0 で終わります（何も生成しません）。
- それ以外の未知の引数はエラーになり、使い方を標準エラーへ出して非 0 で終わります。

> [!IMPORTANT]
> fandhe-frontend 以外のサイトでは `--no-page-sections` を必ず付けてください。付けない場合は本サイト専用の生成節の登録表が使われ、登録されていないページを持つ `nav.toml` は、書き出しの前にエラーで失敗します。付けると、Themes・Primitives・Blocks・Wireframes のショーケースの注入と、その専用アセットの出力も止まります。

最小構成のサイトを生成すると、出力は次のとおりです。

- ページごとの `index.html`（例: `index.html`、`usage/index.html`）
- `404.html`
- `assets/` 配下の `site.css`、`skip-nav.css`、`site.js`、`theme-init.js`、`favicon.svg`、`search-index.json`、`search-index/<セクション>.json`

内部リンクが 1 件でも壊れていると、何も書き出さずに非 0 で終了します。

## nav.toml の書き方

`<root>/site/nav.toml` が、サイトの全ページの一覧・順序・URL を決めます。置き場所は固定です。

### 書式

読めるのは TOML のサブセットです。

- 使えるもの: `#` コメント、`[site]`、`[[section]]`、`[[section.page]]`、`[[section.group]]`、`[[section.group.page]]`、`[[menu]]`、`[[menu.item]]`、`key = "value"`。
- 値はダブルクォートの文字列だけです。エスケープは `\"` `\\` `\n` `\t` を使えます。
- 整数・真偽値・配列・inline table・複数行文字列は使えません。
- 未知のキーと重複したキーはエラーになります。サイズの上限は 1 MiB です。

### テーブルとキー

- `[site]`: `title` と `base_path` が必須です。`title` は、`brand` を指定しないときのフッターのブランド名になります。`base_path` は空文字か、`/` で始まり `/` で終わらない文字列です。GitHub Pages のプロジェクトサイトは `/<リポジトリ名>`、ユーザーサイトや独自ドメイン直下は空文字にします。
- `[[section]]`: `title` と `index_path`。`index_path` は、そのセクション配下に実在する `page.path` のどれかと完全に一致させます。ページもグループも持たないセクションは作れません。セクションは 1 つ以上必要です。
- `[[section.page]]` と `[[section.group.page]]`: `title`・`source`・`path`。`path` は `/` で始まり `/` で終わり、各セグメントは英数字・`-`・`_` だけで、サイト全体で一意にします。`source` は `--root` からの相対パスで、実在するファイルを指します。絶対パス・`..`・`\` は使えません。`site/` の外のファイルも指せます。
- `[[section.group]]`: `title` のみ。入れ子は 1 段までです。
- `[[menu]]`（`title`・`index_path`・`source`）と `[[menu.item]]`（`section`・`description`）: 複数のセクションをヘッダーの 1 項目と集約ページへ束ねる任意の機能です。`index_path` はどの `page.path` とも重複させません。

> [!WARNING]
> `path = "/"` のページは実質的に必須です。ヘッダーのブランドと 404 ページが `base_path` 直下へのリンクを持つため、トップのページが無いとリンク検査で失敗します。

### 最小構成の例

次の例は、そのままビルドできます。値は例示用のダミーです。

```toml
[site]
title = "my-project"
base_path = "/my-project"
brand = "my-project"
repository_url = "https://github.com/your-org/my-project"
tagline = "my-project のドキュメント"
copyright = "© 2026 your-org"
version_badge = ""
lang = "ja"
brand_mark = "m"
brand_color = "#0f766e"

[[section]]
title = "Docs"
index_path = "/"

[[section.page]]
title = "はじめに"
source = "site/index.md"
path = "/"

[[section.page]]
title = "使い方"
source = "site/usage.md"
path = "/usage/"
```

この `nav.toml` は `source` で 2 つの原稿を参照するため、ビルドの前に `<root>/site/` へ作成してください。存在しないと、ビルドは書き出しの前に失敗します。最小の原稿は次のとおりです（本文はダミーです）。

```markdown
# はじめに

my-project のドキュメントへようこそ。
```

上を `site/index.md` に、次を `site/usage.md` に保存します。

```markdown
# 使い方

ここに使い方を書きます。
```

```sh
mkdir -p site
printf '# はじめに\n\nmy-project のドキュメントへようこそ。\n' > site/index.md
printf '# 使い方\n\nここに使い方を書きます。\n' > site/usage.md
```

### 任意のファイル

- `site/redirects.toml`: `[[redirect]]` の `from` と `to` で、旧 URL の移転案内ページを作れます。
- `site/assets/`: 直下の通常ファイルが、出力の `assets/` へコピーされます。サブディレクトリとシンボリックリンクはエラーです。

## `[site]` のブランドキー

次の 8 キーは、すべて任意です。値が検証に通らないと、行番号付きのエラーでビルドが失敗します（値そのものはエラーに出ません）。値は既定のエスケープを通って出力されます。

| キー | 許容する値 | 未指定のとき | 反映先 |
|------|------------|--------------|--------|
| `brand` | 1〜64 文字。制御文字なし。空白のみは不可 | ヘッダーは `fandhe-frontend`、フッターは `title` | ヘッダーとフッターのブランド名 |
| `repository_url` | `https://` で始まる ASCII。空白とバックスラッシュ（`\`）なし。ホストあり。2048 バイト以下 | fandhe-frontend のリポジトリ | ヘッダーとフッターのリポジトリリンク（文言とアイコンはホストで決まります） |
| `tagline` | 1〜200 文字。制御文字なし。空白のみは不可 | fandhe-frontend の文言 | フッターのタグライン |
| `copyright` | `tagline` と同じ | fandhe-frontend の著作権表記 | フッター下段 |
| `version_badge` | 0〜32 文字。制御文字なし。空文字は非表示。空白のみは不可 | fandhe-frontend-core の版数 | ヘッダーのバッジ |
| `lang` | BCP 47 の形（`ja`、`en-US`、`zh-Hant-TW` など）。35 文字以下 | `ja` | `<html lang>`。生成器が出す固定のクローム文言の言語も選びます |
| `brand_mark` | ASCII 英数字ちょうど 1 文字 | 既定の図案 | `assets/favicon.svg` とヘッダーのマーク |
| `brand_color` | `#` と 16 進 6 桁 | `#3182ce` | マークのタイルの塗り色 |

注意点です。

- `brand` を指定しないと、ヘッダーのブランド名が `fandhe-frontend` のままになります。外部サイトでは必ず指定してください。
- `version_badge` を指定しないと、fandhe-frontend-core の版数が表示されます。外部サイトでは、空文字（非表示）か自前の文字列を指定してください。
- `lang` は `<html lang>` の属性に加えて、生成器が出す固定のクローム文言（検索ボタンと検索ダイアログのラベル・プレースホルダ、前後ページのリンク、404 ページ、リダイレクト案内）の言語も選びます。先頭のサブタグが `ja`（大文字小文字は区別しません。未指定の既定も `ja`）なら日本語、それ以外（`en`、`en-US`、`fr` など）は英語です。選べる言語は日本語と英語の 2 つで、文言を自由に差し替えるキーはありません。`tagline` と `copyright` を指定しない場合の既定文言は日本語のままなので、英語のサイトでは両方を指定してください。
- `repository_url` のホストが `github.com`（`www.github.com` を含む。大文字小文字は区別しません）のときは、リンクの文言が "GitHub" で、GitHub のマークが付きます。それ以外のホスト（GitLab や自前のサーバーなど）では、文言が "Repository" で、特定のサービスを示さない汎用のアイコンになります。`repository_url` にバックスラッシュは使えません（ブラウザが `/` として解釈し、ホスト判定と実際のリンク先がずれるため。検証の追加です）。ホストは、ユーザー名やポート番号を除いて判定します（`https://github.com@example.com/` のホストは `example.com` です）。
- `brand_color` と白い文字のコントラストは検証されません。読みやすい色を選んでください。
- 画像ファイルをロゴに使うキーはありません。
- ページの `<title>` は、各ページの `title` だけです。サイト名は付きません。

## 予約パスと予約アセット名

### 予約パス

次のパスは、`--no-page-sections` を付けないビルドで、fandhe-frontend のショーケースが差し込まれる接頭辞です。

- `/themes/<部品名>/`
- `/primitives/<部品名>/`
- `/blocks/<id>/`
- `/wireframes/<名前>/`

`--no-page-sections` を付ければ注入されず、通常のページとして使えます。付け忘れた場合に備えて、これらの接頭辞は避けるのが無難です。

### 予約アセット名

`site/assets/` の直下に次の名前のファイルを置くと、生成物と衝突するため、ビルドエラーになります。

```text
site.css
site-primitives.css
skip-nav.css
pre-styled-ui.css
primitives-showcase.css
admonition.css
site.js
theme-init.js
favicon.svg
search-index.json
image-demo.svg
blocks.css
wireframes.css
blocks-demo-product.svg
blocks-demo-avatar.svg
blocks-demo-logo.svg
blocks-demo-screenshot.svg
blocks-demo-background.svg
```

`index_path = "/assets/"` の `[[menu]]` を持つサイトでは、`site/assets/index.html` も置けません。

## Markdown の対応範囲

原稿は、次の範囲の Markdown を描画できます。

| 使えるもの | 補足 |
|------------|------|
| 見出し `#` から `######` | `id` が付き、ページ内リンクの宛先にできます |
| 段落・リスト（入れ子可）・表・引用 | |
| フェンスコード | 色分けは `rust` `toml` `html` のみ。他の言語は色なしで表示されます |
| インラインコード・`*強調*`・`**太字**` | |
| リンク | `http` / `https` / 相対のみ |
| admonition | 引用の 1 行目に、単独で `[!NOTE]` `[!TIP]` `[!IMPORTANT]` `[!WARNING]` `[!CAUTION]`（大文字） |

使えないものは次のとおりです。

- 画像（`!` とリンクとして描画されます）。
- `_強調_`。
- 生 HTML と HTML コメント（エスケープされ、文字としてそのまま表示されます）。
- 自動リンク、参照形式リンク、`mailto:` などのスキーム。

### リンクの検査

- nav に登録した `.md` ファイルへの相対リンクは、公開 URL へ自動で書き換わります。
- nav に無い `.md` へのリンク、存在しない `#anchor`、存在しない絶対パスはビルドエラーです。絶対パスは `base_path` を含めて書きます。
- 外部 URL は文字列として出力するだけで、到達確認はしません。

## 帰属表記とライセンス

docs-site は `MIT OR Apache-2.0` で提供されています。ライセンス本文は、fandhe-frontend のリポジトリの [LICENSE-MIT](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-MIT) と [LICENSE-APACHE](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-APACHE) にあります。

フッター下段の表記は、次のように切り替わります。

- `lang` 以外のブランドキー（`brand` `repository_url` `tagline` `copyright` `version_badge` `brand_mark` `brand_color`）を 1 つでも指定すると、帰属表記 `Built with fandhe-frontend docs-site (MIT OR Apache-2.0)` になります。"fandhe-frontend docs-site" は fandhe-frontend のリポジトリへ、MIT と Apache-2.0 はそれぞれのライセンス本文へリンクします。
- 7 つとも未指定のときは、`Licensed under MIT OR Apache-2.0` になります。リンク先は同じ 2 つのライセンス本文です。
- これらのリンク先は、`repository_url` の値に関係なく、fandhe-frontend を指します。

帰属表記を消す、または差し替える設定キーはありません。`[site]` は未知のキーを拒否するため、そのようなキーを書くとビルドエラーになります。生成後の HTML から帰属表記を取り除く後処理も行わないでください。

利用者自身のコンテンツのライセンスは、この表記の対象外です。`copyright` や本文で、利用者が別に示してください。ライセンスの扱いの詳細は、上記のライセンス本文を確認してください。

## GitHub Pages へのデプロイ

### 前提

リポジトリの Settings の Pages で、Source を "GitHub Actions" にします。

### ワークフローの流れ

1. checkout する。submodule は不要で、`persist-credentials: false` を指定する。
2. Rust stable の toolchain を用意する。
3. 上記の `cargo install` で `docs-site` を導入する。
4. `docs-site --out "${RUNNER_TEMP}/dist" --no-page-sections` で生成する。
5. 生成物を Pages の成果物として渡す。
6. deploy ジョブで公開する。

権限は最小にします。ワークフロー全体は `contents: read` とし、deploy ジョブにだけ `pages: write` と `id-token: write` を付けます。サードパーティの action は commit SHA で固定してください。

action の SHA やバージョンは、すぐに古くなるため本ガイドには書きません。実例は、本リポジトリの [docs-site ワークフロー](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/.github/workflows/docs-site.yml) を参照してください。このワークフローは、公開リポジトリ `Fandhe-AI/actions` の再利用ワークフロー `pages-deploy.yml` を呼んでいます。再利用ワークフローの既定のランナーは self-hosted のため、`runner-label: ubuntu-latest` を明示しないと、ジョブが待機したまま止まります。

### ローカルでの確認

生成された HTML は、`base_path` 付きの絶対パスでアセットを参照します。`file://` で開くとスタイルが崩れるため、HTTP サーバーで配信してください。

1. 出力を `<任意のディレクトリ>/<base_path>/` に置く（例: `base_path = "/my-project"` なら `preview/my-project/`）。
2. 親ディレクトリを `python3 -m http.server --bind 127.0.0.1 --directory <親ディレクトリ>` で配信する。
3. ブラウザで `http://127.0.0.1:8000/<base_path>/` を開く。

本リポジトリの `make docs-preview` も同じ方式です。

## 関連

- [デプロイガイド](./deployment.md): fandhe-frontend で作るアプリのデプロイ方法（Vercel など）。
- [設計文書](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/docs-site-external-use.md): `[site]` の拡張と帰属表記の決定記録。
- [setup-github-pages スキル](https://github.com/Fandhe-AI/agent-util-skills/tree/main/skills/setup-github-pages): 足場の作成からデプロイ設定までを自動化するスキル。
