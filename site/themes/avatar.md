# Avatar

`fandhe-frontend-pre-styled-ui` の `avatar` mod が提供するスタイル済み Avatar 部品です。

ユーザーやチームの識別に使うアイコン画像・イニシャル表示部品です。ImageStatus（Loading/Loaded/Error）に応じて image パーツと fallback パーツの表示・非表示を CSS の data-state セレクタで切り替えます。

size（Xs〜Xl）・shape（Circle/Rounded/Square）・variant（Subtle/Solid/Outline）・colorPalette（6 値、既定 Neutral）の 4 軸を持ちます（イシュー #1554 で参照サイト〔chakra-ui/Radix Themes〕基準へ調整）。

イシュー #2044 で shadcn/ui と突合し、`group`（複数の Avatar を重ねて表示す
るレイアウト専用パート）と `badge`（右下に配置する状態ドット）を pre-styled-
only パートとして補完しました。`group` は `stacked: true` にした Avatar を
並べて重なり表示を作り、残数表示（`+N`）は独立したパートを新設せず
`root(stacked) + fallback("+3")` の組み合わせで表現します。`badge` は
`with_badge: true` にした Avatar の右下へ絶対配置され、`AvatarBadgeProps`
で `size`（既定 Md）・`palette`（既定 Accent）を指定します。

`Avatar` の Demo は `ImageStatus` を固定し、画像読み込み成功時の表示と
フォールバック（イニシャル等）表示の両方、variant・colorPalette の組み合わせ、
および `group`/`badge` の合成パターンを掲示しています。

イシュー #3130 で 3 点を opt-in で追加しました。`badge` は
`AvatarBadgeProps::placement`（`AvatarBadgePlacement`、既定 `BottomEnd`）で
右下/右上を切り替えられます。`group` の重なり順は `group_with` +
`AvatarGroupProps`（`AvatarGroupStacking`、既定 `LastOnTop`=DOM 順）で、
`FirstOnTop` にすると先頭の Avatar が最前面になります（既存の `group` は
シグネチャ・出力とも変更していません）。共通 `Size` enum（最大 56px）を
超える大サイズは新規 API を追加せず、`root` 呼び出し側の `attrs` へ
inline `style` を渡して上書きします。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
