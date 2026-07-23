# docs サイト骨格への pre-styled-ui 適用可否の評価と判断記録

**本文書のステータス**: 確定（イシュー #694）。

## 1. 背景

PR #679（pre-styled-ui ショーケースの GitHub Pages 統合、イシュー #609 系）
は、pre-styled-ui コンポーネントを実レンダリングして掲載するショーケース
ページ（`/components/pre-styled-ui/`、`crates/docs-site/src/showcase.rs`）を
docs サイトへ追加した。その実装過程で、docs サイト骨格そのもの（Linear 風
2 カラムレイアウト、`crates/docs-site/src/layout.rs` / `nav.rs` /
`markdown.rs` が生成する class 契約、`site/assets/site.css`）へも
pre-styled-ui の styled 部品・テーマトークンを適用するかが検討されたが、
「`site_css_contract` と見た目を壊すリスクに対して利得がない」という判断
がエージェント報告にのみ残り、リポジトリ内の恒久文書には記録されていな
かった（暗黙の見送り）。

本書は `docs/policy/intentional-non-adoption.md` と同趣旨で、この判断を
評価軸とともに恒久記録し、再評価トリガーを明示することでイシュー #694 の
「暗黙の見送り」状態を解消する。

## 2. 評価対象と評価軸

サイト骨格（`site/assets/site.css` が固定するクラス名契約、
`crates/docs-site/tests/site_css_contract.rs` が検証する実出力）への
pre-styled-ui 適用候補を、以下 4 軸で評価する。

- **利得**: 適用によって得られる視覚的・保守上の改善
- **`site_css_contract` への影響**: `layout.rs` / `nav.rs` / `markdown.rs`
  が生成する class 契約・CSS セレクタの作り替えが必要か
- **意味論整合**: pre-styled-ui 部品の WAI-ARIA ロール・anatomy が、適用先
  の文書サイト要素の意味論と一致するか（誤適用はアクセシビリティを毀損する）
- **保守コスト**: 二重管理（`--docs-*` トークンと `--fandhe-*` トークン等）
  が生じないか

## 3. 適用候補ごとの評価

### 3.1 サイドバーナビ（`nav.rs::sidebar`）

pre-styled-ui には文書ナビゲーション（アプリ内固定サイドバーのリンク一覧）
に相当する部品が存在しない。最も近い `menu` 部品は WAI-ARIA `menu` ロール
（キーボード操作を伴う操作ドロップダウン・コマンドリスト向け）であり、
`nav` 要素 + リンクリストという文書ナビの意味論とは異なる。`menu` ロールを
文書ナビへ転用すると、スクリーンリーダー利用者に「操作可能なメニュー」と
誤って伝わりアクセシビリティを毀損する。

**判断**: 見送り（意味論不整合）。

### 3.2 前後ページャ（`nav.rs::prev_next_nav`、`nav.prev-next` のカード風リンク）

pre-styled-ui の `card` 部品は `data-scope` anatomy を持つ `div` ベースの
コンテンツカードであり、アンカー要素全体をカード化するリンク部品ではない。
適用するには `site_css_contract.rs` が固定する class 契約（`prev-next` /
`prev` / `next`）自体の作り替えが必要になり、見た目上の利得もない。

**判断**: 見送り（`site_css_contract` への影響が大きく利得なし）。

### 3.3 注記ブロック（`alert` 部品）

**導入済み（イシュー #715）**。当初（イシュー #694 時点）は `markdown.rs`
が admonition（注記）構文を持たず、`> ...` はそのまま素の `blockquote` と
して出力されるのみだった。`alert` 部品を適用するには Markdown 側に新しい
拡張構文（例: `> [!NOTE]`）を新設する必要があり、これはイシュー #694 の
スコープ（サイト骨格への適用可否評価）を超えるとして見送っていた
（§5 再評価トリガー 2）。

イシュー #715 でこの拡張自体の導入可否を評価した結果、サイト掲載
19 ページの棚卸しで「意味的に admonition である」ラベル付き注記が
6 ブロック実在すること、GFM alerts（`> [!NOTE]` 等）が GitHub 上でも
ネイティブ描画されリポジトリ直読みとサイト描画の意味が保たれること、
構文が行頭マーカーの固定文字列一致で決定的にパース可能（AI 保守前提の
評価軸を満たす）ことから、**導入する**と判断した。実装は
`crates/docs-site/src/markdown.rs`（GFM alerts 5 種のマーカー判定・
`AlertStatus` への固定マッピング）・`crates/docs-site/src/admonition.rs`
（専用 CSS の分離配線）で、3.5 のショーケースページと同じ「分離 CSS
方式」を踏襲し `site.css` のカスケードには影響させていない。

