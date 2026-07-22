# サンプル集（examples/）

`fandhe-frontend` フレームワークの動くサンプルは、リポジトリルート
`examples/` 配下に「正本」として置かれています。いずれも crates.io へ
公開済みのクレート（v0.1.0）へのバージョン依存のみで完結する、独立した
cargo プロジェクトです。本ページはサンプルの一覧と選び方、`fw new
--example` による取得手順をまとめます。

## 1. サンプル一覧

| サンプル | 学べること | 主要クレート |
|---|---|---|
| [ssr-routing](../../examples/ssr-routing/README.md) | `Loader` trait の自作実装・`respond_with` による SSR 応答組み立て・`Router` によるパスパラメータ処理・既定エスケープ（REQ-1） | `fandhe-frontend-core` / `-app` / `-server` |
| [ssg-blog](../../examples/ssg-blog/README.md) | `generate_pages` による静的サイト書き出し・パス検証の fail-closed 契約・View Transitions の有効化 | `fandhe-frontend-core` / `-server` |
| [dist-server-docker](../../examples/dist-server-docker/README.md) | 単一バイナリ配布・`FROM scratch` の Docker イメージ最小化・外部依存利用時の静的アセット配信の制約と対処 | `fandhe-frontend-dist-server` |
| [interactive-view-transitions](../../examples/interactive-view-transitions/README.md) | `Component` trait による状態機械・`dispatch`/`hydrate`・`start_router` による SPA 内 View Transitions の自動有効化 | `fandhe-frontend-core` / `-app` / `-interactive`（+ `-wasm-full`） |
| [headless-pre-styled-ui](../../examples/headless-pre-styled-ui/README.md) | `fandhe-frontend-headless-ui`（ark-ui 相当）の anatomy・`data-*`・WAI-ARIA 属性（Tabs/Accordion/Dialog/Switch/RadioGroup/Avatar） | `fandhe-frontend-core` / `-headless-ui` |

## 2. 選び方

- **まずサーバーサイド描画を試したい**: [ssr-routing](../../examples/ssr-routing/README.md) から始めてください。`Loader`・`respond_with`・`Router` という SSR の基本要素が揃っています。
- **静的サイト生成（ブログ等）を作りたい**: [ssg-blog](../../examples/ssg-blog/README.md) が `generate_pages` の使い方と fail-closed なパス検証を実演します。
- **単一バイナリ・Docker でデプロイしたい**: [dist-server-docker](../../examples/dist-server-docker/README.md) が `fandhe-frontend-dist-server` を使った最小構成の配布サーバーを示します。
- **クライアント側の状態管理・ページ遷移アニメーションを試したい**: [interactive-view-transitions](../../examples/interactive-view-transitions/README.md) が `fandhe-frontend-interactive` の状態機械と View Transitions を実演します。
- **headless UI コンポーネント（ark-ui 相当）を試したい**: [headless-pre-styled-ui](../../examples/headless-pre-styled-ui/README.md) が `fandhe-frontend-headless-ui` の Tabs/Accordion/Dialog/Switch/RadioGroup/Avatar を実演します。

## 3. `fw new --example` での取得

各サンプルは `fw` CLI（`fandhe-frontend-cli`）の `--example` オプションで
自分のプロジェクトとして展開できます。`fw` の導入手順は
[クイックスタート](./quickstart.md) を参照してください。

```bash
cargo install fandhe-frontend-cli
fw new my-app --example ssr-routing
```

`--example` に指定できるサンプル名は `ssr-routing` / `ssg-blog` /
`dist-server-docker` / `interactive-view-transitions` /
`headless-pre-styled-ui` の 5 種類です。展開
されたプロジェクトはリポジトリの `examples/` 配下と全ファイルバイト一致
（パッケージ名の置換は行いません）で、そのまま `cargo build` / `cargo
test` / `fw gate --project .` が通る状態です。

## 4. 共通規約

各サンプルは以下の構成に従います（詳細は [`examples/README.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/examples/README.md) を参照）。

- root workspace から独立した `[workspace] members = ["."]`（サンプル単体で `cargo build`/`cargo test` が完結する）
- `structure.toml` / `clippy.toml` / `deny.toml` を同梱し、`fw gate --project examples/<name>` がそのまま通る
- 既定エスケープ（REQ-1）: ノード木 API のみで HTML を組み立て、`raw_html()` や `format!` による HTML 文字列の直接組み立てを行わない
- README は「概要 / 学べること / 前提 / 動かし方 / 主要ファイル / 関連ガイド」の節構成

[headless-pre-styled-ui](../../examples/headless-pre-styled-ui/README.md) は
作成当初（イシュー #552）、依存する `fandhe-frontend-headless-ui` が
crates.io 未公開だったため path 依存の意図的な例外でしたが、前提クレート
公開（イシュー #608）を受けてイシュー #609 でバージョン依存へ切り替え、
`fw new --example` にも対応済みです。

## 5. 次のステップ

- [クイックスタート](./quickstart.md): `fw new --template app` から始める入門ガイド
- [コンポーネント作成ガイド](./component-authoring.md): ページ・コンポーネントの実装方法
- [API Reference](../api/component-api.md): 各 API の詳細仕様
