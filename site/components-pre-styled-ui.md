# pre-styled-ui コンポーネントショーケース

`fandhe-frontend-pre-styled-ui` が提供するスタイル済み UI コンポーネントの
実レンダリング結果を掲載するページです。以下の各コンポーネントは docs サイトの
ビルド時に Rust 関数（`crates/docs-site/src/showcase.rs`）が実際に組み立てた
ノード木であり、スタイルはテーマトークンと slot recipe から生成した専用 CSS
（`assets/pre-styled-ui.css`）で適用されています。

Tabs / Accordion / Dialog / Menu / Select / Popover / Tooltip / Switch /
RadioGroup などの状態機械を持つコンポーネントは、選択中・開いた状態やチェック
状態を固定した静的マークアップとして掲示しています（クリック等の状態遷移は
wasm 層の責務で、本ページのスコープ外です）。Dialog / Menu / Select / Popover /
Tooltip はトリガー起点のオーバーレイ部品のため、開いた状態のまま掲示すると
本来の配置（画面全体を覆う・トリガーの直下にかぶさる等）ではページ内の他の
セクションと重なってしまいます。そのためこのページでは、掲示専用の CSS で
オーバーレイをページの流れの中へ収めています（実際のアプリケーションでの
overlay 配置は pre-styled-ui の recipe CSS がそのまま担います）。Avatar は
画像読み込み状態（`ImageStatus`）を固定し、フォールバック表示・画像表示の両方を
掲示しています。

API の詳細は [fandhe-frontend-pre-styled-ui API](../docs/api/pre-styled-ui-api.md)
と [pre-styled-ui slot recipe API](../docs/api/pre-styled-recipe-api.md) を
参照してください。