**判断**: 導入する（詳細設計・実装は `crates/docs-site/src/markdown.rs`・
`crates/docs-site/src/admonition.rs` の rustdoc・イシュー #715 実装計画を
参照）。

### 3.4 テーマトークン（`--fandhe-*` の `site.css` への波及）

`site/assets/site.css` は `--docs-*` トークンで自己完結する単一の静的
ファイルであり、「外部参照ゼロ・単一ファイルのみでレイアウト・タイポグ
ラフィが完結する」ことを不変条件とする（ファイル冒頭コメント参照）。
`fandhe_frontend_pre_styled_ui::theme::Theme::to_css` の出力を注入するには
`site.css` をビルド生成物化する必要があり、「静的ファイルを読み取り検証
する」という `site_css_contract.rs` の前提、および「`site.css` が正」と
いう契約の作り替えを要する。加えて Linear 風の既存配色（`--docs-*`）と
pre-styled-ui のトークン体系（`--fandhe-*`）が二重管理になり、保守コスト
が増す一方で視覚的な統一以外の利得がない。

**判断**: 見送り（不変条件の作り替えを要し、利得に対してコストが見合わない）。

### 3.5 ショーケースページ・admonition（適用済み範囲）

PR #679 で `/components/pre-styled-ui/` ページに適用済み。ショーケース
専用の分離 CSS（`assets/pre-styled-ui.css`、`crates/docs-site/src/showcase.rs::stylesheet`）を `crate::layout::docs_page_with_assets` の
追加 `<link>` で読み込む方式を採り、`site.css` のカスケードへは一切
影響させていない。

イシュー #715 で導入した admonition（3.3）も同型の適用境界を踏襲する。
専用の分離 CSS（`assets/admonition.css`、
`crates/docs-site/src/admonition.rs::stylesheet`）を admonition を実際に
含むページにのみ `docs_page_with_assets` の追加 `<link>` で読み込ませ、
含まないページ・フィクスチャサイトのビルド結果は変えない
（`crates/docs-site/src/build.rs` の配線）。

これらが現時点で pre-styled-ui を docs サイトへ組み込む「適用境界」の
全体であり、本書が定める見送り判断（3.1・3.2・3.4）はこの適用境界を
変更しない。

## 4. 結論

サイト骨格（Linear 風 2 カラムレイアウト・`site.css`・ナビゲーション
生成ロジック）への pre-styled-ui の styled 部品・テーマトークンの適用は、
3.1・3.2・3.4 は見送る。3.3（注記ブロック）はイシュー #715 で導入済み。
適用範囲は 3.5 のショーケースページ・admonition（いずれも分離 CSS 方式）
に限定したまま維持する。

## 5. 再評価トリガー

以下のいずれかが発生した場合、本書の判断を再評価する。再評価提案は
`docs/policy/intentional-non-adoption.md` の運用（再導入提案時は評価軸の
充足確認を Issue・PR に明記する）に準拠すること。

1. pre-styled-ui にレイアウト・ナビゲーション系部品（Breadcrumb /
   Pagination / 文書ナビ向け Link リスト / Container 等）が追加されたとき
   （3.1・3.2 の意味論不整合が解消され得る）
2. ~~docs-site の Markdown レンダラへ admonition（注記）構文を追加すると
   き（3.3 の `alert` 適用を再評価する）~~ → イシュー #715 で消化済み
   （3.3 参照。admonition 構文を追加し `alert` 部品での描画を導入した）
3. サイト骨格（Linear 風 2 カラムレイアウト・`site.css`）の大規模リ
   デザインを行うとき（3.4 のトークン波及コストの前提が変わり得る）
4. `--docs-*` と `--fandhe-*` のトークン二重管理が実際の同期不具合・
   保守コストを生んだとき（3.4 の「利得なし」判断の再検証が必要になる）

## 6. 関連文書

- `docs/policy/intentional-non-adoption.md`: AI 開発・保守前提の評価軸
  （明示性・決定性・機械検証可能性・コンテキスト消費）と非採用記録の
  運用ルール本体
- `crates/docs-site/src/showcase.rs`: 適用済み範囲（ショーケースページ）
  の実装
- `crates/docs-site/src/markdown.rs` / `crates/docs-site/src/admonition.rs`:
  イシュー #715 で導入した admonition 構文のパース・alert 部品への描画・
  専用 CSS の分離配線の実装
- `crates/docs-site/tests/site_css_contract.rs`: `site.css` とレイアウト
  生成ロジックの class 契約を検証する回帰テスト（admonition の
  `fd-alert--status-*` は `admonition::stylesheet()` 側の契約であることの
  ドリフト検知も含む）
