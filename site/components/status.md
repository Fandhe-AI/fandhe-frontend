# Status

`fandhe-frontend-pre-styled-ui` の `status` mod が提供するスタイル済み Status 部品です。

オンライン/オフライン等の状態を示す最小部品です。colorPalette 軸でセマンティック色を選択でき、root/indicator の 2 パーツのみで構成します。role="status" は付与しない設計で、非同期の状態遷移を伴う場合は呼び出し側が明示的に role/aria-live を足す契約です。
