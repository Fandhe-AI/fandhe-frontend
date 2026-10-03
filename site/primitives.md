# Primitives コンポーネント索引

対応クレートは `fandhe-frontend-headless-ui` です。同クレートが提供する
構造（anatomy）・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態
（`data-*`）のみを持つ unstyled な headless UI コンポーネント（Primitives）の
索引ページであり、各部品は個別ページ（`/primitives/<kebab-name>/`）へ
分解されています。

Primitives は見た目のスタイルを持ちません。スタイル済みの表示例が必要な場合は
`fandhe-frontend-pre-styled-ui` が提供する [Themes](./themes.md) セクションを
参照してください（Themes は Primitives の上に見た目のスタイルを重ねた層です）。

> [!NOTE]
> 本ページは repo main（開発中の最新コード）を対象としています。crates.io
> 公開版のモジュール収録状況が本ページと異なることがあります。実際に
> インストールしたバージョンの収録内容は
> `https://docs.rs/fandhe-frontend-headless-ui/<version>` で確認してくだ
> さい（詳細は `docs/api/pre-styled-ui-api.md` §2a）。
Primitives ページには CSS 変数表がありません（headless-ui に CSS の概念が
無いため）。また Anatomy・`data-*` 属性表は Demo の実レンダリングから機械
導出しているため、その部品のデモに現れないパーツ・属性は表に出ず、部品に
よっては節ごと省略されます（未実装・不具合ではありません。詳細は
`docs/design/docs-site-component-pages.md` §7b を参照してください）。

各部品ページの Demo 節にある枠線・余白は docs サイト側が付与したデモ枠で
あり、`fandhe-frontend-headless-ui` 自体は見た目のスタイルを一切持ちません
（Themes セクションのスタイル済み recipe とは無関係のデモ表示用の枠です）。

## 関連 API

- [fandhe-frontend-headless-ui API](../docs/api/headless-ui-api.md): headless API（anatomy・data-*・WAI-ARIA 契約）
