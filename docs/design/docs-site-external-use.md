# docs サイトの外部リポジトリ利用に向けた `[site]` 拡張方針

- ステータス: 設計確定（実装中）。`brand` / `repository_url` は #3720、`brand_mark` / `brand_color` は #3722、`tagline` / `copyright` / `version_badge` / `lang` は #3721 で実装済み。フラグは #3716 / #3717 で実装済み。外部利用の契約テストは #3724 で追加した（§7）。利用者向けの手順書は `docs/guides/docs-site-external-repos.md`（#3726）
- 起票元: #3715（親 #3714、ルート #3713）
- 範囲: `site/nav.toml` の `[site]` への追加キー、CLI フラグ `--no-page-sections`、帰属表記の方針。本文書はコード・CI・ruleset を変更しない
- 記載の区別: 「決定事項」は #3715 に記載された確定済みの判断（2026-10-05）。「本文書で定めた詳細」は決定事項を実装可能にするために本文書で補った提案で、実装イシューのレビューで確認する。後者をユーザー承認済みとは扱わない

## 1. 結論

- 旧方針「`[site]` スキーマは拡張しない」を、「サイトの同一性を示す表示値に限り、任意キーとして拡張する」へ改める
- 追加キーは 8 個（`brand` / `repository_url` / `tagline` / `copyright` / `version_badge` / `lang` / `brand_mark` / `brand_color`）。すべて任意で、未指定時は現行の出力と 1 バイトも変わらない
- registry 無効化は CLI フラグ `--no-page-sections` で行い、ショーケース注入も同じスイッチで止める
- 外部サイトでブランドを差し替えても、帰属表記と `LICENSE-MIT` / `LICENSE-APACHE` へのリンクは残す。消す・差し替えるキーは設けない

## 2. 現状の棚卸し（rev cf5edb9b8 時点）

行番号は変わるため関数名・定数名で示す。

| 対象 | 所在 | 現状 |
|------|------|------|
| `[site]` パース | `nav.rs` の `parse_nav`（`Ctx::Site` 分岐）・`Site`・`validate_base_path` | `title` / `base_path` の 2 キーが必須。未知キー・重複キーは `NavError::Parse`。値はダブルクォート文字列のみ。入力上限 1 MiB |
| ヘッダーのブランド名 | `layout.rs` の `brand()` | `"fandhe-frontend"` 固定（`[site].title` は使わない） |
| フッターのブランド名 | `site_footer.rs` の `site_footer()` | `[site].title` を表示。ヘッダーと供給元が異なる |
| GitHub リンク | `layout.rs` の `github_link()`、`site_footer.rs` の Resources 列、`landing.rs` | すべて `REPOSITORY_URL` 定数 |
| 版数 badge | `layout.rs` の `brand_version()`、`site_version.rs` | `core v{version}`。`crates/core/Cargo.toml` から取得し、解析失敗時は badge なし |
| crates.io リンク | `site_footer.rs` の `CRATES_IO_URL` | `fandhe-frontend-core` 固定 |
| タグライン・著作権 | `site_footer.rs` の `FOOTER_TAGLINE` / `COPYRIGHT_TEXT` | 固定文言 |
| ライセンス表記 | `site_footer.rs` の下段 | `Licensed under MIT OR Apache-2.0`（`LICENSE_MIT_URL` / `LICENSE_APACHE_URL` の 2 リンク）。"Built with ..." の文言は現状どこにも無い |
| favicon・ブランドマーク | `favicon.rs` の `mark_node()` | 塗り `#3182ce` の角丸タイルに `rect` で描いた白い "f"。`aria-label="fandhe-frontend"`。テストが `<text` 等を禁止 |
| `<html lang>` | `layout.rs` 末尾、`redirect.rs` の `redirect_document` | どちらも `"ja"` 固定。404 は layout 経由 |
| CLI | `main.rs` の `parse_args` | `--out` / `--root` のみ。未知引数はエラー |
| registry | `page_sections.rs` の `REGISTRY` / `EMPTY_REGISTRY`、`build.rs` の `build_site` / `build_site_with` | `build_site` は本番 `REGISTRY` 固定。Blocks / Wireframes / `component_page` の注入は registry と無関係に `page.path` 一致で走る |
| `/assets/` 集約ページ | `build.rs`・`menu_index.rs` | カードと CSS を registry を介さず直接配線する（`EMPTY_REGISTRY` でも出る） |

