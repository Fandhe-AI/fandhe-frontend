# Clipboard

`fandhe-frontend-pre-styled-ui` の `clipboard` mod が提供するスタイル済み Clipboard 部品です。値のコピー・コピー済み表示を扱う複合部品です。本ページは `showcase` の部品ページレジストリに未登録のため、Demo（実レンダリング掲示）は掲載していません（Examples は使用コードのみを示します）。

## Features

- Root / Label / Control / Input / Trigger / Indicator / ValueText の 7 anatomy パーツで構成する。
- コピー済みかどうかの 2 値状態機械（`data-copied` 存在属性で表現、値語彙ではなく存在の有無）。
- コピー対象の `value` は状態機械に持たせず、呼び出し側が各パーツ関数へ都度渡す描画パラメータ（`root` が出力する `data-value` 属性からクライアント側が読み取る）。
- コピー完了後の自動リセット（タイムアウト）は本モジュールの責務外（`fandhe-frontend-wasm-full` 側のクライアント配線層が担う）。
- `"clipboard:copy"` / `"clipboard:reset"` の 2 アクションを提供する（アクション名は `Runtime<C>` が複数コンポーネントを無条件配線する構成のため名前空間修飾されている）。

## Anatomy

パーツ構成は `crates/headless-ui/src/clipboard.rs` の `ANATOMY.part(...)` 呼び出しから採取しています。

```
root
label
control
input
trigger
indicator
value-text
```

## API Reference

### Arguments

| Name | Type | Default | Description |
|---|---|---|---|
| `value` | `&str` | （必須） | コピー対象の値。`root` の `data-value` 属性・`input` の `value` 属性へ渡る。 |
| `copied` | `bool` | `false` | コピー済み状態。`data-copied` 存在属性・`indicator` の表示切替へ反映する。 |

### Data Attributes

| Part | Attribute | Observed Values |
|---|---|---|
| `root` / `control` / `input` / `trigger` / `indicator` | `data-copied` | 付与時のみ存在（`copied=true`）。 |
| `indicator` | `data-variant` | `copied` / `idle`（2 変種の indicator が同じ scope/part を共有するための区別）。 |

## Examples

```rust
use fandhe_frontend_pre_styled_ui::clipboard;

let node = clipboard::root("https://example.com/shared-link", false, vec![], vec![]);
```

## Accessibility

### Keyboard Interactions

| Key | Description |
|---|---|
| `Enter` / `Space`（trigger にフォーカス時） | クリップボードへのコピーを実行する（`navigator.clipboard.writeText`、wasm 層実装）。 |

### WAI-ARIA

| Attribute | Description |
|---|---|
| `readonly` | `input` パーツに常時付与し、キーボード操作でのコピー対象値の書き換えを防ぐ。 |
| `type="button"` | `trigger` パーツに常時付与し、フォーム送信への誤混入を防ぐ。 |

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
