# Toast

一時的な通知の queue 表示です。`fandhe-frontend-headless-ui` の `toast`
mod は group（live region）/ root（通知 1 件）/ title / description /
action-trigger / close-trigger の 6 anatomy パーツと、複数通知を有界な
キューとして管理する状態機械 Toaster を提供します。`aria-live` は
`ToastStatus` から決定的に導出され（Error のみ `assertive`、他は
`polite`）、group は `role="region"` + 必須の `aria-label` を固定付与
します。タイマーによる自動 dismiss の実配線は本部品のスコープ外です。

スタイル済みの表示例は [Toast](../themes/toast.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