外部利用側は、生成後の HTML を後処理で置換して回避している。ヘッダーが固定文字列、フッターが `[site].title` という供給元の不一致も、後処理が必要になる一因である。

## 3. 方針の見直し

### 3.1 旧方針と見直しの理由

旧方針は次の 3 か所に残っていた。

- `layout.rs` の `REPOSITORY_URL` の rustdoc（`[site]` スキーマは拡張しない）
- `site_footer.rs` の `FOOTER_TAGLINE` の rustdoc（`nav.toml` のスキーマは広げない）
- `docs-site-styled-blocks-redesign.md` の「フッター 3 列」節（同趣旨）

docs-site を fandhe-frontend 以外のリポジトリから使う需要が出たため、表示値を後処理で置換させるより、検証付きの設定として公式に受けるほうが安全と判断する。

### 3.2 新方針の境界

今後 `[site]` へキーを足すときは、次の 6 条件をすべて満たす。

1. 任意キーで、未指定時は現行の出力と 1 バイトも変わらない
2. `parse_nav` 内で fail-closed に検証する（未知キー拒否は維持）
3. 出力は `text()` / `el()` の属性経由だけ。`raw_html()` と HTML 文字列の組み立ては使わない
4. 任意の HTML・CSS・JS、任意スキームの URL、ファイルパスを受け取るキーは設けない
5. 帰属表記を消す・差し替えるキーは設けない
6. 契約テストで固定する

## 4. `[site]` 追加キー

### 4.1 共通ルール（本文書で定めた詳細）

- 8 キーはすべて任意。`title` / `base_path` は従来どおり必須
- 値はダブルクォート文字列のみ（既存パーサの制約。真偽値は導入しない）
- 重複キー・未知キーは既存どおり `NavError::Parse`（行番号付き）。新しいエラー種別は作らない。エラーメッセージは英語
- 検証はエスケープ解釈後の値に対して行う（`\n` / `\t` で制御文字を持ち込めるため）
- 正規化（trim・大文字小文字変換）はしない。書かれた値をそのまま既定エスケープ経由で出す
- モデルは「未指定」を区別できる形（`Option`）で保持し、既定値の解決は 1 か所のアクセサにまとめる。`brand` のフッター側フォールバックと、帰属表記の切り替え判定（§6。`lang` 以外の 7 キーのいずれかが指定されたか）に必要なため

### 4.2 キー表

キー名と未指定時の方針は決定事項。許容値の細目は本文書で定めた詳細。

| キー | 許容する値 | 未指定時 | 反映先 | 実装 |
|------|------------|----------|--------|------|
| `brand` | 1〜64 文字。制御文字なし。空白のみは不可 | ヘッダーは `fandhe-frontend`、フッターのブランド名は `[site].title`、マークの `aria-label` は `fandhe-frontend`（現行の 3 か所をそれぞれ維持） | 指定時はヘッダー `a.docs-brand`・フッター `p.docs-footer-brand-name`・マークの `aria-label` の 3 か所へ同じ値 | #3720（`aria-label` は #3722） |
| `repository_url` | `https://` 始まりのみ。ASCII。空白・制御文字なし。ホスト部が空でない。2048 バイト以下 | `REPOSITORY_URL` 定数 | ヘッダーの GitHub リンクとフッター Resources 列の GitHub リンク。ライセンスリンクと帰属表記のリンクには影響させない | #3720 |
| `tagline` | 1〜200 文字。制御文字なし。空白のみは不可 | `FOOTER_TAGLINE` | フッター `p.docs-footer-tagline` | #3721 |
| `copyright` | 1〜200 文字。制御文字なし。空白のみは不可 | `COPYRIGHT_TEXT` | フッター下段 | #3721 |
| `version_badge` | 0〜32 文字。制御文字なし。空文字は「非表示」を表す唯一の指定 | `core v{version}`（解析失敗時は badge なし、現行どおり） | ヘッダー `span.docs-brand-version`。指定時（空文字を含む）は Resources 列の crates.io リンクも出さない | #3721 |
| `lang` | BCP 47 の構文サブセット。先頭サブタグは英字 2〜3 文字、後続は英数字 1〜8 文字を `-` 区切り、全体 35 文字以下。IANA レジストリとの照合はしない | `ja` | 通常ページ・404・リダイレクト案内の `<html lang>` | #3721 |
| `brand_mark` | ASCII 英数字ちょうど 1 文字 | 現行の `rect` で描いた "f" | `assets/favicon.svg` とヘッダーのインライン SVG（同じ図案） | #3722 |
| `brand_color` | `#` + 16 進 6 桁（大文字・小文字可）。3 桁省略形・色名・`rgb()` は不可 | `#3182ce` | マークのタイルの `fill` 属性（`style` 属性は使わない） | #3722 |

