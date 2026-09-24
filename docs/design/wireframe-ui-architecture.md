# fandhe-frontend-wireframe-ui アーキテクチャ設計

**本文書のステータス**: 確定（イシュー #2601。ツリー全体はルート #2599、Phase 0 は #2600）。

- **関連**: `docs/design/animation-core-architecture.md`（新規クレート位置づけ文書の書式先例）・`docs/design/docs-site-blocks-section.md`（同型の書式先例）・`docs/policy/intentional-non-adoption.md` §3.25（責務境界の判断軸）・`docs/design/pre-styled-ui-size-and-color-palette-axes.md`（`Size`/`ColorPalette` 軸の先例）

本文書自身が 49 部品一覧（§8）・視覚差分方針（§3）・`Size` 軸命名規約（§4）・責務境界（§5）・Figma
プロパティ変換規約（§6）の正である。起票時の作業メモ（`_/local-plans/wireframe-ui-tree.md` 等）は
git 追跡対象外のローカル作業ファイルであり、本文書とは独立に破棄され得るため正としては参照しない。

## 1. 位置づけ・スコープ

`fandhe-frontend-wireframe-ui`（`crates/wireframe-ui/`）は、`fandhe-frontend-headless-ui`（Primitives）・
`fandhe-frontend-pre-styled-ui`（Themes）とは**独立した第 3 の UI コンポーネント層**である。両者への依存・
両者からの依存のいずれも持たない。依存クレートは `fandhe-frontend-core` のみとする（外部依存ゼロ・
`crates/headless-ui/`・`crates/pre-styled-ui/` との path 依存を一切持たない構成）。

目的は、ローファイ・モノクロのワイヤーフレーム／プロトタイピング表現に特化した部品セットを提供する
ことである。Primitives（構造・アクセシビリティ）・Themes（実運用向けスタイル）とは別の関心事、すなわち
「画面設計初期段階での配置イメージの提示」に特化する。

本クレートは **SSR のみ**を提供し、`wasm-full` 配線（ハイドレーション・インタラクション配線）は行わない
（静的 SSR 専用）。このため REQ-11 の gzip 200KB 計測（既存の `wasm-full`/`dist-server` 経路）には非影響
である。

この静的 SSR 専用という制約に伴い、`wireframe-ui` の全部品は**非インタラクティブな表示専用の
プレースホルダー**として設計する。modal・tooltip・accordion・tabs・select・slider のような、
Primitives/Themes では通常キーボード操作・フォーカス管理・状態遷移を伴う部品についても、
`wireframe-ui` 版は画面設計上の見た目（anatomy・配置イメージ）のみを静的に示し、対話的な
WAI-ARIA セマンティクス（`role`・`aria-expanded`・`aria-haspopup` 等）や `tabindex`・キーボード
イベントハンドラは付与しない（§5・§7 参照）。ネイティブに対話セマンティクスを持つ HTML 要素
（`button`/`input`/`select`/`a[href]` 等）自体も出力せず、`div`/`span` 等の非対話要素でレイアウトの
みを表現する。実際に操作可能・アクセシブルな同種部品が必要な場合は `fandhe-frontend-headless-ui`
（Primitives）／`fandhe-frontend-pre-styled-ui`（Themes）を利用する。

## 2. blocks.pm 参照方針

blocks.pm（https://www.blocks.pm/）が持つ 35 部品分のカタログ構成を**対応範囲（どの部品を含めるか）
の参考**とする。忠実再現を目標とせず Rust API として使いやすい形へ調整してよいという裁量
（ユーザー承認 2026-09-22）自体は維持するが、その裁量の使い方（外観・anatomy・プロパティ構成を
blocks.pm から直接転用してよいか）は下記のライセンス保留により制約される。

**参照スクリーンショットの本リポジトリへの取り込みは不採用と確定した（イシュー #2602）。**
blocks.pm は Figma プラグイン「Blocks – Wireframe」（Figma Community 配布）である。プラグインページ
本体（`https://www.figma.com/community/plugin/1332372435133832847/blocks-wireframe`）下部の明示リンクと
Figma 公式ヘルプセンター記事（無料プラグインは既定で Community Free Resource License の下で公開される
旨の記載）を一次情報として突き合わせ、当該プラグインへの Community Free Resource License 適用を確認済み
である（確認日 2026-09-22、根拠・引用は `docs/design/reference-screenshots/README.md` の「blocks.pm
ライセンス適用の一次情報確認」節を参照）。同ライセンスは第三者への再配布・derivative work 作成を
明示的に禁止し、スクリーンショット掲載を許諾する記載を持たない。本リポジトリの既存参照 4 サイト
（chakra-ui / Ark UI / Radix Primitives・Themes / shadcn/ui）が持つ「MIT ライセンスの GitHub
リポジトリ」という積極的な再配布許諾根拠を blocks.pm は持たないため、`docs/design/
reference-screenshots/wireframe-<kebab>.png` は配置しない（fail-closed 判断、詳細・出典表は
`docs/design/reference-screenshots/README.md` の「出典・ライセンス・再配布根拠」節を参照）。
各部品からの視覚参照は https://www.blocks.pm/ への外部リンクに限る。

