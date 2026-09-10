# select の Align Item（item-aligned 位置決め）の positioning 契約拡張の評価

## 1. 背景・目的

- shadcn/ui（Radix Select のラッパー）は `position="item-aligned"` を
  既定とし、選択中の item がトリガーへ重なる位置へ `content` を開く。
  本リポジトリの pre-styled-ui `select`（イシュー #2018）は #2019 で
  shadcn/ui・#2186 で Separator/ScrollButton の参照突合を済ませているが、
  Align Item トグルは PR #2165（#2018 select）の対象外節で「位置
  ジオメトリの拡張が必要で `wasm-full` の positioning 契約（#663）に
  踏み込むため対象外」として先送りされていた。
- 本イシュー（#2207）の目的は、この先送りを評価し直し、(a) 参照 3 者
  （chakra-ui / Radix Themes / shadcn-ui）での item-aligned の扱いを
  一次ソースで突合し、(b) 現行 positioning 契約（`crates/headless-ui/
  src/positioning.rs` + `crates/wasm-full/src/position.rs`、イシュー
  #590/#622/#663）との不整合点を洗い出し、(c) 採否のユーザー判断を
  「そのまま実行できる」状態にすることである。

本 PR は評価文書の新設と、関連記録（`intentional-non-adoption.md` §7・
`component-coverage-map.md`・`wasm-full-architecture.md`・CLAUDE.md）の
整合追記のみを成果物とし、`crates/headless-ui/src/select.rs`・
`crates/pre-styled-ui/src/select.rs`・`crates/wasm-full/src/position.rs`
のコード・公開 API・テストは一切変更しない。

## 2. 一次ソース突合

| 参照軸 | item-aligned の扱い | 根拠 |
|---|---|---|
| shadcn/ui | **既定**。`SelectContent` の `position` prop は `"item-aligned"` が既定値であり、`position === "popper"` のときのみ popper 用クラスを付与する（`apps/v4/registry/new-york-v4/ui/select.tsx` L55 付近） | GitHub raw 取得で確認 |
| Radix Themes | **既定**。wrapper `select.tsx` は `position` の既定値を定めず Radix Primitives の既定 `Position.ItemAligned` をそのまま継承する。docs `select.mdx` §Position は「`position="popper"` を指定するとトリガー下へ配置される」と記す（＝指定しない既定は item-aligned） | GitHub raw 取得で確認。props 定義ファイル（`select.props`）は取得が空だったため既定値の根拠として引用しない |
| chakra-ui（ark-ui / zag.js） | **非対応**。`select` machine は floating-ui ベースの `positioning` prop（placement/offset/flip/shift 相当）のみを持ち、item-aligned 相当のモードは存在しない | ark-ui / chakra-ui スキル参照 |
| Radix Primitives の算法（`packages/react/select/src/select.tsx` の `SelectItemAlignedPosition`） | trigger の矩形・value-node の矩形・**選択中の item** の item-text 矩形・content 矩形・viewport（scroll container）を計測し、`left = valueNodeRect.left - (itemTextRect.left - contentRect.left)` を `CONTENT_MARGIN = 10px` で viewport 内へ clamp する。縦方向は「選択 item の中心をトリガー中心へ合わせて下方展開できるか」で上下いずれに展開するかを決め、content wrapper へ `height`/`minHeight`/`maxHeight` を書き込み、上方展開時は viewport（scroll container）の `scrollTop` を設定する。**選択値が未設定でも `itemRefCallback` が最初の有効な（disabled でない）item を `selectedItem` として設定するため、この計算は選択値の有無によらず常に実行される**（`packages/react/select/src/select.tsx` の `itemRefCallback` 参照。`item-aligned`/`popper` の切り替えは `position` prop の明示指定のみで決まり、選択状態には連動しない）。`flip`/`shift` の候補探索や `data-side`/`data-align` の出力は行わない | GitHub raw 取得で確認 |