### 4.3 判断理由と既知の制約

- `version_badge` だけ空文字を許すのは、真偽値を持たないパーサで「非表示」を表すため。他のキーの空文字は誤記とみなして拒否する
- `version_badge` の非空の空白のみ（例: `" "`）は拒否する（#3721 で補った細目）。見えない badge を作らず、非表示は空文字だけに一本化するため。キー名・行番号・規則だけをエラーに出し、値は表示しない
- 実装状況: `tagline` / `copyright` / `version_badge` / `lang` は #3721 で実装済み（`Site::is_brand_customized` が `brand` / `repository_url` / `tagline` / `copyright` / `version_badge` の指定有無で帰属表記への切り替えを判定する。#3722 は自分のキーをこの判定へ足す）
- crates.io リンクを `version_badge` に連動させるのは、追加キーを決定事項の 8 個から増やさないため。badge も crates.io リンクも `fandhe-frontend-core` を指しており、badge を差し替える・消す外部サイトでは同じリンクも意味を失う
- `brand` 未指定時にヘッダーとフッターで供給元が異なるのは現行の挙動で、バイト一致のために維持する。指定時に初めて両者が揃う
- `lang` が変えるのは属性だけ。検索 UI の日本語文言、"Skip to content"、"On this page"、"Menu"、リダイレクト案内の文言は固定のままで、クローム文言の多言語化は本ツリーの範囲外
- `repository_url` を GitHub 以外へ向けても、リンクの文言 "GitHub" とアイコンは固定
- `brand_mark` 指定時のグリフ（#3722 で決定）: 指定時のみ SVG `<text>` で描く（`x=16 y=24 text-anchor=middle font-family=sans-serif font-size=22 font-weight=700 fill=#ffffff`、`dominant-baseline`・`style`・外部フォントは使わない）。未指定時は現行の `rect` 3 本の "f" を維持し、`"f"` の明示指定も `<text>` 側で描く（正規化しない）。`brand_color` のみの指定は "f" のままタイルの `fill` だけを変える。`favicon.rs` の `<text` 禁止テストは既定経路で維持する。既知の制約: (1) 描画はシステムの汎用フォント依存で、小文字や descender のある文字は上下中央が厳密には揃わない。(2) インライン SVG の `<text>` は `a.docs-brand` の DOM `textContent` に 1 文字加わる（ラッパーが `aria-hidden` のためアクセシブルネームと検索インデックスには影響しない）。62 文字ぶんの図形表は量が過大なため採らなかった
- `brand_color` と白いグリフのコントラストは検証しない（利用者の責任）
- 画像ファイルのパスを受け取るキーは設けない（パストラバーサルの入口を作らない）
- `landing.rs` の `REPOSITORY_URL` 参照と CLI 版数 badge は本番 registry 専用の内容で、`--no-page-sections` では出力されない
- `docs-site-three-column-redesign.md` の対応表は `[site].title` がヘッダーのブランドに対応すると書いていたが、実装は固定文字列だった。#3720 で `brand` を入れ、表記を合わせた
- #3720 の実装細目: 検証は `parse_nav` の `[site]` アーム内で行い（行番号付きの `NavError::Parse`）、エラーメッセージにはキー名と理由だけを載せ値は載せない。`repository_url` のホスト部は `https://` の後から最初の `/` `?` `#` までを authority とし、最後の `@` より後・最初の `:` より前が空でないことで判定する。ヘッダーへは `layout::SiteChrome`（`docs_page_with_chrome`）経由で渡し、マークの `aria-label` への `brand` 反映は #3722 で実装した。本番 registry のランディング（`landing.rs`）の GitHub リンクは `repository_url` の影響を受けない（外部利用は `--no-page-sections` 前提のため）
- #3722 の実装細目: `favicon::BrandMark`（`glyph` / `color`）・`mark_node_for` / `svg_for` を追加し、`SiteChrome.mark` を介してヘッダーのインライン SVG と `assets/favicon.svg` が同じ入力・同じ関数から作られる。マークの `aria-label` は `[site].brand`（未指定は `fandhe-frontend`）。検証は `parse_nav` の `[site]` アーム内（バイト列で判定、エラーに値を載せない）。`csp.rs` は変更しない

