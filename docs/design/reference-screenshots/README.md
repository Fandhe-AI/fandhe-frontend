# reference-screenshots

UI 部品スタイル調整（参考サイト基準への調整、ルート issue #1420）の Issue ツリーで参照するスクリーンショット。

## 内容

- `chakra-<slug>-<n>.png` / `ark-<slug>-<n>.png` / `radixp-<slug>-<n>.png` / `radixt-<slug>-<n>.png`: 参考サイト（chakra-ui / Ark UI / Radix Primitives / Radix Themes）の各部品ページの先頭デモ領域（取得日 2026-08-30、viewport 1280x900）。画像ごとの取得元 URL は `SOURCES.md` を参照
- `themes-<kebab>.png` / `primitives-<kebab>.png`: 本リポジトリ docs サイト（`make docs` 出力）の各部品ページ Demo 領域（ライトテーマ、同日取得）

## 出典・ライセンス・再配布根拠

参考サイト由来の画像は、各サイトのドキュメント原稿・デモ実装を含む以下の MIT ライセンスリポジトリの内容を
レンダリングしたものであり、MIT ライセンス（改変・再配布可、著作権表示と許諾表示の保持が条件）に基づき
本リポジトリへ複製・配布する。用途は本リポジトリの UI 部品との視覚比較（設計資料）に限る。

| サイト | 元リポジトリ | ライセンス | 著作権表示 |
|---|---|---|---|
| chakra-ui (https://chakra-ui.com) | https://github.com/chakra-ui/chakra-ui | MIT | Copyright (c) Segun Adebayo |
| Ark UI (https://ark-ui.com) | https://github.com/chakra-ui/ark | MIT | Copyright (c) Chakra UI |
| Radix Primitives / Radix Themes (https://www.radix-ui.com) | https://github.com/radix-ui/website（原稿・デモ）, https://github.com/radix-ui/primitives, https://github.com/radix-ui/themes | MIT | Copyright (c) WorkOS |

各リポジトリの LICENSE 全文（MIT 許諾表示）は上記 URL を参照する。各サイトのロゴ・商標は本ディレクトリに
含めない（取得対象はデモ領域のみ）。

issue 本文からはコミット SHA 固定の raw URL で参照する。