**blocks.pm の外観・anatomy・プロパティ構成の実装への転用は保留する（PR #2670 codex レビュー
P1 指摘、2026-09-22）。** derivative work 作成を禁止する配布元ライセンスの下では、スクリーンショット
を保存しないだけでは「blocks.pm の Figma コンポーネント構造を Rust API へ翻案する」という実装行為
自体の許諾問題は解消しない。したがって Phase 1〜9 の各部品実装は、blocks.pm の Figma プロパティ
（variant 列挙・boolean スロット構成・具体的な instance swap 構造等）を閲覧・書き写して構造的に
一致させる作業を行わない。§4 の `Size` 軸・§6 の変換規約は blocks.pm 固有の schema ではなく、
一般的な wireframe/UI キット設計で広く使われる汎用パターン（段階的サイズ軸・固定スロットの
boolean 爆発畳み込み等）として独立に設計し直したものである（§4・§6 参照）。blocks.pm への外部
リンクは、対応範囲の確認・画面設計上のインスピレーション確認用に限り、部品ごとの厳密な仕様書
としては用いない。**再評価トリガー**: 作者 Hexa（love@blocks.pm）から derivative work 作成
（実装への翻案）を含む書面での明示的な許諾が得られた場合。それまでは §4・§6・§8 の変換規約・
分類は本節の保留の範囲内でのみ有効とする。

## 3. 視覚差分方針

`wireframe-ui` の配色はモノクロ・グレースケールとし、黒塗り二値（白黒二値）表現を強制しない。
**読みやすさ優先**の基準で独自に調整する（§2 のとおり blocks.pm の原案を模写しない）。

色トークン（`fandhe-frontend-pre-styled-ui::ColorPalette`）には依存しない。`ColorPalette` 軸自体を
`wireframe-ui` へ持ち込まない。

## 4. `Size` 軸の命名規約

`crates/pre-styled-ui/src/recipe.rs` の `pub enum Size { Xs, Sm, Md, Lg, Xl }`（5 段階）と**同名の
5 段階**（`Xs`/`Sm`/`Md`/`Lg`/`Xl`）を `wireframe-ui` 独自の `Size` 列挙型として定義する。
`pre-styled-ui` への依存はしない（独自定義で名前のみ揃える）。

部品によってはこの 5 段階を超えるサイズ区分が必要に見える場合があるが、`wireframe-ui` は常に
この 5 段階のみを採用し、5 段階を超える最大値は最大バケットである `Xl` へ畳み込む（§2 のとおり
blocks.pm 側の具体的な段階数・命名を実装の根拠にはしない、独立設計）。この変換規約は個別部品
ごとの判断揺れを防ぐため本文書に集約する。

全部品が 5 段階すべてを使う必要はなく、部品によっては一部バケットのみ使用してよい（既定値は
原則 `Md`）。

## 5. 責務境界（機能非搭載の判断軸）

`wireframe-ui` が提供するのは構造（anatomy）と配置イメージ（レイアウト）のみであり、バリデーション・
状態管理・実インタラクションといった**アプリケーションロジックは持たない**。

これは `docs/policy/intentional-non-adoption.md` §3.25（規則 1: アプリケーションロジックを内包する
部品は実装しない）と同型の判断軸である。ただし headless-ui／pre-styled-ui が担う「anatomy・
アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）まで」という既存原則は、
`wireframe-ui` にはそのまま適用しない。`wasm-full` 配線を持たない静的 SSR 専用（§1）という制約の
もとでは、対話部品に対してキーボード操作・フォーカス管理・状態遷移のアクセシビリティ契約を実際には
満たせないため、見た目上は操作可能に見えて実際には操作不能な UI を提供する不整合を避ける。
`wireframe-ui` が担うのは構造（anatomy）と、選択済み・無効化等の見た目上の表示状態を示す `data-*`
属性までとし、対話的な WAI-ARIA セマンティクス（`role`・`aria-expanded`・`aria-haspopup` 等）・
キーボード操作・フォーカス管理・状態遷移は一切実装しない（§1）。modal・tooltip・accordion・tabs・
select・slider を含む全部品は非インタラクティブな表示専用プレースホルダーであり、実際にアクセシブルな
同種部品が必要な利用者は `fandhe-frontend-headless-ui`／`fandhe-frontend-pre-styled-ui` を再利用・
配線すること。