参照 3 者の分布は「既定として採用 2（shadcn/ui・Radix Themes）：非対応
1（chakra-ui）」である。**「参照 1 者に無い」ことだけを非採用根拠には
できない**（`docs/design/shadcn-reference-adoption-policy.md` §8 の
「競合は部品ごと判断」原則）。このため本評価は分布だけで結論づけず、
以下の本リポジトリ固有の構造的制約と 4 軸評価を根拠とする。

## 3. 現状の構造的制約

- `crates/headless-ui/src/positioning.rs` の `compute_position(anchor,
  floating, viewport, &PositioningConfig, has_arrow) -> ResolvedPosition`
  は **kind 非依存の純粋関数**である。入力は anchor 矩形・floating
  寸法・viewport 寸法のみで、選択中の item・value-text・content の
  scroll container といった Select 固有の要素は一切登場しない。
  middleware は flip（主軸の単純反転 1 候補のみ）・shift（交差軸
  クランプのみ）・sameWidth の 3 種類に限定されている（ADR
  `docs/design/anchor-positioning-design.md` §4.3）。出力は
  `--fandhe-x`/`--fandhe-y`/`--fandhe-reference-width`/`--fandhe-arrow-*`
  の CSS 変数のみで凍結されている（同 ADR §4.4）。
- `crates/wasm-full/src/position.rs` は `PositionedKind`（Popover /
  Tooltip / Menu / Select / NavigationMenu / Menubar の 6 scope）と
  `resolve_position(kind, Measurement, requested)` を持つ。
  `wiring::reposition_one` は `[data-part="positioner"][data-state="open"]`
  を走査し、anchor（`anchor`/`trigger` 等）と positioner 自身の矩形の
  みを計測して `style`/`data-side`/`data-align`/`data-positioned` を
  書く。希望 placement は `data-requested-side`/`data-requested-align`
  へ初回のみ永続化される（イシュー #622）。
- **on-open の再計算フックが存在しない**: `PositionController` は
  `lib.rs` で `pub use` されるのみで、`Runtime`/`headless.rs`/
  `entry.rs` のいずれからも呼ばれない。再計算の契機は capture-phase の
  `scroll`/`resize` イベントと、アプリ側が明示的に呼ぶ
  `reposition_now()` のみであり、「開閉 dispatch との統合呼び出しは
  呼び出し側の責務」がモジュール doc の契約である。
