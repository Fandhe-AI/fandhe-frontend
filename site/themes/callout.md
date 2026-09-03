# Callout

`fandhe-frontend-pre-styled-ui` の `callout` mod が提供するスタイル済み Callout 部品です。

本文フロー中に置く補足情報を強調表示する静的な部品です。[Alert](alert.md) と異なり `role="alert"`（WAI-ARIA live region）を付与しないため、支援技術への割り込み通知を発生させません。強調の意味づけは variant（soft/surface/outline）と colorPalette の組み合わせのみで表現し、緊急度の意味論は持ちません。

size（xs〜xl、既定 md）は Radix Themes Callout の size 1〜3 に対応し、padding・gap・角丸・文字サイズが連動して変化します。配色はセマンティック色（accent/info/success/warning/danger/neutral）の淡色トークンを背景に、本文サイズでも 4.5:1 以上の WCAG コントラストを満たす文字色を組み合わせます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
