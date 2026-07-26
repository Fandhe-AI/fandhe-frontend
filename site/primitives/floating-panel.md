# Floating Panel

ドラッグ移動・リサイズ可能な浮遊パネルです。`fandhe-frontend-headless-ui`
の `floating_panel` mod は Root / Trigger / Positioner / Content / Header /
Title / Control / StageTrigger / CloseTrigger / Body の 10 anatomy パーツ
と、開閉・`Stage`（Default/Minimized/Maximized）・座標を持つ状態機械を
提供します。`content` は `role="dialog"` を固定付与しますが、非モーダル
overlay のため `aria-modal` は出力しません（ユーザーは他の要素を操作し
続けられます）。実際のドラッグ・リサイズ操作は JS ランタイムの責務として
スコープ外です。

スタイル済みの表示例は [Floating Panel](../themes/floating-panel.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
