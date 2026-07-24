# docs サイト骨格への pre-styled-ui 適用可否の評価と判断記録

**本文書のステータス**: 確定（イシュー #694）。§3.4・§5 はイシュー #904
で再評価済み（詳細は各節参照）。

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

### 3.1 サイドバーナビ（`nav.rs::sidebar`）【解消済み（イシュー #756）】

（当初の判断、イシュー #694 時点）pre-styled-ui には文書ナビゲーション
（アプリ内固定サイドバーのリンク一覧）に相当する部品が存在しない。最も
近い `menu` 部品は WAI-ARIA `menu` ロール（キーボード操作を伴う操作
ドロップダウン・コマンドリスト向け）であり、`nav` 要素 + リンクリストと
いう文書ナビの意味論とは異なる。`menu` ロールを文書ナビへ転用すると、
スクリーンリーダー利用者に「操作可能なメニュー」と誤って伝わり
アクセシビリティを毀損する。

**当初の判断**: 見送り（意味論不整合）。

**再評価（イシュー #756）**: `docs/api/headless-ui-api.md` §4b の検討で
「文書ナビ向け Link リスト」を最優先の追加候補と判断し、`role` を一切
付与しない専用 headless 部品 `nav_list`（`crates/headless-ui/src/nav_list.rs`）
+ styled ラッパー（`crates/pre-styled-ui/src/nav_list.rs`）を新設した。
`nav.rs::sidebar` は本部品（`heading`/`list`/`item`/`link` の headless
自由関数、`root` は headless 直接呼び出しで `class="sidebar"` を温存）へ
移行済みであり、意味論不整合は解消した。視覚スタイルは §3.4 の不変条件
どおり `site.css` の既存タグ・class セレクタが変更なしで適用され続ける
（`nav_list` は class を持たない素の `nav`/`h2`/`ul`/`li`/`a` を出力する
ため）。現在ページの表現は `class="current"` を廃止し `aria-current="page"`
（+ `data-current`）のみへ一本化した。

**判断**: 導入する（実装詳細は `crates/docs-site/src/nav.rs::sidebar` の
rustdoc・イシュー #756 実装計画を参照）。

### 3.2 前後ページャ（`nav.rs::prev_next_nav`、`nav.prev-next` のカード風リンク）【解消済み（イシュー #756）】

（当初の判断、イシュー #694 時点）pre-styled-ui の `card` 部品は
`data-scope` anatomy を持つ `div` ベースのコンテンツカードであり、アンカー
要素全体をカード化するリンク部品ではない。適用するには
`site_css_contract.rs` が固定する class 契約（`prev-next` / `prev` /
`next`）自体の作り替えが必要になり、見た目上の利得もない。

**当初の判断**: 見送り（`site_css_contract` への影響が大きく利得なし）。

**再評価（イシュー #756）**: `docs/api/headless-ui-api.md` §4b の検討で
chakra-ui の LinkBox/LinkOverlay パターンに倣った `link_overlay`
（`crates/headless-ui/src/link_overlay.rs`、Root/Overlay の 2 anatomy）+
styled ラッパー（`crates/pre-styled-ui/src/link_overlay.rs`）を新設した。
`nav.rs::prev_next_nav` は `div.prev`/`div.next`（`link_overlay::root`）配下に
`[data-part="overlay"]`（`link_overlay::overlay`）を持つ構造へ移行し、
カード全面クリック化の意味論を LinkOverlay 部品で表現するようになった
（本ページの用途では overlay がカードの唯一の子でありフロー内で全面を
占めるため、`link_overlay` の一般的な `position: absolute` 拡張パターンは
使わず、overlay 自体に従来のカード CSS をそのまま当てる。`site_css_contract`
が固定する class 契約は `prev-next`/`prev`/`next` のまま変更なし、属性
セレクタ `[data-part="overlay"]` は同テストの class 抽出対象外）。

**判断**: 導入する（実装詳細は `crates/docs-site/src/nav.rs::prev_next_nav`
の rustdoc・イシュー #756 実装計画を参照）。

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

**当初の判断**: 見送り（不変条件の作り替えを要し、利得に対してコストが
見合わない）。

**再評価（イシュー #904）**: 親トラッキング #899（GitHub Pages デザイン
刷新・3 カラム化）を受け、§5 再評価トリガー 3「サイト骨格の大規模リ
デザインを行うとき」が発動した。前提が「大規模リデザインを行わない」
から「行う」へ変わったため、評価軸 4 点を再充足確認する。

- **利得**: 大規模リデザイン（3 カラム化、`docs/design/docs-site-three-column-redesign.md`
  参照）では骨格 CSS をどのみち全面的に書き直す。このとき
  `fandhe_frontend_pre_styled_ui::theme::Theme::to_css` が既に提供する
  トークン体系（色・余白・タイポグラフィ）とダークモード基盤
  （`prefers-color-scheme` メディアクエリ + `data-theme` 属性上書きの
  両対応）を流用できれば、同等の機能を `--docs-*` として独自に再実装
  するコストを避けられる。当初評価時点（イシュー #694）には無かった
  「どのみち書き直す」という前提が新たに成立したことで、利得が実際に
  発生する状況になった。
- **`site_css_contract` への影響**: class 契約自体は 3 カラム化
  （`docs-toc-aside`・`docs-header-nav` 等の新設 class、
  `docs-site-three-column-redesign.md` §3.1 参照）に伴っていずれ作り
  替えが必要になる。当初評価が「作り替えコスト」として計上していた分は、
  3 カラム化それ自体で発生する作り替えコストに吸収され、テーマトークン
  波及固有の**限界的コスト**（テスト取得元の差し替えのみ、同文書 §5）
  に縮小した。
