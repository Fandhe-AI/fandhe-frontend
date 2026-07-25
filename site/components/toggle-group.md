# Toggle Group

`fandhe-frontend-pre-styled-ui` の `toggle_group` mod が提供するスタイル済み Toggle Group 部品です。
複数トグルボタンのグループで、高々 1 項目が押下される single モード
（`ToggleGroup`）と複数項目が同時に押下される multiple モード
（`MultiToggleGroup`）の 2 状態機械を選べます。

> [!IMPORTANT]
> 本ページは Demo（showcase 掲示）を持ちません。`showcase::stylesheet()` の
> CSS 束（`crates/docs-site/src/showcase.rs`）に `toggle_group::stylesheet()`
> がまだ配線されておらず、本 PR（#946）は「`showcase.rs` を変更しない」という
> 受け入れ条件のため Demo の追加を見送っています。Demo 復活には
> `showcase.rs` への 2 行程度の追記が必要であり、フォローアップ課題として
> 記録しています（本 PR ではイシュー起票は行わず、レビューへ要否を委ねます）。
> 以下の節は Rust の機械生成ではなく、本 Markdown 側で手書きしています。

## Features

- 複数トグルボタンのグループ。ark-ui の Toggle Group を参考に Root / Item の 2 anatomy パーツを持つ。
- 各 item は単体の `Toggle` と同じ「押下状態を持つネイティブ `<button type="button">`」であり、`aria-pressed`/`data-state`（`"on"`/`"off"`）の語彙を揃える。
- `root` のみがグループ化コンテナとして `role="group"` を持つ（`role="radiogroup"` を持つ `RadioGroup` とはボタングループ・input グループの違いがある）。
- `orientation` は CSS 用途の `data-orientation` のみを出力し、`aria-orientation` は付与しない（`role="group"` は toolbar/radiogroup 等の方向性を持つロールではないため）。
- `size` / `color-palette` variant で `root` にクラスを付与する。

## Anatomy

`crates/headless-ui/src/toggle_group.rs` の `ANATOMY.part(...)` 呼び出しから転記（手書き。Demo 復活時は他部品と同じ機械導出へ寄せる）。

```
root
item
```

## API Reference

### Arguments

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `size` | `Size` | `Size::Md` | `root` へ付与するサイズ variant。 |
| `disabled` | `bool` | `false` | `root` の `data-disabled` へ反映される。 |
| `orientation` | `Option<Orientation>` | `None` | `Some` のとき `root` へ `data-orientation` を出力する。 |
| `labelled_by` | `Option<&str>` | `None` | `Some` のとき `root` へ `aria-labelledby` を出力する。 |
| `pressed`（item） | `bool` | | 各 item の押下状態。`aria-pressed` / `data-state`（`"on"`/`"off"`）へ反映される。 |
| `value`（item） | `&str` | | 各 item の識別子。`data-value` として既定エスケープ経由で出力される。 |

## Accessibility

### Keyboard Interactions

| Key | Description |
| --- | --- |
| `Space` / `Enter` | ネイティブ `<button>` のブラウザ既定動作により各 item の押下状態を切り替える。 |

### WAI-ARIA

| Attribute | Description |
| --- | --- |
| `role="group"` | `root` に固定付与。`labelled_by` が `Some` のときのみ `aria-labelledby` が付与される。 |
| `aria-pressed` | 各 item に付与。押下状態（true/false）を表す。 |

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
