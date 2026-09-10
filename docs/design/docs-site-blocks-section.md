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

## 11. `signup-01`（#2094）実装記録

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