- **意味論整合**: テーマトークンは色・余白・タイポグラフィという視覚
  表現のみを提供し、WAI-ARIA ロール・anatomy には一切関与しない。3.1
  （`nav_list`）・3.2（`link_overlay`）・3.3（`alert`）が対処した
  「部品の意味論を誤って持ち込む」リスクとは性質が異なり、当初評価から
  不変（毀損リスクなし）。
- **保守コスト**: `--docs-*` と `--fandhe-*` の二重管理は §5 トリガー 4
  が警告した同期不具合の温床である。3 カラム化を機に `--fandhe-*` へ
  一本化すれば、二重管理そのものが発生源から解消される（トリガー 4 が
  懸念する事態を「実際に起きてから対処する」のではなく「そもそも起こ
  らない設計にする」ことで先回りして解消する）。

**結論**: 見送り → **導入へ転換**。実装は Phase 2（イシュー #905、
`docs/design/docs-site-three-column-redesign.md` §4「CSS 供給方式」）で
行う。CSS 供給方式（`Theme::to_css` + 骨格 CSS を `StyleSheet` で組み立て
`build.rs` から `write_css_file` する方式）・`site_css_contract.rs` の
契約作り替え方針（同文書 §5）・トークン一本化の詳細は同文書に譲る。

**実装状態の注記**: 本イシュー（#904）時点ではコードは変更していない。
`site/assets/site.css` は静的単一ファイルのまま、`crates/docs-site/src/layout.rs`
は 2 カラム骨格のまま据え置かれている。導入の実装完了は Phase 2〜4
（#905〜#913）の完了をもって確定する。

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

これらに加え、イシュー #756 で 3.1（`nav_list` headless 部品）・3.2
（`link_overlay` headless 部品）を導入済みである（headless 部品の
markup 適用のみで、視覚スタイルは §3.4 の不変条件どおり `site.css` が
自己完結したまま担う。pre-styled-ui の styled `stylesheet()`/生成 CSS は
どちらも導入しない）。3.4（テーマトークン波及）は本イシューでも見送りの
まま維持する。

## 4. 結論

サイト骨格（Linear 風 2 カラムレイアウト・`site.css`・ナビゲーション
生成ロジック）への pre-styled-ui 適用は、3.1・3.2 はイシュー #756 で
「headless 部品による markup 導入（視覚スタイルは `site.css` 継続）」の
形で解消済み。3.3（注記ブロック）はイシュー #715 で導入済み。3.4
（テーマトークン波及）はイシュー #904 の再評価で見送りから導入へ転換
した（実装は Phase 2〜4、#905〜#913）。pre-styled-ui の styled 部品・
生成 CSS（`stylesheet()`）の適用範囲は、従来の 3.5（ショーケースページ・
admonition、分離 CSS 方式）に加え、3.4 の転換により**サイト骨格全体
（`site.css` 自体のビルド生成物化）へも拡張される**。適用範囲の統治は
本文書から `docs/design/docs-site-three-column-redesign.md` へ引き継ぐ
（同文書 §1 参照）。

## 5. 再評価トリガー

以下のいずれかが発生した場合、本書の判断を再評価する。再評価提案は
`docs/policy/intentional-non-adoption.md` の運用（再導入提案時は評価軸の
充足確認を Issue・PR に明記する）に準拠すること。

1. ~~pre-styled-ui にレイアウト・ナビゲーション系部品（Breadcrumb /
   Pagination / 文書ナビ向け Link リスト / Container 等）が追加されたとき
   （3.1・3.2 の意味論不整合が解消され得る）~~ → イシュー #756 で消化済み
   （3.1 参照。`nav_list`/`link_overlay` headless 部品を追加し
   `nav.rs::sidebar`/`nav.rs::prev_next_nav` へ適用、意味論不整合を
   解消した）
2. ~~docs-site の Markdown レンダラへ admonition（注記）構文を追加すると
   き（3.3 の `alert` 適用を再評価する）~~ → イシュー #715 で消化済み
   （3.3 参照。admonition 構文を追加し `alert` 部品での描画を導入した）
3. ~~サイト骨格（Linear 風 2 カラムレイアウト・`site.css`）の大規模リ
   デザインを行うとき（3.4 のトークン波及コストの前提が変わり得る）~~
   → イシュー #904 で消化済み（3.4 参照。親トラッキング #899 の 3 カラム
   化を受けて発動し、評価軸 4 点を再充足確認した結果「見送り→導入」へ
   転換した。実装詳細は `docs/design/docs-site-three-column-redesign.md`
   参照）
4. ~~`--docs-*` と `--fandhe-*` のトークン二重管理が実際の同期不具合・
   保守コストを生んだとき（3.4 の「利得なし」判断の再検証が必要になる）~~
   → イシュー #904 でトリガー 3 の消化に伴い併せて消化済み（3.4 参照。
   実際の同期不具合の発生を待たず、3 カラム化を機に `--fandhe-*` への
   一本化へ先回りして転換したため、二重管理そのものが発生しない設計に
   なった）

## 6. 関連文書

- `docs/design/docs-site-three-column-redesign.md`: イシュー #904 で
  作成した 3 カラム新レイアウトの設計文書。3.4 の導入転換を受けた
  CSS 供給方式・class 契約・breakpoint・契約テスト作り替え方針・
  ヘッダードロップダウンの意味論評価を確定する（Phase 2〜4、#905〜#913
  が参照する統治文書）
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
