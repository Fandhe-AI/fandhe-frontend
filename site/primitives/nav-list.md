# Nav List

見出し + リンクリストのみで構成する文書ナビ向けの静的なリンク集です。`fandhe-frontend-headless-ui` の `nav_list` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。`menu` ロールへの誤読を避けるため `role` 属性を一切付与せず、素の `nav`/`ul`/`li`/`a` の暗黙 ARIA ロールに依拠します。

スタイル済みの表示例は [Nav List](../themes/nav-list.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