## 6. Figma ライクなプロパティを Rust API へ落とす際の共通規約

Figma のコンポーネントプロパティパネルに典型的に現れる表現（固定スロットの boolean 爆発・
instance swap・サイズ／強調の組み合わせ等）を Rust API へ落とす際の共通規約を以下に定める。
これは blocks.pm 固有の schema を書き写したものではなく、Figma ベースの wireframe/UI キットに
広く見られる汎用的な表現パターンから独立に設計した変換規約である（§2 のライセンス保留により、
個別部品の実装時に blocks.pm の具体的なプロパティ定義を参照・転記することはしない）。

- **boolean 爆発の畳み込み**: ページ番号・星評価・タブのような、固定スロットを個別の boolean
  で持つ表現パターンは、Rust では**スライス／数値引数**へ畳む（例: 星評価は `rating: u8` 1 引数、
  タブ項目は `&[TabItem]` 等）。
- **instance swap の受け方**: アイコン差し替え等の instance swap は **`Node` スロット引数**で受ける
  （`fandhe-frontend-core` のノード木 API に従う）。
- **Text/Paragraph 系の共通パターン**: `Text`/`Paragraph` 系は `Size` + `Bold`(bool) + `Text`(文字列)
  の 3 点セットで表現する（テキスト系 wireframe 部品に共通して現れる一般的な構成）。

## 7. 全 49 部品共通の前提

後続 Phase 1〜9 の全部品実装イシューは、以下を共通前提とする。

- `#![forbid(unsafe_code)]`（REQ-2）
- 依存は `fandhe-frontend-core` のみ（外部依存ゼロ、他クレートへの依存もなし）
- テキスト引数は `fandhe-frontend-core` の既定エスケープ経由（REQ-1）。各部品イシューの受け入れ条件に
  XSS 回帰テストを含める
- `wasm-full` 配線は行わない（静的 SSR のみ）
- 非インタラクティブな表示専用プレースホルダーとする（§1/§5）。対話的な WAI-ARIA セマンティクス
  （`role`・`aria-expanded`・`aria-haspopup` 等）・`tabindex`・キーボードイベントハンドラ・
  フォーカス管理・状態遷移は付与しない。`button`/`input`/`select`/`a[href]` 等ネイティブに対話
  セマンティクスを持つ HTML 要素も出力しない（`div`/`span` 等の非対話要素でレイアウトのみを表現する）。
  modal・tooltip・accordion・tabs・select・slider を含む全部品にこの制約が適用される
- 各部品イシューは showcase 実装 + `/wireframes/<kebab>/` ページ（docs サイト、#2607 が基盤整備）+
  テストを同梱する
- **blocks.pm の Figma プラグイン・スクリーンショットを開いて外観・プロパティ構成を書き写さない**
  （§2 のライセンス保留）。blocks.pm 由来（35）に区分される部品も、anatomy・variant・プロパティ
  構成は本文書（§4・§6）と各部品イシューの記述に基づいて独立に設計する

## 8. 49 部品一覧（Phase 別表）

本節の表を正とする（実イシュー番号を含む）。Phase 構成は **Phase 0〜9 の 10 Phase・子イシュー計 59 件**
である。起票時の作業メモ（`_/local-plans/wireframe-ui-tree.md` 等）に記載されていた「11 Phase / 66 件」
というプロース記述は誤りであり、本節の表が最終的な正となる。

| Phase | 親イシュー | 内容 | 子部品（kebab: イシュー番号） |
|---|---|---|---|
| 0 | #2600 | 基盤 | design-doc:#2601 / reference-screenshots:#2602 / crate-scaffold:#2603 / ci-integration:#2604 / common-api:#2605 / icon-base:#2606 / docs-section:#2607 |
| 1 | #2608 | レイアウト骨格 | frame:#2609 / stack:#2610 / grid:#2611 / divider:#2612 |
| 2 | #2613 | テキスト・注釈 | text:#2614 / paragraph:#2615 / rich-text:#2616 / annotation:#2617 / link:#2618 / tag:#2619 |
| 3 | #2620 | Forms A | button:#2621 / input:#2622 / textarea:#2623 / select:#2624 / checkbox:#2625 / radio:#2626 / switch:#2627 / slider:#2628 |
| 4 | #2629 | Forms B | question:#2630 / ratings:#2631 / calendar:#2632 / file-drop:#2633 / stepper:#2634 |
| 5 | #2635 | Navigation | nav-item:#2636 / menu:#2637 / tabs:#2638 / breadcrumbs:#2639 / pagination:#2640 / accordion:#2641 / cursor:#2642 |
| 6 | #2643 | Overlay・Feedback | tooltip:#2644 / modal:#2645 / alert:#2646 / toast:#2647 / progress:#2648 / spinner:#2649 |
| 7 | #2650 | Data display | avatar:#2651 / icon:#2652 / brand:#2653 / emoji:#2654 / counter:#2655 / stat:#2656 / list:#2657 / card-basic:#2658 |
| 8 | #2659 | Media・Data | image:#2660 / media:#2661 / table:#2662 / chart:#2663 / map:#2664 |
| 9 | #2665 | 仕上げ | golden-tests:#2666 / example:#2667 / crates-io-publish:#2668 |

