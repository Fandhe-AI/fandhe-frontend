# Wireframes

`fandhe-frontend-wireframe-ui` は `fandhe-frontend-headless-ui`（Primitives）・
`fandhe-frontend-pre-styled-ui`（Themes）とは独立した第 3 の UI コンポーネント層です。
画面設計初期段階での配置イメージ提示に特化した、モノクロ・ローファイなワイヤーフレーム
部品を提供します。

このクレートの部品は SSR 専用・非インタラクティブな表示専用プレースホルダーです。
`role`/`aria-*`・`tabindex`・キーボードイベントハンドラ・フォーカス管理・状態遷移は
持たず、`button`/`input`/`select`/`a[href]` のようなネイティブに対話セマンティクスを
持つ HTML 要素も出力しません。実際に操作可能な部品が必要な場合は、[Primitives](./primitives.md)
または [Themes](./themes.md) を使用してください。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) です。ライセンス上の理由により、
本セクションには blocks.pm のスクリーンショットや外観・プロパティ構成をそのまま
取り込みません（詳細は `docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は
外部リンク先を直接確認してください。

## ページ構成

各部品ページは以下の 3 節からなる簡略テンプレートです。Primitives/Themes の部品ページが
持つ Anatomy・`data-*` 属性表・CSS 変数表・Accessibility 節はここでは省略します。

- **Demo**: 部品の実レンダリング結果
- **引数表**: Rust API の引数・型・既定値・説明
- **原案差分メモ**: 独自設計の判断（blocks.pm との対応範囲・Primitives/Themes の同名部品との違い等）
