# examples/

`fandhe-frontend` フレームワークの動くサンプル集です。いずれも crates.io へ
公開済みのクレートへのバージョン依存のみで完結する、独立した
cargo プロジェクトです（`templates/`「`fw new` が展開する生成の雛形」とは
異なり、本ディレクトリは「実際に動く正本」です）。

docs サイトで読みやすく整形されたバージョンは
[docs/guides/examples.md](../docs/guides/examples.md)（公開後は
https://fandhe-ai.github.io/fandhe-frontend/examples/ ）でも閲覧できます。

## 1. サンプル一覧

| サンプル | 学べること | 主要クレート |
|---|---|---|
| [ssr-routing](./ssr-routing/README.md) | `Loader` trait の自作実装・`respond_with` による SSR 応答組み立て・`Router` によるパスパラメータ処理・既定エスケープ（REQ-1） | `fandhe-frontend-core` / `-app` / `-server` |
| [ssg-blog](./ssg-blog/README.md) | `generate_pages` による静的サイト書き出し・パス検証の fail-closed 契約・View Transitions の有効化 | `fandhe-frontend-core` / `-server` |
| [dist-server-docker](./dist-server-docker/README.md) | 単一バイナリ配布・`FROM scratch` の Docker イメージ最小化・外部依存利用時の静的アセット配信の制約と対処 | `fandhe-frontend-dist-server` |
| [interactive-view-transitions](./interactive-view-transitions/README.md) | `Component` trait による状態機械・`dispatch`/`hydrate`・`start_router` による SPA 内 View Transitions の自動有効化 | `fandhe-frontend-core` / `-app` / `-interactive`（+ `-wasm-full`） |
| [headless-pre-styled-ui](./headless-pre-styled-ui/README.md) | `fandhe-frontend-headless-ui`（ark-ui 相当）の anatomy・`data-*`・WAI-ARIA 属性（Tabs/Accordion/Dialog/Switch/RadioGroup/Avatar） | `fandhe-frontend-core` / `-pre-styled-ui`（headless 層は再エクスポート経由） |

## 2. 選び方

- **まずサーバーサイド描画を試したい**: [ssr-routing](./ssr-routing/README.md) から始めてください。SSR の基本要素（`Loader`・`respond_with`・`Router`）が揃っています。
- **静的サイト生成（ブログ等）を作りたい**: [ssg-blog](./ssg-blog/README.md) が `generate_pages` と fail-closed なパス検証を実演します。
- **単一バイナリ・Docker でデプロイしたい**: [dist-server-docker](./dist-server-docker/README.md) が `fandhe-frontend-dist-server` を使った最小構成の配布サーバーを示します。
- **クライアント側の状態管理・ページ遷移アニメーションを試したい**: [interactive-view-transitions](./interactive-view-transitions/README.md) が `fandhe-frontend-interactive` の状態機械と View Transitions を実演します。
- **headless UI コンポーネント（ark-ui 相当）を試したい**: [headless-pre-styled-ui](./headless-pre-styled-ui/README.md) が `fandhe-frontend-headless-ui` の Tabs/Accordion/Dialog/Switch/RadioGroup/Avatar を実演します。

## 3. `fw new --example` での取得

`fw` CLI（`fandhe-frontend-cli`）の `--example` オプションで、各サンプルを
自分のプロジェクトとして展開できます。

```bash
cargo install fandhe-frontend-cli
fw new my-app --example ssr-routing
```

`--example` に指定できる名前は `ssr-routing` / `ssg-blog` /
`dist-server-docker` / `interactive-view-transitions` /
`headless-pre-styled-ui` の 5 種類です
（本ディレクトリ直下のディレクトリ名と一致します）。展開されたプロジェクト
は本ディレクトリ配下の該当サンプルと全ファイルバイト一致（パッケージ名の
置換は行いません）で、そのまま `cargo build` / `cargo test` / `fw gate
--project .` が通る状態です。

`fw` の導入手順は [docs/guides/quickstart.md](../docs/guides/quickstart.md)
を参照してください。

## 4. 共通規約

各サンプルは以下の構成に従います（`examples/ssr-routing` が初例、
イシュー #499）。

- **crates.io バージョン依存のみ**: `fandhe-frontend-*` クレートは vendor
  同梱せず、通常の crates.io バージョン依存として `Cargo.toml` に宣言する
- **独立ワークスペース**: root workspace から独立した
  `[workspace] members = ["."]`（サンプル単体で `cargo build`/`cargo test`
  が完結し、リポジトリ全体のビルドに影響しない）
- **`fw gate` 対応**: `structure.toml` / `clippy.toml` / `deny.toml` を
  同梱し、`fw gate --project examples/<name>` がそのまま通る
- **既定エスケープ（REQ-1）**: ノード木 API のみで HTML を組み立て、
  `raw_html()` や `format!` による HTML 文字列の直接組み立てを行わない
- **README の節構成**: 「概要 / 学べること / 前提 / 動かし方 / 主要ファイル /
  関連ガイド」（本サンプル固有の補足節があれば追加してよい）

## 5. 正本と同梱コピー

`crates/cli/embedded-examples/` は `fw new --example` の
`cargo package`/`cargo publish` 制約（クレートディレクトリ外ファイルの
同梱禁止）に対応するための、本ディレクトリのバイト単位同梱コピーです。
**正本は本ディレクトリ（`examples/`）のまま**であり、同梱コピー側を直接
編集しないでください。乖離は
`crates/cli/tests/example_publish_copy_drift.rs` が検知します。

## 6. 関連ドキュメント

- [docs/guides/quickstart.md](../docs/guides/quickstart.md): `fw` 導入から
  プロジェクト作成までの入門ガイド
- [docs/guides/examples.md](../docs/guides/examples.md): 本ページの
  docs サイト版（選び方の解説つき）

## 7. 経緯: `headless-pre-styled-ui` の path 依存解消（イシュー #552/#609）

`headless-pre-styled-ui` は作成当初（イシュー #552）、依存する
`fandhe-frontend-headless-ui` が crates.io 未公開だったため、§4「crates.io
バージョン依存のみ」の原則に対する意図的な例外として path 依存を使い、§3 の
`fw new --example` にも非対応でした。前提クレート公開（イシュー #608）を
受けてイシュー #609 でバージョン依存へ切り替え、`fw new --example
headless-pre-styled-ui`・`crates/cli/embedded-examples/`（§5）への同梱に
対応済みです。他の 4 サンプルと同じ規約に従います。
