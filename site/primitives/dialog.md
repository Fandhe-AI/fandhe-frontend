# Dialog

モーダルダイアログです。`fandhe-frontend-headless-ui` の `dialog` mod は
Root / Trigger / Backdrop / Positioner / Content / Title / Description /
CloseTrigger の 8 anatomy パーツと、`DialogRole`（Dialog/Alertdialog）で
切り替えられる `role`・`aria-modal`・`aria-haspopup="dialog"` を提供します。
フォーカストラップ・Escape キーでの閉鎖・外側クリックでの閉鎖は JS ランタイム
側の責務としてスコープ外であり、本部品は SSR での属性出力のみを担います。

スタイル済みの表示例は [Dialog](../themes/dialog.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