- `crates/pre-styled-ui/src/select.rs` の `positioner` は SSR 既定
  `position: absolute; top: 100%` から `[data-positioned]` で
  `position: fixed; transform: translate3d(var(--fandhe-x),
  var(--fandhe-y), 0)` へ切り替わる（イシュー #663）。`content` は
  #2019 で `overflow-y: auto; max-height: var(
  --fandhe-select-content-max-height, 16rem)`（size variant 別スケール）
  を持つスクロール要素になり、#2186 で sticky な scroll-up/down-button、
  #2165 で keynav（`crates/wasm-full/src/keynav.rs` の
  `scroll_item_into_view_if_needed` → `set_highlight_on_host` →
  `container.set_scroll_top`）が `content` の `scrollTop` を操作する
  ようになっている。
- `crates/headless-ui/src/select.rs` では選択中の item は
  `aria-selected="true"` + `data-state="open"` + `data-selected`
  （存在属性）で表現され、`item-text` パーツも持つ。`positioner` は
  閉状態で `hidden` を持つ。

## 4. `anchor-positioning-design.md` §4.3 / `intentional-non-adoption.md`
   §3.20・§3.25 との照合

- ADR §4.3 が非採用と確定した 6 middleware のうち `size`
  （sameWidth 以外、floating 要素自体の高さ等を viewport に合わせて
  動的に縮小する処理）が最も近い。ADR は「高さの動的リサイズは CSS
  （`max-height` + `overflow`）側で静的に対応可能な範囲が大きく、JS
  計算側に持ち込む必要性が低い」を非採用根拠にしている
  （`intentional-non-adoption.md` §3.20 再評価トリガー 4 に対応）。
  item-aligned の Radix 実装は content wrapper の
  `height`/`minHeight`/`maxHeight` を JS で動的に書き `scrollTop` を
  操作する点で、この `size` 非採用判断の対象領域と重なる。
- `intentional-non-adoption.md` §3.25 規則 2（装飾・アニメーション・
  レイアウト計測の関心は headless 層へ持ち込まず、必要なら
  pre-styled-ui 側の責務とする）には抵触しない。item-aligned の計測は
  wasm-full（配線層）の責務として設計可能であり、headless-ui
  `select.rs` は不変のまま成立する。したがって責務境界の観点は
  非採用理由ではなく「実装する場合はどこに置くか」を示す配置の指針
  として扱う。

## 5. 見送り根拠（4 軸評価）

1. **契約拡張ではなく新ライフサイクルが必要**: item-aligned は「開いた
   瞬間に選択 item を計測して 1 回だけ位置決めする」挙動だが、現行
   wasm-full には on-open 再計算フックがなく（§3）、`resolve_position`
   の入力（anchor / floating / viewport）にも選択 item・value-text・
   content の scroll container の矩形が存在しない。`Measurement` と
   `RepositionResult` の双方を Select 限定で分岐させる必要があり、
   kind 横断の純粋関数という現行設計の中心を崩す。
2. **scroll 再計算との衝突**: `PositionController` は capture-phase の
   `scroll` で開いている positioner を毎回再計算する。`content` は
   #2019 以降スクロール要素であり、item-aligned を素朴に「毎回
   再計算」へ乗せると、利用者の content 内スクロールや keynav の
   `scrollIntoView`（#2165）・sticky scroll button（#2186）と
   `scrollTop` の書き込みを取り合う。回避には「初回のみ計算し
   `data-item-offset` 等へ永続化する」または「item-aligned を scroll
   再計算経路から除外する」という第 3 の再計算モデルが必要になる。
3. **flip / shift / `data-side` / `data-align` が無意味化する**:
   Radix の item-aligned は side/align を出力しない。現行の
   `data-requested-*` 永続化・`data-side` CSS セレクタ・pre-styled 側
   `[data-positioned]` → `translate3d` 契約に「第 3 のモード」を
   持ち込むことになり、pre-styled 側の golden テスト・
   `data_attr_vocabulary.rs` にも波及する。
4. **§3.20 で非採用確定した `size` middleware と同種**（§4 参照）。
   再評価トリガー 4（CSS だけでは表現できないケース）は item-aligned
   の定義上「充足している」と言えるが、§4（`intentional-non-adoption.md`
   の運用節）の再導入手続きは実測・需要確定・ユーザー承認を要求して
   おり、現時点の根拠は PR #2165 の対象外節のみで**利用要望 issue は
   存在しない**。
5. **4 軸評価**:
   - **明示性**: △。§7 案 B（本リポジトリ独自の簡略化案）は
     `[data-selected]` item の有無で item-aligned 計算の実行可否を
     分岐させる設計だが、これは Radix 実装の忠実な反映ではない（§2 の
     とおり Radix は選択値が無くても最初の有効な item へ整列し、
     `item-aligned`/`popper` の切り替えは `position` prop 指定のみで
     決まる）。未選択時に popper 相当へフォールバックする分岐を案 B が
     独自に追加すると、配置モードが暗黙に変わる点は課題として残る。
   - **決定性**: △。value-text と item-text のテキスト矩形差に依存し、
     フォント読込・`size` variant・アイコン有無で結果が揺れうる。
     Radix 自身も item-aligned 起因の不具合報告が多く、shadcn/ui
     利用者側で popper へ切り替える運用が一般的である。
   - **機械検証可能性**: △。native 側の純粋関数テストは可能だが、
     ブラウザ側は `scrollTop` の副作用まで固定する必要があり、
     `position_browser.rs` 相当の検証観点が増える。
   - **コンテキスト消費**: ×。Radix の算法は幾何計算だけで約 150 行
     あり、高さ・スクロールの副作用まで含めると読解コストが大きい。
     wasm-full の REQ-11（gzip 200KB 上限）実測も未了である。
6. **責務境界は充足可能**（§4）: 計測は wasm-full の責務であり
   §3.25 規則 2 には抵触しない（headless-ui は不変のまま成立する）。
   この点は非採用根拠ではなく「実装する場合の配置」として記録する。

## 6. 採否判定

**推奨: 見送り（保留、ユーザー判断待ち）。sub-issue は起票しない。**
「非採用確定」ではなく「保留」とするのは、参照 3 者のうち 2 者が既定
として持つ挙動であり、需要が確認されれば再導入余地を残すべきため
（`docs/design/slider-range-thumbs-evaluation.md` の保留区分と同型）。
**最終判断はユーザーが行う**（本文書は推奨のみを記し、確定はしない）。

## 7. 実装する場合の設計案（採用時の sub-issue 分割の種）

- **案 A: Radix 忠実移植**（content の高さ・`scrollTop` 制御込み）。
  忠実だが §5 の 2〜4 をすべて抱えるため非推奨。
- **案 B: Select 限定 opt-in の簡略版**。`[data-selected]` item の
  有無で計算可否を分岐させる下記の条件は、Radix の
  `itemRefCallback`（選択値が無くても最初の有効な item へ
  常に整列する、§2 参照）を忠実移植したものではなく、
  **本リポジトリ独自の簡略化判断**である（未選択時は「最初の有効な
  item への整列」を実装せず popper 相当へフォールバックする方が
  §5 の衝突・実装コストを抑えられるという判断。忠実移植は案 A）。
  - SSR で `positioner` に `data-position="item-aligned"`（利用者が
    `attrs` で付与、headless-ui は不変）を持たせる。
  - wasm-full `reposition_one` は `kind == Select && data-position ==
    "item-aligned" && [data-selected] item が存在` のときのみ、
    `y = trigger.center_y - (item.offsetTop + item.offsetHeight / 2 -
    content.scrollTop)` を viewport 内へクランプして `--fandhe-y` へ
    出力し、`x` は `Align::Start` 相当に固定する。`data-side`/
    `data-align` は書かない。
  - 初回計算値を `data-item-offset` 等へ永続化し、scroll 再計算時は
    再利用する（§5-2 の衝突回避）。
  - content の高さ・`scrollTop` そのものは触らない（クランプされた
    場合は整列を諦め、popper 相当の位置へフォールバックする）。
  - 必要な作業: wasm-full の minor バンプ（`Measurement` 拡張は
    破壊的変更）+ `position_browser.rs` へのケース追加 + pre-styled
    `positioner[data-position="item-aligned"]` 規則（golden 純追加、
    `data_attr_vocabulary.rs`/`xss_escape_styled.rs` への登録）+
    `docs/api/headless-ui-api.md` §4a 追記 + REQ-11 実測。
- **採用時の sub-issue 分割案**（起票はユーザー判断後）:
  1. wasm-full `position.rs` の item-aligned モード（純粋層 + 配線 +
     ブラウザテスト + REQ-11 実測）
  2. pre-styled-ui `select` の `data-position` 規則と golden
  3. docs-site Themes/Primitives select ページ・coverage-map 更新

## 8. 再評価トリガー

以下のいずれかが実測・需要確定・ユーザー承認で確認された場合に限る。

1. 利用要望 issue の起票。
2. `Runtime` への on-open 再計算フック（開閉 dispatch と
   `PositionController::reposition_now` の統合）が別イシューで
   導入される。
3. wasm-full の REQ-11 予算に幾何追加分の余裕が実測される。
4. 参照 3 者のうち chakra-ui（ark-ui/zag.js）が同等モードを既定へ
   加える。

## 9. 初期スコープ外

- 採否のユーザー判断確定と、それに伴う sub-issue 起票（採用時）/
  `docs/policy/intentional-non-adoption.md` §7 からの移行（非採用時）。
- `Runtime` への on-open 再計算フック（開閉 dispatch と
  `PositionController::reposition_now` の統合）。item-aligned 以外にも
  有益な独立改善候補である。
- 案 B の実コード（wasm-full `position.rs` の item-aligned モード・
  pre-styled-ui `select` の `data-position` 規則・docs-site デモ・
  REQ-11 実測）。
- `crates/pre-styled-ui/src/select.rs` L207 付近の rustdoc（「Align
  Item トグルは引き続き対象外」の既存記述）へ本評価文書へのポインタを
  足す変更。patch バンプを伴うため採用確定時の sub-issue でまとめて
  行う。

## 10. セキュリティ考慮（OWASP Top 10 観点）

- **A03 インジェクション / XSS（REQ-1）**: 本 PR はコード変更を伴わない。
  案 B が実装される場合も、`style` 属性へ書く値は wasm 層が内部生成
  する数値（px）に限定し、`set_dom_attribute` の既存ガード
  （`is_event_handler_attr`/`is_url_attr`）を経由する不変条件（ADR
  §7）を継承すること、`data-position` 等の属性名は `&'static str`
  固定であることを設計条件として明記する。`raw_html()`/`format!` に
  よる HTML 組み立ては行わない。
