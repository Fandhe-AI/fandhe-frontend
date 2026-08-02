# サンプル集（examples/）

`fandhe-frontend` フレームワークの動くサンプルは、リポジトリルート
`examples/` 配下に「正本」として置かれています。いずれも crates.io へ
公開済みのクレートへのバージョン依存のみで完結する、独立した
cargo プロジェクトです。本ページはサンプルの比較・読む順・
`fw new --example` による取得手順をまとめます。

## 1. 5 サンプルの比較

全サンプルに共通する前提は「Rust ツールチェーン（`cargo`）」「crates.io
（`https://index.crates.io` / `https://static.crates.io`）への到達性」
「`fw gate --project examples/<name>` を実行する場合の clippy component /
cargo-deny（`tools/ci/ensure-gate-tools.sh` で導入）」の 3 点です。下表の
「追加で必要なもの」はこの共通前提からの差分のみを示します。

| サンプル | 目的（何が作れるか） | 主要クレート | 追加で必要なもの | 所要目安 |
|---|---|---|---|---|
| [ssr-routing](../../examples/ssr-routing/README.md) | `Loader`・`respond_with`・`Router` による SSR ページの構築 | `fandhe-frontend-core` / `-app` / `-server` | なし | 短 |
| [ssg-blog](../../examples/ssg-blog/README.md) | `generate_pages` / `generate_assets` による静的サイト（ブログ + sitemap.xml / robots.txt）の書き出し | `fandhe-frontend-core` / `-server` | なし | 短 |
| [dist-server-docker](../../examples/dist-server-docker/README.md) | 単一バイナリ配布・Docker イメージでのデプロイ | `fandhe-frontend-dist-server` | Docker（イメージのビルド・起動を試す場合） | 中 |
| [interactive-view-transitions](../../examples/interactive-view-transitions/README.md) | クライアント側状態管理・View Transitions の実演 | `fandhe-frontend-core` / `-app` / `-interactive`（+ `-wasm-full`） | `rustup target add wasm32-unknown-unknown` と、`wasm/Cargo.lock` の解決版に一致する wasm-bindgen-cli（ブラウザでの実動作確認時のみ） | 長 |
| [headless-pre-styled-ui](../../examples/headless-pre-styled-ui/README.md) | Primitives / Themes 2 層 UI コンポーネントのショーケース | `fandhe-frontend-core` / `-pre-styled-ui`（headless 層 API は再エクスポート経由） | なし | 短 |

> 「所要目安」は追加ツール導入の有無と手順ステップ数から算出した目安であり、実測値ではありません。

## 2. 読む順

> [!TIP]
> どれから読むか迷う場合は ssr-routing から始めてください。

1. **Step 1（最初に読む）: [ssr-routing](../../examples/ssr-routing/README.md)**
   `Loader` / `respond_with` / `Router` と既定エスケープ（REQ-1）という、
   他の全サンプルが前提にする基本要素が揃っています。examples 規約の
   初例（#499）でもあり、`structure.toml` / `clippy.toml` / `deny.toml` の
   共通構成もここで理解できます。
2. **Step 2（目的で分岐）**
   - 静的サイト（ブログ等）を作りたい → [ssg-blog](../../examples/ssg-blog/README.md)
   - 単一バイナリ / Docker でデプロイしたい → [dist-server-docker](../../examples/dist-server-docker/README.md)
   - クライアント側の状態管理・ページ遷移アニメーションを試したい → [interactive-view-transitions](../../examples/interactive-view-transitions/README.md)
   - UI 部品（Primitives / Themes 2 層）を試したい → [headless-pre-styled-ui](../../examples/headless-pre-styled-ui/README.md)
3. **Step 3（応用）**
   目的別ガイド（[コンポーネント作成ガイド](./component-authoring.md) 等）と
   [API Reference](../api/component-api.md) へ進んでください。

## 3. 各サンプルの詳細

### 3.1 ssr-routing

`Loader` trait の自作実装・`respond_with` による SSR 応答組み立て・
`Router` によるパスパラメータ処理・既定エスケープ（REQ-1）を学べます。
加えて `el_owned` / `attr_if` / `attr_if_value`（`fandhe-frontend-core`
0.2.0 以降、イシュー #1121）による条件付き属性の組み立ても実演します。
`el` の `Vec<(&str, &str)>` は `format!`/`to_string` した動的な属性値と
相性が悪く（借用元が呼び出し元スタックフレームより長生きする必要がある）、
`el_owned` は属性を `Vec<(String, String)>` として直接受け取ることで
この制約を外します。`attr_if`/`attr_if_value` が返す
`Option<(String, String)>` を `chain`/`flatten` で合成すると、条件不成立
時は属性自体が出力から欠落します（`hidden`/`disabled` のような真偽属性の
慣例に沿う設計）。実装例は `src/main.rs` の `hello_response` を参照して
ください。関連: [API Reference](../api/component-api.md)。

