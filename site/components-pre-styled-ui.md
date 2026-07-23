# pre-styled-ui コンポーネントショーケース

`fandhe-frontend-pre-styled-ui` が提供するスタイル済み UI コンポーネントの
実レンダリング結果を掲載するページです。以下の各コンポーネントは docs サイトの
ビルド時に Rust 関数（`crates/docs-site/src/showcase.rs`）が実際に組み立てた
ノード木であり、スタイルはテーマトークンと slot recipe から生成した専用 CSS
（`assets/pre-styled-ui.css`）で適用されています。

Tabs / Accordion / Switch / RadioGroup などの状態機械を持つコンポーネントは、
選択中・開いた状態やチェック状態を固定した静的マークアップとして掲示しています
（クリック等の状態遷移は wasm 層の責務で、本ページのスコープ外です）。Avatar は
画像読み込み状態（`ImageStatus`）を固定し、フォールバック表示・画像表示の両方を
掲示しています。

API の詳細は [fandhe-frontend-pre-styled-ui API](../docs/api/pre-styled-ui-api.md)
と [pre-styled-ui slot recipe API](../docs/api/pre-styled-recipe-api.md) を
参照してください。
