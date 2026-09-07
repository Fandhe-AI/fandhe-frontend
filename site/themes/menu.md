# Menu

`fandhe-frontend-pre-styled-ui` の `menu` mod が提供するスタイル済み Menu 部品です。
トリガー起点のオーバーレイ + アクション項目リストで、サブメニューは親 Menu の
`content` 内に子 Menu インスタンス由来の `trigger_item` を入れ子で配置して
表現します。チェック可能な項目（CheckboxItem）・単一選択項目（RadioItemGroup）
は開閉状態とは独立した checked 状態機械を別途持ちます。グループ + ラベル
（`item_group`/`item_group_label`）・アイコンなし項目の位置合わせ（`data-inset`）・
危険操作項目（`data-danger`）・ショートカット表示（独立部品 `kbd` との合成
パターン）にも対応します。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
