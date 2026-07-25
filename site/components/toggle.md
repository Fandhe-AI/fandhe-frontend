# Toggle

`fandhe-frontend-pre-styled-ui` の `toggle` mod が提供するスタイル済み Toggle 部品です。
「ボタンの押下状態」を表す 2 状態ボタンで、ネイティブ `<button type="button">`
自身が押下状態を持ちます（[`crate::switch::Switch`] のようなフォーム送信用の
hidden input は持ちません）。

> [!IMPORTANT]
> 本ページは Demo（showcase 掲示）を持ちません。`showcase::stylesheet()` の
> CSS 束（`crates/docs-site/src/showcase.rs`）に `toggle::stylesheet()` が
> まだ配線されておらず、本 PR（#946）は「`showcase.rs` を変更しない」という
> 受け入れ条件のため Demo の追加を見送っています。Demo 復活には
> `showcase.rs` への 2 行程度の追記が必要であり、フォローアップ課題として
> 記録しています（本 PR ではイシュー起票は行わず、レビューへ要否を委ねます）。
> 以下の節は Rust の機械生成ではなく、本 Markdown 側で手書きしています。

## Features

- 押下状態を持つ 2 状態ボタン。ark-ui の Toggle を参考に Root / Indicator の 2 anatomy パーツを持つ。
- 状態機械は `Switch` と同じ `Checkable`（checked/unchecked の 2 値）を内部で再利用するが、公開する `data-state` の語彙は `"on"`/`"off"`（`aria-pressed` と `data-pressed` 存在属性を併記）で `Switch` とは異なる。
- `root` 自身がネイティブ `<button type="button">` であり、フォーカス・クリック・Space/Enter キー操作はブラウザ既定動作で成立する（hidden input を介さない）。
- `size` / `color-palette` variant で `root` にクラスを付与する。

## Anatomy

`crates/headless-ui/src/toggle.rs` の `ANATOMY.part(...)` 呼び出しから転記（手書き。Demo 復活時は他部品と同じ機械導出へ寄せる）。

```
root
indicator
```

## API Reference

### Arguments

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `size` | `Size` | `Size::Md` | `root` へ付与するサイズ variant。 |
| `pressed` | `bool` | | 押下状態。`aria-pressed` / `data-state`（`"on"`/`"off"`） / `data-pressed` へ反映される。 |
| `disabled` | `bool` | `false` | ネイティブ `disabled` 属性と `data-disabled` を出力する。 |

## Accessibility

### Keyboard Interactions

| Key | Description |
| --- | --- |
| `Space` / `Enter` | ネイティブ `<button>` のブラウザ既定動作により押下状態を切り替える。 |

### WAI-ARIA

| Attribute | Description |
| --- | --- |
| `aria-pressed` | `root` に付与。押下状態（true/false）を表す。 |
