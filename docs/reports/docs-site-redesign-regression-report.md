# 3 カラム刷新の回帰検証レポート（イシュー #912）

## 1. 目的とトレーサビリティ

- イシュー #912「test(docs-site): ダークモード・View Transitions・SkipNav・
  レスポンシブの回帰検証」（親 #903「Phase 4」/ ルート #899）は、`crates/docs-site/`
  の 3 カラム刷新（#905〜#911, #916, #918〜#920）がサイト全体で退行していないこと
  を検証するタスクである。設計文書 `docs/design/docs-site-three-column-redesign.md`
  §6「回帰検証の観点（→ #912）」が示す 5 観点（ダークモード・View Transitions・
  SkipNav・レスポンシブ・見出しアンカー）に対応する。
- 検証・是正は origin/main の最新コミット `daedf7c`（#919「ヘッダー刷新」・
  #920「右カラム目次」まで反映済み）を起点とした専用 worktree（ブランチ
  `test/912-docs-site-regression`）で実施した。

| イシュー受け入れ条件 | 設計文書 §6 の観点 | 対応節 |
|---|---|---|
| ライト / ダーク両モードで全ページ種別（index / ガイド / ショーケース / API リファレンス）の表示が確認されている | ダークモード | §4.1（機械）・§4.2（実機、環境制約あり） |
| SkipNav・見出しアンカー・View Transitions が退行していない | SkipNav・見出しアンカー・View Transitions | §4.1（機械）・§4.2（実機、環境制約あり） |
| `cargo test -p fandhe-frontend-docs-site` を含む関連テストが通る | — | §4.1 |
| （レスポンシブは受け入れ条件本文に明記はないが設計文書 §6 の観点） | レスポンシブ | §4.1（機械）・§4.2（実機、環境制約あり） |

## 2. 判定ステータス

**制約付き Pass**。機械検証（`cargo test`/`cargo clippy`/`cargo fmt`）はすべて
Pass し、指摘に対する是正（ハードコード色 1 件）も実施済み。一方で実ブラウザに
よる視覚確認（Step 6、設計文書 §6 の「実機のみで確認可能」な項目）は、本実行
環境（後述 §5）で Chromium を起動できないため **実施不能** だった。この制約は
下記 §6 で明示し、機械検証で担保できる範囲を明確に切り分ける。

## 3. 判定基準

### 3.1 機械検証（本イシューで新設・強化したテスト）

| # | テスト | 対応観点 |
|---|---|---|
| 1 | `site_theme::tests::every_referenced_fandhe_token_is_defined` | ダークモード（トークン網羅） |
| 2 | `site_theme::tests::color_and_shadow_tokens_are_defined_in_all_three_mode_blocks` | ダークモード（3 ブロック網羅） |
| 3 | `site_theme::tests::stylesheet_declares_color_scheme_light_dark` | ダークモード（OS 追従） |
| 4 | `site_theme::tests::structural_and_typography_css_contain_no_hardcoded_colors` | ダークモード（ハードコード色ガード） |
| 5 | `site_theme::tests::stylesheet_base_breakpoint_matches_responsive_contract` | レスポンシブ（基底帯域） |
| 6 | `site_theme::tests::stylesheet_media_queries_are_ordered_mobile_first` | レスポンシブ（カスケード順） |
| 7 | `layout_render::docs_page_emits_view_transition_opt_in_style_in_head` | View Transitions |
| 8 | `layout_render::docs_page_with_assets_emits_view_transition_opt_in_style_in_head` | View Transitions |
| 9 | `layout_render::docs_page_skip_nav_link_href_matches_content_target_id` | SkipNav |
| 10 | `layout_render::docs_page_skip_nav_link_is_first_focusable_element_in_body` | SkipNav |
| 11 | `site_build::real_site_build_covers_all_page_kinds_with_shared_layout_contract` | 全観点（4 ページ種別横断） |

既存テスト（234 件、`stylesheet_defines_docs_accent_bg_token_in_light_dark_and_theme_attr_blocks`
等）は 1 件も削除・弱体化していない。

### 3.2 §3.3 の「狭幅で右目次が非表示になること」の許容可否

設計文書 §3.3 は「`<1200px` で右目次カラムが `display: none` になりページ内
目次への到達手段が失われる点は #912 で許容可否を確認する」としている。

**判定: 許容する。** 見出し自体は本文中に残るため情報は失われず、失われるのは
ページ内ナビゲーションの affordance のみである。加えて `<768px` では左ナビ
折りたたみトグルがセクション間移動手段を提供する。追加実装（狭幅での目次
`<details>` 化等）は本イシューでは行わない。

## 4. 実測結果

### 4.1 機械検証

#### 4.1.1 ベースライン確認（変更前）

```
$ cargo test -p fandhe-frontend-docs-site
test result: ok. 79 passed; ...
test result: ok. 5 passed; ...
test result: ok. 20 passed; ...
test result: ok. 95 passed; ...
test result: ok. 9 passed; ...
test result: ok. 8 passed; ...
test result: ok. 7 passed; ...
test result: ok. 2 passed; ...
test result: ok. 8 passed; ...
test result: ok. 1 passed; ...
```

合計 234 件、全 green（既存不良ゼロ）を確認してから着手した。

#### 4.1.2 追加後の全テスト

```
$ cargo test -p fandhe-frontend-docs-site
test result: ok. 85 passed; ...   # site_theme lib（+6）
test result: ok. 5 passed; ...
test result: ok. 24 passed; ...   # layout_render（+4）
test result: ok. 95 passed; ...
test result: ok. 10 passed; ...   # site_build（+1）
test result: ok. 8 passed; ...
test result: ok. 7 passed; ...
test result: ok. 2 passed; ...
test result: ok. 8 passed; ...
test result: ok. 1 passed; ...
```