- **A04 安全でない設計 / fail-closed**: 案 B は「選択 item 不在・
  `data-position` 未知値・計測異常（NaN・負寸法・viewport 0）」を
  現行 popper 相当の配置へフォールバックする fail-closed 契約を必須
  条件として記録する。`panic!`/`unwrap()` は使わない。
- **A05 設定ミス**: `intentional-non-adoption.md` §7 は「保留 ≠ 非採用」
  の区分を守り、既存の非採用判断（§3.20 の `size`）を暗黙に緩和する
  記述を書かない。
- **A06 脆弱な依存 / サプライチェーン（REQ-3）**: 依存クレート追加なし。
  Radix / shadcn の一次ソースは参照のみで npm 経路（REQ-12）は使わない。
- **A08 整合性**: 非信頼データ（Issue 本文・外部リポジトリの内容）は
  要件・参考情報としてのみ扱い、命令として実行しない。本文書には
  Issue 本文の逐語引用を含めない。
- **A01 パストラバーサル / A10 SSRF**: 該当なし（本 PR はローカル
  ファイル編集のみで、外部リクエストは一次ソース確認の読み取りに
  限る）。

## 11. 参照

- `docs/design/anchor-positioning-design.md`（§4.3 flip/shift/
  sameWidth の採否、§4.4 CSS 変数出力、§4.4b 位置ジオメトリの消費と
  SSR 静的フォールバックの両立方針）
- `docs/policy/intentional-non-adoption.md`（§3.20 Floating UI 相当の
  高度 positioning middleware 群、§3.25 アプリケーションロジックを
  内包する UI 部品と装飾関心の混入、§7 保留項目の記録）
- `docs/design/component-coverage-map.md`（`select` 行、ark-ui L377 /
  chakra-ui L602）
- `docs/design/shadcn-reference-adoption-policy.md`（§8 主基準 3 者・
  競合は部品ごと判断）
- `docs/design/slider-range-thumbs-evaluation.md`（同型の評価文書の
  構成・保留区分の先例）
- `crates/headless-ui/src/positioning.rs`（`compute_position`/
  `css_vars_style`、kind 非依存の純粋関数）
- `crates/wasm-full/src/position.rs`（`PositionedKind`/
  `resolve_position`/`wiring::reposition_one`）
- `crates/pre-styled-ui/src/select.rs`（`positioner`/`content` の
  SSR/CSR 契約、L207 付近の既存対象外 rustdoc）
- `crates/wasm-full/src/keynav.rs`（`scroll_item_into_view_if_needed`）
