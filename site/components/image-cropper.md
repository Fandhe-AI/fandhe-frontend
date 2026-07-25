# Image Cropper

`fandhe-frontend-pre-styled-ui` の `image_cropper` mod が提供するスタイル済み Image Cropper 部品です。画像の crop 矩形（`x`/`y`/`width`/`height` の `u32`）を扱う決定的な状態機械で、canvas 描画命令・変換行列・ピクセルバッファは一切保持しません。本ページは `showcase` の部品ページレジストリに未登録のため、Demo（実レンダリング掲示）は掲載していません（Examples は使用コードのみを示します）。

## Features

- Root / Viewport / Image / Selection / Handle / Grid の 6 anatomy パーツで構成する。
- 座標系は画像の自然寸法（`image_width`/`image_height`、共に `u32`・`>= 1`）を基準とする整数ピクセル空間。
- crop 矩形の不変条件: `1 <= width`（`min_size` 以上）、`x + width <= image_width`（`y`/`height`/`image_height` も同様）。
- dispatch（`"move"`/`"resize"`/`"set"`/`"reset"`）による決定的な状態機械。同一の dispatch 列からは常に同一の矩形が再現される（浮動小数点・デバイス依存座標を扱わない）。
- アスペクト比固定時は `height` を `width` から決定的な整数丸めで算出する。
- 実際の画像切り出し（canvas によるピクセルデータ生成）は本部品のスコープ外。表示は CSS（`object-position`/`clip-path`/inset + custom property）で百分率アクセサの値を反映する想定。

## Anatomy

パーツ構成は `crates/headless-ui/src/image_cropper.rs` の `ANATOMY.part(...)` 呼び出しから採取しています。

```
root
viewport
image
selection
handle
grid
```

## API Reference

### Arguments

| Name | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `Md` | styled root に付与する寸法 variant。 |
| `state` | `&ImageCropper` | （必須） | crop 矩形の決定的状態機械。動的な値は `selection` の 4 個の `--fandhe-cropper-*` custom property のみで伝搬する。 |

### Data Attributes

| Part | Attribute | Observed Values |
|---|---|---|
| `selection` | `style` に埋め込む custom property | `--fandhe-cropper-x` / `-y` / `-width` / `-height`（いずれも百分率）。 |

## Examples

```rust
use fandhe_frontend_headless_ui::image_cropper::ImageCropper;
use fandhe_frontend_pre_styled_ui::image_cropper;
use fandhe_frontend_pre_styled_ui::Size;

let state = ImageCropper::new(640, 480, 0, 0, 320, 240, None, 32);
let node = image_cropper::root(Size::Md, &state, vec![], vec![]);
```

## Accessibility

### Keyboard Interactions

| Key | Description |
|---|---|
| `ArrowLeft` / `ArrowRight` / `ArrowUp` / `ArrowDown` | crop 矩形を 1px 単位で移動する（dispatch `"move"`、wasm 層実装）。 |
| `Shift` + 矢印キー | crop 矩形のサイズを変更する（dispatch `"resize"`、wasm 層実装）。 |

### WAI-ARIA

支援技術向けの動的なライブ領域・role は本部品の対象外です（視覚操作専用の crop UI であり、切り抜き結果は状態機械の値としてフォーム等の他部品へ受け渡す想定）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
