# shadcn/ui Blocks の docs サイト取り込み方針

**本文書のステータス**: 確定（イシュー #2007。承認は本 PR のマージをもって成立）。

## 1. 背景・目的

イシュー #2001「shadcn/ui（components / blocks / charts）を 4 本目の参照軸として
UI 部品を見直し・追加する」の Phase 0（#2002）配下で、shadcn/ui Blocks
（dashboard / sidebar / login / signup）という「**既存部品の合成例**（新規
コンポーネントではない）」の置き場所が未確定のままだった。

置き場所の候補は 3 案ある。

- **案 A**: docs サイトに `/blocks/` セクションを新設し SSG で掲載する
- **案 B**: `examples/` 配下に独立したサンプルプロジェクトとして追加する
- **案 C**: 両方（docs サイト掲載 + `examples/` 追加）

調査の結果、この決定はすでに Phase 6（#2087、子イシュー #2088〜#2095）
として**案 A 前提でフル起票済み**であることを確認した。#2088〜#2095 は
いずれも「Phase 0 の blocks 方針イシューで案 A が採用されることを前提に
起票している。別案となった場合は `update-issue-tree` で本 Phase を是正
する」という同一の前提文言を持ち、対象 7 件（dashboard-01 / sidebar-07 /
sidebar-03 / login-01 / login-04 / signup-01 / signup-05）はイシュー #2007
起票時点の候補と完全一致する。

したがって本文書の役割は、新規の意思決定ではなく**すでに既定路線として
実装が進行しうる状態にある前提を正式な決定記録として文書化し、Phase 6 が
是正不要であることを確認する**ことにある。

## 2. 決定: 案 A（docs サイト `/blocks/` セクション）を採用する

理由:

- 既存の `site/nav.toml` + `crates/docs-site` の SSG 機構だけで完結し、
  `crates/cli/embedded-examples/` のバイト一致同期（`example_publish_copy_drift.rs`）
  や `fandhe-frontend-cli` の semver バンプ連鎖を誘発しない。これは
  `docs/ci/example-overlay-browser-interaction-testing-evaluation.md`（イシュー
  #1210）が examples 追加のコストとして既に評価した論点と同型である。
- docs サイトは無 JS が既定（`crates/docs-site/tests/no_js_contract.rs`）で
  あり、sidebar の開閉のような JS ハイドレーション前提の挙動は、静的な
  複数状態（例: expanded / collapsed の 2 ページ、または 1 ページ内で両状態
  を並べて表示する）として表現する制約を持つ。この制約は #2088 側の実装が
  従う前提として本文書で明記する。

案 B（`examples/` 追加）・案 C（両方）は不採用とする。

- 案 B は #1210 と同型の同期コスト（`embedded-examples` バイト一致・cli
  semver バンプ連鎖）を、新規コンポーネントを含まない合成例のためだけに
  払うことになり見合わない。
- 案 C は案 A・B 双方のコストを二重に負担するだけで、案 A 単独に対する
  追加の利用者価値がない。

## 3. 掲載対象は 7 件で確定する

dashboard-01 / sidebar-07 / sidebar-03 / login-01 / login-04 / signup-01 /
signup-05 の 7 件とする。これは既存の #2088〜#2095 が対象とする一覧と
完全に一致する。

**login-03 を対象外とする根拠**: `login-03` はスクリーンショット取得済み
（PR #2000、`docs/design/reference-screenshots/shadcn-block-login-03-1.png`）
であり、イシュー #2007 の本文中の参照画像としても使われた。しかし
構造的には `login-01`（単一カラムのシンプルなカード）とほぼ同型の合成
であり、7 件の代表セットが目指す「構造パターンの網羅（単一カラム／2 カラム
画像付き／アイコン折りたたみサイドバー／サブメニュー付きサイドバー／SNS
プロバイダ付きサインアップ）」に新規パターンを追加しない。したがって
`login-03` はビルド対象に含めない。この一文を Phase 7（#2096）の棚卸しが
参照する一次情報とする。

**Phase 7（#2530）による追加**: 親トラッキング #2530「Phase 7: Motion+
部品化」配下のイシュー #2547 により、Motion+ `sections/pricing-sections`
に相当する `pricing-tiers-morph`（月額/年額 billing 切替 + 3 段ティア
カードのクロスフェード）・`pricing-usage-slider`（利用量スライダーと
価格 `stat` の静的な組）の 2 件が追加された。これは上記「7 件で確定」の
判断（#2088〜#2095 の一覧が最終であるという記述）を変更するものではなく、
新規タスク（#2530 系）による純追加であることを示す別段落である。

**Phase 7（#2530）による追加（#2548）**: 同じ親トラッキング #2530 配下の
イシュー #2548 により、Motion+ の testimonials 系レイアウトを参照した
`testimonials-stack`（`card`/`blockquote`/`avatar` を合成し、積層した
testimonial カードを表す合成例）の 1 件が追加された。積層オフセットに
`pre-styled-ui::recipe::STAGGER_INDEX_VAR`（`--fandhe-motion-stagger-index`）
を用いる判断・自動ローテーションを行わない判断（docs-site は JS
ハイドレーションを行わない設計のため）は
`crates/docs-site/src/blocks/marketing/testimonial/testimonials_stack.rs`
のモジュール doc を正とする。これも上記と同じ純追加であり「7 件で確定」の
判断は変更しない。