## 5. `--no-page-sections` とショーケース注入

- フラグ名は `--no-page-sections`（値なし、決定事項）。指定時は `build_site_with(.., &EMPTY_REGISTRY)` 相当、未指定時は現行どおり本番 `REGISTRY`（実装は #3716）
- 指定時は、`/themes/<部品名>/`・`/primitives/<部品名>/`・`/blocks/<id>/`・`/wireframes/<名前>/` に一致するページへのショーケース注入と、ショーケース用アセットの出力も止める（実装は #3717）。ショーケース専用の別フラグは設けず、スイッチは 1 つにする
- `/assets/` 集約ページ（`[[menu]]`、`menu_index.rs`）は `nav.toml` 由来で registry に依存しない（`build.rs` が registry を介さず直接配線する）ため、フラグ指定時も使える
- フラグと `[site]` キーは直交する。帰属表記の規則はフラグの有無に依存しない
- `nav.toml` のキーではなく CLI フラグにした理由: registry はバイナリに焼き込まれた本サイト専用の内容であり、サイト設定ではなくビルドの呼び出し方の選択だから

## 6. 帰属表記

- 決定事項: 外部サイトでブランドを差し替えても、"Built with fandhe-frontend docs-site" の帰属表記と `LICENSE-MIT` / `LICENSE-APACHE` へのリンクを残す。消す・差し替えるキーは設けない。`[site]` の未知キー拒否により、`attribution = ""` のような指定はビルドエラーになる
- 現状: 下段は `Licensed under MIT OR Apache-2.0` のみで、"Built with" の文言は無い

表示規則（本文書で定めた詳細）。

- 既定構成（`brand` / `copyright` / `repository_url` / `tagline` / `version_badge` / `brand_mark` / `brand_color` の 7 キーがすべて未指定）: 現行の下段をそのまま出す。バイト一致を保つ。`lang` は表示物を差し替えないため判定に含めない
- ブランド差し替え構成（上記 7 キーのうち 1 つでも指定。`brand_mark` / `brand_color` / `repository_url` / `tagline` / `version_badge` のみの指定も含む）: ライセンス行を `Built with fandhe-frontend docs-site (MIT OR Apache-2.0)` へ切り替える。"fandhe-frontend docs-site" は fandhe-frontend のリポジトリへ、MIT / Apache-2.0 は各ライセンス本文へリンクする
- どちらの構成でも、2 本のライセンスリンクがちょうど 1 回ずつ出る
- 切り替える理由: 外部の著作権表記の隣に "Licensed under MIT OR Apache-2.0" を置くと、外部サイトの内容がそのライセンスで提供されていると誤読される。対象を docs-site（生成器）に限定した文言へ変える
- リンク先（ライセンス本文 2 本と帰属表記のリポジトリ URL）は `repository_url` と独立した定数にし、`repository_url` の既定値と同じ値でも共有しない
- 実装の分担: `brand` は #3720、`copyright` と切り替え本体は #3721。#3720 だけがマージされた中間状態でも既存のライセンス行が残るので制約は満たされる
- #3724 の「帰属表記以外に `fandhe-frontend` が出ない」検査のため、帰属表記の DOM 範囲を識別できる形にすることを推奨する（class を足す場合は `STRUCTURE_CLASS_CONTRACT` への追加が要る。採否は #3721）。**#3721 の結論: class・属性は足さない**。フッター class は全件に `site.css` のセレクタを要求する契約（`site_css_contract.rs`）があり、class を足すと `site.css` が変わって未指定時のバイト一致が崩れるため。帰属表記は `div.docs-footer-bottom` の 2 番目の `p` で範囲を特定できる。目印が必要になった場合は、差し替え構成のときだけ `data-*` 属性を出す形で後から足せる（追加的な変更で、未指定時の出力は変わらない）。生成 CSS の `--fandhe-*` 変数名などは表示ではないので、検査範囲は #3724 で次のとおり定めた（#3724 で定めた詳細。ユーザー承認済みではない）。
  - 対象は全出力ファイル（HTML・CSS・JS・JSON・SVG・リダイレクト案内）と出力の相対パス名。検査語は `fandhe-frontend` と `Fandhe-AI`（ASCII 大文字小文字無視）。
  - 許容は、クロームを持つ HTML の `div.docs-footer-bottom` の 2 番目の `p`（帰属表記）のみ。
  - `fandhe` だけを含む内部識別子（`--fandhe-*`・`#fandhe-skip-nav`・localStorage キー `fandhe-docs-theme`）と、CLI の標準出力・標準エラーは対象外。
  - 帰属表記の外から `fandhe-frontend` を消すのに必要十分なキーは `brand` / `repository_url` / `copyright` / `version_badge` の 4 つ。`tagline` / `lang` / `brand_mark` / `brand_color` は消去には不要。
