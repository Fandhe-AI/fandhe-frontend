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
- **#2604（CI 組み込み、完了）**: §7 の共通前提（`forbid(unsafe_code)`・REQ-1 既定エスケープ・wasm-full 非配線・非インタラクティブ制約）を前提に、`deps-check` 計測対象化（実測 packages=1/60 depth=1/6）・`release.yml` 選択肢追加・`ZERO_DEP_CRATES` 非登録・公開クレート化（`publish = false` なし、初回公開は #2668）を実施した。詳細は `docs/ci/version-bump-publish-order-gap.md` §11 参照
- **#2605（共通 API）**: §4 の `Size` 軸命名規約・§6 の Figma プロパティ変換規約を前提とする
- **#2606（アイコン基盤）**: §6 の instance swap（`Node` スロット引数）規約を前提とする
- **#2607（docs サイトセクション）**: §7 の「`/wireframes/<kebab>/` ページ同梱」方針と、§8 の kebab 命名（特に `media`）を前提とする
