# File Upload

ドロップゾーン・ボタン起点のファイル選択・受理済みファイル一覧を持つアップロード部品です。ファイルメタデータ（ファイル名・サイズ・MIME タイプ）のみを扱い、`File` オブジェクト自体や実際のアップロード処理は保持しません（accept/max-files 等の検証は決定的な純粋関数で行いますが、実送信・保存は利用者側の責務です）。

`fandhe-frontend-headless-ui` の `file_upload` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

スタイル済みの表示例は [File Upload](../themes/file-upload.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