### 3.2 ssg-blog

`generate_pages` による静的サイト書き出し・パス検証の fail-closed 契約・
View Transitions の有効化を学べます。加えて `generate_assets`
（`fandhe-frontend-server` 0.2.0 以降）による `sitemap.xml` / `robots.txt`
の書き出しも実演します（イシュー #1135）。

`generate_assets` は `generate_pages` と異なり任意のファイル名を持つ
非 HTML 生成物を書き出す汎用 API です。利用手順:

```rust
use fandhe_frontend_server::ssg::generate_assets;
use std::path::Path;

// (リクエストパス, コンテンツ文字列) の列を組み立てる。
let assets = vec![
    ("/sitemap.xml".to_string(), sitemap_xml),
    ("/robots.txt".to_string(), robots_txt),
];

// generate_pages と同じ fail-closed のパス検証を経由して dist/ へ書き出す。
generate_assets(&assets, Path::new("dist"))?;
```

`generate_assets` を使う際は以下の 3 点に注意してください。

- **fail-closed**: `assets` 全件のパスを書き出し前に検証し、正規化後の
  重複も検出します。1 件でも不正・重複があれば **1 つも書き出さずに**
  エラーを返します（`generate_pages` と同型）。
- **既定エスケープ（REQ-1）は適用されない**: コンテンツは無加工で書き出され
  ます（`Node` 木・`fandhe_frontend_core::render` を経由しません）。**HTML
  ページの生成には使わず `generate_pages` を使ってください**。`sitemap.xml`
  の URL 等、コンテンツ内部のエスケープ（XML エスケープ等）は呼び出し側の
  責務です。
- **`fandhe-frontend-server` 0.2.0 以降が必要**: `generate_assets` は
  イシュー #1119 で追加された API です（0.1.x には含まれません）。

実装例は [ssg-blog](../../examples/ssg-blog/README.md) の
`src/main.rs`（`build_assets` / `main`）を参照してください。

さらに `fandhe_frontend_core::json_ld`（`fandhe-frontend-core` 0.2.0 以降、
イシュー #1117）による JSON-LD 構造化データの
`<script type="application/ld+json">` 埋め込みも実演します（記事一覧
ページ（`/`）のみ、`src/main.rs` の `website_json_ld` / `layout` の
`head_extra` 引数参照）。`json_ld` は既定エスケープ（REQ-1）の経路では
なく、シリアライズ済みの JSON 文字列を受け取って `<` `>` `&` 等の
HTML 活性文字だけを `\uXXXX` 中立化する専用 API です。渡す文字列は
既にシリアライズ済みの JSON である必要があり（`serde_json::to_string`
の結果等）、HTML エスケープ済み文字列を渡すと JSON が壊れます。

### 3.3 dist-server-docker

単一バイナリ配布・`FROM scratch` の Docker イメージ最小化・外部依存利用時の
静的アセット配信の制約と対処を学べます。

### 3.4 interactive-view-transitions

`Component` trait による状態機械・`dispatch`/`hydrate`・`start_router` に
よる SPA 内 View Transitions の自動有効化を学べます。関連:
[Interactive API](../api/interactive-api.md)。

### 3.5 headless-pre-styled-ui

Primitives 層（`fandhe-frontend-headless-ui` 相当、anatomy・`data-*`・
WAI-ARIA 属性）と Themes 層（`fandhe-frontend-pre-styled-ui`、スタイル済み
部品）の 2 層 UI コンポーネント構成（Tabs/Accordion/Dialog/Switch/
RadioGroup/Avatar 等）を学べます。加えて `Theme::upsert_color` /
`Theme::upsert_space`（`fandhe-frontend-pre-styled-ui` 0.38.0 以降、
イシュー #1138）によるテーマトークンの上書き・追加も実演します。
`push_color` 等は同名トークンを `DuplicateTokenName` で fail-closed 拒否
するため、`Theme::default()` の既定パレット（`accent`）を差し替える正規
経路は `upsert_color` です（不在トークンに対しては `push_color` と同じ
挿入動作になります）。実装例は `src/main.rs` の `build_stylesheet` を
参照してください。関連: [Pre-styled UI API](../api/pre-styled-ui-api.md)。

## 4. `fw new --example` での取得

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

## 5. 共通規約

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

## 6. 次のステップ

- [クイックスタート](./quickstart.md): `fw new --template app` から始める入門ガイド
- [コンポーネント作成ガイド](./component-authoring.md): ページ・コンポーネントの実装方法
- [API Reference](../api/component-api.md): 各 API の詳細仕様
- 迷ったら本ページの「2. 読む順」に戻り、ssr-routing から段階的に進めてください
