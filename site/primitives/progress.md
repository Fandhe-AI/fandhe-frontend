# Progress

進捗表示（Linear）の unstyled 部品です。`value = None` は不定進捗（indeterminate）を表し、`min`/`max`/`value` は fail-closed に正規化されます。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Progress](../themes/progress.md) を参照してください。

**アクセシビリティ**

- `role="progressbar"` は `root` へ固定付与します（参考サイト ark-ui/Zag.js は `track` へ置きますが、本実装は Radix Primitives に合わせて `root` へ配置します）。
- `value_text` へは `aria-live="polite"` を無条件付与します（数値の更新を支援技術へ非割り込みで通知します）。
- キーボード操作はありません（`progressbar` は非インタラクティブなロールであり、参照サイト 4 件〔ark-ui / Radix Primitives / Radix Themes / chakra-ui〕もキーボード操作表を持ちません）。
- `aria-label`/`aria-labelledby` の既定値は持ちません。意味論的なラベル関連付けは呼び出し側が `attrs` 経由で配線してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