`media` の kebab は blocks.pm 表示名 `Placeholder` ではなく **`media`** を正とする。#2607 の
`/wireframes/<kebab>/`（docs サイトのページ URL）および Rust API 側の識別子（showcase 関数名・コンポーネント名）
が参照する kebab と一致させるため、本表で明示的に固定する（#2602 の結論により `wireframe-<kebab>.png` は
配置しないため、画像ファイル名を命名根拠とはしない）。

各部品の対応範囲の区分（blocks.pm カタログの同名部品と対応範囲を揃えるか／`wireframe-ui` 独自に
追加するか）は以下のとおりである（本文書が正）。§2 のとおり、対応範囲を揃えることと外観・
プロパティ構成を転用することは別であり、以下の区分はあくまで「同名の部品を用意するかどうか」の
対応範囲一覧であって、実装が blocks.pm の具体的な Figma 構造に由来することを意味しない。

- **blocks.pm カタログと対応範囲を揃える部品（35）**: annotation, avatar, brand, breadcrumbs,
  button, card-basic, chart, checkbox, counter, cursor, divider, emoji, icon, image, input, link,
  map, menu, nav-item, pagination, paragraph, media（blocks.pm 表示名は Placeholder）, progress,
  question, radio, ratings, rich-text, select, slider, switch, table, tabs, tag, text, tooltip
- **`wireframe-ui` 独自に追加する部品（14）**: frame, stack, grid, textarea, modal, alert, toast,
  accordion, stepper, list, stat, calendar, file-drop, spinner

## 9. 後続イシューへの委譲

- **#2602（参照スクリーンショット取り込み、完了）**: §2 のとおりスクリーンショット取り込みは不可（fail-closed）と確定した。各部品からの視覚参照は https://www.blocks.pm/ への外部リンクに限る。加えて §2 のとおり、外観・anatomy・プロパティ構成の実装への転用も書面許諾が得られるまで保留する（PR #2670 codex レビュー指摘、2026-09-22）
- **#2603（crate 雛形）**: §1 の位置づけ・依存方針（`fandhe-frontend-core` のみ、Primitives/Themes 非依存）を前提とする
- **#2604（CI 組み込み、完了）**: §7 の共通前提（`forbid(unsafe_code)`・REQ-1 既定エスケープ・wasm-full 非配線・非インタラクティブ制約）を前提に、`deps-check` 計測対象化（実測 packages=1/60 depth=1/6）・`release.yml` 選択肢追加・`ZERO_DEP_CRATES` 非登録・公開クレート化（`publish = false` なし、#2668 で v0.52.0 の初回公開実施済み）を実施した。詳細は `docs/ci/version-bump-publish-order-gap.md` §11 参照
- **#2605（共通 API、完了）**: §4 の `Size` 軸命名規約・§6 の Figma プロパティ変換規約を前提に、`Size` 列挙・共通型（`Bold`/`Primary`/`Active`/`Disabled`/`Orientation`）・モノクロトークン・`wireframe_css()` 出力関数を実装した。詳細は §10 参照
- **#2606（アイコン基盤、完了）**: §6 の instance swap（`Node` スロット引数）規約を前提に、SVG ラインアート
  アイコンセット（`icon::<name>(Size) -> Node`、21 種、`ALL` レジストリ）を実装した。詳細は §11 参照
- **#2607（docs サイトセクション、完了）**: §7 の「`/wireframes/<kebab>/` ページ同梱」方針と、§8 の kebab 命名（特に `media`）を前提に、`/wireframes/` セクション基盤（nav 登録・レジストリ・簡略レンダラ・契約テスト・CI dist check）を実装した。詳細は §12 参照

## 10. class 命名規約・CSS 出力規約（イシュー #2605）

本節は `crates/wireframe-ui/src/`（`class.rs`/`css.rs`/`size.rs`/`tokens.rs`/`props.rs`）の実装契約を記す。
Rust 側の値表（`size::SCALE`〔非公開 const〕・`tokens::TOKENS` 等）を正とし、本節では値を書き写さない（二重管理回避）。

