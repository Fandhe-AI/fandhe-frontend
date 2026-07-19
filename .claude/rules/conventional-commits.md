# Conventional Commits 規約

## 形式

```
<type>(<scope>): <日本語の要約>

<本文（任意・日本語）>
```

## type

| type | 用途 |
|------|------|
| feat | 機能追加 |
| fix | バグ修正 |
| docs | ドキュメントのみの変更 |
| style | 動作に影響しない整形（rustfmt 適用等） |
| refactor | 機能変更を伴わない構造改善 |
| perf | 性能改善 |
| test | テストの追加・修正 |
| build | ビルドシステム・依存関係の変更 |
| ci | CI 設定の変更 |
| chore | その他（リポジトリ運用・設定） |

## scope

クレート・領域名を使う: `core` / `interactive` / `app` / `server` / `dist-server`（fandhe-frontend-dist-server: 単一バイナリ配布サーバー） / `wasm-client` / `wasm-full` / `xtask` / `spec` / `claude`（.claude 体系） / `global`（横断）

例:

```
feat(core): テキスト補間の既定エスケープを製品仕様として固定
fix(server): SSG 出力時のルーティング解決の不具合を修正
ci(xtask): 依存グラフ上限 (60件/深さ6) の自動計測を追加
```

## Breaking Change

- 破壊的変更は `!` を付ける（例: `feat(core)!: render() の戻り値型を変更`）か、フッターに `BREAKING CHANGE:` を記載する

## 厳守事項

- **`--no-verify` の使用禁止**（pre-commit フックを必ず通す）
- 1 コミット 1 論理変更。無関係な変更を混ぜない
- コミット前に staged 差分からシークレット混入（`.env`・トークン等）がないか確認する
- create-commit スキルを使用してコミットする