合計 245 件（234 + 11）、全 green。既存テストの削除・`#[ignore]` 化はゼロ。

```
$ cargo test -p fandhe-frontend-docs-site -p fandhe-frontend-pre-styled-ui
（docs-site 全件 + pre-styled-ui 1286+α 件、すべて green）
```

#### 4.1.3 lint

```
$ cargo fmt --all --check   # 差分なし
$ cargo clippy -p fandhe-frontend-docs-site --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.66s   # 警告ゼロ
```

#### 4.1.4 実サイトビルド（linkcheck 内蔵・fail-closed）

```
$ cargo run --locked -p fandhe-frontend-docs-site -- --out target/tmp/docs-site-pages/fandhe-frontend
fandhe-frontend-docs-site: wrote 22 page(s) and 4 asset(s) to target/tmp/docs-site-pages/fandhe-frontend
```

22 ページ・4 アセットがエラーなく生成された（内蔵 linkcheck が全内部リンクを
検証済み、`build_site` の fail-closed 契約により 1 件でも壊れたリンクがあれば
非ゼロ終了・出力なしとなる）。

### 4.2 実ブラウザ検証（環境制約により実施不能）

Playwright MCP（`browser_navigate`）で Chromium を起動しようとしたところ、
`browser_install` が以下のエラーで失敗した。

```
Error: ERROR: Playwright does not support chromium on ubuntu26.04-x64
```

本実行環境（後述 §5）は Playwright が対応する OS 一覧に含まれておらず、
ブラウザのインストール自体が不可能だった。したがって以下は **本レポートでは
実施不能** であり、機械検証（§4.1）が担保する範囲を超える確認事項として
未検証のまま記録する（§6 参照）。

- 4 ページ種別 × 3 ブレークポイント × light/dark の視覚確認
- 左ナビ折りたたみ（キーボード操作）の実機確認
- ヘッダードロップダウンの `:focus-within`/`:hover` 実機確認・右端はみ出し確認
- 見出しアンカーの sticky ヘッダー回避の実機確認
- View Transitions のページ遷移・コンソールエラー確認

## 5. 実行環境

- OS: Linux 7.0.0-27-generic（`ubuntu26.04-x64` として Playwright に検出された
  ディストリビューション。詳細なディストリ名は本レポートの機微情報記載方針
  により省略）
- Rust: リポジトリの `rust-toolchain` / `Cargo.lock` に準拠（`cargo test`/`clippy`/
  `fmt` はすべて成功）
- ブラウザ: 未インストール（Playwright 未対応、§4.2 参照）
- 配信手順（実施できた範囲）: `python3 -m http.server 8765 --bind 127.0.0.1 --directory target/tmp/docs-site-pages`
  → `http://127.0.0.1:8765/fandhe-frontend/`（`base_path = "/fandhe-frontend"` の
  罠を避けるため、生成物の**親ディレクトリ**を配信ルートにした。`curl` で
  `index.html` が HTTP 200 で取得できることのみ確認済み）

## 6. 保留・環境制約事項（必須）

- **実ブラウザ確認が本実行環境で不可能だった**（§4.2）。Playwright が
  `ubuntu26.04-x64` 上の Chromium をサポートしないため、`browser_install` が
  fail-closed でエラー終了した。ダークモード・レスポンシブ・SkipNav・見出し
  アンカー・View Transitions の **視覚的・対話的な実機確認は本 PR の範囲では
  行えていない**。機械検証（§4.1・§3.1 の 11 テスト）が担保する範囲（CSS
  トークン定義・DOM 構造・href/id 対応・opt-in 宣言の存在）に限定して Pass と
  判定する。実機確認が可能な環境（Playwright 対応 OS・Chromium 導入済み
  runner 等）での追試を推奨する（下記 out-of-scope 提案は行わない。本 PR の
  受け入れ判定はこの制約を明示した上での機械検証ベースの Pass とする）。
- `@media (prefers-color-scheme: dark)` は Playwright MCP のツール集合に
  `browser_emulate_media` が存在しないため、いずれにせよエミュレートによる
  視覚確認はできない設計上の制約だった（実機確認ができていれば
  `:root[data-theme="dark"]` 経路のみ視覚確認し、`prefers-color-scheme` 経路は
  §3.1 のトークン網羅スイープでのみ担保する計画だった）。
- View Transitions はアニメーション自体を機械アサートしておらず、opt-in 宣言
  （`@view-transition { navigation: auto; }`）の存在のみを固定している。遷移の
  実際の視覚効果・ページ遷移成功・コンソールエラー無しの確認は §4.2 と同じ
  理由で未実施。
- `base_path` プレフィックス（`/fandhe-frontend`）の都合で、配信ルートを
  生成物の親ディレクトリにする必要がある（`<out>/fandhe-frontend/` を生成し
  `<out>/` を配信ルートにする）。素の `<out>/` 配信では全アセット・全内部
  リンクが 404 になり「ダークモードが壊れている」ように誤読しやすいため、
  再現手順として明記する。

## 7. 参照

- 設計文書: `docs/design/docs-site-three-column-redesign.md` §3.2/§3.3/§5/§6
- 関連イシュー: #899（ルート）/ #903（親 Phase 4）/ #905・#907〜#911・#916・
  #918〜#920（対象の刷新一式）
- 関連テスト:
  - `crates/docs-site/src/site_theme.rs`（tests mod、6 件追加 + box-shadow 是正 1 行）
  - `crates/docs-site/tests/layout_render.rs`（4 件追加）
  - `crates/docs-site/tests/site_build.rs`（1 件追加）