### 10.1 class 命名規約

全 class は `class::CLASS_PREFIX`（`fw-wire-`）で始まる。

- **部品ルート**: `fw-wire-<kebab>`（kebab は §8 の表の値。例: `fw-wire-button`）
- **部品内パート**: `fw-wire-<kebab>-<part>`（例: `fw-wire-button-label`）。BEM の `__`/`--` は使わず
  単一ハイフン連結に統一する（`fw-wire-size-md` と同じ形）
- **共通修飾**（部品名を含まない横断 class。`.fw-wire-button.fw-wire-size-md` のように部品ルートと
  結合して使う）:
  - `fw-wire-size-<xs|sm|md|lg|xl>`（`size::Size::class()`）
  - `fw-wire-bold`（`props::Bold::class()`）
  - `fw-wire-primary`（`props::Primary::class()`）
  - `fw-wire-horizontal` / `fw-wire-vertical`（`props::Orientation::class()`）
- **表示状態は class ではなく `data-*` 属性**で表す（`data-active`/`data-disabled`。今後追加する
  `data-selected` 等も同様。`props::Active`/`props::Disabled` が `fandhe_frontend_core::attr_if`
  経由で生成する）
- **CSS カスタムプロパティ**は `--fw-wire-*` プレフィックス。pre-styled-ui の `--fandhe-*`・`fd-*` class
  とは意図的に別プレフィックスとし、docs サイト（#2607）で両スタイルシートが同一ページに載っても
  衝突しない
- **基盤パート class（唯一の例外）**: `fw-wire-icon-glyph`（`icon` モジュール、§11）は部品ルートを持たずに
  単独使用する唯一の例外的パート class である。`data-icon` 属性はアイコン名の識別子であり、表示状態を
  表す `data-*`（前項）ではない
- **部品固有の修飾 class**（`fw-wire-<kebab>-<modifier>`、例: `fw-wire-frame-bordered`、イシュー #2609）は
  部品ルートと結合して使う横断修飾ではない、1 部品専用の修飾 class である。共通修飾（前項の 4 種）とは
  別物であり、`props.rs` の共通型へ昇格させるかは部品横断で再利用が見えた時点で判断する

`class::class_list(base, modifiers)` は `base` に `Some` の修飾子のみを半角スペース連結する。引数は
`&'static str` に限定し、利用者入力が class へ流れ込む経路を型で塞ぐ（REQ-1・A03 対応）。

### 10.2 共通型（`props.rs`）

`Bold`/`Primary`/`Active`/`Disabled`/`Orientation` は blocks.pm のプロパティ schema をそのまま転写した
ものではなく、「wireframe/UI キット表現として独立に定義した」最小集合である（§2 のライセンス保留）。
視覚修飾型（`Bold`/`Primary`）は class を、表示状態型（`Active`/`Disabled`）は `data-*` 属性を返す。
対話セマンティクス（`role`/`aria-*`/`tabindex`）に相当する型は持たない（§1/§5/§7 の制約の一般化）。

### 10.3 モノクロトークン

`tokens::TOKENS` がグレースケール固定値（`paper`/`fill-subtle`/`fill`/`line-subtle`/`line`/`ink-muted`/`ink`
の 7 段）と補助トークン（`line-width`/`radius`/`font-family`）を保持する唯一の正である。固定 light 値の
みを持ち `prefers-color-scheme` による分岐は行わない。ダークモード対応・docs サイトのテーマトグル配下
での扱いは #2607 側の判断事項とする（§8「スコープ外候補」）。

### 10.4 `wireframe_css()` / `PARTS` 追記契約

`css::wireframe_css() -> &'static str` が全部品 CSS を集約する唯一の入口関数である（イシューでの呼称
`WIREFRAME_CSS` に対応する。`concat!` はリテラルしか受け付けずモジュール間の `const` 連結ができないため、
`std::sync::OnceLock` による遅延構築を採用する）。出力順は `:root` トークン（`tokens::css()`）→ `Size` 5 段
のスコープ付きカスタムプロパティ（`size::css()`）→ `css::PARTS` 登録順、で連結する。

Phase 1 以降の部品イシューは、自分のモジュールに `pub const <PART>_CSS: &str` を定義し `css::PARTS` へ
1 要素追記する以外の場所で CSS を出力してはならない（意図的な摩擦点。`crates/wireframe-ui/tests/common_api.rs`
がセレクタ行の `fw-wire-` プレフィックス一致・`PARTS` の重複禁止を機械固定する）。

