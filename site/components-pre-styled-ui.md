# pre-styled-ui コンポーネントショーケース

`fandhe-frontend-pre-styled-ui` が提供するスタイル済み UI コンポーネントの
実レンダリング結果を掲載するページです。以下の各コンポーネントは docs サイトの
ビルド時に Rust 関数（`crates/docs-site/src/showcase.rs`）が実際に組み立てた
ノード木であり、スタイルはテーマトークンと slot recipe から生成した専用 CSS
（`assets/pre-styled-ui.css`）で適用されています。

Tabs / Accordion / Dialog / Menu / Select / Popover / Tooltip などの状態機械を
持つコンポーネントは、選択中・開いた状態を固定した静的マークアップとして
掲示しています（クリック等の状態遷移は wasm 層の責務で、本ページのスコープ
外です）。Dialog / Menu / Select / Popover / Tooltip はトリガー起点の
オーバーレイ部品のため、開いた状態のまま掲示すると本来の配置（画面全体を
覆う・トリガーの直下にかぶさる等）ではページ内の他のセクションと重なって
しまいます。そのためこのページでは、掲示専用の CSS でオーバーレイをページの
流れの中へ収めています（実際のアプリケーションでの overlay 配置は
pre-styled-ui の recipe CSS がそのまま担います）。

API の詳細は [fandhe-frontend-pre-styled-ui API](../docs/api/pre-styled-ui-api.md)
と [pre-styled-ui slot recipe API](../docs/api/pre-styled-recipe-api.md) を
参照してください。
