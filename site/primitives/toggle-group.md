# Toggle Group

複数の押下可能なボタンをグループ化する unstyled 部品です。「高々 1 項目が押下される」状態機械（常時 deselectable）を提供し、各項目は独立した Toggle の集合として振る舞います。各 `item` は `root` と同じ `orientation` が `Some` のときに `data-orientation` を持ち、`ToggleGroupProps::roving_focus`（既定 `false`）を有効にすると SSR 側から roving tabindex を opt-in で出力できます（イシュー #1630）。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Toggle Group](../themes/toggle-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