**例外（`size::SCALE` 等の単一の正から動的に導出する生成 CSS）**: `size::css()`（`Size` 5 段のスコープ付き
カスタムプロパティ）と `frame::frame_padding_css()`（Frame の padding 5 段、イシュー #2609）は値が実行時に
`size::SCALE` を走査して決まるため `pub const <PART>_CSS: &str` として `const` 化できない。この 2 つに限り
`wireframe_css()` が `PARTS` を経由せず個別に連結する。新たな非 `PARTS` 経路を追加してよいのは、その CSS が
`size::SCALE` 等の単一の正（別モジュールが既に持つ値表）から導出される場合に限る。単に `const` 化が面倒と
いう理由での逸脱は許容しない。

**golden テストの追記契約（イシュー #2666、2026-09-24 に値ベース全単射検証へ全面刷新）**: `css::PARTS` へ
`<PART>_CSS` を追記する部品 PR は、対応する golden 期待値サブモジュール（`crates/wireframe-ui/tests/golden/<snake>.rs`、
`pub const EXPECTED_CSS: &str` のみを持つ）を新規追加し、`crates/wireframe-ui/tests/golden_css.rs` の
`goldens()` レジストリへ当該部品の `GoldenEntry`（実装定数への参照・golden 参照・`in_parts: true`）を
追記することを同じ PR に同梱すること。手順の詳細（サブモジュールの追加手順・`goldens()` への登録手順・
機械ダンプ手順・部品対応表）は `docs/internal/wireframe-ui-golden-test-update-guide.md` を正とし、本節
では手順を重複して書かない。`crates/wireframe-ui/tests/golden_css.rs` はソーステキストを走査せず、
`PARTS` の値の多重集合と `in_parts` な golden エントリの `actual` 値の多重集合が一致することを実行時に
検証するため、この手順を怠るとテストが FAIL する（旧方式が使っていた `tests/<snake>_css.rs`・
`golden_coverage.rs`・`PENDING` 定数はいずれも廃止済みであり、本節・更新手順書のいずれからも参照しない）。

### 10.5 `Size` と pre-styled-ui の段階名パリティ

wireframe-ui `size::Size` は pre-styled-ui `recipe::Size`（`crates/pre-styled-ui/src/recipe.rs`）と段階名
（`Xs`/`Sm`/`Md`/`Lg`/`Xl`、既定 `Md`）を一致させるが、依存は追加しない（§4）。両者が同時に段階を増減する
変更は `crates/xtask/tests/wireframe_ui_size_parity.rs`（ソース走査による variant 名突合）の更新を伴う
（意図的な摩擦点）。

## 11. SVG アイコン基盤・`Node` スロット規約（イシュー #2606）

本節は `crates/wireframe-ui/src/icon.rs` の実装契約を記す。値の正は Rust 側であり、表の項目名以外の
値（座標・CSS 宣言本文等）は本節へ書き写さない（§10 と同じ二重管理回避方針）。

### 11.1 モジュール構成

- `icon::<name>(size: Size) -> Node`（例: `icon::search(Size::Md)`）が個別アイコンの公開関数。引数は
  `size` のみで、方向付きキャレットは列挙型ではなく `caret_up`/`caret_down`/`caret_left`/`caret_right`
  の関数 4 本に分ける
- `icon::ALL: &[IconEntry]`（`IconEntry = (&'static str, fn(Size) -> Node)`）が名前 → コンストラクタの
  レジストリ（宣言順）。#2652（`/wireframes/icon/` ページ）の一覧表示元、および契約
  テスト（`tests/icon.rs`）が全アイコン × 全 `Size` を走査する基点（#2607 時点では `/wireframes/`
  セクション自体は基盤のみで個別部品ページを持たないため、一覧表示元の実装は #2652 が担う。§12 D8）
- `icon::ICON_GLYPH_CSS: &str` がグリフの CSS（1 セレクタ）。`css::PARTS` へ最初に登録された要素
- クレートルートでは `pub mod icon;` のみを公開し、`pub use` による関数再エクスポートは行わない
  （`icon::plus` の名前空間で使わせる）

### 11.2 出力契約

各アイコンは次の `<svg>` ルート属性を持つ（すべて `&'static str` の固定リテラル。利用者入力を含まない）:

| 属性 | 値 | 根拠 |
|---|---|---|
| `class` | `fw-wire-icon-glyph fw-wire-size-<段階>` | §10.1 命名規約（`class_list`） |
| `data-icon` | アイコン名（`ALL` の名前と同一） | 識別子。表示状態ではない |
| `viewBox` | `0 0 24 24` | 24 グリッド固定 |
| `width` / `height` | `1em` | CSS 未読込時のフォールバック |
| `fill` | `none` | 線画契約 |
| `stroke` | `currentColor` | モノクロ・祖先文字色追従 |
| `stroke-width` | `1.5` | `tokens::TOKENS` の `line-width` と同値 |
| `stroke-linecap` / `stroke-linejoin` | `round` | 線画の統一 |
| `aria-hidden` | `true` | 装飾用途。対話的 ARIA は付与しない |
| `focusable` | `false` | 非インタラクティブ（§7） |

