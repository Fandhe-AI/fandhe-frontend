# ガイド一覧

Guides セクションは、目的別の実践ガイドをまとめています。コンポーネント
の書き方から既存ページへの部分埋め込み、ビュー遷移、静的アセットの取り
込みまで、実装時に必要な手順を個別ページで解説します。`fw new` からの
最短経路を知りたい場合は、まず[クイックスタート](../docs/guides/quickstart.md)
を参照してください。

## コンポーネント記述ガイド

[コンポーネント記述ガイドを見る](../docs/guides/component-authoring.md)

マクロ DSL に依存せず、純粋な Rust のノード木 API でコンポーネントを
記述する方法を解説します。既定エスケープを保ったままコンポーネントを
組み立てる基本パターンを扱います。

## 最小埋め込みガイド

[最小埋め込みガイドを見る](../docs/guides/embedding-guide.md)

既存の HTML ページの `<div>` へコンポーネントを部分的にマウントする
手順を解説します。同じ描画関数を SSR 構成と共有できる点が特徴です。

## View Transitions

[View Transitions ガイドを見る](../docs/guides/view-transitions.md)

クロスドキュメントおよび SPA 内でのビュー遷移を有効化する方法を解説
します。

## NPM アセットビルド

[NPM アセットビルドガイドを見る](../docs/guides/npm-asset-build.md)

`--ignore-scripts` を既定としたサプライチェーン対策付きの、静的アセット
取り込みパイプラインの利用方法を解説します。

## JS ゼロ SSG での利用ガイド

[JS ゼロ SSG での利用ガイドを見る](../docs/guides/no-js-ssg.md)

クライアント側 JavaScript（`fandhe-frontend-wasm-full` ハイドレーション）を
読み込まない静的サイト構成で、`fandhe-frontend-headless-ui` /
`fandhe-frontend-pre-styled-ui` の部品がどう振る舞うか、開閉 UI の代替
パターンを解説します。
