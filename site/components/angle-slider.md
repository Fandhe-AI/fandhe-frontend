# Angle Slider

`fandhe-frontend-pre-styled-ui` の `angle_slider` mod が提供するスタイル済み Angle Slider 部品です。`0..=360` 度の角度を選択する 1 次元スライダーで、円形のダイヤルを回転させる形で操作します。本ページは `showcase` の部品ページレジストリに未登録のため、Demo（実レンダリング掲示）は掲載していません（Examples は使用コードのみを示します）。

## Features

- Label / Control / Thumb / HiddenInput / ValueText の 5 anatomy パーツで構成する。
- `thumb` の回転は headless 中立な `angle_deg`（`0..=359` の整数）から導出する `--fandhe-angle` CSS custom property の 1 点のみで伝搬し、canvas の描画命令列のような内部状態は持たない。
- dispatch（`"set"`/`"increment"`/`"decrement"`）による決定的な状態機械。`"set"` は受理値をそのままではなく `0..=360` の範囲へ丸めてから採用する（fail-closed、クライアント由来の信頼できない入力として扱う）。
- `size`/`palette` variant クラスを付与する styled `root` を提供する（`crate::slider::root` と同型）。

## Anatomy

パーツ構成は `crates/headless-ui/src/angle_slider.rs` の `ANATOMY.part(...)` 呼び出しから採取しています。

```
root
label
control
thumb
hidden-input
value-text
```

## API Reference

### Arguments

| Name | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `Md` | root に付与する寸法 variant。 |
| `palette` | `ColorPalette` | `Accent` | root に付与する colorPalette 軸。 |
| `disabled` | `bool` | `false` | true なら thumb の `tabindex` を `-1` にし `aria-disabled` を付与する。 |

### Data Attributes

| Part | Attribute | Observed Values |
|---|---|---|
| `root` / `control` / `thumb` | `data-disabled` | 付与時のみ存在（`disabled=true`）。 |

## Examples

```rust
use fandhe_frontend_headless_ui::angle_slider::AngleSlider;
use fandhe_frontend_pre_styled_ui::angle_slider;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

let state = AngleSlider::default();
let node = angle_slider::root(Size::Md, ColorPalette::Accent, &state, false, vec![], vec![]);
```

## Accessibility

### Keyboard Interactions

| Key | Description |
|---|---|
| `ArrowUp` / `ArrowRight` | 角度を増加させる（dispatch `"increment"`、wasm 層実装）。 |
| `ArrowDown` / `ArrowLeft` | 角度を減少させる（dispatch `"decrement"`、wasm 層実装）。 |

### WAI-ARIA

| Attribute | Description |
|---|---|
| `role="slider"` | thumb に常時付与する。 |
| `aria-valuemin` / `aria-valuemax` | 常に `"0"` / `"360"`。 |
| `aria-valuenow` / `aria-valuetext` | 現在の角度値とその読み上げ用テキスト表現。 |