- リダイレクト案内ページはサイトクロームを持たないため帰属表記の対象外（現行どおり）

## 7. 後方互換と検証方針

- 不変条件: キーもフラグも指定しないビルドは、変更前と全ファイルがバイト一致する
- 各実装 PR の確認手順: 変更前後で `cargo run -p fandhe-frontend-docs-site -- --out <一時ディレクトリ>` を実行し、`diff -r` が空であること。既存の契約テスト（`crates/docs-site/tests/`）は弱めない
- 各実装イシューで、キーごとの検証テスト（正常値・境界値・拒否値）と、指定時の反映・エスケープのテストを足す

- #3724 で定めた詳細: 外部リポジトリ相当の最小サイトを `crates/docs-site/tests/fixtures/site-external` にチェックインし、`crates/docs-site/tests/external_site_contract.rs` がバイナリ（`--no-page-sections`）経由で生成して、生成ファイル一覧の完全一致・ブランド反映・帰属表記の外の `fandhe-frontend` 不在・予約パス（`/themes/accordion/` と `/primitives/accordion/`）へのショーケース非注入を固定する。#3725 の CI ジョブも同じ fixture を入力にできる

## 8. セキュリティ不変条件

- インジェクション / XSS（A03）: 追加キーの値はすべて `text()` と属性の既定エスケープを通す。`repository_url` は `https://` の許可リスト方式で `javascript:` / `data:` を構造的に排除する。`brand_color` は 16 進 6 桁、`brand_mark` は英数字 1 文字、`lang` は英数字と `-` に限り、SVG や属性へ記号を持ち込ませない
- パストラバーサル（A01）: ファイルパスを受け取るキーは設けない。既存の `source` 検証は変えない
- 安全でない設計（A04）: 検証は fail-closed。未知キー・重複キーの拒否を維持し、既存の入力上限（1 MiB）と各キーの長さ上限で入力量を抑える
- 設定ミス（A05）: CSP（`src/csp.rs`）は変えない。インライン `<script>` / `<style>` は 0 個のまま、色は `fill` 属性で渡す。外部リンクは `external: true`（`target="_blank"` と `rel="noopener noreferrer"`）を維持する。外部フォント・CDN を読まない
- 脆弱な依存（A06）: 依存クレートを足さない。検証は標準ライブラリだけで書く
- 整合性（A08）: 帰属表記とライセンスリンクは設定から独立した定数にし、消す手段を作らない
- SSRF（A10）: ビルド時に `repository_url` へアクセスしない（文字列として出力するだけ）
- エラーメッセージに出すのはキー名・行番号・利用者自身の設定値まで

## 9. 再評価トリガーの判定

`docs-site-three-column-redesign.md` §10 のトリガー 1〜7 に対する判定。

| # | 判定 | 根拠 |
|---|------|------|
| 1〜4 | 非該当 | 任意キーとフラグの追加だけで、骨格・CSS 供給・契約テスト・配信方式を変えない |
| 5 | 非該当 | `[section]` の `index_path` 必須検証は変えない |
| 6 | 非該当 | `header_nav` の列挙を変えない |
| 7 | 非該当 | 層セクションの追加・改称・境界変更を行わない |

## 10. 関連文書・関連イシュー

- `docs/design/docs-site-styled-blocks-redesign.md` / `docs-site-three-column-redesign.md` / `docs-site-csp-policy.md`
- `crates/docs-site/src/` の `nav.rs` / `layout.rs` / `site_footer.rs` / `favicon.rs` / `site_version.rs` / `page_sections.rs` / `build.rs` / `main.rs`
- `docs/guides/docs-site-external-repos.md`（利用者向けガイド、#3726）
- #3713（ルート）/ #3714 / #3716 / #3717 / #3718 / #3720 / #3721 / #3722 / #3724 / #3726
