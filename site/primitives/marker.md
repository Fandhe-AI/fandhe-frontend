# Marker

会話スレッド内のインライン注記行（システム注記・日付等の区切り・ラベル付きセパレータ）を表現する shadcn/ui Marker 相当の部品です（参照軸はイシュー #2001）。`fandhe-frontend-headless-ui` の `marker` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root` / `icon` / `content` の 3 anatomy パーツを持ちます。状態機械を持たない静的部品です。

`root` は `variant`（`note` / `divider` / `label`）・`tone`（`neutral` / `info` / `warning` / `danger`）の props を受け取り、それぞれ `data-variant` / `data-tone` として出力します。会話系部品（message / bubble / attachment / marker）が共有する `data-role` / `data-align` は意図的に持ちません。`data-tone` の値語彙は `fandhe-frontend-pre-styled-ui` の `ColorPalette`（`neutral` / `info` / `warning` / `danger`）の同名 4 値の部分集合で、新しい値語彙は作りません。

`divider` / `label` variant の区切り線（行の下の境界線、中央ラベル + 左右の線）は本 mod では一切描画しません。区切り線の描画は `fandhe-frontend-pre-styled-ui` 側（`separator` パーツの再利用または CSS）が担う設計です。`icon` は装飾スロット（アイコン文字等）を、`content` は注記の本文テキストを children で受け取るだけのスロットです。

**アクセシビリティ**

- `icon` には常に `aria-hidden="true"` を固定付与します（呼び出し側の `aria-hidden="false"` 偽装は除去されます）。
- `root` には `role` を固定付与しません。静的な注記に割り込み通知は不要という判断です。ストリーミング中の注記に `role="status"` を付与したい場合は、呼び出し側が `attrs` で明示的に渡してください。

`fandhe-frontend-pre-styled-ui` 側の対応するスタイル済み部品（recipe・golden・Themes ページ）は後続イシューで追加予定です。

自前 CSS の最小例:

```css
[data-scope="marker"][data-part="root"] {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8125rem;
  color: #6b7280;
}
[data-scope="marker"][data-part="root"][data-tone="warning"] {
  color: #b45309;
}
[data-scope="marker"][data-part="root"][data-tone="danger"] {
  color: #b91c1c;
}
[data-scope="marker"][data-part="root"][data-variant="label"] {
  justify-content: center;
}
```

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
