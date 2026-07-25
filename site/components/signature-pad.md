# Signature Pad

`fandhe-frontend-pre-styled-ui` の `signature_pad` mod が提供するスタイル済み Signature Pad 部品です。手書き署名入力を扱う複合部品で、canvas を一切使わず「ストローク（座標列）の列 → SVG path 文字列」の決定的な純粋関数のみで構成します。本ページは `showcase` の部品ページレジストリに未登録のため、Demo（実レンダリング掲示）は掲載していません（Examples は使用コードのみを示します）。

## Features

- Root / Label / Control / Segment（svg） / SegmentPath（ストロークごとの path） / Guide / ClearTrigger / HiddenInput の 8 anatomy パーツで構成する。
- 同一座標列は常に同一の `d` 属性値を生成する決定的な `stroke_path_d`（丸め規則を rustdoc で固定）。
- ポインタイベントから座標を収集する処理は本モジュールの責務外（`fandhe-frontend-wasm-full` が座標列を明示的な `Stroke` へ正規化してから `"add-stroke"` アクションとして dispatch する）。
- `strokes: Vec<Stroke>` + `disabled`/`read_only` を保持する状態機械。

## Anatomy

パーツ構成は `crates/headless-ui/src/signature_pad.rs` モジュール doc の anatomy 表（および `ANATOMY.part(...)` 呼び出し）から採取しています。

```
root
label
control
segment
segment-path
guide
clear-trigger
hidden-input
```

## API Reference

### Arguments

| Name | Type | Default | Description |
|---|---|---|---|
| `disabled` | `bool` | `false` | true なら全パーツを無効化する。 |
| `empty` | `bool` | `true` | ストロークが 1 件も無いかどうか（`root`/`guide` の表示切替に使う）。 |
| `width` / `height` | `u32` | （必須） | `segment`（svg）の viewBox 寸法。 |

### Data Attributes

| Part | Attribute | Observed Values |
|---|---|---|
| `root` / `control` | `data-disabled` | 付与時のみ存在（`disabled=true`）。 |
| `root` | `data-empty` | 付与時のみ存在（ストローク未入力時）。 |

## Examples

```rust
use fandhe_frontend_pre_styled_ui::signature_pad;

let node = signature_pad::root(false, true, vec![], vec![
    signature_pad::control(false, vec![], vec![]),
]);
```

## Accessibility

### Keyboard Interactions

署名の描画自体はポインタ操作（マウス/タッチ/スタイラス）専用であり、キーボードのみでの描画操作は提供しません。

| Key | Description |
|---|---|
| `Enter` / `Space`（clear-trigger にフォーカス時） | 入力済みストロークをすべて消去する（dispatch `"clear"`、wasm 層実装）。 |

### WAI-ARIA

| Attribute | Description |
|---|---|
| `type="button"` | `clear-trigger` パーツに常時付与し、フォーム送信への誤混入を防ぐ。 |

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
