# fandhe-frontend-pre-styled-ui API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-pre-styled-ui`（chakra-ui 参考の
pre-styled UI コンポーネント層、親トラッキング #520・骨格新設 #546）の
公開 API 表面をまとめる。`fandhe-frontend-headless-ui`（ark-ui 相当の下層、
[`docs/api/headless-ui-api.md`](./headless-ui-api.md)）の上に、テーマ
トークン・variant API・静的 CSS 生成を重ね、styled 部品を実装する 2 層
構造の上層を担う。

**spec 未反映の注記**: `fandhe-frontend-headless-ui` と同様、本クレートに
対応する REQ / TASK は `docs/spec/` に存在しない（要件提案は
fandhe-frontend-spec リポジトリの Issue #20 として起票済み、#520 参照）。

## 2. 実装状況（本書作成時点、2026-07-22）

本クレートは **crate 骨格のみ**（イシュー #546）であり、公開 API を持たない
（`src/lib.rs` はクレート doc コメントのみ）。以下は並列進行中のイシューで
あり、本書はそれらのマージ後に更新する。

| イシュー | 内容 | 状態（本書作成時点） |
|---|---|---|
| #547 | テーマトークン・ダークモード基盤 | 実装中（未マージ） |
| #548 | slot recipe 相当の variant API・静的 CSS 生成 | 実装中（未マージ） |
| #550 | Button 等の単純な styled 部品 | 未着手 |
| #551 | headless-ui ラッパー（Accordion/Dialog 等の styled 版） | 未着手 |

`examples/headless-pre-styled-ui`（#552）は本クレートが未実装のため、
headless-ui の `data-scope`/`data-part`/`data-state` セレクタへ手書きで
当てる CSS（`examples/headless-pre-styled-ui/static/ui.css`）を暫定的な
代替として同梱している。本クレートの公開 API が揃い次第、同サンプルへの
統合をフォローアップする。

## 3. 不変条件（実装済み・骨格に記載済み、`src/lib.rs` 参照）

1. コンポーネントは `fandhe_frontend_headless_ui` 経由で
   `fandhe_frontend_core::Node` を返す通常の Rust 関数として実装する
   （REQ-5、マクロ DSL は採用しない）。
2. 出力は `fandhe_frontend_core::render` の既定エスケープを必ず経由する。
   本クレート内で `raw_html()` を使用しない（新たなエスケープ迂回経路を
   作らない）。
3. `#![forbid(unsafe_code)]`（REQ-2）によりクレート全体で `unsafe` を機械的
   に禁止する。
4. 外部依存は `fandhe-frontend-headless-ui`（path）のみ。
   `fandhe-frontend-core` への直接依存は宣言しない（headless-ui 経由で
   間接的に利用する。`fandhe-frontend-core` はスモークテスト用の
   dev-dependency としてのみ許容する）。

これらの不変条件は #547/#548/#550/#551 の実装レビューでもそのまま適用される
（`.claude/rules/coding-rust.md`・`docs/api/headless-ui-api.md` §6 と同一の
制約を上層でも維持する）。

## 4. 設計方針（予定、#547/#548 の実装完了後に本節を更新）

- **テーマトークン**（#547）: 色・スペーシング等のデザイントークンと
  ダークモード切り替えの基盤。chakra-ui の `system`/`recipe` 相当の設計を
  参考にしつつ、静的 SSR 出力（ビルド時に確定する CSS）を前提とする。
- **variant API・静的 CSS 生成**（#548）: chakra-ui の slot recipe 相当。
  コンポーネントの見た目バリエーション（size/variant/colorPalette 等）を
  型安全に選択し、対応する静的 CSS を生成する。
- **styled 部品**（#550/#551）: #550 は Button 等の単純な部品、#551 は
  headless-ui の Accordion/Dialog 等をラップした styled 版を提供する予定。

## 5. 関連ドキュメント

- [`docs/api/headless-ui-api.md`](./headless-ui-api.md): 本クレートの下層
- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレート未実装時点での暫定サンプル（pre-styled-ui 統合について節参照）
- `.claude/skills/chakra-ui/`: 設計時の参考にした chakra-ui リファレンス
  スキル
