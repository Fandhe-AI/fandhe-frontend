# Drawer

画面端からスライドインするパネルです。ark-ui・chakra-ui の Drawer は
WAI-ARIA 上 Dialog パターンの変種であり、`fandhe-frontend-headless-ui` の
`drawer` mod は新規状態機械を作らず `dialog` mod の開閉状態機械をそのまま
再利用します。追加する要素は専用 anatomy（`data-scope="drawer"`）と、
画面のどの端から出現するかを表す `DrawerPlacement`（Start/End/Top/Bottom、
既定 End）のみです。`fandhe-frontend-wasm-full` の Escape・外側クリック
配線は未対応（fail-closed のため未対応でも安全側）です。

スタイル済みの表示例は [Drawer](../themes/drawer.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
