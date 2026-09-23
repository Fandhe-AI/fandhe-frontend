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

## 掲載済み

- [Annotation](./wireframes/annotation.md)
- [Grid](./wireframes/grid.md)
- [Divider](./wireframes/divider.md)
- [Stack](./wireframes/stack.md)
- [Frame](./wireframes/frame.md)

## 掲載予定

以下は Phase 1〜8（#2608〜#2665）で順次掲載する 45 部品です（Phase ごとの内訳）。
掲載が完了した部品は本節へリンク付きの「掲載済み」一覧として追記されます（現時点では
未掲載のため本節はリンクなしの一覧です）。

### Phase 1: レイアウト骨格

### Phase 2: テキスト・注釈

- text
- paragraph
- rich-text
- link
- tag

### Phase 3: Forms A

- button
- input
- textarea
- select
- checkbox
- radio
- switch
- slider

### Phase 4: Forms B

- question
- ratings
- calendar
- file-drop
- stepper

### Phase 5: Navigation

- nav-item
- menu
- tabs
- breadcrumbs
- pagination
- accordion
- cursor

### Phase 6: Overlay・Feedback

- tooltip
- modal
- alert
- toast
- progress
- spinner

### Phase 7: Data display

- avatar
- icon
- brand
- emoji
- counter
- stat
- list
- card-basic

### Phase 8: Media・Data

- image
- media
- table
- chart
- map