**Phase 7（#2530）による追加（イシュー #2549）**: 同じ Motion+ `sections/
bento-grids` に相当する `bento-staggered`（scroll-driven な bento グリッド。
各セルが `animation-timeline: view()` でビューポート進入時にフェード＋
下方向スライドインし、`--fandhe-motion-stagger-index` を `animation-range`
の開始点オフセットへ乗せることで順送りに発火する）・`feature-expand`
（`grid-template-rows: 0fr → 1fr` の CSS のみで hover/`:focus-within` 時に
詳細説明を展開するカードグリッド）の 2 件が追加された。実装記録の詳細
（hover の分類・`content_height.rs` を使わない判断・stagger の表現手法）は
本文書 §14 に記す。

**Phase 7（#2530）による追加（#2550）**: 同じ親トラッキング #2530 配下の
イシュー #2550 により、Motion+ `sections/cta-sections` に相当する
`cta-banner-magnetic`（ポインタに追従して吸い付く magnetic ボタン付き
CTA バナー）・`cta-signup-celebrate`（送信完了で confetti が発火する
サインアップ CTA。既存の confetti 機構をそのまま合成する「使う側」）の
2 件が追加された。これも上記と同じ純追加であり「7 件で確定」の判断は
変更しない。

**Phase 7（#2530）による追加（#2542）**: 同じ親トラッキング #2530 配下の
イシュー #2542 により、Motion+ Cursor（ポインタに spring で追従する
カスタムカーソル）に相当する `cursor-hover-cards`（card 3 枚への hover で
カーソルがリング形状へ変化・ラベル表示・中心吸着の 3 パターンを示す合成
例）の 1 件が追加された。これも上記と同じ純追加であり「7 件で確定」の
判断は変更しない。

**Phase 7（#2530）による追加（#2551）**: 同じ親トラッキング #2530 配下の
イシュー #2551 により、Motion+ `sections/footers` に相当する
`footer-sticky-reveal`（`position: sticky` のみで本文の下から現れる
sticky reveal footer）・`footer-newsletter`（presence 同型 CSS で
「入力」「完了」2 panel の遷移を表現する newsletter footer。無 JS のため
Before/After の 2 インスタンスを静的に併記）の 2 件が追加された。これも
上記と同じ純追加であり「7 件で確定」の判断は変更しない。

## 4. `component-coverage-map.md` との関係

shadcn/ui Blocks は `docs/design/component-coverage-map.md` §2 の「対象外」
区分（chakra-ui Pro blocks と同じ「既存部品の合成例／商用テンプレート集」の
扱い）に該当する。coverage-map へ新規コンポーネント行として追加しない。
coverage-map が管理するのは「参照軸の skill reference ファイル」の対応表
であり、shadcn Blocks はスクリーンショットのみで管理されるため、この
対応表の対象には含めない。

## 5. `/blocks/<name>/` のページ構成

**IA（情報設計）**:

1. H1（block 名 + 一言説明）
2. Demo（本文 1 カラム内で全幅。後述）
3. 使用部品一覧（`sidebar` / `card` 等、Primitives/Themes 該当ページへの
   相互リンク）
4. Rust コード（フェンス `rust` コードブロック）
5. （任意）shadcn 側との差分メモ

**「全幅」の意味**: ビューポート端から端までの真の全幅（3 カラムグリッド
自体の変更）ではなく、`.docs-content` カラム内で最大幅を使う意味である。
`docs-toc`／サイドバーの DOM 構造・`crates/docs-site/tests/layout_render.rs`
の不変条件（`class="docs-toc"` 単一セレクタ等）は変更しない。合成物が
`.docs-content` の最大幅（`--fandhe-space-docs-max-content-width`）より
広くなる場合は、既存の表・showcase と同じ `overflow-x: auto` パターン
（`site_theme.rs` に既存の横スクロール実装）を Demo ラッパへ適用し、
ページ全体の横スクロールは発生させない。

**Rust コード掲載の一次情報源**: `site/blocks/<name>.md` 内のフェンス
`rust` コードブロックは手書きするが、`crates/docs-site/src/blocks/<name>.rs`
の合成関数本体と内容が一致することを、`crates/cli/tests/template_publish_copy_drift.rs`
/ `example_publish_copy_drift.rs` と同型の**ドリフト検知テスト**（新設
`crates/docs-site/tests/blocks_code_drift.rs` 等、実装は #2088 側）で
fail-closed に強制する方針とする。`include_str!` によるビルド時埋め込みは
`crates/docs-site` が「外部依存ゼロ・内部 path 依存のみ」の設計であることと
矛盾しないため次善案として触れるが、正の方針は「手書き + ドリフトテスト」
（他の embedded-copy 系契約と統一した idiom）とする。

## 6. 契約テストの追随範囲

イシュー #2007 本文にある「`linkcheck`」という表現は実在するファイル・
関数を指していないため、実ファイル名で記録し直す。

- `crates/docs-site/tests/site_nav.rs`: `site_nav_registers_six_sections_with_expected_titles`
  （セクション数 6 → 7、関数名含めて是正が必要）、
  `site_nav_registers_all_pages_with_expected_paths`（`assert_eq!(pages.len(), ...)`
  を Blocks 追加分だけ加算した値へ是正。実装者は数値を推測せず、
  `nav.toml` 反映後に実測して埋める）。
- `crates/docs-site/tests/search_index.rs`: `expected_hrefs` は実 `Nav`
  （`real_nav`）から機械導出されるため、`nav.toml` 更新後は自動追随する
  （ハードコード修正は基本不要）。
- `crates/docs-site/tests/no_js_contract.rs`: リダイレクト集合は
  `site/redirects.toml` から機械導出されるため、Blocks は旧 URL を持たず
  対象外（変更不要）。ページ本文の無 JS 制約チェック（該当すれば）は
  #2088 側で確認する。
- `crates/docs-site/tests/layout_render.rs` / `sidebar_group_render.rs` /
  `nav_group_schema.rs`: Blocks セクションは 7 ページのみのため
  `[[section.group]]`（カテゴリ分類）は使わず、フラットな `[[section.page]]`
  一覧とする（Primitives/Themes は 63/110 件のためグループ化が必須だが、
  Blocks の規模では不要と判断。Getting Started/Guides と同型のフラット
  構成）。この決定により `nav_group_schema.rs`/`sidebar_group_render.rs`
  への追随は不要になる。
- `.github/workflows/docs-site.yml` の dist sanity check: `site/**` と
  `crates/docs-site/**` の既存 paths glob がそのまま新規ファイルを捕捉する
  ため paths エントリ追加は不要（themes/primitives 追加時の前例と同型）。
  ただし `test -f "${RUNNER_TEMP}/docs-site-dist/blocks/index.html"` の
  追加が必要（#2088 側の作業）。

## 7. Phase 4（`sidebar`、#2071）への依存

dashboard-01 / sidebar-07 / sidebar-03 の 3 件は `sidebar` 部品（#2071）の
完了が前提となる。login/signup 4 件（login-01 / login-04 / signup-01 /
signup-05）は `sidebar` に依存しないため先行着手可能である。この依存関係は
#2088〜#2095 各イシューの「依存」欄と整合させる。

## 8. セキュリティ上の留意事項（OWASP Top 10）

- **A03 インジェクション/XSS（既定エスケープ）**: Blocks ページの実装は
  既存の `pre-styled-ui` パート関数のみで合成し、
  `format!("<div>{}</div>", ...)` のような HTML 文字列直接組み立てを
  行わない（`.claude/rules/coding-rust.md` の既存制約）。login/signup
  フォームという「ユーザー入力を扱うように見える」部品を初めて docs
  サイトに掲載するため、既定エスケープ経由であることをここで再確認する。
- **A05 セキュリティ設定ミス（安全でない URL スキーム）**: login-04
  （2 カラム画像付きログイン）で使う画像アセットは `data:` URI にしない
  （`crates/core` の `is_safe_url` が `data:` を拒否し属性ごと欠落する
  既知の不具合パターン、イシュー #1562 の前例）。ビルド時生成の
  プレースホルダー SVG/画像を `assets/` 配下へ相対パスで配置する
  （既存 `assets/image-demo.svg` と同型）。shadcn 実サイトの著作物
  （Unsplash 等の写真）をそのまま複製・ホットリンクしない。
- **なりすまし・フィッシング類似 UI への配慮**: login/signup の合成デモは
  実際の認証処理・送信先を持たない静的表示であることを明記し、ブランド名は
  架空のプレースホルダー（実企業名を使わない）とする。フレームワーク自身の
  公開 docs サイトが実在サービスの認証画面に酷似したページを不用意に
  公開しないための予防的措置である。**ソーシャルログインプロバイダ名
  （Google 等）も実企業名のため、一般名（例: 「SSO」）へ置換する**
  （#2092 の `login-01` で確定した規則。signup-05〔#2095〕も同規則を
  踏襲する）。
- **A08 ソフトウェア/データ完全性（ドキュメントのドリフト）**: §5 に
  記載のとおり、`site/blocks/*.md` の手書き Rust コードと実装のドリフトを
  防ぐため、fail-closed なドリフト検知テストの新設を必須とする（実装は
  #2088 側だが、方針をここで固定することでテストなしの手書きコード掲載を
  防ぐ）。

## 9. `update-issue-tree` の要否

Phase 6（#2087・#2088〜#2095）は本決定と完全に整合しており、**是正不要**
である。ルート #2001 本文の「確定済みの判断」節にある「Blocks は docs
サイト `/blocks/` セクション（案 A）前提で Phase 6 を起票している」という
一文の「確定」への更新（`gh issue edit`）は、本 PR のマージ後の別作業として
切り出す（本イシューのスコープ外）。

## 10. 実装記録（#2088）

基盤整備（nav 登録・ページ雛形・契約テスト）の実装で確定した規約。
後続 7 イシュー（#2089〜#2095）はこれをそのまま複製する。

- **セクションの並び順**: ヘッダー順は Getting Started / Guides / Examples /
  Primitives / Themes / **Blocks** / API Reference（Blocks は Themes の
  直後・API Reference の直前）。Issue 本文の提案（Primitives の前）とは
  異なるが、Primitives → Themes → Blocks という粒度の小→大の導線を優先し、
  `crates/docs-site/tests/primitives_nav.rs` が固定する「Primitives は
  index 3・直後が Themes」を変えない配置を選んだ。
- **雛形実例**: `login-01`（`/blocks/login-01/`）。sidebar 非依存かつ使用
  部品が最小（card/field/input/button）のため、雛形検証に十分と判断した。
  shadcn 側との忠実度・スクリーンショット比較は #2092 で完了済み
  （`field::group` の採用・ボタン群を `card::body` 側へ移設・ソーシャル
  ログインボタンのラベル一般化等、`crates/docs-site/src/blocks/login_01.rs`
  のモジュール doc「shadcn 側との構成上の判断」節・`site/blocks/login-01.md`
  「shadcn 側との差分メモ」節参照）。
- **ページ組み立て方式**: `crates/docs-site/src/blocks/mod.rs::insert_generated_sections`
  が Markdown ブロック列の最初の `h2` の直前へ「Demo」「使用部品」の 2 節を
  挿入する（`component_page::generated_content` の「後方追記」とは異なる）。
  `build.rs` は `component_page`/`Layer` 経路とは独立した分岐としてこれを
  呼ぶ（`Layer::from_page_path` の全域判定に Blocks を通さないため）。
- **マーカー規約**: `.rs` 側は `// blocks-code:begin`/`// blocks-code:end`
  の行マーカーで `use` 宣言 + `pub fn demo() -> Node` を囲み、`.md` 側の
  最初の rust フェンス本文（末尾空白のみ trim 許容）と行単位で一致させる。
  `crates/docs-site/tests/blocks_code_drift.rs` が fail-closed に固定する。
- **CSS**: ビルド時生成の専用スタイルシート `assets/blocks.css`
  （`crate::blocks::stylesheet`、`primitive_showcase::stylesheet` と同型）。
  block ページには本 CSS に加え、合成に使う部品自体の見た目のため
  `assets/pre-styled-ui.css`（`crate::showcase::STYLESHEET_REL_PATH`）も
  配線する。`/blocks/` 索引ページにはいずれも配線しない。
- **`<form>` を使わない**: `crate::layout` の無 JS 制約（Enter キーでの
  暗黙 submit 回避）に従い、Demo は `<form>` を持たず、ボタンは
  `button::button` の既定 `type="button"` のまま用いる。遷移先の無い
  「パスワードを忘れた」「サインアップ」は `link::root` の `href="#"` では
  なく `ButtonVariant::Link` の見た目のみリンク風ボタンで表現する。
  `crates/docs-site/tests/blocks_contract.rs` が固定する。
- **nav 構成**: `[[section.group]]` を使わずフラットな `[[section.page]]`
  のみ（1 block = 1 ページで階層化する動機がないため）。

## 11. `login-04`（#2093）実装記録

`login-01`（1 カラム）に続く 2 件目のログイン系 block。フォーム + 画像の
2 カラム構成という shadcn `login-04` 固有の要件から、以下を追加で決定した。

- **画像はビルド時生成アセットの再利用**: 右列画像は `data:` URI ではなく
  `crate::showcase::image_demo_svg` が生成する `assets/image-demo.svg` を
  相対パス（`../../assets/image-demo.svg`）参照する。`data:` URI は
  `fandhe_frontend_core::is_safe_url` の検証で属性ごと欠落する（イシュー
  #1562 と同型の判断）ため、既存の Themes 側 Image demo と同じアセットを
  流用する形にした（新規アセットは追加しない）。
- **見出しは `card::title`（`<h3>`）で表現してよい**: 当初「heading 要素は
  TOC を汚染するため避ける」という判断を検討したが、`crate::layout::
  with_heading_anchors` は Card の `title`（`<h3>`）を含む部品内部の見出しを
  部分木ごと走査対象外とする既存の仕組みを持つため、`card::title` の使用は
  TOC を汚染しない（`login_01` も同じ構成）。`login_04` のモジュール doc
  「見出しは `card::title`（`<h3>`）+ `card::description` で表現する」節を
  正とする。
- **プロバイダロゴの一般化**: shadcn 側の Apple/Google/Meta ロゴ入り
  `IconButton` 3 個は、実企業名・商標ロゴを持ち込まない方針（§8）に従い、
  `sidebar-03` の `geo_icon` と同型の自作幾何アイコン + 一般的な
  `aria-label`（「Login with provider A/B/C」）へ置換した。
- **`field::separator`（#2276）の初採用**: 「Or continue with」区切りは
  headless-ui/pre-styled-ui へ追加済みの `field::separator`（テキスト付き
  区切り）をそのまま使う。`field::group` 内の縦積みリズムを崩さないため、
  `separator::group`/`separator::label`（#2053）ではなくこちらを採用した。
- **`@media` の初使用**: `blocks::LAYOUT_CSS` 系はこれまで `@media` を
  持たなかったが、`< 768px` で右列画像を隠し 1 カラムへ切り替える shadcn
  側の `hidden md:block` 相当の再現に `@media (max-width: 47.99rem)` を
  初めて使った。`StyleSheet::push_css` の検証（`<`・NUL のみ拒否）は
  `@media` ブロックを問題なく通す。
- **カードのクリップ・2 カラム grid**: `card::root` は既定で padding・
  `overflow: hidden` を持たない（`card.rs` doc 参照）ため、
  `[data-blocks-login-04-card] { overflow: hidden; }` と
  `[data-blocks-login-04-body] { padding: 0; display: grid;
  grid-template-columns: 1fr 1fr; }` を block 側 CSS で補い、フォーム側
  ラッパ（`[data-blocks-login-04-form]`）へ `padding: 2rem` を付与する
  構成にした。
## 12. `signup-01`（#2094）実装記録

カード型のシンプルなサインアップフォーム。`login-01`（#2092 確定規則）を
そのまま踏襲し、以下の対応付けで shadcn `signup-01` を合成した。

- **`FieldDescription` → `field::helper_text` + `has_helper_text`**: shadcn
  側の `FieldDescription`（Email/Password/Confirm Password の 3 箇所）は
  `field::helper_text` として描画し、対応する `FieldProps::has_helper_text`
  を `true` にして `aria-describedby` を入力欄へ関連付ける。`has_helper_text`
  が `true` のフィールドは対応する `helper_text` ノードを必ず描画しないと
  `demo_output_has_no_dangling_aria_references_or_duplicate_ids` が
  宙ぶらりん参照として検知する（逆に `false` のまま `helper_text` を描くと
  関連付けが欠落する）。説明文を持たない Full Name は `has_helper_text: false`
  のまま。
- **SSO 一般化の踏襲**: 「Sign up with Google」→「Sign up with SSO」
  （`ButtonVariant::Outline`、全幅）。`login-01` と同じ実企業名不使用の
  判断（§8）。なお `site/blocks/signup-01.md` の「shadcn 側との差分メモ」
  節は説明のため「Google」という語を含むが、これは合成 Demo（HTML 出力）
  の一部ではなく原稿の解説文であり、§8 の「実企業名を持ち込まない」は
  Demo が実際に描画する UI 文言（ボタンラベル等）を指す。
- **`field::error_text` 非出力の踏襲**: `invalid: false` のため常に
  `hidden` になり shadcn 構成にも存在しないため、`login-01` と同じく
  DOM から省く。
- **カード幅 24rem**: `login-01` と同じ `[data-blocks-signup-01-card]`
  の `max-width: 24rem`。フィールド数が多い（4 件）ため `min-height` は
  `login-01`（24rem）より広い `32rem` とした。
- **使用部品**: Card / Field / Input / Button（`login-01` と同一の 4 部品）。

## 13. `signup-05`（#2095）実装記録

shadcn/ui Blocks `signup-05`（registry `new-york-v4/signup-05`）実物を確認
したところ、Issue 本文の見立て（`card`/`separator`/`link`）は実物と一致
しなかった（Card なし・入力欄は Email 1 個のみ・区切りは `FieldSeparator`・
リンクは `<a href="#">` の死リンク）。実物へ合わせて以下のとおり構成した。

- **`card` を不採用**: shadcn 実物に Card が無いため、外枠は素の `div`
  （`data-blocks-signup-05-stack`）で構成した。
- **ブランド見出しに `heading::heading`（H3）を採用**: shadcn 実物は
  `<h1>` だが、Demo 内へ `h1` を置くとページ本体の H1（`crate::layout`
  が生成）と重複するため `HeadingLevel::H3` を選んだ。`heading` は
  `data-scope="heading"` を持つため `crate::layout::with_heading_anchors`
  の TOC 収集対象外であり（`card::title` と同じ機構）、H1 → `## Demo` の
  下に正しくネストする。
- **ロゴを非リンク `div` + `role="img"` の `icon` にした理由**: 死リンク
  不使用方針（本文書 §8 と同型）のため `<a href="#">` を出力しない。
  `icon::IconProps.label` に `Some("Acme Inc.")` を渡すことで
  `role="img"` + `aria-label` が付与され、shadcn 側の sr-only span 相当の
  アクセシブルネームを代替できるため、追加のラッパー要素は不要と判断した。
- **プロバイダ名の一般化**: 実企業名・実ブランド・商標ロゴは持ち込まない
  方針（§8）のため、「Continue with Apple」「Continue with Google」を
  「Continue with provider A」「Continue with provider B」へ置換し、
  アイコンは `sidebar_03`/`sidebar_07` と同型の自作幾何図形を使った。
- **`@media (max-width: 39.99rem)` の採用**: shadcn 側の
  `Field.grid.gap-4.sm:grid-cols-2`（Tailwind `sm` ブレークポイント
  640px 未満で 1 列）相当を、既存 block が使っていない `@media` クエリで
  再現した（`field::separator` の採用は #2276 で追加されたテキスト付き
  separator パーツを充てた）。

不足部品は無かった（`field::separator`・`heading`・`icon`・アイコン付き
`button` はいずれも実装時点で既存）。

## 14. `bento-staggered` / `feature-expand`（#2549）実装記録

Motion+ `sections/bento-grids` に相当する 2 件（親トラッキング #2530
「Phase 7: Motion+ 部品化」→ #2476「Motion/Motion+ 参照アニメーション
充実」配下）。§3 の「7 件で確定」は #2088〜#2095 のツリー限定のスコープ
記録であり、本追加はその確定数の対象外の別系統（Motion+ 参照系）である
ことを§3 追記段落とあわせて明記する。

- **出典は Motion+（shadcn/ui ではない）**: 両 block とも `docs/design/
  motion-reference-adoption-policy.md` §9 に従い、Motion+
  （`motiondivision/plus`、MIT）のコードを転写せず、着想のみを参照して
  Rust/CSS で独自に再実装した。
- **hover は新規配線を行わない（A 群判定）**: 同文書 §4 は hover を A 群
  （CSS `:hover` で足りる大半のケース）に分類し「既存実装済みの範囲のみで
  新規配線を追加しない」と定める。`feature-expand` の hover/`:focus-within`
  展開はいずれも既存の CSS 機構のみで実装し、`fandhe-frontend-wasm-full`/
  `fandhe-frontend-animation` への新規配線を一切行わない。
- **`content_height.rs`（wasm-full の JS 機構）を使わない設計逸脱**:
  Issue 本文が示唆する「`content_height.rs` の既存機構」は文字通り再利用
  できない。`content_height.rs` は `fandhe-frontend-wasm-full` の JS
  ランタイム（ハイドレーション配線）であり、docs サイトは無 JS 前提
  （`crates/docs-site/tests/no_js_contract.rs`）でハイドレーションを一切
  行わない。`feature-expand` は代わりに `grid-template-rows: 0fr → 1fr`
  の CSS のみで `height: auto` への遷移不能問題を解く標準テクニックを
  使う（子要素へ `min-height: 0` を明示し、grid item の既定 `min-height:
  auto` によって `0fr` の収縮が効かなくなるのを避ける）。
- **`bento-staggered` の stagger は `animation-delay`（時間軸）ではなく
  `animation-range` の開始点オフセット（進行度軸）で表現する**:
  `animation-timeline: view()` 配下で `animation-delay` を時間値のまま
  併用した場合の解釈は仕様上複雑で確証が持てないため、進行度軸のみで
  完結させる意図的な設計判断である。`--fandhe-motion-stagger-index`
  （`fandhe_frontend_pre_styled_ui::recipe::stagger_index_style`）の値を
  `animation-range: entry calc(10% + var(...) * 8%) entry 100%` の開始点
  へ直接乗せ、後続セルほど発火が遅れる「順送り」を実現する。
- **`.blocks-demo` の `overflow-x: auto` を `bento-staggered` 限定で
  打ち消す**: `crate::blocks::LAYOUT_CSS` の `.blocks-demo` は
  `overflow-x: auto` を宣言する。CSS Overflow 仕様上 `overflow-y` を
  明示しない場合は `overflow-x` と同じ値へ強制されるため、
  `.blocks-demo` 自身がスクロールコンテナ化し、実際にはスクロールしない
  小さなデモ枠内では `animation-timeline: view()` が意図通り機能しない。
  `.blocks-demo.blocks-bento-staggered { overflow: visible; }` で
  この block 限定で打ち消し、ページ本体のビューポートを基準にする。
- **docs-site の `motion` feature 有効化は既に完了済み**: `motion`
  feature（`crates/docs-site/Cargo.toml`）はイシュー #2524 で有効化済み
  であり（`docs/guides/pre-styled-ui-motion-feature.md` §5）、`bento-
  staggered` は既存の `motion::KEYFRAMES_CSS`（`SLIDE_FROM_BOTTOM_
  KEYFRAMES_NAME`）を Blocks が初めて `push_css` した消費者である
  （`crate::showcase` は `SlotRecipe` の builder 経由で同キーフレームを
  間接的に使うのみで、生の `motion::KEYFRAMES_CSS` 定数を直接 push
  していない）。
- **`<form>` を持たない・実データを持たない**: `crate::blocks` モジュール
  doc の不変条件どおり、両 block とも `<form>` を出力しない。機能名・
  説明文はすべて架空のものであり、実企業名・実サービス名・実クレデンシャル・
  PII を含まない。
- **キーボード到達性（`feature-expand`）**: `:hover` のみでは非マウス
  操作者が展開内容へ到達できないため、各カードへ `button::button`
  （`Ghost` variant）を必ず配置し `:focus-within` の対象にする。展開/
  非展開いずれの状態でも詳細説明は DOM 上に常在し `aria-hidden`/`hidden`
  で隠さない（視覚的な非表示と AT 上の非表示を意図的に一致させない）。

不足部品は無かった（`card`・`icon`・`button` はいずれも実装時点で既存）。


## 15. hero sections 4 件（#2546）実装記録

Motion+ の hero sections に相当する 4 件（親トラッキング #2530「Phase 7:
Motion+ 部品化」配下）。`bento-staggered`/`testimonials-stack` と同じく
§3 の「7 件で確定」の対象外（Motion+ 参照系の別系統）である。

- **出典は Motion+（shadcn/ui ではない）**: 4 件とも `docs/design/
  motion-reference-adoption-policy.md` §9 に従い、着想のみを参照して
  Rust/CSS で独自に再実装した。取得手段・ファイル名・内部識別子は
  記載しない。
- **stagger は時間軸（`animation-delay`）で表現する**: `bento-staggered`
  の scroll-driven stagger（`animation-range`）とは異なり、
  `hero-editorial-stagger`/`hero-terminal` はページ先頭に置かれる hero
  である前提のため、`--fandhe-motion-stagger-index` を
  `animation-delay: calc(...)` へ乗せる時間軸 stagger を使う
  （`@supports (animation-timeline: view())` 不要）。`animation-delay` は
  リテラル `calc()` のため duration トークンの 0 化だけでは消えず、
  個別に `@media (prefers-reduced-motion: reduce)` を持つ。
- **`hero-parallax-layers` は `SlotRecipe::parallax` を直接使う**:
  `crate::showcase::parallax_demo` と同じ手法で、背景・中景・前景の
  3 レイヤーへ `ParallaxSpeed::Slow`/`Normal`/`Fast` を割り当てる。
  抽象図形（CSS グラデーション/`border-radius` のみ、画像は使わない）。
  `data-fandhe-scroll-progress` は docs-site が JS ハイドレーションを
  行わないため付与しない（`parallax_demo` と同じ判断）。
- **`hero-terminal` の typewriter は docs-site 上では静的表示**:
  `text_reveal::typewriter` はマークアップ（opt-in 属性）のみを供給し、
  実際の文字送りは `fandhe-frontend-wasm-full` の `text-animation`
  feature が JS ハイドレーション後に担う。無 JS の docs-site では
  `fd-text-reveal__display` の初期値（目標テキスト）がそのまま表示
  される。
- **`text-split-reveal` が `text_reveal::TEXT_REVEAL_CSS` を初めて
  `push_css` する**: `motion` feature 自体は #2524 で有効化済みだが、
  `text_reveal::TEXT_REVEAL_CSS` はどの block も `push_css` していな
  かった。`blocks::stylesheet()` が本 block の追加にあわせて 1 回だけ
  push する（`bento-staggered` が `motion::KEYFRAMES_CSS` を初めて push
  したのと同型の経緯）。
- **`text_reveal`/`cursor` を `parts` に列挙しない先例を踏襲**: いずれも
  単体の Themes ページを持たないため、4 件とも `parts` には実際に
  Themes ページを持つ部品（badge/heading/text/button/code/kbd）のみを
  列挙する。
- **`<form>` を使わない・実データを持たない**: `crate::blocks` モジュール
  doc の不変条件どおり、4 件とも `<form>` を出力しない。文言・コマンド
  文字列はすべて架空のものであり、実企業名・実サービス名・実クレデン
  シャル・PII を含まない。

不足部品は無かった（`badge`・`heading`・`text`・`button`・`code`・`kbd`
はいずれも実装時点で既存）。

## 16. `game-ui-modal`（#2552）実装記録

親 #2530「Phase 7: Motion+ 部品化」の最終タスク。Motion+
`examples/game-ui`（公開カタログ上のゲーム風 UI カテゴリで唯一の実例、
モーダル入場アニメーション 1 種）の部品化可否を検証し、Blocks 化と判定
した。

### 可否判定

| 構成要素 | 写像先（既存機能） |
|---|---|
| 暗幕 + 中央パネルのモーダル構造 | `pre-styled-ui::dialog` の root/backdrop/positioner/content/title/description/body/footer |
| scale + spring による入場 | `motion::ZOOM_IN_KEYFRAMES_NAME` + `theme::SPRING_EASING_LINEAR`（spring 近似 `linear()`、#2381） |
| 子要素（報酬行）の順送り出現 | `recipe::STAGGER_INDEX_VAR`/`stagger_index_style`（#2384）+ `motion::SLIDE_FROM_BOTTOM_KEYFRAMES_NAME` |
| 操作ボタン | `pre-styled-ui::button`（Solid / Ghost） |
| 報酬・ステータス表示 | `pre-styled-ui::badge` |

構成要素はすべて既存機能への写像で表現でき、DOM 計測・rAF・WAAPI を要する
要素は無いため **Blocks 化**（新規部品なし）と判定した。`wasm-full`/
`frontend-animation`/`pre-styled-ui`/`headless-ui` は一切変更していない。

### 設計判断

- **出典は Motion+**: `docs/design/motion-reference-adoption-policy.md`
  §9 に従い、着想のみを参照して Rust/CSS で独自に再実装した。取得手段・
  ファイル名・内部識別子は記載しない。
- **`trigger` を置かない**: docs-site は JS ハイドレーションを行わない
  ため、無 JS では開閉を切り替えられない `dialog::trigger` は表示上の
  意味を持たない（`testimonials-stack`/`sidebar-07` と同じ判断）。本
  Demo はモーダルが既に開いた静的な初期状態のみを描く。
- **dialog の中和 CSS は本 block スコープに閉じる**: `dialog::positioner`/
  `backdrop` は本来 `position: fixed; inset: 0` のビューポート全体
  オーバーレイだが、`.blocks-demo` 枠内へ収める必要があるため、
  `.blocks-game-ui-modal [data-scope="dialog"][data-part="..."]`
  （本 block のデモ class を前提とする属性セレクタ）に限定して
  `position: relative`/`inset: auto` へ差し替える。他 block や
  `/themes/dialog/` ページの `dialog` 表示には一切影響しない。
  `.blocks-demo.blocks-game-ui-modal` は scale の overshoot が枠で
  クリップされないよう `overflow: visible` にする（`bento-staggered`
  先例と同じ理由）。
- **入場アニメーションは既存の `presence_transition` と併走可能**:
  content には `ZOOM_IN_KEYFRAMES_NAME` + `SPRING_EASING_LINEAR` を
  `@keyframes` アニメーションとして適用する。既存の
  `presence_transition`（`content` の `transition` + `@starting-style`、
  #2387）と同方向（opacity/scale のフェードイン）であり、`animation`
  と `transition` は独立した CSS プロパティのため併走しても破綻しない。
- **reduced-motion は追加 `@media` なしで縮退**: duration はすべて
  `--fandhe-motion-duration-*` トークン参照であり、`Theme::to_css` の
  既定出力が reduced motion 環境で一括して 0ms 化する。`@keyframes` は
  無限反復・scroll-driven のいずれでもないため個別の `@media` は不要
  （新規 `@keyframes` 定義も追加していない。既存の
  `motion::KEYFRAMES_CSS` が持つ `ZOOM_IN`/`SLIDE_FROM_BOTTOM` を
  `animation` プロパティで参照するのみ）。
- **`<form>` を使わない・実データを持たない**: 報酬名・クエスト名は
  すべて架空のものであり、実企業名・実クレデンシャル・PII を含まない。

不足部品は無かった（`dialog`・`badge`・`button` はいずれも実装時点で
既存）。

### スコープ外

- `motion-reference-adoption-policy.md` §5 棚卸し表への game-ui 行追記
  （#2549/#2550 と同じく任意の後続）
- reference screenshot 追加（Motion+ 出典の再配布リスク、#2549/#2550 と
  同判断）
- 無 JS サイト上での実際の開閉・spring の wasm 側再生（利用者側の
  `wasm-full` 配線に委ねる。既存 `dialog` 配線で成立する）

## 17. カテゴリ属性・索引ページ生成化（#2733）実装記録

親トラッキング #2730（目的別パーツ拡充ツリー、新規約 300 block）の前提
整備として、`Block` に区分（`BlockSection`）とカテゴリ（`BlockCategory`）
を追加し、`/blocks/` 索引ページを「区分 → カテゴリ」の階層見出しで構成
するようにした。以後追加される block はすべて `category` を持つことが
要求される。

### 型設計

- `crates/docs-site/src/blocks/category.rs` に `BlockSection`（4 種:
  Marketing/Application/Ecommerce/Docs）と `BlockCategory`（66 種、
  イシュー #2733 本文記載順）を新設した。`site/nav.toml` を表す
  `crate::nav::Section` との名前衝突を避けるため `Section` ではなく
  `BlockSection` と命名した。
- `BlockCategory::section()`/`label()`/`kebab()` はいずれも `_ =>` を
  使わない全 variant 明示の `match` とし、新規カテゴリ追加時に実装更新を
  怠るとコンパイルエラーになる設計にした（未知カテゴリのコンパイル時
  排除という受け入れ条件を型で満たす）。
- **`Ecommerce::CategoryListing` の命名**: イシュー本文の ecommerce 区分
  にはカテゴリ名そのものが `category`（商品カテゴリ一覧ページ）として
  列挙されている。この enum 自体の概念名（カテゴリ）と variant 名が
  衝突すると読み手を混乱させるため、variant 名は `CategoryListing` とし、
  `kebab()` が厳密な文字列 `"category"` を返すことで実際の分類名との
  ズレを吸収した。

### 既存 22 block の割当

login-01/login-04/signup-01/signup-05 → Application/Auth、dashboard-01 →
Application/Dashboard、sidebar-07/sidebar-03 → Application/Sidebar、
pricing-tiers-morph/pricing-usage-slider → Marketing/Pricing、
testimonials-stack → Marketing/Testimonial、bento-staggered →
Marketing/Bento、feature-expand → Marketing/Feature、
cta-banner-magnetic/cta-signup-celebrate → Marketing/Cta、
footer-sticky-reveal/footer-newsletter → Marketing/Footer、
hero-editorial-stagger/hero-parallax-layers/hero-terminal/
text-split-reveal → Marketing/Hero（`text_split_reveal` はモジュール doc
に「hero sections に相当する合成例」と明記されているため hero 扱いと
した）、game-ui-modal → Application/Dialog（合成部品の中心が `dialog`
であるため）。

判断を要した 2 件:

- **`cursor-hover-cards`**: Application/Card とした。合成部品が `card`
  のみで、マーケティング訴求文脈ではなく汎用カード hover 演出のデモで
  あるため（Marketing/Feature も次点候補として `category.rs` の
  `BlockCategory::Card` 割当コメントに残す判断だったが、実装ファイル側
  コメントには記載していない。次点は本節にのみ記録する）。
- **`game-ui-modal`**: Application/Dialog とした。合成部品の中心が
  `dialog` であるため。

### 索引のレジストリ生成化

`site/blocks.md` の手書き「掲載済み」箇条書き（22 行）を撤去し、
イントロ文のみへ縮小した。索引本文は `crate::blocks::insert_generated_sections`
が `page_path == INDEX_PATH`（`/blocks/`）のときに
`index_generated_sections` を呼んで `BlockSection::ALL` × `BlockCategory::ALL`
から `BLOCKS` を走査し組み立てる。0 件の区分・カテゴリは見出しごと省略
する。カテゴリ内の表示順は `BLOCKS` の宣言順ではなく `path` の辞書順と
した（並列 PR による `BLOCKS` への追記順は安定しないため、索引の表示順を
レジストリ追記順から独立させる判断）。

既存の `insert_generated_sections`/`splice_before_first_h2`（「最初の `h2`
の直前へ挿入する、無ければ末尾へ追加する」全域関数）をそのまま索引ページ
にも適用できたため、`crate::build::build_site` 側の呼び出し箇所は無変更
で済んだ。索引化のために新設したのは `index_generated_sections` 関数と
`INDEX_PATH` 定数のみであり、当初想定した専用の分岐追加は不要だった。

### テスト

- `crates/docs-site/src/blocks/mod.rs` の `insert_generated_sections_is_noop_for_non_block_pages`
  は入力パスを `/blocks/`（索引ページになったため non-noop）から
  `/blocks/no-such-block/`（未登録パス）へ差し替えた。
- 新規 `insert_generated_sections_builds_index_with_section_and_category_headings`
  で、実在する区分・カテゴリの見出し・リンクが出力され、0 件カテゴリ
  （例: `Faq`）の見出しが出力されないことを固定した。
- `crates/docs-site/tests/blocks_nav.rs::blocks_index_page_links_to_the_registered_block`
  は生の Markdown ソースを読む方式から、実サイトビルド
  （`support/shared_site.rs`）の `blocks/index.html` を読み、
  `blocks::all_blocks()` 全件への `href` をループ検証する方式へ書き換えた。
  個別イシュー番号ごとの手書き `assert!` 列挙を廃し、将来 block が
  増えても本テストへの追記が不要なレジストリ駆動の網羅チェックへ
  移行した。

### `site/nav.toml`（サイドバー構成）は変更しない

`site/nav.toml` の Blocks セクションは引き続きフラット構成
（`[[section.page]]` のみ、`[[section.group]]` は使わない）を維持する。
`blocks_nav.rs::blocks_section_is_registered_immediately_after_themes` が
`section.groups.is_empty()` を固定しており、これを崩す変更（サイドバーの
`[[section.group]]` 化）は本イシューのスコープ外の別判断とする。
Themes セクションは本イシューと同じ「索引ページ本文内のカテゴリ見出し」
方式であり、本実装はこの Themes 方式を踏襲した（Primitives セクションが
使う `[[section.group]]` 方式とは異なる）。

**この判断は #2735 で置き換えられた（§19 参照）。** 上記「変更しない」は
索引ページ本文の生成方式（本節が扱うスコープ）についての判断であり、
サイドバー自体のグループ化可否は明示的に別イシューへ委ねられていた
（本節冒頭の記述どおり）。#2735 がその「別イシュー」であり、以後は
`site/nav.toml` の Blocks セクションが `[[section.group]]` を使う。

## 18. カテゴリ別モジュール分割（#2734）実装記録

### 背景

ルート #2730「Blocks 目的別パーツ拡充トラッキング」は #2733 で導入した
66 カテゴリへ向けて今後 300 件規模の block を並列 PR で追加していく計画
だが、#2733 時点の `crates/docs-site/src/blocks/mod.rs` は `mod` 宣言・
`BLOCKS` 配列・`stylesheet()` の 3 箇所が単一ファイルへフラットに列挙され
ており、block を 1 件追加するだけの通常の変更が全 PR で同じ行域を編集
することになる。並列実装が本格化する前の Phase 0 として、この構造上の
欠陥を解消する。

### ディレクトリ構成

`crates/docs-site/src/blocks/` を次の 3 階層へ分割した。

```
blocks/
  mod.rs                    -- Block/LayoutCss 定義・all_blocks()/block_for_path()/
                                stylesheet() のみを持つ薄いトップ
  category.rs                -- 既存のまま（BlockSection/BlockCategory、#2733）
  marketing/ application/ ecommerce/ docs/
    mod.rs                    -- 区分内の全カテゴリの mod 宣言 + blocks() 集約
    <category>.rs              -- 空カテゴリの雛形（`pub(super) fn blocks() -> Vec<Block> { Vec::new() }`）
    <category>/mod.rs          -- block を持つカテゴリ（`mod <block>;` 宣言 + blocks() 集約）
    <category>/<block>.rs      -- 個別 block の実装（既存 22 件を git mv で移設）
```

66 カテゴリ全件を本 Phase で先行スキャフォールドした（populate 済みの
カテゴリだけ mod 宣言する遅延方式は採らなかった）。後続 Phase では複数
カテゴリの「そのカテゴリ最初の block」が並列 PR として同時進行するため、
カテゴリ登録を遅延させると区分側 `mod.rs` の編集で衝突が再発するためで
ある。ディレクトリ/ファイル名は `BlockCategory::kebab()` の `-` を `_`
へ置換した snake_case とし、`CategoryListing`（`kebab() == "category"`）
のみ `category_listing` を使う（`crate::blocks::category` 型定義モジュール
との名前衝突回避）。

「カテゴリの卒業」手順（空雛形 → ディレクトリ化）: 最初の block を追加
する際は空雛形ファイル（例 `marketing/banner.rs`）を `git mv` で
`banner/mod.rs` へ改名し、block 実装ファイルを同じディレクトリへ追加した
上で `blocks()` を書き換える。この変更はカテゴリ内で完結し、区分側
`mod.rs`（`marketing` 等）・トップレベル `crate::blocks` 側は
`pub(super) fn blocks()` のシグネチャが不変のため変更不要。

### `Block` レジストリの関数化（`BLOCKS` 配列の廃止）

66 個の可変長カテゴリを stable Rust の `const fn` だけで単一の
`&'static [Block]` へ連結する手段（`generic_const_exprs` なしでは手書き
連結コードしか選択肢がない）は可読性・保守性の観点で見合わないため、
`pub const BLOCKS: &[Block]` を廃止し `pub fn all_blocks() -> Vec<Block>`
（4 区分の `blocks()` を `.extend()` で連結するだけ）へ置き換えた。
`Block` は `Clone, Copy` のためコピーコストは無視できる（呼び出しは
docs サイトビルド時・テスト時のみでホットパスではない）。

呼び出し側の追随:

- `block_for_path` の戻り値を `Option<&'static Block>` から
  `Option<Block>`（`all_blocks().into_iter().find(...)`）へ変更した。
- `crates/docs-site/tests/blocks_nav.rs`/`blocks_code_drift.rs`/
  `blocks_contract.rs` の `blocks::BLOCKS` 参照を `blocks::all_blocks()`
  ベースへ書き換えた。`&'static str` フィールド（`path` 等）のみを使う
  `.iter().map(...).collect()` は一時 `Vec` から借用しても値の寿命が
  `'static` のため単一式のまま書き換えられたが、`.find(...).expect(...)`
  の結果を複数行で使い回す 3 箇所（`login_04_block`/`block`（signup-05
  デモ検証）/`block`（login-01 使用部品リンク検証））は
  `.into_iter().find(...)` で所有権ごと受け取る形へ変更した
  （一時 `Vec` から得た参照をステートメントを跨いで保持できないため）。

### block 固有 CSS のレジストリ駆動化

`Block` へ新フィールド `layout_css: LayoutCss` を追加した。

```rust
pub enum LayoutCss {
    Static(&'static str),
    Dynamic(fn() -> String),
}
```

`stylesheet()` は共通フレーム CSS・共有フレームワーク CSS
（`motion::KEYFRAMES_CSS`/`cursor::CURSOR_CSS`/`text_reveal::TEXT_REVEAL_CSS`）
を先頭で push した後、`all_blocks()` を走査して各 block の `layout_css`
を `push_css` するだけになった。カテゴリ側モジュールは CSS の集約経路を
別途持つ必要がなく、`Block` 自身のフィールドへ寄せることで二重の集約
経路を作らない設計とした。各 block 実装ファイル側は `LAYOUT_CSS`
（`&'static str` 定数）/`layout_css()`（`fn() -> String`）の可視性を
`pub(super)` から**ファイル内 private** へ縮小した（`BLOCK` 定数の組み立て
時に自己完結的に取り込むだけになったため）。

### 生成物の同一性（受け入れ条件との整合）

- `dist/blocks/**` の HTML は分割前後で **バイト単位で完全一致**する
  （実測: `diff -rq` で差分ゼロ）。各 block の `demo()` 実装は無変更、
  索引ページはカテゴリ内 `path` 辞書順ソートのため走査順に依存せず、
  `nav.toml` は既存 22 件のエントリを一切動かしていないため前後ナビも
  不変。
- `assets/blocks.css` は**内容（ルール集合）が完全に同一**（実測:
  `}` 区切りでルール単位に分解し正規化した多重集合として比較し、
  分割前後とも 255 ルールで完全一致）だが、**連結順序は変わる**
  （手書きのカテゴリを跨いだ追記順 → 区分 → カテゴリ → 登録順）。
  全 block の CSS セレクタは `.blocks-<name>`/`[data-blocks-<name>-*]`
  の形で block ごとに名前空間分離されており、カスケード順に依存する
  規則は存在しないため、順序変更に副作用はない。受け入れ条件の
  「生成物が分割前と同一」は「ページ HTML はバイト同一、CSS はルール
  集合として同一（順序は不問）」と解釈する。

### `site/nav.toml` の競合対策

既存 22 件の `[[section.page]]`（前後ナビ順序保持のため）は一切動かさず、
末尾（`footer-newsletter` の直後・Wireframes セクション定義の直前）へ
`BlockCategory::ALL` の宣言順で 66 個のカテゴリ用コメントアンカーを追記
した。以後 block を追加する際は該当カテゴリの見出し行の直後へ
`[[section.page]]` を追記する運用とする。異なるカテゴリの block を追加
する PR 同士は異なる見出し行を編集するため通常はコンフリクトしない
（同カテゴリ内での競合のみ許容範囲）。アンカーはコメント行のみのため
`parse_nav` のパース結果・`blocks_nav.rs` の三方突合には影響しない
（実測で確認済み）。

### `.github/workflows/docs-site.yml` の `test -f` 再編

既存 22 行の `test -f` は削除・弱体化せず、`BlockCategory::ALL` 順で
66 カテゴリの見出しコメントを挿入してグループ化した。純粋な bash
スクリプトの再編でありサイト生成物には影響しない。

### 新規ガードテスト

`crates/docs-site/tests/blocks_categories.rs` を新設し、以下を固定した。

- `every_registered_block_rust_source_matches_its_category_directory`:
  各 `Block.rust_source` が `category.section()`/`category.kebab()`
  から機械導出される `crates/docs-site/src/blocks/<section>/<category>/`
  配下を指していること。
- `every_category_has_a_scaffold_file_or_directory`: `BlockCategory::ALL`
  全 66 件について、空雛形 `.rs` またはディレクトリ化済み `mod.rs` の
  いずれかが実在すること。
- `no_category_has_both_a_flat_scaffold_and_a_directory`: 「カテゴリの
  卒業」手順が中途半端な状態（空雛形とディレクトリの両方が同時に存在）
  を残さないこと。

`crates/docs-site/tests/blocks_contract.rs::blocks_source_does_not_use_raw_html_or_build_html_strings`
の `collect_rs_files` は導入当初から再帰的（`path.is_dir()` なら再帰）
であり、block 実装をサブディレクトリへ分割しても無改造で機能することを
確認済み。

### delegation 表の欠落（out of scope）

`crates/docs-site/` は CLAUDE.md の delegation パス切り替え表・
`.claude/rules/delegation-impl.md` のいずれにも明示エントリが無い。
本実装は実務上の慣行（tooling-builder 相当）に従ったが、delegation 表
自体の整備は本イシューのスコープ外とし、別途 Issue 化を検討する
（`.claude/rules/out-of-scope-tracking.md` 参照）。

## 19. block 一覧索引（イシュー #2736）

`CLAUDE.md`・`.claude/rules/ci.md` は block 追加のたびに個別の設計判断を
長文追記する運用を続けてきたが、22 件の記述が蓄積した時点で可読性を
失い、全 PR の競合点になっていた（イシュー #2736）。以後 block 追加 PR は
両ファイルを編集せず、個別の設計判断の索引を本節に一本化する。表の
「正の所在」列は既存記録がある場合はその節番号、無い場合は該当
`.rs` パス（`crates/docs-site/src/blocks/` を省略した相対表記）+
対応する `site/blocks/<kebab>.md` を指す。

| slug | issue | 正の所在 |
|---|---|---|
| login-01 | #2088 | §10, `application/auth/login_01.rs` |
| dashboard-01 | #2089 | §10, `application/dashboard/dashboard_01.rs` |
| sidebar-07 | #2090 | §10, `application/sidebar/sidebar_07.rs` |
| sidebar-03 | #2091 | §10, `application/sidebar/sidebar_03.rs` |
| login-04 | #2093 | §11, `application/auth/login_04.rs` |
| signup-01 | #2094 | §12, `application/auth/signup_01.rs` |
| signup-05 | #2095 | §13, `application/auth/signup_05.rs` |
| pricing-tiers-morph | #2547 | §3 追記段落, `marketing/pricing/pricing_tiers_morph.rs` |
| pricing-usage-slider | #2547 | §3 追記段落, `marketing/pricing/pricing_usage_slider.rs` |
| testimonials-stack | #2548 | §3 追記段落, `marketing/testimonial/testimonials_stack.rs` |
| bento-staggered | #2549 | §3 追記段落・§14, `marketing/bento/bento_staggered.rs` |
| feature-expand | #2549 | §3 追記段落・§14, `marketing/feature/feature_expand.rs` |
| cta-banner-magnetic | #2550 | §3 追記段落, `marketing/cta/cta_banner_magnetic.rs` |
| cta-signup-celebrate | #2550 | §3 追記段落, `marketing/cta/cta_signup_celebrate.rs` |
| cursor-hover-cards | #2542 | §3 追記段落, `application/card/cursor_hover_cards.rs` |
| footer-sticky-reveal | #2551 | §3 追記段落, `marketing/footer/footer_sticky_reveal.rs` |
| footer-newsletter | #2551 | §3 追記段落, `marketing/footer/footer_newsletter.rs` |
| hero-editorial-stagger | #2546 | §15, `marketing/hero/hero_editorial_stagger.rs` |
| hero-parallax-layers | #2546 | §15, `marketing/hero/hero_parallax_layers.rs` |
| hero-terminal | #2546 | §15, `marketing/hero/hero_terminal.rs` |
| text-split-reveal | #2546 | §15, `marketing/hero/text_split_reveal.rs` |
| game-ui-modal | #2552 | §16, `application/dialog/game_ui_modal.rs` |
| banner-cookie-consent | #2740 | `marketing/banner/banner_cookie_consent.rs` |
| banner-email-signup | #2741 | `marketing/banner/banner_email_signup.rs` |
| banner-announcement-pill | #2739 | `marketing/banner/banner_announcement_pill.rs` |
| banner-floating-card | #2742 | `marketing/banner/banner_floating_card.rs` |
| banner-full-width-bar | #2743 | `marketing/banner/banner_full_width_bar.rs` |
| blog-featured-article | #2808 | `marketing/blog/blog_featured_article.rs` |
| blog-featured-with-list | #2809 | `marketing/blog/blog_featured_with_list.rs` |
| blog-grid-image | #2810 | `marketing/blog/blog_grid_image.rs` |
| blog-grid-text | #2811 | `marketing/blog/blog_grid_text.rs` |
| bento-asymmetric-rows | #2744/#2745/#2746 | `marketing/bento/bento_asymmetric_rows.rs` |
| blog-list-image | #2812 | `marketing/blog/blog_list_image.rs` |
| blog-overlay-cards | #2813 | `marketing/blog/blog_overlay_cards.rs` |
| blog-split-header-grid | #2814 | `marketing/blog/blog_split_header_grid.rs` |
| bento-three-column-tall | #2748 / #2749 | `marketing/bento/bento_three_column_tall.rs` |
| bento-two-column | #2750 | `marketing/bento/bento_two_column.rs` |
| content-article | #2751 | `marketing/content/content_article.rs` |
| content-article-toc | #2752 | `marketing/content/content_article_toc.rs` |
| content-columns-screenshot | #2753 | `marketing/content/content_columns_screenshot.rs` |
| content-image-tiles | #2754 | `marketing/content/content_image_tiles.rs` |
| content-split-image | #2755 | `marketing/content/content_split_image.rs` |
| content-with-testimonial | #2756 | `marketing/content/content_with_testimonial.rs` |
| careers-split-photo-list | #2817 | `marketing/careers/careers_split_photo_list.rs` |
| changelog-accordion | #2818 | `marketing/changelog/changelog_accordion.rs` |
| changelog-stacked-list | #2819 | `marketing/changelog/changelog_stacked_list.rs` |
| changelog-timeline | #2820 | `marketing/changelog/changelog_timeline.rs` |
| comparison-cards | #2822 | `marketing/comparison/comparison_cards.rs` |
| comparison-feature-rows | #2823 | `marketing/comparison/comparison_feature_rows.rs` |
| comparison-split-table | #2824 | `marketing/comparison/comparison_split_table.rs` |
| careers-card-grid | #2815 | `marketing/careers/careers_card_grid.rs` |
| careers-split-accordion | #2816 | `marketing/careers/careers_split_accordion.rs` |
| changelog-timeline-subscribe | #2821 | `marketing/changelog/changelog_timeline_subscribe.rs` |
| cta-feature-links | #2757 | `marketing/cta/cta_feature_links.rs` |
| cta-split-actions | #2758 | `marketing/cta/cta_split_actions.rs` |
| cta-split-image | #2759 | `marketing/cta/cta_split_image.rs` |
| feature-accordion-image | #2760/#2761/#2762 | `marketing/feature/feature_accordion_image.rs` |
| contact-dialog-form | #2827 | `marketing/contact/contact_dialog_form.rs` |
| feature-alternating-rows | #2763 | `marketing/feature/feature_alternating_rows.rs` |
| contact-form-testimonial | #2828 | `marketing/contact/contact_form_testimonial.rs` |
| feature-image-cards | #2765 | `marketing/feature/feature_image_cards.rs` |
| feature-large-screenshot | #2766 | `marketing/feature/feature_large_screenshot.rs` |
| contact-image-info | #2829 | `marketing/contact/contact_image_info.rs` |
| contact-info-columns | #2830 | `marketing/contact/contact_info_columns.rs` |
| contact-split-info | #2835 | `marketing/contact/contact_split_info.rs` |

（`.rs` パスは `crates/docs-site/src/blocks/` を省略した相対表記。上記
64 件は個別の Markdown 原稿〔`site/blocks/<kebab>.md`〕を持つ block の
全件であり、`crates/docs-site/src/blocks/` 配下にはこれとは別に
イシュー #2730 系トラッキング配下で追加された空雛形・カテゴリ別
モジュール（§17/§18 参照）が存在するが、それらは個別の Markdown 原稿を
まだ持たず本表の対象外である。以後の block 追加は本表への 1 行追加と、
`.claude/rules/ci.md` が定める `.github/workflows/docs-site.yml` の
dist sanity check `test -f` 対象への 1 行追加（生成物の存在を fail-closed
に検証する既存契約、削除・弱体化しない）の 2 点で足りる。`CLAUDE.md`・
`.claude/rules/ci.md` の説明本文（経緯の長文追記）は編集しない。）

## 20. Blocks 共通のデモ用ダミー素材ヘルパ（#2737）実装記録

親トラッキング #2731「Blocks 目的別パーツ拡充ツリー」配下（phase:0）。
今後追加される目的別パーツ block の多くが商品画像・人物アバター・会社
ロゴ・スクリーンショット枠・グラフ用数値等のプレースホルダー素材を
必要とするため、`crates/docs-site/src/blocks/dummy_assets.rs`
（`pub(crate) mod dummy_assets;`、`all_blocks()` レジストリには乗らない
第 5 の Rust 生成コンテンツ供給元）へ共通ヘルパとして一元化した。

- **画像 5 種**（商品・人物アバター・会社ロゴ・スクリーンショット枠・
  汎用背景タイル）はいずれも `showcase::image_demo_svg`（イシュー #1562）
  と同じ「ビルド時生成 SVG を相対パスアセットとして書き出す」方式を
  踏襲する。`data:` URI は `fandhe_frontend_core::url::is_safe_url` が
  拒否するため使わない。出力先は `assets/blocks-demo-{product,avatar,
  logo,screenshot,background}.svg`、`crate::build::build_site` が
  `dummy_assets::IMAGE_ASSETS`（`(相対パス, 生成関数)` のディスパッチ
  テーブル、`crate::blocks::Block` レジストリと同型の単一情報源設計）を
  1 ループ走査するだけで href 登録・書き出しの両方を行う。書き出し条件は
  既存 22 block の使用有無を問わず「Blocks ページが 1 件でも存在すれば
  無条件（`has_blocks_page` と同条件）」とし、個別 block の実際の消費
  状況を走査する複雑な条件判定は導入しない。
- **モノトーン・抽象図形限定**: 5 種とも共通パレット定数（背景/輪郭/
  強調の 3 色グレースケール）のみで構成し、実在の人物・企業・ブランドを
  模さない（会社ロゴは六角形 + 中心円の抽象バッジ、人物アバターは
  円 + 弧のみのシルエット）。
- **文言・数値セット**: 架空の人名（`PERSON_NAMES`）・役職
  （`JOB_TITLES`）・社名（`COMPANY_NAMES`、既知の実データセット由来
  名称は避けた完全架空のセット）・一言レビュー（`TESTIMONIAL_QUOTES`）・
  価格帯（`SAMPLE_PRICE_TIERS`）・グラフ用サンプル系列
  （`SAMPLE_CHART_CATEGORIES`/`SAMPLE_CHART_SERIES_A`/`_B`）を
  `pub(crate)` 定数として提供する。出力は呼び出し側 block が
  `fandhe_frontend_core::text()` 経由で行う契約（本モジュール自体は
  `Node` を組み立てない）。
- **既存 22 block への適用は任意・本実装のスコープ外**: 出力差分レビュー
  を要するため、本イシューでは新設と CI 組み込みのみを行い、既存 block
  の書き換えは行わない。
- **未使用コードの警告抑制**: 本モジュール新設時点では `crate::blocks`
  の他モジュールから未参照の定数・関数がある（後続 #2730 系 block から
  順次参照される想定）ため、`#![allow(dead_code)]` をモジュール冒頭に
  明示し `cargo clippy -- -D warnings` を赤くしないようにした。
- **契約テスト**: `crates/docs-site/tests/blocks_dummy_assets.rs` が
  実サイトビルド成果物に対して 5 SVG の存在・非空・`<svg` ルート・
  `data:`/`<script`/イベントハンドラ属性不在を検証する。単体テスト
  （`dummy_assets.rs` 内 `#[cfg(test)]`）は生成関数の戻り値を直接検証し、
  役割を分離している。
## 21. サイドバーのカテゴリ階層化（#2735）実装記録

親トラッキング #2730（目的別パーツ拡充ツリー、約 300 block への拡充計画）
を前に、`/blocks/` のサイドバー（`crates/docs-site/src/nav.rs::sidebar`）を
カテゴリ見出し付きへ変更した。§17 まではフラット列挙（索引 1 件 + block
22 件）のままサイドバーに直接並んでいたが、300 block 規模では目的の
パーツを探せなくなるため、本イシューで対応した。

### 採用案: `BlockCategory` 単位の `[[section.group]]` 化

- `crates/docs-site/src/nav.rs` は既に Primitives（`PrimitiveCategory` 6
  グループ）・Themes（カテゴリ 6 グループ）の両セクションで
  `[[section.group]]`/`<details>`/`<summary>` によるグループ化
  （イシュー #939/#940）を実装済みであり、これが本リポジトリで唯一実在
  するサイドバーのグループ化手段だった。`site/nav.toml`・`nav.rs` の
  パーサ・`sidebar()` の描画ロジックは一切変更せず、Primitives/Themes と
  同じ構文を Blocks セクションへ追記するだけで実現した。
- グループの分類元は §17 で追加済みの `BlockCategory`
  （`crates/docs-site/src/blocks/category.rs`）とし、`category.label()` を
  そのままグループ `title` に用いた（索引ページの `h3` 見出しと同じ表記で
  UI の一貫性を保つ）。
- 索引ページ（`/blocks/`）は他セクションと同様 `[[section.page]]` の
  直下ページのまま残し（`blocks_index_page_links_to_the_registered_block`
  が `section.pages` から直接 `/blocks/` を検索する契約を維持するため
  必須）、それ以外の全 block を `[[section.group]]`/
  `[[section.group.page]]` へ変換した。

### 不採用案: レジストリからサイドバー用 `Nav` を動的導出する

`blocks::BLOCKS` の `category` を使って、ビルド時に `nav::sidebar()` へ
渡す `Nav` を都度組み替える（`/blocks/` 索引ページ本文の
`index_generated_sections` と同型の発想）案も検討したが、不採用とした。
理由:

1. 本リポジトリのサイドバーグループ化は Primitives/Themes とも nav.toml
   手書き `[[section.group]]` の一択であり、動的導出は前例がない新規
   メカニズムになる（`nav.rs` を Blocks 固有ロジックへ結合するか、
   `build.rs` 側に「サイドバー専用の派生 `Nav`」という新しい概念を導入
   する必要があり、いずれも既存の実績あるパスより複雑・高リスク）。
2. §17 末尾が「サイドバー構成は変更しない、変更する場合は別イシューでの
   判断」と明記しており、その「別イシュー」である本件が同じドキュメント
   の直前の決定と矛盾する新方式を持ち込むのは一貫性を欠く。
3. Primitives（75 部品）・Themes（123 部品）は同じ手書き方式で既に本番
   運用されており、Blocks が目指す規模（当面 22 だが将来 ~300）でも
   同方式が破綻する理由がない。

### グループの粒度: `BlockCategory`（現在 12 グループ）であって `BlockSection`（4 種）ではない

`BlockSection` は 4 分類のみで、300 block 到達時は 1 グループ平均 75 件と
なり「探せない」問題が再燃する。`BlockCategory`（実装時点で 22 block が
12 カテゴリへ分散）は索引ページの `h3` と同じ粒度であり、「カテゴリ見出
し」という issue タイトルの字面にも忠実なため、グループは `BlockCategory`
単位とした。

### 2 段ネスト（区分 → カテゴリ）は今回やらない

`nav.rs` の `[[section.group]]` は 1 段ネスト限定
（`[[section.group.group]]` は未知テーブルとして明示的にエラーになる）
であり、`BlockSection` を外側の見出しとして併用するには `nav.rs` の中核
パーサ・`sidebar()` 描画ロジックを拡張する必要がある。Primitives/Themes
を含む全セクション共通のコードへ手を入れる大きめの変更になるため、本
イシューのスコープ（「カテゴリ見出し」単数）を超えると判断し見送った。
ただし `BlockCategory::ALL` の宣言順（区分ごとにまとまっている）でグルー
プを並べたため、明示的な区分見出しは無くても実質的に区分単位でまとまっ
た順序になっている。将来カテゴリ数が増えすぎて破綻する場合は改めて評価
する（新規 Issue の起票はユーザー承認事項のため、本節には観察事項として
のみ記録する）。

### グループ内のページ順序: 宣言順ではなく `path` の辞書順

`primitives_nav.rs` は「カタログの宣言順」をそのまま nav.toml の並びに
要求しているが、Blocks の場合は `index_generated_sections`（§17）が既に
「並列 PR による `BLOCKS` への追記順は安定しないため `path` の辞書順に
独立させる」と判断している。同じ不安定性は本イシューにも当てはまるため、
Primitives の慣習をそのまま踏襲せず、グループ内のページは `path` 昇順で
`site/nav.toml` に記述し、対応するテストもこの順序で比較する（索引ページ
本文との表示順の一貫性も得られる）。

### 変更ファイル

- `site/nav.toml`: Blocks セクションの `[[section.page]]`（索引を除く
  22 件）を、1 件以上 block を持つ `BlockCategory` 12 件の
  `[[section.group]]`/`[[section.group.page]]` へ書き換えた。
  グループ順序は `BlockCategory::ALL` の宣言順のうち block を持つものの
  み（Hero → Feature → Cta → Pricing → Testimonial → Footer → Bento →
  Sidebar → Auth → Dialog → Card → Dashboard）、グループ内は `path` 昇順。
  Wireframes セクション直前のコメント（旧「Blocks と同じ理由でフラット
  構成」）も、Blocks が本イシューでグループ化された結果誤りになるため、
  Wireframes 自身の理由（Phase 進行順管理・49 部品規模ではカテゴリ化の
  動機がない）へ書き換えた。
- `crates/docs-site/tests/blocks_nav.rs`:
  `blocks_section_is_registered_immediately_after_themes` を「索引ページ
  のみが直下ページ・groups が非空」の検証へ反転し、
  `primitives_nav.rs` と同型の新規テスト 2 件
  （`blocks_section_groups_match_registry_category_order` /
  `blocks_group_pages_match_registry_category_assignments`。後者は宣言順
  ではなく `path` 昇順で比較する点が Primitives 側と異なる）を追加した。
- `crates/docs-site/tests/site_build.rs`:
  `real_site_blocks_sidebar_shows_category_groups` を追加し、実ビルドの
  `blocks/login-01/index.html`（`BlockCategory::Auth`）のサイドバーが
  「`docs-nav-group` を含む」「open な `<details>` がちょうど 1 件」
  「その `<summary>` が `Auth` ラベルを含む」「リンクがすべて `/blocks/`
  配下」「`<h2>` は 1 件のまま」を満たすことを固定した。

### 変更不要と判断した箇所（確認のみ）

- `crates/docs-site/src/nav.rs`（`sidebar()`/`group_node()` は無改修で
  流用できた）。
- `crates/docs-site/src/blocks/mod.rs` / `category.rs`（レジストリ側は
  無変更）。
- `crates/docs-site/tests/site_nav.rs`（`.filter().len()`/`.contains()`
  による件数・集合検証のみで順序非依存のため無改修で PASS した）。
- `crates/docs-site/tests/nav_group_schema.rs` /
  `sidebar_group_render.rs`（汎用 fixture ベースのテストで実サイトの
  nav.toml を読まないため無改修）。
- `.github/workflows/docs-site.yml` / `.claude/rules/ci.md`（新規ページ・
  新規 dist 出力パスを追加しないため paths glob・`test -f` 一覧の追随は
  不要）。

### out-of-scope 観察事項

- 2 段ネスト（区分見出し）は `nav.rs` の 1 段制限のため見送った（上記
  「2 段ネストは今回やらない」節参照）。
- ヘッダードロップダウン（`src/nav.rs::header_nav`）の Blocks 項目数が
  索引 1 件のみに縮小するのは、Primitives/Themes と同型の既知の副作用
  であり、本イシューでは対処しない。
