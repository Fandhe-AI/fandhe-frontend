# API リファレンス

API Reference セクションは、公開 API の仕様をクレート別にまとめています。
凍結された公開 API の表・呼び出し規約・セキュリティ不変条件はここに掲載
する `docs/api/` 配下のページが正です。issue/PR 番号やロードマップなど
の実装経緯・進行管理の記述は `docs/internal/` 配下の実装ノートへ分離して
おり、`site/nav.toml` に登録していないため本サイトには掲載していません
（本リポジトリは public のため「非公開」ではなく「サイト非掲載」です）。

## core（描画コア）

- [コンポーネント記述 API](../docs/api/component-api.md)

## app / server（アプリ構築・ルーティング）

- [fandhe-frontend-app API](../docs/api/app-api.md)
- [ルーター パスマッチング](../docs/api/router-path-matching.md)
- [fandhe-frontend-server SSG API](../docs/api/server-api.md)

## interactive（状態管理）

- [状態管理 API](../docs/api/interactive-api.md)

## wasm（CSR / ハイドレーション）

- [hydrate() API](../docs/api/hydration-api.md)
- [ハイドレーション状態フォーマット](../docs/api/hydration-state-format.md)

## headless-ui

- [fandhe-frontend-headless-ui API](../docs/api/headless-ui-api.md)

## pre-styled-ui

- [fandhe-frontend-pre-styled-ui API](../docs/api/pre-styled-ui-api.md)
- [pre-styled-ui slot recipe API](../docs/api/pre-styled-recipe-api.md)

## 部品ごとの詳細

部品ごとの Demo・Anatomy・`data-*` 属性表・利用例は、本セクションでは
なく Themes セクションの[Themes 索引](./themes.md)
配下にある部品ページ（`/themes/<kebab>/`。イシュー #1017 で
`/components/<kebab>/` から移行）が正です。
