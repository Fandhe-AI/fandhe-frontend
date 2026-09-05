# QR Code

QR コード（ISO/IEC 18004 Model 2、byte モード）を表示する unstyled 部品です。外部依存ゼロのエンコーダで文字列からモジュール行列を導出する純粋な変換のみを提供し、符号化対象の文字列自体はマークアップへ出力されません。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [QR Code](../themes/qr-code.md) を参照してください。

**アクセシビリティ**

- `frame`（`svg`）は `aria_label` を指定したときのみ `role="img"` + `aria-label` を付与します。未指定時はどちらも付与しません（WAI-ARIA 1.2 `img` ロールは Accessible Name Required のため、名前のない `img` ロールを名乗って偽の説明文を捏造しません）。`aria-labelledby` 等の別経路で名前付けしたい場合は `attrs` に `role`/`aria-labelledby` を渡してください。
- `frame` には `xmlns="http://www.w3.org/2000/svg"` を固定付与します（SVG 単体をシリアライズしても名前空間解決できるようにするため）。
- キーボード操作はありません（QrCode は非インタラクティブな表示部品です。参照サイト ark-ui / chakra-ui もキーボード操作表を持ちません。Radix Primitives / Radix Themes に QR Code は存在しません）。
- `DownloadTrigger`（QR 画像のダウンロードボタン）は提供しません。生成画像を `a[download]` の `data:` href に載せる経路が必要になりますが、`fandhe_frontend_core::is_safe_url` が `data:` URL を拒否し属性ごと欠落するため、静的 SSR の枠内では動作しない部品を追加しない判断です。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