子要素は `path`/`circle`/`line`/`polyline`/`polygon`/`rect` のみで、属性は固定リテラル。`href`/
`xlink:href`/`on*`/`style` は一切出力しない。子要素にも `fill` は付けない（ルートの `fill="none"` を
継承する）。

### 11.3 サイズ機構

`ICON_GLYPH_CSS` が `.fw-wire-icon-glyph { width: 1em; height: 1em; font-size: var(--fw-wire-font-size,
1rem); ... }` を宣言し、`<svg>` 自身に付与した `fw-wire-size-<段階>`（`size::css()` が定義する
`--fw-wire-font-size`）が同一要素上で実寸を決める。`size::SCALE` の値はここへ書き写さない。

### 11.4 `Node` スロット規約

アイコン差し替え（Figma の instance swap 相当）を受ける部品は、`Option<Node>` のスロット引数として
`icon::<name>(size)` の戻り値をそのまま受け取る設計を標準とする（例: `leading: Option<Node>` /
`trailing: Option<Node>`）。呼び出し側は `Some(icon::search(size))` を渡し、部品側は children へ合成
する。ホスト要素は `div`/`span` のような非対話要素を使う（`button` 等の対話要素は出力しない、§7）。
`Node` はエスケープ済みの構築済みノードであるため、スロットへ渡すこと自体が REQ-1 の既定エスケープを
損なうことはない（`render()` の既定エスケープ・属性名ホワイトリスト・URL 検証を通る）。最小例は
`icon` モジュールの rustdoc（doctest として実行される）を参照する。部品イシュー（button / input /
link / nav-item / select / tag 等）はこの形を標準とする。

**例外（menu、イシュー #2637）**: `menu` は項目ごとのアイコン差し替え（Figma instance swap 相当）を
対応範囲外とし、`Option<Node>` スロットを持たない（`MenuItem` はラベル + 無効状態のみ）。検索行先頭の
アイコンも固定パートの `icon::search` とし `Option<Node>` 化しない（`select` のドロップダウン指示子
`caret_down` と同じ判断: 利用者が省略・差し替えできない部品の同一性を担う要素のため）。将来項目アイコン
が必要になった場合は別イシューで本節の対象へ追加する（`crate::menu` モジュール doc・
`site/wireframes/menu.md` の「原案差分メモ」参照）。

### 11.5 ジオメトリの出自

全アイコンは 24×24 グリッド上の単純図形として独自に描く。§2 の blocks.pm 外観書き写し禁止に加え、
Lucide / Feather / Heroicons 等の既存アイコンセットのパスデータもコピーしない（帰属表示付きライセンス
であり、本リポジトリにその受け入れ方針の記録がないため。`docs/policy/intentional-non-adoption.md` 系の
先例と同じ fail-closed 判断）。

### 11.6 #2652 との継ぎ目

イシュー #2652（`icon` 部品、Phase 7）は本モジュールへ `pub fn icon`（部品ルート `fw-wire-icon`・
`role="img"`/`aria-label` 付与の判断込み）・`ICON_CSS`・docs ページ相当の拡張を追加する見込みである。
名前衝突を避けるため、#2606 では以下の名前を使わない:

- `pub fn icon`（個別アイコンは `icon::plus` のように公開する）
- 部品ルート class `fw-wire-icon`（グリフ class は `fw-wire-icon-glyph` とし、将来の `icon` 部品の
  パート class としても整合させる）
- CSS 定数名 `ICON_CSS`（`ICON_GLYPH_CSS` とする）

### 11.7 追記契約

新規アイコン追加は `icon::ALL` への登録を必須とする（`tests/icon.rs` の契約テストが自動網羅するため）。

イシュー #2642 で `cursor-arrow`/`cursor-hand` の 2 種を追加した（計 23 種）。`cursor`（Phase 5
「Navigation」）が使う既存アイコンが 1 つもなかったため、`file_drop`（#2633）とは異なりアイコン追加
経路を選んだ（`crate::cursor` モジュール doc・`crate::file_drop` モジュール doc 参照）。

## 12. docs サイト `/wireframes/` セクション（イシュー #2607）

