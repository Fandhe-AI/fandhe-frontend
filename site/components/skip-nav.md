# Skip Nav

`fandhe-frontend-pre-styled-ui` の `skip_nav` mod が提供するスタイル済み Skip Nav 部品です。キーボード操作時のみ視覚的に現れる「本文へスキップ」リンク（WCAG 2.1 SC 2.4.1 Bypass Blocks 対応）で、`fandhe-frontend-docs-site` 自身が全ページのレイアウト骨格へ常時 1 個挿入しています。本ページは `showcase` の部品ページレジストリに未登録のため、Demo（実レンダリング掲示）は掲載していません（本ページの `<body>` 先頭に実際の Skip Nav リンクが常設されており、Tab キーで直接触れられます）。

## Features

- `link`（`<a>`）/ `content`（`<div>`）の 2 パーツ構成。時間変化する内部状態を持たない純粋関数のみで構成する。
- `link` は呼び出し側から任意の URL を受け取らず、常に `#<id>`（フラグメントのみ）を内部で組み立てる（`javascript:` 等のスキーム注入経路を構造的に持たない）。
- `content` の `tabindex="-1"` はプログラム的フォーカスのみを許可し、Tab 順序には加えない。
- `link` の `href` と `content` の `id`/`tabindex` は呼び出し側 `attrs` に同名キーが含まれていても除去してから合成する（fail-closed）。
- focus 時のみ視覚的に現れる表示は `fandhe-frontend-pre-styled-ui` 側の `StateCondition::FocusVisible`（`:focus-visible` 疑似クラス）による純 CSS で実現し、`data-focus-visible` 属性配線を必要としない。

## Anatomy

パーツ構成は `crates/headless-ui/src/skip_nav.rs` の `ANATOMY.part(...)` 呼び出しから採取しています。

```
link
content
```

## API Reference

### Arguments

| Name | Type | Default | Description |
|---|---|---|---|
| `id` | `&str` | `DEFAULT_ID` | `link` の `href="#<id>"` と `content` の `id`/スキップ先を結び付ける識別子。 |

### Data Attributes

| Part | Attribute | Observed Values |
|---|---|---|
| `link` | `data-scope` | `skip-nav` |
| `content` | `data-scope` | `skip-nav` |

## Accessibility

### Keyboard Interactions

| Key | Description |
|---|---|
| `Tab`（ページ先頭で押下） | `link` へフォーカスが移動し、`:focus-visible` により視覚的に出現する。 |
| `Enter` | `href="#<id>"` へ遷移し、`content` の `tabindex="-1"` により実 DOM フォーカスも本文側へ移動する。 |

### WAI-ARIA

| Attribute | Description |
|---|---|
| `tabindex="-1"` | `content` に付与し、プログラム的フォーカスのみを許可する（Tab 順序には加えない）。 |
