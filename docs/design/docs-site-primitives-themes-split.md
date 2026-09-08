# docs サイト Primitives / Themes 2 層分割の設計

**本文書のステータス**: 確定(イシュー #1015、親 #1019 / トラッキング #1035)。
Phase 3(#1016〜#1018)・Phase 4(#1020〜#1022)・Phase 5(#1024〜#1029)の
設計正である。実測値の基準は `origin/main` = d3013f1(再実測手順は §6 に記載)。

## 1. 背景・目的

トラッキング #1035「Radix UI 同型の Primitives / Themes 2 層 docs サイトへの
刷新」配下で、Phase 3(#1019)以降の 4 Phase・15 issue 超が同一の前提に
立って実装できる設計の正が存在しなかった。特に以下 3 点が未確定のままだと
後続で矛盾・手戻りが確定的に発生する。

1. **URL 移行の是非と根拠**: `docs/design/docs-site-component-pages.md` §8 が
   「`/components/pre-styled-ui/` の URL を変更しない」と確定済みであり、
   Phase 3 の `/themes/` 移行はこの確定と正面から衝突する。上書きの明示なし
   に移行すると設計文書と実装の不整合として後続レビューが止まる。
2. **旧 URL 互換方式と 4 つの fail-closed 契約への例外の形**: GitHub Pages に
   サーバーサイドリダイレクトが無いため生成ページに頼らざるを得ず、
   `nav.toml` / `search-index.json` / `linkcheck` / `site_nav.rs` の 4 契約
   すべてに例外が要る。例外の形(汎用除外リストか、構造的分離か)を後続の
   実装者に委ねると、fail-closed 性を壊す汎用除外リストが生まれるリスクが
   高い。
3. **Primitives 台帳・カテゴリ分類の一次情報の所在**: `component-coverage-map.md`
   は過去に「実装済み mod 38」と記載しながら実測 67 という既知ドリフト
   (#934 で全件再実測が必要になった)を抱えている。同じ失敗を繰り返さない
   ため、台帳は「コードから機械判別できる規則」として文書化する必要がある。

本文書は上記 3 点を含む 7 項目を 1 文書へ確定し、#1016 / #1017 / #1018 /
#1020 / #1021 / #1022 / #1024〜#1029 の実装者が本文書だけで判断を完結できる
状態にすることを目的とする。

**既存文書との責務境界**:

| 文書 | 責務の正 |
|---|---|
| `docs/design/component-coverage-map.md` | ark-ui / chakra-ui / Radix の 3 参照軸コンポーネント対応表 |
| `docs/design/docs-site-component-pages.md` | 部品ページ IA(雛形構成・カテゴリ分類の一般規約) |
| 本文書 | docs サイトの 2 層分割・URL 体系・Primitives 台帳判別規約 |

## 2. 対応関係と命名

| サイト表示名 | 対応クレート | セクション URL |
|---|---|---|
| Primitives | `fandhe-frontend-headless-ui` | `/primitives/` |
| Themes | `fandhe-frontend-pre-styled-ui` | `/themes/` |

表示名は Radix パリティのため **Primitives / Themes** とする。CLAUDE.md の
既定方針(headless-ui = Radix Primitives、pre-styled-ui = Radix Themes)に
従った命名であり、新規の判断ではない。

**各セクショントップ(`/primitives/` = #1021、`/themes/` = #1018)の冒頭で
対応クレート名を明記する**ことを両 issue への必須要件とする。利用者が
crate 名と結び付けられるようにするためである。

ヘッダー上の並びは **Primitives → Themes**(Radix の並びに合わせる。#1021
実装内容 4 と整合)。

## 3. URL 体系

| 対象 | URL | 件数 |
|---|---|---|
| Primitives 索引 | `/primitives/` | 1 |
| Primitives 部品 | `/primitives/<kebab>/` | 67(イシュー #2059 で `button_group` が追加され 63→64、イシュー #2062 で `input_group` が追加され 64→65、イシュー #2065 で `item` が追加され 65→66、イシュー #2068 で `command` が追加され 66→67) |
| Themes 索引 | `/themes/` | 1 |
| Themes 部品 | `/themes/<kebab>/` | 113(`/components/<kebab>/` から移転。イシュー #1683 で 107→108、イシュー #1685 で 108→109、イシュー #1687 で 109→110、イシュー #2063 で 110→111、イシュー #2066 で 111→112、イシュー #2060 で 112→113) |
| 旧 URL(移転案内) | `/components/<kebab>/` ほか | 115 |

`<kebab>` は mod 名の `_` → `-` 置換で機械導出する
(`docs-site-component-pages.md` §4 の規約をそのまま継承。`nav::validate_page_path`
のセグメント allowlist から導かれた選択であり緩和ではない)。

115 件の内訳は「110 実在部品ページ(イシュー #1683 で追加した
`/components/collapsible/`・イシュー #1685 で追加した `/components/field/`
を含む) + 1 実在索引(`/components/pre-styled-ui/`)
+ 1 未提供 URL(`/components/`)への予防的移転案内」である
(イシュー #2063 で `/components/input-group/` が加わり 113 件、イシュー
#2066 で `/components/item/` が加わり 114 件、イシュー #2060 で
`/components/button-group/` が加わり 115 件になった)。`/components/` は
**現在ページとして存在しない URL**(`site/nav.toml` に該当 `path` なし)だが、
将来の誤アクセス・外部からの推測アクセスに備えて移転案内対象へ含める。

### §8 の上書き宣言

`docs-site-component-pages.md` §8 は次のとおり確定している(原文引用)。

> URL(`/components/pre-styled-ui/`)は既存の被リンク(`docs/api/pre-styled-ui-api.md`
> / `docs/api/pre-styled-recipe-api.md` からのリンク、外部からのブックマーク)を
> 壊さないため **変更しない**。

**本設計はこの確定を上書きする(URL は変更する)**。

上書き後も §8 の本来の意図は保たれる。§8 の理由は「既存の被リンクを壊さない」
ことであって「URL 文字列そのものを不変に保つ」ことではない。§4 の移転案内
機構により旧 URL は 404 にならず、被リンク・外部ブックマークは移転先へ到達
する。加えて #1017 はリポジトリ内(`docs/api/*.md` 等)の被リンクを新 URL へ
張り替えるため、リポジトリ内リンクは移転案内を経由せず canonical を直接
指す。

**`docs-site-component-pages.md` §8 の本文改訂は Phase 6 で行う。本 issue
(#1015)・本文書では `docs-site-component-pages.md` を編集しない。**

## 4. 旧 URL 互換の方式

GitHub Pages に静的リダイレクト機能は無い。旧 URL を維持する唯一の手段は
生成ページである。

生成する `<from>/index.html` は次の 4 要素を必ず持つ。

1. `<meta http-equiv="refresh" content="0; url={to}">`
2. `<link rel="canonical" href="{to}">`
3. `<meta name="robots" content="noindex">`
4. **JS を使わない本文フォールバック**(移転先への `<a>` リンク + 1 行説明)。
   JS 無効環境・`meta refresh` を無視する環境でも遷移できること
   (`no_js_contract.rs` の思想と整合)。

### 4 契約への例外の「形」(fail-closed 性の毀損を最小化する構造規定)

リダイレクトが「特定のページを検査から外す汎用の除外リスト」を生まない
よう、例外を**構造的分離**として実装する。

> リダイレクトは `Nav` のページ集合へ一切入らない**別の宣言・別の型・別の
> レンダラ**として実装する。宣言は `nav.toml` の `[[section.page]]` とは
> 独立した供給元(`site/redirects.toml` の新設、または `nav.toml` 内の専用
> トップレベルテーブル `[[redirect]]`。どちらにするかは #1016 の実装判断と
> する)から読み、`Nav` の走査結果には現れない。
>
> この構造を取ると `nav.toml` 由来のサイドバー・ヘッダー・目次、
> `search_index::page_entry` の収集、`linkcheck::check_links` の解決先集合、
> `site_nav.rs` のページ数期待値は**いずれも Nav ページのみを対象とする現行
> 実装のままで、除外述語を 1 つも書かずに例外が成立する**。「特定ページを
> 検査対象から外すフラグ・除外リスト」は導入しない(`security.md` A05、
> fail-closed 方針)。

### 契約ごとの帰結(#1016 の受け入れ条件へ 1:1 対応)

| 契約 | 扱い |
|---|---|
| `site/nav.toml` | リダイレクトは nav に登録しない(そもそも別供給元)。サイドバー・ヘッダー・目次に現れない |
| `search-index.json`(`src/search_index.rs`) | Nav ページのみを索引するため自動的に含まれない。既存の決定性・エスケープ・サイズ上限テストは弱めない |
| `linkcheck`(`src/linkcheck.rs`) | `to` が実在 Nav ページであることは検証する。`from` はサイト内リンクの解決先として扱わない。加えて肯定形の規約: **サイト内リンクは 1 本たりとも `from`(旧 URL)を指してはならない**(#1017 は全リポジトリ内リンクを canonical `/themes/<kebab>/` へ張り替える。旧 URL を linkcheck の allowlist へ足して解決するのは禁止。移転案内は外部トラフィック・ブックマーク専用) |
| `site_nav.rs` のページ数期待値 | 本体ページ数へ含めない。リダイレクト件数は別の期待値として fail-closed に固定する(#1018 完了時点で 109、#1683 で 110、#1685 で 111、#1687 で 112、#2063 で 113、#2066 で 114、#2060 で 115) |

### ビルド時 fail-closed 検証(#1016 の必須要件)

- `to` が実在 Nav ページを指さない宣言はビルド失敗。
- `from` が既存の本体ページ `path` と衝突する宣言はビルド失敗(リダイレクト
  が実ページを覆い隠す事故の防止)。
- `to` はサイト内絶対パスに限る(§7 セキュリティ参照)。`nav::validate_page_path`
  と同一のセグメント allowlist(`/` 始まり・`/` 終わり・英数と `-` `_` のみ)
  を通し、スキーム付き URL・protocol-relative `//`・`..` を拒否する。
- `from` / `to` の HTML 属性値出力は `fandhe_frontend_core` のノード木 API
  経由とする(`raw_html()` 禁止、REQ-1)。

## 5. 部品ページ雛形の層差

関数名で参照する(行番号は `d3013f1` 時点のもので refactor により陳腐化する
ため、関数名を主・行番号を従とする)。

| 節 | Themes(pre-styled-ui) | Primitives(headless-ui) |
|---|---|---|
| Demo | `showcase::generated_content`(pre-styled-ui API 直呼び) | 新規供給が必要(#1022)。pre-styled-ui の再エクスポート経由で headless-ui 部品を呼ぶ unstyled デモ。`crates/docs-site/Cargo.toml` へ headless-ui 直接依存を追加しない(#693 方針) |
| Anatomy | `component_page::collect_anatomy_parts`(d3013f1 時点 L777)が Demo ノード木の `data-part` を走査 | 同一ロジックを流用可(層非依存。走査対象は `data-scope`/`data-part` のみで層に依存しない) |
| `data-*` 属性表 | `component_page::collect_data_attrs_from_tree`(同 L846)が Demo ノード木の `data-*` を走査 | 同一ロジックを流用可(層非依存) |
| CSS 変数表 | `component_page::collect_css_vars_for_scope`(同 L927)が `showcase::stylesheet()` から `--fandhe-<scope>-*` を抽出 | 恒常的に省略。headless-ui に CSS の概念が無く、抽出元が存在しない |
| Demo ラッパ class | `div class="pre-styled-showcase"`(同 L390 固定) | 引数化が必要(#1021)。新 class 名は `site_css_contract.rs` の双方向契約表へ登録する |
| Features / API 引数表 / Examples / Accessibility | `component_specs*`(`component_specs/` + `component_specs_overlay.rs` + `component_specs_nav_data.rs` + `component_page_specs_948.rs`) | 新規の原稿データセット(#1021 でレジストリ新設、#1024〜#1029 で充填) |

**既存制約の継承**: `component_page.rs` モジュール doc(d3013f1 時点 L21-32)
の「Anatomy / `data-*` は Demo が実際に描画したパーツの部分集合しか出ない
(デモが描画しなかったパーツは導出結果に含まれない)」という制約は、走査
ロジックが層非依存であることの帰結として **Primitives 側でも同じく成立
する**。Primitives の原稿執筆者(#1024〜#1029)は「Anatomy 表に出ないパーツ
がある = バグ」と誤読しないこと。完全列挙のための headless-ui へのパーツ
列挙 API 追加は、公開クレートのバンプと #693 方針への抵触を伴うため引き
続き見送る。

節が出力されない理由の全体(4 分類)と分類ごとの編集者の対応の正は
`docs-site-component-pages.md` §7b(イシュー #1082)であり、本節では
規則本文を再掲しない。

## 6. Primitives 台帳の判別規約

**判別規則**: `crates/headless-ui/src/*.rs` のうち `anatomy(` を呼ぶもの
(`anatomy.rs` 自身を除く)= **部品 67 件**(イシュー #2059 で `button_group`・
イシュー #2062 で `input_group` がそれぞれ追加され 63→65、イシュー #2065 で
`item` が追加され 65→66、イシュー #2068 で `command` が追加され 66→67)。

**基盤モジュール 9 件**(部品ではない): `anatomy` / `aria` / `color` /
`data_attrs` / `date` / `format` / `positioning` / `qr_encode` / `state`。

66 + 9 + `lib.rs` = **76** = `crates/headless-ui/src/*.rs` の総数(旧
63 + 9 + 1 = 73、直近は 65 + 9 + 1 = 75)。

**`collapsible` / `field` / `fieldset` は `anatomy()` を持つ実部品**であり
基盤ではない(CLAUDE.md の記述から基盤と誤読しないこと)。

**glob の穴を塞ぐ規定**: 上記規則はトップレベルの `src/*.rs` glob 上で定義
されている。将来 `crates/headless-ui/src/<name>/mod.rs` のような入れ子
モジュールディレクトリが追加されると glob をすり抜けて台帳から静かに漏れる。
#1020 の fail-closed テストは同一 glob を使うだけでなく、**`crates/headless-ui/src/`
直下にディレクトリが現れた場合にも落ちること**を要件とする。

**手書きリストの目視同期に委ねない**。#1020 は「headless-ui の全モジュール
が、台帳か基盤リストのどちらかちょうど一方に属する」ことを検証し、
(a) 未登録の新規モジュール追加、(b) 台帳掲載モジュールの消滅、(c) 台帳と
基盤リストの重複、のいずれでも落ちるテストを置く。

### 再実測手順

`component-coverage-map.md` / `docs-site-component-pages.md` の慣例に倣い、
コマンドをそのまま埋める。

```bash
# 部品 67 件（イシュー #2059 で button_group・イシュー #2062 で
# input_group・イシュー #2065 で item・イシュー #2068 で command が
# それぞれ追加され 63→65→66→67）
grep -l 'anatomy(' crates/headless-ui/src/*.rs | grep -v '/anatomy.rs' | wc -l   # => 67
# 総数 77(= 67 + 基盤 9 + lib.rs、旧 73)
ls crates/headless-ui/src/*.rs | wc -l                                            # => 77
# Themes 部品ページ 113 件（イシュー #1017 で site/components/ から site/themes/ へ移行済み、
# イシュー #2063 で input_group、イシュー #2066 で item、イシュー #2060 で
# button_group の Themes ページが加わり 110→113）
ls site/themes/*.md | wc -l                                                       # => 113
```

## 6a. ラップ状態の判別規約(層をまたぐ対応関係、イシュー #1064)

§6 は headless-ui ソース ↔ Primitives 台帳の**レイヤー内**ドリフト検知
(`tests/primitives_catalog.rs`)の規約である。本節は Primitives(67 部品、
イシュー #2059 で `button_group`・イシュー #2062 で `input_group`・
イシュー #2065 で `item`・イシュー #2068 で `command` をそれぞれ追加、
旧 63)と Themes(114 部品、イシュー #2063 で `input_group`・イシュー #2066
で `item`・イシュー #2060 で `button_group`・イシュー #2070 で `command`
をそれぞれ追加、旧 110)の**層をまたぐラップ状態**
(どの Themes ページが
どの headless 部品をラップしているか)の判別規約であり、対応する契約
テストは `crates/docs-site/tests/wrap_state.rs`(イシュー #1064)。

**Primitives 側の未ラップ判定(イシュー #2059/#2065/#2068)**: `button_group`
はイシュー #2059 で headless-ui 層のみを実装しており、pre-styled-ui
recipe・Themes ページを持たなかったが、イシュー #2060 で pre-styled-ui 側
（`crates/pre-styled-ui/src/button_group.rs`・`/themes/button-group/`）を
新設し `WRAPPED_SAME_NAME` へ移った(`HEADLESS_UNWRAPPED`
(`tests/wrap_state.rs`)・`PRIMITIVES_WITHOUT_THEMES_PAGE`
(`src/primitives_catalog.rs`)の両リストからは除外済み)。`item` も同様に
イシュー #2065 で headless-ui 層のみを実装していたが、イシュー #2066 で
pre-styled-ui recipe・Themes ページを追加し `WRAPPED_SAME_NAME` へ移った
(両リストからは除外済み)。`command` も同様にイシュー #2068 で
headless-ui 層のみを実装していたが、イシュー #2070 で pre-styled-ui
recipe・Themes ページを追加し `WRAPPED_SAME_NAME` へ移った(両リストから
除外済み)。

### 名寄せキー

`site/nav.toml` の `source = "site/themes/<kebab>.md"` から取り出した
`<kebab>` を `_` へ戻したもの(snake_case)を Themes 側キー、
`primitives_catalog::PRIMITIVES` の `module` を Primitives 側キーとする
(`path` ではなく `source` をキーにする理由は `tests/primitives_catalog.rs`
と同じ。URL 移転(#1017 の `/components/` → `/themes/`)に耐えるため)。

### 判別シグナル(コードレベル、rustdoc を根拠にしない)

`crates/pre-styled-ui/src/**/*.rs` の各モジュールについて、**非コメント行**
に現れる `fandhe_frontend_headless_ui::<module>` のうち `<module>` が
Primitives 台帳の 67 部品名(イシュー #2059 で `button_group`・イシュー
#2062 で `input_group`・イシュー #2065 で `item`・イシュー #2068 で
`command` をそれぞれ追加、旧 63)に一致するものを
「コード委譲あり」とする。

rustdoc(`//!` / `///`)の言及は**ラップの根拠にしない**。rustdoc に 1 文
足すだけでカテゴリが変わる壊れやすい契約を避けるため、コード実体(`pub use`
や関数呼び出し)を伴う参照のみを「ラップ済み」と呼ぶ。

### Themes 114 部品の 4 バケット分割

| バケット | 件数 | 定義 |
|---|---|---|
| WRAPPED_SAME_NAME | 67 | 同名の Primitives 部品が存在し、かつ同名 headless モジュールへコード委譲している(イシュー #1685 で `field`、イシュー #1687 で `fieldset`、イシュー #2063 で `input_group`、イシュー #2066 で `item`、イシュー #2060 で `button_group`、イシュー #2070 で `command` を追加) |
| WRAPPED_CROSS_NAME | 4 | 同名 Primitives 部品は無いが、別名の headless 部品へコード委譲している |
| DOC_REFERENCE_ONLY | 5 | headless 部品への参照が rustdoc のみ(コード委譲なし) |
| PRE_STYLED_ONLY | 38 | headless 部品への参照がコード・rustdoc いずれにも無い |

一覧の正は `crates/docs-site/tests/wrap_state.rs` の定数
(`WRAPPED_SAME_NAME` / `WRAPPED_CROSS_NAME` / `DOC_REFERENCE_ONLY` /
`PRE_STYLED_ONLY`)であり、本設計文書へは複製しない(`.claude/rules/ci.md`
の「yml・ci.md では二重管理しない」と同じ二重管理回避の方針)。

**`DOC_REFERENCE_ONLY` は設計上ドリフトしやすいバケットである**。C ↔ D の
移動は多くの場合「欠陥」ではなく「台帳更新イベント」であり、テストが落ちた
らまず実態(rustdoc の言及有無)を確認し、定数を更新して PR 本文に理由を
書く運用とする。

**受け入れ条件の中核**: 「同名 Themes ページがあるのに同名 headless へ委譲
していない」部品は現時点で 0 件であり、この 0 件を
`no_same_name_themes_page_reimplements_its_primitive` テストが恒久固定する
(`docs/policy/intentional-non-adoption.md` §3.25 が最も警戒する独自実装の
兆候)。

**イシュー #2059/#2060/#2062/#2063/#2065/#2066/#2068 追記(2026-09-08)**:
headless-ui 層のみを新設した `button_group`(#2059)・`input_group`(#2062)・
`item`(#2065)・`command`(#2068)はいずれも新設時点では上記「0 件」を暫定的
に増やしていたが、`input_group` は #2063 で、`item` は #2066 で、
`button_group` は #2060 で、それぞれ Themes 側
(`crates/pre-styled-ui/src/{input_group,item,button_group}.rs`・
`/themes/{input-group,item,button-group}/`)を新設し `WRAPPED_SAME_NAME`
バケットへ分類されたため、`HEADLESS_UNWRAPPED`/`PRIMITIVES_WITHOUT_THEMES_PAGE`
はいずれも `command` のみの 1 件へ戻った(#2068 の Themes ページは後続
#2070 で追加予定)。

### Primitives 側の未ラップ判定

`crates/pre-styled-ui/src/**/*.rs` 全体のコード行から、どこからも参照され
ていない headless 部品モジュールを求めると現時点で 1 件
(`HEADLESS_UNWRAPPED = ["command"]`。上記「イシュー
#2059/#2060/#2062/#2063/#2065/#2066/#2068 追記」節参照。`collapsible` は
イシュー #1682、`fieldset` はイシュー #1686、`input_group` はイシュー
#2063、`item` はイシュー #2066、`button_group` はイシュー #2060 でそれぞれ
コード委譲済みになったため本台帳から外れた。`command` はイシュー #2068
時点では headless-ui 層のみの新設のため引き続き本台帳に残る)。`field` は
イシュー #1685 で `/themes/field/` ページを新設し
`PRIMITIVES_WITHOUT_THEMES_PAGE` から除外済みで、`WRAPPED_SAME_NAME`
バケットへ分類される。`fieldset` も同様にイシュー #1687 で
`/themes/fieldset/` ページを新設し `PRIMITIVES_WITHOUT_THEMES_PAGE` から
除外済みで、`WRAPPED_SAME_NAME` バケットへ分類される。`input` / `textarea` /
`native_select` の 3 モジュールは引き続き `field` を別名でコード委譲して
いる(`WRAPPED_CROSS_NAME`)。2 つの台帳(ページレベルの `PRIMITIVES_WITHOUT_THEMES_PAGE` と、
コードレベルの `HEADLESS_UNWRAPPED`)が独立にドリフトして片方が黙って
嘘をつく事故を防ぐため、
`unwrapped_ledger_is_consistent_with_primitives_without_themes_page` が
`HEADLESS_UNWRAPPED ⊆ PRIMITIVES_WITHOUT_THEMES_PAGE` と
`PRIMITIVES_WITHOUT_THEMES_PAGE ∖ HEADLESS_UNWRAPPED == ∅` を機械検査する。

### モジュール解決(`crates/pre-styled-ui/src/`)

Themes ページ 113 件すべてが `crates/pre-styled-ui/src/` 配下のちょうど
1 つのソースへ解決する。解決順は (1) `charts` ページの特例
(`src/charts/mod.rs`)、(2) トップレベル `src/<mod>.rs`、(3) `src/charts/<mod>.rs`。
トップレベルと `charts/` のステムが衝突する既知の 1 件(`tooltip`。トップ
レベルが汎用 headless Tooltip のラッパー、`charts/tooltip.rs` はチャート
内部専用の非ページモジュール)はトップレベルを優先する
(`TOP_LEVEL_WINS_COLLISIONS`)。

`crates/pre-styled-ui/src/` の走査規約は §6 のフラット規約(ディレクトリの
出現そのものを panic させる)とは意図的に異なる。`charts/` が正当に存在
するため、`NESTED_MODULE_DIRS = ["charts"]` のみ許容し、深さは 1 段まで、
それ以外のサブディレクトリ・非 `.rs` エントリ・symlink は fail-closed に
panic する。§6 の弱体化ではなく別レイヤー向けの規約である。

### 再実測手順

```bash
# Themes 部品ページ 113 件（イシュー #2063 で input_group、イシュー #2066 で
# item、イシュー #2060 で button_group が加わり 110→113）
grep -oE 'source = "site/themes/[a-z0-9-]+\.md"' site/nav.toml | wc -l          # => 113
# Primitives 部品 67 件（イシュー #2059 で button_group・イシュー #2062 で
# input_group・イシュー #2065 で item・イシュー #2068 で command が
# それぞれ追加され 63→65→66→67）
grep -c 'path: "/primitives/' crates/docs-site/src/primitives_catalog.rs        # => 67
# pre-styled ソース総数 127(トップレベル 113 + charts/ 14)
ls crates/pre-styled-ui/src/*.rs | wc -l                                        # => 113
ls crates/pre-styled-ui/src/charts/*.rs | wc -l                                 # => 14
```

## 7. Primitives のカテゴリ分類

**本節は #1024〜#1029 の「対象部品」節からの転記であり、再導出・変更をして
はならない**。変更すると Phase 5 の 6 issue と矛盾する。各グループの部品名
は対応 issue が一次情報。

| グループ | 部品数 | 対応 issue |
|---|---|---|
| Forms A | 11 | #1024 |
| Forms B | 11 | #1025 |
| Forms C・日付・状態表示 | 10 | #1026 |
| Overlay / Disclosure | 10 | #1027 |
| Navigation | 11 | #1028 |
| Data Display / Utilities | 10 | #1029 |
| **合計** | **63** | — |

各グループの部品名(`_` 表記。ページ path は `_` → `-`)。

- **Forms A(11、#1024)**: `angle_slider` `checkbox` `checkbox_group`
  `color_picker` `combobox` `editable` `field` `fieldset` `file_upload`
  `image_cropper` `listbox`
- **Forms B(11、#1025)**: `number_input` `password_input` `pin_input`
  `radio_group` `rating_group` `segment_group` `select` `signature_pad`
  `slider` `switch` `tags_input`
- **Forms C・日付・状態表示(10、#1026)**: `calendar` `date_input` `date_picker`
  `download_trigger` `toggle` `toggle_group` `clipboard` `timer` `progress`
  `qr_code`
- **Overlay / Disclosure(10、#1027)**: `accordion` `collapsible` `dialog`
  `drawer` `floating_panel` `hover_card` `popover` `toast` `toggle_tip`
  `tooltip`
- **Navigation(11、#1028)**: `action_bar` `breadcrumb` `link` `link_overlay`
  `menu` `menubar` `nav_list` `navigation_menu` `pagination` `tabs` `toolbar`
- **Data Display / Utilities(10、#1029)**: `avatar` `carousel`
  `json_tree_view` `scroll_area` `skip_nav` `splitter` `steps` `tour`
  `tree_view` `visually_hidden`

**転記の検証記録**: 上記 6 グループの和集合と §6 のコード導出 63 件(#2062
以前の値)を `diff` で突合し、**差分なし(集合一致)** を確認済み。

**イシュー #2062 追記(2026-09-08、Phase 4)**: 上記の逐語転記(#1024〜
#1029 の対応 issue 一次情報)は変更しない。本イシューは Forms A へ
`input_group`(12 件目、`image_cropper` と `listbox` の間)を追加し、
Forms A は 11 → 12、6 グループ合計は 63 → 64 になった。§6 のコード導出
64 件との一致は `crates/docs-site/tests/primitives_catalog.rs::category_counts_and_order_follow_the_design_spec`
が機械検査する(本節末尾の表・部品名一覧は据え置き、本追記のみを正とする)。

```bash
grep -l 'anatomy(' crates/headless-ui/src/*.rs | grep -v '/anatomy.rs' \
  | xargs -n1 basename | sed 's/.rs$//' | sort > /tmp/code63.txt
# #1024〜#1029 の「対象部品」節を連結して sort したものを /tmp/issue63s.txt とする
diff /tmp/code63.txt /tmp/issue63s.txt   # -> 差分なし(IDENTICAL)
```

`site/nav.toml` の `[[section.group]]` 構成(#1021)はこの分類・この順序に
従う。**Phase 4/5 の実装者が独自の下位グループを発明することを禁じる**
(`docs-site-component-pages.md` §5 の同趣旨規定を継承)。

**追記(イシュー #2059、Phase 4 #2057、親 #2058)**: #1024〜#1029 完了後に
新設された部品 `button_group`(shadcn/ui Button Group 相当、参照軸 #2001)
を Navigation カテゴリへ追加する(`toolbar`(既存 Navigation 部品)の直後、
`breadcrumb` の後に配置)。上記の転記本文(#1024〜#1029 節)は再導出・書き
換えの対象外のため変更しないが、現在の実カテゴリ内訳は Navigation
11→12・合計 63→64 である。カテゴリ選定の根拠: 最も近い先例 `toolbar` の
所属グループであること、および本節が新規下位グループの発明を禁じている
ことからの推論(#2058/#2057 自体はカテゴリを指定していない)。後続の
Phase 4 部品(`input_group` 等)も同じ規則でカテゴリを決めることを想定する。

**base 再取り込み時点の合算(2026-09-08)**: #2059(`button_group`、
Navigation 11→12)と #2062(`input_group`、Forms A 11→12)は独立に
進行し、いずれも起点は 63 件だった。両方を取り込んだ現在の実カテゴリ
内訳は Forms A 12・Navigation 12・6 グループ合計 63→65 であり、§6 の
コード導出 65 件との一致は
`crates/docs-site/tests/primitives_catalog.rs::catalog_has_65_entries_in_six_categories_in_spec_order`
が機械検査する(本節末尾の表・部品名一覧・上記各追記は据え置き、本追記
のみを合算値の正とする)。

**追記(イシュー #2068)**: shadcn/ui のみに存在する `command`（Command、
参照軸 #2001）を Forms A カテゴリへ `combobox` の直後（ARIA 系譜が近い
`combobox`/`listbox` と同じグループ、`editable` の前）に追加する。
`button_group` #2059 と同じ経緯で headless-ui 層のみを先行実装し
Themes ページは後続 #2070 で追加するため、`PRIMITIVES_WITHOUT_THEMES_PAGE`
/ `HEADLESS_UNWRAPPED`（`tests/wrap_state.rs`）へも `command` を追加した
（2 件目）。現在の実カテゴリ内訳は Forms A 13・6 グループ合計 65→66 で
あり、§6 のコード導出 66 件（headless-ui `crates/headless-ui/src/*.rs`
総数 76 = 部品 66 + 基盤 9 + `lib.rs`）との一致は
`crates/docs-site/tests/primitives_catalog.rs::catalog_has_66_entries_in_six_categories_in_spec_order`
/ `module_counts_are_consistent_with_the_source_tree` が機械検査する
（本節末尾の表・部品名一覧・上記各追記は据え置き、本追記のみを合算値の
正とする）。

**訂正追記(イシュー #2068、PR #2234 レビュー指摘)**: 上記追記は起点を
誤って 65 として計算していた。`item`（イシュー #2065）は本追記より先に
base（main）へマージ済みで、`command` 着手前の実カテゴリ内訳は既に
Forms A 12・6 グループ合計 66 件だった。`command` 追加後の正しい合算は
Forms A 13・6 グループ**合計 66→67**であり、§6 のコード導出は
**部品 67 件**（headless-ui `crates/headless-ui/src/*.rs` 総数
**77** = 部品 67 + 基盤 9 + `lib.rs`）、`HEADLESS_UNWRAPPED` は
`button_group`/`command`/`item` の**3 件**である。一致は
`crates/docs-site/tests/primitives_catalog.rs::catalog_has_67_entries_in_six_categories_in_spec_order`
/ `module_counts_are_consistent_with_the_source_tree` が機械検査する
（本節末尾の表・部品名一覧・上記各追記は据え置き、本訂正追記のみを
合算値の正とする）。

## 8. `component-coverage-map.md` との関係

`docs/design/component-coverage-map.md` は ark-ui / chakra-ui / Radix の
3 参照軸コンポーネント**対応表の正**である(`radix-primitives-inventory.md`
/ `radix-themes-survey.md` が Radix 側の一次記録)。

本文書は**サイトの IA(2 層分割・URL 体系・台帳判別規約)の正**であり、
**対応表の正を移動させない**。

分担の再掲:

- 網羅性・実装済み/未実装の区分 = `component-coverage-map.md`
- 部品ページ雛形と Themes 側 6 カテゴリ = `docs-site-component-pages.md`
- 2 層分割と URL 体系と Primitives 台帳 = 本文書

関連文書・実装(相互参照): `component-coverage-map.md` /
`docs-site-component-pages.md` / `docs-site-three-column-redesign.md` /
`docs-site-api-reference-split.md` / `docs-site-search-design.md`、および
`crates/docs-site/src/{nav,linkcheck,search_index,component_page,showcase}.rs`、
`site/nav.toml`、`.github/workflows/docs-site.yml`。

## 9. セキュリティ上の不変条件(OWASP 観点)

### A01 アクセス制御 / パストラバーサル・オープンリダイレクト

- リダイレクト宣言の `to` は**サイト内絶対パスに限る**。`nav::validate_page_path`
  と同一のセグメント allowlist(`/` 始まり・`/` 終わり・セグメントは英数と
  `-` `_` のみ)を通し、**スキーム付き URL(`http://`・`javascript:` 等)・
  protocol-relative(`//evil.example`)・`..` セグメントを拒否する**。加えて
  `to` は実在 Nav ページへ解決できることをビルド時に検証する。
  - 検証を欠いた `to` を `<meta http-equiv="refresh" content="0; url={to}">`
    / `<link rel="canonical" href="{to}">` / フォールバック `<a href="{to}">`
    へ補間すると**オープンリダイレクト**になる。これが本設計で最も鋭い
    セキュリティ項目であり、#1016 の実装は必ず本節の検証を実施すること。
- `from` も同 allowlist を通し、既存本体ページ `path` との衝突をビルド失敗
  とする(リダイレクトが実ページを覆い隠す事故の防止)。
- `/primitives/<kebab>/` の kebab-case は同 allowlist から導出された規約
  であり緩和ではない(`docs-site-component-pages.md` §10 A01 を継承)。
  `page.source` も既存 `nav::validate_sources`(絶対パス禁止・`..` 禁止・
  `\` 禁止・`repo_root` 配下の実在確認)を Primitives セクションでもそのまま
  適用し、新セクション経路だけ検証を迂回しない。

### A03 インジェクション / XSS(REQ-1)

- Primitives 雛形(#1021)・リダイレクトページ(#1016)とも `fandhe_frontend_core`
  のノード木 API のみで組み立て、`render()` の既定エスケープを通す。
  **`raw_html()` および HTML 文字列の直接組み立て(`format!("<meta ... url={}>", to)`)
  を明示的禁止事項とする**(`component_page.rs` モジュール doc の既存方針を
  継承)。
- 原稿 Markdown は既存 `markdown.rs` 経路を通す(新たな迂回経路を作らない)。
- `search_index.rs` の多層エスケープ(`"` `\` / 制御文字 / `<` `>` `&` /
  U+2028/U+2029)は Primitives ページ追加後も維持する。

### A04 安全でない設計

- nav スキーマの fail-closed(未知キー・未知テーブル・空グループ・2 段
  入れ子・path 重複はすべて行番号付きエラー)を Primitives セクション追加
  後も維持する。`MAX_INPUT_BYTES`(1 MiB)上限も維持。
- リダイレクト宣言のパーサも同様に fail-closed(未知キー・欠落・不正 path
  が行番号付きエラー)。
- 「宣言のみで実体が無い移転先」を許さない(`to` の実在検証)。

### A05 セキュリティ設定ミス

- **本設計の中核規定**: 4 契約への例外を「特定ページを検査から外すフラグ・
  除外リスト」として実装しない。リダイレクトを `Nav` ページ集合の**外**に
  置く構造的分離により、除外述語ゼロで例外を成立させる。汎用除外リストは
  「任意のページを fail-closed 検査から外せる抜け道」となるため禁止する。
- `site_css_contract.rs` の双方向 fail-closed 契約・`site_typography_contract.rs`
  は弱体化・`#[ignore]` 化しない。#1021 が追加する Demo ラッパ class は契約
  表へ**登録する**(登録漏れで契約が緩むことを防ぐ)。
- `docs-site.yml` の dist sanity check(`test -f` 群)は削除・弱体化しない。
  Primitives 追加後は代表 Primitives ページの `test -f` 追加を #1021 の任意
  項目として言及する(必須化はしない — CI 契約の二重管理を避ける
  `.claude/rules/ci.md` の方針に従い、件数・ページ総数は yml へ書かず
  テスト側で固定する)。

### A09 ログ・エラーの機微情報

- 新規 `RedirectError` 等の `Display` は行番号と理由のみを含み、入力全文・
  絶対パス・環境変数を含めない(既存 `NavError` 方針を継承)。

### 秘密情報の混入

- 本文書自体は設計記述のみであり、API キー・トークン・実クレデンシャルを
  含まない。

## 10. Phase 対応表

| 本文書の節 | 対応イシュー |
|---|---|
| §2 命名・セクション対応 | #1018 / #1021 |
| §3 URL 体系・§8(上書き対象) | #1017 / #1018(`docs-site-component-pages.md` §8 本文改訂は Phase 6) |
| §4 リダイレクト方式・4 契約例外 | #1016 |
| §5 雛形の層差 | #1021 / #1022 |
| §6 台帳判別規約 | #1020 |
| §7 カテゴリ分類 | #1021 / #1024〜#1029 |

## 11. 申し送り(issue 化はしない。PR 本文で提案するに留める)

2 層分割後、`/primitives/accordion/` と `/themes/accordion/` のように
**同名タイトルの検索インデックスエントリが 2 件生成される**部品が多数出る
(`accordion` `avatar` `carousel` `dialog` 等)。#958 の検索 UI が重複候補と
して提示することになるが、どの受け入れ条件にも含まれていない。層ラベル
付きタイトル(例: `Accordion (Primitives)`)等の対処が要るなら別 issue と
する。

## 12. イシュー #2072 追記(2026-09-09、sidebar 追加)

`crates/headless-ui/src/sidebar.rs` の新設(headless-ui 層のみ先行実装、
Themes recipe は後続イシュー #2073)に伴い、§2/§6/§7 の件数を以下のとおり
更新した(実装コミット側の一次情報は `crates/docs-site/src/primitives_catalog.rs`
と `crates/docs-site/tests/primitives_catalog.rs`。本節では差分のみ記録し
本文中の全出現箇所は書き換えない、二重管理回避)。

- Primitives 部品: 67 → **68**(`sidebar` が Navigation カテゴリへ追加)
- `crates/headless-ui/src/*.rs` 総数: 77 → **78**(基盤 9 件 + `lib.rs` は不変)
- §7 Navigation カテゴリ件数: 12 → **13**(グループ内順序は
  `pagination` の次、`tabs` の前)
- `PRIMITIVES_WITHOUT_THEMES_PAGE` / `HEADLESS_UNWRAPPED`(§9 相当の
  「headless-ui 層のみ先行実装」台帳、`command` と同型の経緯): `["command"]`
  → **`["command", "sidebar"]`**
- Themes 部品(§2/§6 の件数)は本イシューでは変化しない(`/themes/sidebar/`
  は後続イシュー #2073 の責務)。