本節は `crates/docs-site/src/wireframes/mod.rs` の実装契約を記す。§7 の
「各部品イシューは `/wireframes/<kebab>/` ページを同梱する」方針の受け皿
（nav 登録・レジストリ・簡略レンダラ・契約テスト・CI dist check）を、
Phase 1〜8（#2608〜#2665）の着手前に本イシューで固定する。設計判断は
以下の決定表（D1〜D8）を正とする。

| # | 論点 | 決定 |
|---|---|---|
| D1 | `component_page::Layer` 統合 vs 独立分岐 | **独立分岐**（`crate::blocks` と同型）。`Layer::from_page_path` は `/primitives/` 以外を全部 `Themes` と判定する全域関数であり、Wireframes を通すと `pre-styled-ui.css` 配線判定と混線するため統合しない |
| D2 | 雛形実例ページを同梱するか | **同梱しない**（`WIREFRAMES` は空レジストリ）。掲載予定 49 部品の kebab は Phase 1〜8 の各部品イシューが所有するため、本イシューで作ると衝突する |
| D3 | docs-site → wireframe-ui の path 依存 | **本イシューで追加**。Phase 1 の全部品イシューが同時に依存追加すると `Cargo.toml`/`Cargo.lock`/`structure.toml` が PR 間で衝突するため、基盤側で 1 回だけ入れる |
| D4 | 専用 CSS の構成 | `assets/wireframes.css` = `wireframe_css()`（wireframe-ui 側で全部品 CSS を `PARTS` 集約済み）+ docs 専用のデモ枠 CSS（`.wireframes-demo`）のみ。部品追加時に docs-site 側の `stylesheet()` を編集しない契約（`crate::blocks` の部品ごと `LAYOUT_CSS` 追記点とは異なる） |
| D5 | ダークモード | デモ枠 `.wireframes-demo` に `color-scheme: light` を固定し、docs サイトのテーマトグルで反転させない（wireframe トークンは固定 light 値のみ、紙面メタファーとして最も単純で決定的） |
| D6 | Rust コードのドリフト検知（`blocks_code_drift.rs` 相当） | **持たない**。テンプレート（Demo + 引数表 + 原案差分メモ）に Rust コード節がなく、引数表はレジストリから機械生成する |
| D7 | 「原案差分メモ」の扱い | Markdown 原稿側の手書き H2（見出し文言は `原案差分メモ` 固定、`wireframes::DIFF_NOTES_HEADING`）。§2 のライセンス保留により blocks.pm の外観は参照できないため、内容は「独自設計の判断・Primitives/Themes 同名部品との違い」を書く欄と位置づける |
| D8 | アイコン一覧（§11.1）の表示元 | **#2652（`/wireframes/icon/`）へ委譲**。本イシューでは表示しない（D2 と同じ衝突回避） |

### 12.1 ページ組み立て方式

`crate::blocks` と同型に、Markdown 本文の**最初の `h2` の直前**へ「Demo」
「引数表」の 2 節を挿入する（後方追記ではない）。`wireframes::insert_generated_sections`
が `crate::build::build_site` の `render_markdown` 直後・`linkcheck::rewrite_md_links`
前で呼ばれる。

### 12.2 レジストリ契約（Phase 1〜8 が複製する契約）

`Wireframe { path, title, args, demo }` 1 件 = 1 部品ページ。Phase 1〜8 の
各部品イシューが触る箇所は以下の定型である。

- `site/nav.toml` の Wireframes セクションへ `[[section.page]]` を 1 ブロック追記
- `site/wireframes/<kebab>.md` を 1 件追加（H1 → 導入 → `## 原案差分メモ`）
- `crates/docs-site/src/wireframes/<snake>.rs` の `WIREFRAME` 定数を追加し、
  `mod.rs` の `WIREFRAMES` へ 1 行追記
- `crates/docs-site/tests/site_nav.rs`・`site_build.rs` のページ数 +1
- `.github/workflows/docs-site.yml` の `verify: dist sanity check` へ
  `test -f` を 1 行追加（最初の 1 件は `assets/wireframes.css` の
  `test -f` も併せて追加する）
- `site/wireframes.md` の「掲載予定」冒頭に「掲載済み」節を新設し
  （最初の部品イシューのみ）、以降はリンクを 1 行追加

### 12.3 セキュリティ不変条件

生成コンテンツはすべて `fandhe_frontend_core` のノード木 API で組み立て、
`raw_html()`・HTML 文字列の直接組み立てを使わない。`ArgRow` の各フィールドは
`&'static str` に限定し、利用者入力が引数表へ流れ込む経路を型で塞ぐ。
`wireframes_contract.rs` が XSS 回帰・非対話制約（`<form>`/`<button>`/
`<input>`/`<select>`/`<a href>` 不在）・CSS 配線を固定する。
