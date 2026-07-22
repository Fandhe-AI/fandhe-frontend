# fandhe-frontend

Rust 製のフロントエンドフレームワークです。AI 時代のセキュリティリスクを下げることを目的に、プレーンな HTML / JavaScript / CSS を尊重しつつ、SSR / SPA / SSG / トランジションなどモダン機能を網羅します。部分埋め込みの最小構成からフル機能構成までのグラデーションを持ち、単一実行ファイルでのデプロイ（Docker 想定）を目標とします。

> 正式名称は `fandhe-frontend`（確定、2026-07-19）です。決定根拠・新旧マッピング表・段階的移行計画は [`docs/design/framework-naming.md`](./docs/design/framework-naming.md) を参照してください。現行のクレート名は `fandhe-frontend-core` 系のまま段階的移行中です（仕様上のクレート名の想定については [仕様リポジトリの `05-tasks.md`](./docs/spec/05-tasks.md) 参照）。

## ドキュメント

公開 docs サイト（Getting Started / Guides / API Reference 構成）は以下で閲覧できます。

- https://fandhe-ai.github.io/fandhe-frontend/

ローカルでビルドして確認する場合は次のコマンドを実行してください。

```bash
cargo run -p fandhe-frontend-docs-site -- --out dist/
```

生成物は `dist/index.html` から閲覧できます。ビルドには内部リンク検証（linkcheck）が
内蔵されており、リンク切れを検出した場合は非 0 終了します。コンテンツの正は
`docs/guides/`・`docs/api/` 配下、サイト構成は `site/nav.toml` です。

## インストール

### CLI ツール

`fw` コマンド（プロジェクト生成・検査ツール）は crates.io で公開されており、
以下のコマンドで導入できます。

```bash
cargo install fandhe-frontend-cli
```

### ライブラリとして使う場合

コア描画ライブラリやアプリ構築層などのクレートは個別に crates.io から利用できます。
例えば、描画コアの `fandhe-frontend-core` を使う場合は以下のコマンドを実行してください。

```bash
cargo add fandhe-frontend-core
```

その他のクレート（`fandhe-frontend-app`・`fandhe-frontend-server` 等）も同様に
追加・導入できます。詳細はドキュメントサイトの Getting Started を参照してください。

#### UI コンポーネント 2 層構造（headless-ui / pre-styled-ui）

ark-ui / chakra-ui を参考にした 2 層の UI コンポーネント構成（親トラッキング
#520）として `fandhe-frontend-headless-ui`（headless UI コンポーネント層）・
`fandhe-frontend-pre-styled-ui`（pre-styled UI コンポーネント層）を提供します。
両クレートとも v0.1.0 で crates.io へ公開済み（イシュー #608）のため、
`cargo add fandhe-frontend-headless-ui` 等で通常どおり導入できます（実例は
[`examples/headless-pre-styled-ui`](./examples/headless-pre-styled-ui/README.md)、
API 詳細は
[`docs/api/headless-ui-api.md`](./docs/api/headless-ui-api.md) /
[`docs/api/pre-styled-ui-api.md`](./docs/api/pre-styled-ui-api.md) を参照）。

## サンプル集（examples/）

動くサンプルは [`examples/`](./examples/) 配下に「正本」として置かれています。
いずれも crates.io へ公開済みのクレートへのバージョン依存のみで完結する
独立した cargo プロジェクトです。

| サンプル | 学べること | 主要クレート |
|---|---|---|
| [ssr-routing](./examples/ssr-routing/README.md) | SSR（`Loader`・`respond_with`・`Router`）+ 既定エスケープ（REQ-1） | `fandhe-frontend-core` / `-app` / `-server` |
| [ssg-blog](./examples/ssg-blog/README.md) | `generate_pages` による静的サイト書き出し・fail-closed なパス検証 | `fandhe-frontend-core` / `-server` |
| [dist-server-docker](./examples/dist-server-docker/README.md) | 単一バイナリ配布 + `FROM scratch` の Docker イメージ最小化 | `fandhe-frontend-dist-server` |
| [interactive-view-transitions](./examples/interactive-view-transitions/README.md) | 状態管理（`Component`/`dispatch`）+ View Transitions の自動有効化 | `fandhe-frontend-core` / `-app` / `-interactive` |
| [headless-pre-styled-ui](./examples/headless-pre-styled-ui/README.md) | headless-ui コンポーネント（Tabs/Accordion/Dialog/Switch/RadioGroup/Avatar）の anatomy・`data-*`・WAI-ARIA | `fandhe-frontend-core` / `-headless-ui` |

`fw` CLI の `--example` オプションで、上記のサンプルをすべて自分の
プロジェクトとして展開できます。

```bash
cargo install fandhe-frontend-cli
fw new my-app --example ssr-routing
```

一覧・選び方の詳細は [`examples/README.md`](./examples/README.md)、docs サイト版は
https://fandhe-ai.github.io/fandhe-frontend/examples/ を参照してください。

## 仕様

仕様書（ブレスト〜PoC〜要件定義〜タスク分解〜ロードマップ）は [Fandhe-AI/fandhe-frontend-spec](https://github.com/Fandhe-AI/fandhe-frontend-spec) で管理し、`docs/spec/` にサブモジュールとして取り込んでいます。

```bash
git clone --recurse-submodules git@github.com:Fandhe-AI/fandhe-frontend.git
# 既存クローンの場合
git submodule update --init
```

| ドキュメント | 内容 |
|-------------|------|
| [`docs/spec/04-requirements.md`](./docs/spec/04-requirements.md) | MoSCoW 優先度付き要件・受け入れ基準 |
| [`docs/spec/05-tasks.md`](./docs/spec/05-tasks.md) | タスク分解（依存関係・工数） |
| [`docs/spec/06-roadmap.md`](./docs/spec/06-roadmap.md) | マイルストーン MS-1〜MS-5・着手判定（Go・2026-07-08） |

## 開発の進め方

`docs/spec/06-roadmap.md` のマイルストーンに従って実装します。

- **MS-1**: 安全性基盤＋純 Rust 記述方式の確立（既定エスケープ・依存グラフ上限の CI 計測）
- **MS-2**: コア三モード描画（SSR / SPA / SSG）とグラデーションの実装
- **MS-3**: 実ブラウザ実証＋WASM 完全方式の確立（実装内 Go/No-Go 確認あり）
- **MS-4**: 配布・DX の確立（単一実行ファイル・Docker・cargo 統合ビルドチェーン）
- **MS-5**: AI 自己保守・改修フックの確立（`impact` → 変更適用 → `gate`）

実装着手の最初のタスクは TASK-1.1（`fandhe-frontend-core` 既定エスケープの製品化）です。

## ライセンス

本プロジェクトは [MIT ライセンス](./LICENSE-MIT) と [Apache License 2.0](./LICENSE-APACHE) の
デュアルライセンスで提供されます。

あなたが明示的に別段の定めをしない限り、あなたが本プロジェクトへ提出する Contribution は、
Apache License 2.0 のセクション 5 に定義される条件に従い、追加の条項なしに上記デュアル
ライセンスの下で提供されるものとみなされます。
