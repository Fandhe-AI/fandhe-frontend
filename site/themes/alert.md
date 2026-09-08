# Alert

`fandhe-frontend-pre-styled-ui` の `alert` mod が提供するスタイル済み Alert 部品です。

ユーザーへ注意喚起・状態変化を伝える部品です。role="alert" を状態に関わらず固定で付与するため、五つの状態（Info/Success/Warning/Error/Neutral）いずれでもスクリーンリーダーへ通知が届きます。見た目は variant（Subtle/Surface/Solid/Outline、既定 Subtle）と size（Xs〜Xl、既定 Md）でも調整できます。値の変化を伴わない静的な警告表示には [Status](status.md) を、一時的な進捗通知には [Progress](progress.md) を検討してください。

shadcn/ui の variant「default」「destructive」は、独自の名称を持ち込まず既存の status/variant 軸の組み合わせで表現します（Neutral + Surface が「default」相当、Error + Outline が「destructive」相当）。shadcn/ui の `AlertAction`（右上のアクション併記）に相当する構成は、本部品が独自に追加した pre-styled-only レイアウトパート `action` で表現します。`action` は root の末尾に置くだけでアクション（button 等）を右側（RTL では左側）へ寄せる表示専用パートで、クリック配線・送信処理は内包しません（呼び出し側が children に渡すノードへ配線します）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
