//! styled Menu（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::menu`（イシュー #540）の Root / Trigger /
//! Indicator / Positioner / Content / Arrow / ArrowTip / Item / ItemGroup /
//! ItemGroupLabel / Separator 11 anatomy パーツを再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・スコープ外
//! 事項は [`crate::dialog`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Menu` 型・headless
//! `root` を再エクスポートしない理由、イシュー #729）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::dialog::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子のみを選択的に再
//! エクスポートする。状態機械 [`fandhe_frontend_headless_ui::menu::Menu`] は
//! **あえて**再エクスポートしない（[`crate::switch`]/[`crate::dialog`] の
//! 状態機械非再エクスポートと同じ理由）。`Menu` による状態管理・hydration が
//! 必要な呼び出し側は `fandhe_frontend_headless_ui::menu::Menu` を直接
//! import し、実際の描画は本モジュールの styled [`root`]（および再エクスポート
//! 済みのパーツ関数）を組み合わせて構築すること。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! `trigger`/`content` の開閉 `data-state`（open/closed）に応じた見た目の
//! 切り替えを [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]、
//! イシュー #643。`serialize_rule` を直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系属性の反映（イシュー #643）
//!
//! `item` は headless 層（`crates/headless-ui/src/menu.rs`）の virtual focus
//! パターン（イシュー #581）でハイライトされる。実 DOM フォーカスは
//! `trigger` に留まり続け、選択中の項目には `data-highlighted` 属性が
//! 付与される契約のため、`item` の highlight 表示は
//! [`crate::recipe::StateCondition::Attr`]`("data-highlighted")` で反映し、
//! `:focus-visible` は付けない（フォーカスが実際に来ないパーツへ付けても
//! 発火しないため）。`trigger` は実際にフォーカスを受けるボタン要素のため
//! `:focus-visible` によるフォーカスリングを登録する。
//!
//! # `--fandhe-reference-width` の消費（イシュー #643）
//!
//! `crates/wasm-full/src/position.rs::reposition_one`（イシュー #588）が
//! `positioner` の `style` 属性へ書き込む `--fandhe-reference-width`
//! （`trigger` の実測幅、CSS カスタムプロパティ継承で子孫の `content` から
//! 参照可能）を `content` の `min-width` が `var(--fandhe-reference-width,
//! 10rem)` として消費し、chakra-ui の `sameWidth` 相当（listbox 幅がトリガー
//! 幅へ追随する見た目）を実現する。wasm 未稼働の SSR 静的表示では変数が
//! 未定義のため `10rem`（従来の固定値）へフォールバックする。
//! # 位置ジオメトリ（`--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`）の消費（イシュー #663）
//!
//! SSR 静的フォールバック（`positioner` の `position: absolute; top: 100%;
//! left: 0`、`root` の `position: relative` をローカル座標系とする）と、
//! `crates/wasm-full/src/position.rs::wiring::reposition_one` が書き込む
//! 確定座標（`getBoundingClientRect`/`window.innerWidth/innerHeight` 由来の
//! **viewport 原点**座標）は座標系が異なるため、`positioner` へ書き込まれる
//! `data-positioned`（値なしの存在マーカー、wasm 層のみが付与し headless 層
//! の SSR/SSG 出力には決して現れない）の有無で `position` 種別ごと切り替える
//! （`docs/design/anchor-positioning-design.md` §4.4b 参照）。マーカーが
//! 無い場合は本来の静的フォールバックのまま（wasm 未稼働環境でも表示が
//! 壊れない fail-closed 動作）:
//!
//! ```css
//! [data-scope="menu"][data-part="positioner"][data-positioned] {
//!   position: fixed;
//!   top: 0;
//!   left: 0;
//!   margin-top: 0;
//!   transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);
//! }
//! ```
//!
//! `arrow`/`arrow-tip`（Menu のみ、ADR §4.2 で Select は arrow 非対象）は
//! マーカー切り替え不要で変数フォールバックのみで両立する。
//! `reposition_one` は positioner の `style`（CSS カスタムプロパティは
//! 子孫へ継承される）に加えて arrow 要素自身の `style` へも同じ値を複製
//! するため、arrow の base 規則で直接 `var(--fandhe-arrow-x, 50%)`/
//! `var(--fandhe-arrow-y, 0)` を参照できる（フォールバック値は SSR 既定
//! placement（bottom）で anchor 中央上端に相当する）。
//!
//! # arrow / arrow-tip の `data-side` 連動（イシュー #2210）
//!
//! `positioner` の `data-side`（wasm 層のみが書き込む、headless SSR 出力
//! には現れない属性。上記「位置ジオメトリ」節参照）に連動して、
//! `arrow-tip` の回転角を anchor に面する辺へ先端が向くよう切り替える。
//! CSS custom property の継承（`positioner[data-side=X]` state が
//! `--fandhe-menu-arrow-rotate` を**定義**し、`arrow-tip` の base 規則
//! が `var(..., 45deg)` で**消費**する）で実現し、[`crate::recipe::
//! SlotRecipe`] が持たない子孫結合子（イシュー #708 で意図的に非採用）は
//! 使わない。SSR は常に bottom 配置のため `positioner` 自体のジオメトリ
//! （position/top/left 等）は `data-side` state で宣言しない（純追加に
//! 保つ判断）。回転値は floating（positioner）が anchor のどちら側に
//! 出るかで決まる（`arrow`/`arrow-tip` は `border-left`/`border-top` 固定
//! + `translate(-50%, -50%)` で辺上に中心配置する前提）:
//!
//! **ネストしたサブメニューへの継承漏れ対策（Bugbot 指摘、イシュー
//! #2210）**: `--fandhe-menu-arrow-rotate` は継承される CSS custom
//! property のため、`data-side` state（`top`/`left`/`right`）だけが
//! この変数を定義し既定（bottom 相当・`data-side` 未指定）の
//! `positioner` が何も定義しないと、祖先 `positioner`（例: 画面端で
//! `top` へ反転した親メニュー）の値をネストしたサブメニューの
//! `positioner` が継承してしまい、子の `arrow-tip` が誤った向きを
//! 指す。これを防ぐため `positioner` の **base** 規則
//! （`data-side` 未指定にもマッチする、詳細度 2）でも
//! `--fandhe-menu-arrow-rotate: 45deg;` を明示的に再定義し、既定側の
//! 継承を子孫ごとにローカルへ断ち切る。`data-side` state（詳細度 3）
//! は base 規則より詳細度が高いため、top/left/right への切り替えは
//! 従来どおり上書きされる。
//!
//! | `data-side` | floating の位置 | 先端の向き | rotate |
//! |---|---|---|---|
//! | bottom（既定） | anchor の下 | 上 | `45deg` |
//! | top | anchor の上 | 下 | `225deg` |
//! | left | anchor の左 | 右 | `135deg` |
//! | right | anchor の右 | 左 | `315deg` |
//!
//! # positioner のオーバーレイ配置（PR #575 Bugbot 指摘対応）
//!
//! `positioner` に `position: absolute` を設定し、開いた menu が通常のフローに
//! 残らずオーバーレイ表示になるようにする（[`crate::dialog`] の `positioner`・
//! [`crate::select`] の `positioner` と同じ配置責務）。`trigger`/`positioner` は
//! headless 側 `root`（`crates/headless-ui/src/menu.rs`）の子として並置される
//! 兄弟要素であり、`trigger` は `positioner` の祖先になれない。そのため
//! containing block を提供する `position: relative` は共通の祖先である `root`
//! に付与する（PR #575 Bugbot 指摘 1 対応、`trigger` への誤付与を修正）。

//!
//! # 担当パートの是正（イシュー #1525、親 #1524 の 1/3 分割。`trigger` /
//! `positioner` / `content` / `arrow`（`arrow-tip` 含む）のみ担当）
//!
//! 親イシュー #1524 の 7 軸チェックリスト（サイズ / バリアント / 色 / 状態 /
//! ダーク / フォーカス / 余白・角丸・影 + hover / disabled / トランジション）
//! に対し、本イシューが担当するパートで実施した是正・意図的に合わせなかった
//! 点を記録する（`item`/`item-group`/`item-group-label`/`separator`/
//! `indicator` は 2/3（#1526）、`checkbox_item`/`radio_item`/
//! `trigger_item`（サブメニュー）系は 3/3（#1527）の担当のため一切
//! 触れていない）。
//!
//! - **`trigger`**: `border-radius` の生リテラル（`0.375rem`）を
//!   `var(--fandhe-radius-md)` へトークン化（値は同一、外観不変。
//!   select 1/2 #1774 と同じ判断）。[`crate::recipe::hover_bg_muted`] +
//!   [`crate::recipe::StateCondition::Hover`] +
//!   [`crate::recipe::hover_surface_declarations`] で hover 背景、
//!   [`crate::recipe::disabled_declarations`] +
//!   `StateCondition::Attr("data-disabled")` で
//!   headless（`crates/headless-ui/src/menu.rs`）が `disabled` 属性と対で
//!   付与する `data-disabled` の視覚反映、
//!   [`crate::recipe::transition_declarations`] で `border-color,
//!   background, color` の遷移を追加した。`:focus-visible` の直書き
//!   outline 2 宣言は [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Token`。menu は `ColorPalette` 軸を持たないため、
//!   select 1/2・combobox 1/2・date-picker 1/3 と同じ選択）へ置換した。
//!   `data-state="open"` の border-color 切り替えは実装済みのため維持した。
//! - **`content`**: `border-radius`（生 `0.375rem` →
//!   `var(--fandhe-radius-md)`）・`box-shadow`（生
//!   `0 4px 6px rgba(0, 0, 0, 0.15)` → `var(--fandhe-shadow-md)`）を
//!   トークン化した（select 2/2 #1775 と同型）。ダーク側の見た目差は
//!   `Theme` 側のトークン再定義経由で自動成立するため個別対応は不要。
//!   `data-state="closed"` の `visibility: hidden` 切り替えは実装済みの
//!   ため維持した。
//! - **content の開閉トランジションは追加しない（意図的な非対応）**:
//!   headless 層（`crates/headless-ui/src/menu.rs`）は `positioner`/
//!   `content` の closed 時に `hidden` 存在属性を同一フレームで即時
//!   付与・除去する契約であり、遷移前フレームが描画されないため CSS
//!   トランジションが発火しない。dialog（イシュー #1693/PR #1795
//!   codex-review P1 指摘）で同じ理由により追加を取り下げた判断を継承
//!   する。
//! - **`positioner` の位置ジオメトリは変更しない**: `position`/`top`/
//!   `left`/`margin-top`/`data-positioned` 切り替えと
//!   `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`/
//!   `--fandhe-reference-width` は wasm positioning 契約（イシュー
//!   #663/#588）に紐づくため触れていない。`z-index: 10` もトークンが
//!   theme に存在しないため現状維持。
//! - **`arrow`/`arrow-tip` の座標・寸法は変更しない**: 位置ジオメトリと
//!   同じ配置契約（イシュー #663）に紐づく幾何値（`0.5rem` 等）であり、
//!   色（`background`/`border-color`）は既にトークン参照済みのため是正
//!   対象がない（`arrow-tip` の回転角のみイシュー #2210 で `data-side`
//!   連動化した。上記「arrow / arrow-tip の `data-side` 連動」節参照）。
//! - **`size` variant 軸**: 既存の Xs〜Xl 5 段（イシュー #729/#1681）を
//!   変更なしで維持。
//! - **`color-palette`/variant 軸**: menu は元々これらの軸を持たない
//!   （2/3 #1526 の item highlight 配色が対象領域になり得るため、本
//!   イシューでは追加しない）。
//!
//! # 担当パートの是正（イシュー #1526、親 #1524 の 2/3 分割。`item` /
//! `item-group` / `item-group-label` / `separator` / `indicator` を担当。
//! `checkbox_item`/`radio_item`/`trigger_item`（サブメニュー）系は 3/3
//! （#1527）の担当のため触れていない）
//!
//! **スコープ解釈の注記（#2033 時点の訂正）**: 本イシュー（#1526）着手時点
//! ではイシュータイトルの「item-text / item-indicator」は headless
//! `menu` anatomy（本モジュール冒頭 rustdoc・`ANATOMY.part(...)` 一覧参照）
//! に存在しないと判断していたが、これは #1651（本イシューより後）が
//! 当該 2 パートを追加する**前**の時点の判断であり、現在は誤りである。
//! `crates/headless-ui/src/menu.rs` は #1651 で `item-text`/
//! `item-indicator` を追加済みであり、pre-styled 側の再エクスポート・
//! `SLOTS`・CSS 着装漏れはイシュー #2033（shadcn/ui 突合）で是正した
//! （本モジュール rustdoc「担当パートの是正（イシュー #2033）」節参照）。
//! 1/3（#1525）が本モジュール rustdoc に記録した分担（`item`/`item-group`/
//! `item-group-label`/`separator`/`indicator` = 2/3 本イシュー）自体は
//! 変更なく踏襲する。
//!
//! - **`item`**: select 2/2（#1502）・combobox 2/2 と同型で是正した。
//!   `display: flex` / `align-items: center` /
//!   `gap: var(--fandhe-space-2)` を追加してレイアウトを整え、
//!   `border-radius` の生リテラル（`0.25rem`）を
//!   `var(--fandhe-radius-sm)` へトークン化（値は同一、外観不変）。
//!   [`crate::recipe::hover_bg_muted`]、
//!   [`crate::recipe::StateCondition::HoverExceptAttr`]`("data-highlighted")`、
//!   [`crate::recipe::hover_surface_declarations`] で hover 背景を追加
//!   した（`Hover`（無条件）ではなく `HoverExceptAttr` を使う理由は
//!   select 2/2・combobox 2/2〔PR #1745 codex-review P1 指摘対応〕と同じ:
//!   素の `:hover:not([data-disabled])` は selector specificity
//!   （0,4,0）が `[data-highlighted]`（0,3,0）より高く、highlight 中の
//!   item にポインタが重なると muted 背景が accent 背景〔virtual focus
//!   の視覚状態〕を上書きしてコントラストが崩れるため）。
//!   [`crate::recipe::disabled_declarations`]、
//!   `StateCondition::Attr("data-disabled")` で headless
//!   （`crates/headless-ui/src/menu.rs::item`）が `disabled` 引数と対で
//!   付与する `data-disabled` を反映し、
//!   [`crate::recipe::transition_declarations`] で `background, color` の
//!   遷移を追加した。既存の `data-highlighted`（accent 背景 +
//!   accent-fg）は維持した。
//! - **`indicator`**: select 1/2（#1501）と同型。base 追加:
//!   `display: inline-block`（`transform: rotate()` を効かせるため）+
//!   `color: var(--fandhe-color-fg-muted)`。2 回目の base 登録で
//!   `transition_declarations("transform", MotionDuration::Fast)` を
//!   純追加した。headless `indicator`（`crates/headless-ui/src/menu.rs`）
//!   が反映する `data-state="open"` で `transform: rotate(180deg)` へ
//!   切り替える state を追加した。
//! - **`item-group` / `item-group-label` / `separator` は現状維持
//!   （意図的な非対応）**:
//!   - `item-group`: 参照サイトでも構造コンテナのみで独自視覚なし。
//!     select 2/2 も未スタイル（`crate::select` 同モジュール rustdoc
//!     参照）のため同じ判断を踏襲する。
//!   - `item-group-label`: select の canonical 形（`fg-muted` /
//!     `font-size-xs` / padding）と既に同一のため是正不要。
//!   - `separator`: 既にトークン経由（`border-muted` / `space-2`）で
//!     参照サイトと同等のため是正不要。
//! - **`color-palette`/variant 軸は追加しない（意図的非採用）**: 1/3
//!   の rustdoc が「2/3 の item highlight 配色が対象領域になり得る」と
//!   申し送っていたが、select 2/2・combobox 2/2 と同じく accent トークン
//!   直による highlight を維持し palette 軸は追加しない（同型部品間の
//!   一貫性優先。ダーク側はトークン再定義経由で自動成立する）。
//!
//! # 担当パートの是正（イシュー #1527、親 #1524 の 3/3 分割）
//!
//! **スコープ解釈の注記**: イシュータイトルの「option-item」は headless
//! `menu` anatomy（`ANATOMY.part(...)` 一覧参照）に存在しないパート名
//! である。2/3（#1526）が rustdoc に残した同型のスコープ解釈（「item-text
//! / item-indicator は anatomy に存在しない」）に倣い、「option-item =
//! checkbox-item / radio-item（ark-ui 旧世代の Option Item 呼称由来と
//! みられる）」と読み替え、1/3・2/3 の rustdoc が申し送った分担
//! （`checkbox_item`/`radio_item`/`trigger_item` 系 = 3/3）に従う。
//!
//! 本イシュー着手時点で `SLOTS` に `trigger-item`/`context-trigger`/
//! `checkbox-item`/`radio-item-group`/`radio-item` の 5 パートが未登録
//! （headless `ANATOMY.part(...)` には存在するが CSS が一切当たっていな
//! い状態）だったため、以下のとおり是正した。
//!
//! - **`checkbox-item` / `radio-item`**: `item`（2/3 是正）と同型の
//!   レイアウト（`display: flex` / `align-items: center` /
//!   `gap: var(--fandhe-space-2)` / `border-radius: var(--fandhe-radius-sm)`
//!   / [`crate::recipe::hover_bg_muted`] / transition）。padding は size
//!   variant の root スコープ変数 `--fandhe-menu-item-padding` を `item`
//!   と共有するため、variant 定義自体の追加は不要（size は自動適用され
//!   る）。headless（`crates/headless-ui/src/menu.rs::checkbox_item` /
//!   `radio_item`）が反映する `data-state="checked"` を select/listbox の
//!   選択済み表示（`crate::select`/`crate::listbox` 参照）と同一強度の
//!   `--fandhe-color-bg-muted` で表現し、highlight（accent 背景、`item`
//!   と同型）より弱い視覚的重みにして区別する。state 登録順は
//!   checked → highlighted → disabled → hover（同 specificity の後勝ちで
//!   highlight が checked を上書きできる順序、2/3 の `item` と同型）。
//! - **`trigger-item`**: menubar `sub-trigger`（`crate::menubar`）+ `item`
//!   の合成。`justify-content: space-between` はサブメニュー示唆の子要素
//!   を右端へ寄せる menubar 先例を踏襲。headless
//!   （`crates/headless-ui/src/menu.rs::trigger_item`）が反映する
//!   `data-state="open"`（サブメニュー側の開閉状態）を menubar
//!   `sub-trigger` の open 表示（`--fandhe-color-accent-subtle`）と同型で
//!   表現する。登録順は open → highlighted → disabled → hover（menubar と
//!   同じく highlight が open を後勝ちで上書きする順序）。
//! - **`context-trigger`**: 1/3（#1525）が是正した `trigger` と同型
//!   （cursor / bg / fg / border / `border-radius: var(--fandhe-radius-md)`
//!   / padding / hover / transition / open 時の border-color /
//!   `:focus-visible` リング / hover）。**disabled 規則は登録しない**:
//!   headless `context_trigger`（`crates/headless-ui/src/menu.rs`）は
//!   disabled 引数を持たず `data-disabled` が決して付与されないため、
//!   vacuous な規則を置かない。
//! - **`radio-item-group`**: 規則なし（意図的な非対応）。`item-group`
//!   （2/3 是正で「参照サイトでも構造コンテナのみで独自視覚なし」と判断
//!   済み）と同じ判断を踏襲する。`role="group"` の構造コンテナであり
//!   独自の視覚状態を持たない。
//!
//! **ネスト時のサブメニュー配置は静的 CSS で対応しない（意図的な非対応）**:
//! menubar `sub-content`（`top: 0; left: 100%` の side 配置）と異なり、
//! menu のサブメニューは親 `content` 内に子 Menu インスタンス由来の
//! `trigger-item`/`positioner`/`content` を入れ子配置する構成であり、
//! ネスト側も**同一の `data-part` 名**を持つ。理由は 3 点:
//! (1) [`crate::recipe::SlotRecipe`] は子孫セレクタ機構を意図的に持たない
//! （#708 確定、`crate::recipe::StateCondition::NthChildEven` rustdoc
//! 参照）ため、同一 part 名のネスト深さによる区別が静的 CSS では表現でき
//! ない。(2) 実行時配置は既存の `positioner[data-positioned]` 規則
//! （wasm positioning 契約、#663/#588、本モジュール rustdoc 参照）が
//! ネストした positioner にもそのまま適用されるため、ランタイムの
//! サブメニュー配置は既存 CSS で成立する。(3) SSR 静的フォールバックでは
//! 既存の `top: 100%; left: 0` が適用され表示は壊れない（fail-closed）。
//! `content` への `position: relative` 追加（menubar PR #1000 型）も、
//! arrow の配置基準（`--fandhe-arrow-x/y` の解決先）へ影響し得るため本
//! イシューでは行わない。
//! # 担当パートの是正（イシュー #2033、shadcn/ui 突合。`item-text` /
//! `item-indicator` の CSS 着装漏れ・`item` のグループ化/ショートカット/
//! inset/destructive 合成パターンの補完を担当）
//!
//! ルート #2001 起票時点では shadcn/ui を「補完参照」と位置づけていた
//! （`docs/design/shadcn-reference-adoption-policy.md` §7「（改訂前）」）
//! が、2026-09-07 のユーザー判断（イシュー #2153）により同文書は改訂され
//! （§8）、現在は shadcn/ui を chakra-ui / Radix Themes と並ぶ「主基準の
//! 1 つ」（3 者共同、pre-styled-ui の視覚言語に限る）と位置づけている
//! （§2「決定事項」）。本節の補完（`item-text`/`item-indicator` の CSS
//! 着装漏れ是正・`data-danger`/`data-inset` 状態追加）はこの主基準化後の
//! 位置づけの下でも、既存 variant の CSS 出力を変えない純追加（§2「golden
//! への影響方針」）として正当化される。
//!
//! - **`item-text`/`item-indicator` の CSS 未着装（構造的な見落とし、
//!   shadcn 突合以前から存在）**: `crates/headless-ui/src/menu.rs` は
//!   #1651 で当該 2 パートを anatomy へ追加済みだったが、本モジュールの
//!   `pub use` 再エクスポート一覧・[`SLOTS`]・[`recipe`] のいずれにも
//!   反映されておらず、`fandhe-frontend-pre-styled-ui` のみに依存する
//!   呼び出し側から到達不能だった（2/3 #1526 の rustdoc「スコープ解釈の
//!   注記」は #1651 以前の時点の誤認であり、現在は訂正が必要）。本イシュー
//!   で再エクスポート・`SLOTS` 追加・`item-text` への `flex: 1; min-width:
//!   0;`（[`crate::listbox`] の `item-text` と同型、ショートカット表示
//!   `kbd` 合成を項目右端へ押し出す用途）を追加した。
//! - **`item-indicator` に `.base` を追加しない（意図的、
//!   select/combobox の既知の教訓を踏襲）**: headless
//!   （`crates/headless-ui/src/menu.rs::item_indicator`）は unchecked 時に
//!   `hidden` 存在属性を付与する契約であり、styled 側で `display` を
//!   宣言すると author 規則（詳細度 (0,2,0)）が UA の
//!   `[hidden] { display: none }`（(0,1,0)）に勝って表示制御が壊れる
//!   （[`crate::select`]/[`crate::combobox`] の `item-indicator` rustdoc に
//!   既知の教訓として明記済み）。加えて menu の checkbox/radio 項目は
//!   チェックマークが項目**先頭**（左）に来る配置（shadcn/ark-ui 共通）で
//!   あり、select/combobox（`margin-left: auto` で末尾へ寄せる設計）と
//!   機械的に同一化しない。`item` 側 `gap`（`--fandhe-space-2`）が既に
//!   `item-indicator`/`item-text` 間の余白を担うため、追加の `.base`
//!   なしで先頭配置レイアウトが成立する。
//! - **destructive（危険操作）項目 `data-danger`（値なし存在属性）**:
//!   `data-variant`（`crates/headless-ui/src/clipboard.rs` が
//!   `copied`/`idle` という別意味論で既に使用しており
//!   `docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B-2「同名は
//!   同一意味論でのみ再利用」に反する）・`destructive`（issue チェック
//!   リストの用語をそのまま持ち込まない）のいずれも使わない。`item()` の
//!   `ITEM_RESERVED`（`crates/headless-ui/src/menu.rs`）に含まれないため
//!   呼び出し側が `attrs` へ `("data-danger", "")` を渡せばそのまま
//!   出力される（headless 側の変更は不要）。反映するのは
//!   `--fandhe-color-danger-fg-subtle` の文字色のみ（[`crate::theme`] の
//!   コントラストペアに登録済みのトークンを流用し、新規コントラスト検証の
//!   追加は不要）。**highlighted と重なる場合の背景色変更**はイシュー
//!   #2203 で [`crate::recipe::StateCondition::AttrAll`]（値なし存在属性
//!   同士の AND を表現する新 variant）を追加して実装済み: `item` へ
//!   `[data-danger][data-highlighted]` 専用規則
//!   （`--fandhe-color-danger-subtle` 背景 + `--fandhe-color-danger-fg-subtle`
//!   文字色、いずれも [`crate::theme`] のコントラストペアに登録済みの
//!   トークンを流用）を追加した。この合成規則の specificity は属性
//!   セレクタ 2 個分 (0,4,0) で単体規則（各 (0,3,0)）より必ず高いため、
//!   両属性が同時に立つ場合は登録順に関わらずこの規則が勝つ。**素の
//!   ポインタ hover（`data-highlighted` が付かない状態）時の赤系背景化**
//!   は引き続き意図的に対応しない: 既存の `item` hover 規則
//!   （`StateCondition::HoverExceptAttr("data-highlighted")`）を書き換える
//!   と既存 golden のバイト出力が変わり「純追加」の原則に反するため見送る。
//!   `data-danger` の state 規則は `data-highlighted` の state 規則より
//!   **前**に登録したまま据え置く（golden バイト安定のための順序維持であり、
//!   合成規則追加により両者の勝敗自体は specificity で決まるため登録順は
//!   もはや非本質）。`checkbox-item`/`radio-item`/`trigger-item` への対応
//!   も行わない（shadcn の destructive item は素の item 用途が中心のため
//!   過剰実装を避ける）。
//! - **inset 項目 `data-inset`（値なし存在属性）**: アイコン/インジケータを
//!   持たない項目のテキスト位置を、持つ項目と視覚的に揃える。
//!   `padding-inline-start` のみを `--fandhe-space-6`
//!   （`item-indicator` の実測幅ではなく、gap + インジケータ相当幅の
//!   近似値）へ上書きする。既存の `--fandhe-menu-item-padding` shorthand
//!   （root スコープの size variant）は上書きしない（属性セレクタの詳細度
//!   （0,3,0）が base の `[data-scope][data-part]`（0,2,0）より高いため、
//!   `padding-inline-start` のみが後勝ちする）。
//! - **`data-danger`/`data-inset` は recipe variant 軸ではない**: 1/3
//!   （#1525）・2/3（#1526）の「`color-palette`/`variant` 軸は追加しない
//!   （意図的非採用）」は root スコープの
//!   [`crate::recipe::SlotRecipe::variant`] 軸（`Size` 等、呼び出し側が
//!   コンポーネント全体へ選ぶ列挙値）を指す判断であり、`data-danger`/
//!   `data-inset` は呼び出し側が個別の `item` インスタンスへ都度付与する
//!   pre-styled-only の状態属性（[`crate::recipe::StateCondition::Attr`]
//!   経由）であるため、この既存の非採用判断とは対象が別であり矛盾しない。
//! - **`item-group`/`item-group-label`/`separator`/`checkbox-item`/
//!   `radio-item`/`trigger-item`/`context-trigger` は現状維持（意図的な
//!   非対応）**: shadcn 突合の結果、2/3（#1526）・3/3（#1527）が既に施した
//!   是正（グループ・ラベル・checked 表示・highlight・サブメニュー表示等）
//!   が shadcn dropdown-menu/context-menu と同等の構造・状態表現を
//!   カバーしており、追加の是正は不要と判断した。
//! - **ショートカット（`kbd` 合成）は新規 anatomy パートを追加しない**:
//!   `crates/pre-styled-ui/src/kbd.rs`（`kbd::kbd`）が既に独立した
//!   pre-styled-only 部品として実装済みのため、呼び出し側が
//!   `item(...)` の children に `item_text(...)` + `kbd::kbd(...)` を
//!   並べる合成パターンで実現する（Demo は
//!   `crates/docs-site/src/showcase.rs::menu_section` 参照）。
//! - **`crates/docs-site/src/component_specs_overlay.rs::MENU` の
//!   `examples` を埋めた**: グループ+ラベル・checkbox/radio 項目・
//!   サブメニュー・ショートカット・inset・destructive の 6 パターンを
//!   ノード木 API のコード例として追加した（`format!` によるマークアップ
//!   直接組み立てはしない、REQ-1 遵守）。
//!
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root`・状態機械 `Menu` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::menu` を直接 import する。
// `MenuCheckboxItem`/`MenuRadioItemGroup` は `Menu` と異なり root への
// inherent メソッドを持たず、未スタイル root の静かな適用漏れが起きないため
// 従来通り再エクスポートを維持する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::menu::{
    arrow, arrow_tip, checkbox_item, content, context_trigger, indicator, item, item_group,
    item_group_label, item_indicator, item_text, positioner, radio_item, radio_item_group,
    separator, trigger, trigger_item, MenuCheckboxItem, MenuRadioItemGroup,
};
// `trigger`/`trigger_item`/`context_trigger` 等の `state` 引数・
// `MenuCheckboxItem`/`MenuRadioItemGroup` の `Component::Action`
// （dispatch 対象）はいずれも `state` モジュール由来で上記選択的再エクスポート
// では到達しない。呼び出し側が `fandhe-frontend-pre-styled-ui` のみに依存して
// 呼び出せることを保証するための明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{
    CheckableAction, DisclosureAction, OpenState, SingleSelectAction,
};

/// headless `menu` anatomy の `data-part` 一覧（`crates/headless-ui/src/menu.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "indicator",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
    "item",
    "item-group",
    "item-group-label",
    "separator",
    "trigger-item",
    "context-trigger",
    "checkbox-item",
    "radio-item-group",
    "radio-item",
    "item-text",
    "item-indicator",
];

/// この styled Menu の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("menu", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "padding",
                    "var(--fandhe-menu-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （select 1/2 #1774・combobox 1/2・date-picker 1/3 と同型のパターン、
        // イシュー #1525）。
        .base(
            "trigger",
            transition_declarations("border-color, background, color", MotionDuration::Fast),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
                // イシュー #2210 Bugbot 指摘: `--fandhe-menu-arrow-rotate`
                // は CSS custom property であり継承される。data-side が
                // 既定（未指定 = bottom 相当）の positioner にこの base
                // 規則で明示的にフォールバック値（45deg）を再定義して
                // おかないと、祖先 positioner（ネストしたサブメニュー等）
                // が top/left/right の値を持つ場合にそれを継承してしまい
                // arrow-tip の向きを誤る。`positioner[data-side=...]`
                // state（詳細度 3）はこの base 規則（詳細度 2）より
                // 常に優先されるため、既定以外の分岐は従来どおり上書き
                // される。
                decl("--fandhe-menu-arrow-rotate", "45deg"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl(
                    "padding",
                    "var(--fandhe-menu-content-padding, var(--fandhe-space-2))",
                ),
                decl("min-width", "var(--fandhe-reference-width, 10rem)"),
            ],
        )
        // イシュー #663: arrow はマーカー切り替え不要（モジュール rustdoc
        // 参照）。フォールバック値は SSR 既定 placement（bottom）で anchor
        // 中央上端に相当する。
        .base(
            "arrow",
            vec![
                decl("position", "absolute"),
                decl("left", "var(--fandhe-arrow-x, 50%)"),
                decl("top", "var(--fandhe-arrow-y, 0)"),
                decl("transform", "translate(-50%, -50%)"),
            ],
        )
        .base(
            "arrow-tip",
            vec![
                decl("width", "0.5rem"),
                decl("height", "0.5rem"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border-left", "1px solid var(--fandhe-color-border)"),
                decl("border-top", "1px solid var(--fandhe-color-border)"),
                // イシュー #2210: `positioner[data-side=...]` state が
                // 定義する `--fandhe-menu-arrow-rotate` を消費する。
                // フォールバック値 45deg は無指定（SSR 既定の bottom
                // 配置）時の従来値と一致する（回帰なし）。
                decl("transform", "rotate(var(--fandhe-menu-arrow-rotate, 45deg))"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （1/3・select 2/2 と同型のパターン、イシュー #1526）。
        .base(
            "item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "indicator",
            vec![
                // `transform: rotate()` を効かせるための display（select 1/2
                // #1501・accordion `item-indicator` と同じ根拠、モジュール
                // rustdoc「担当パートの是正（#1526）」節参照）。
                decl("display", "inline-block"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "indicator",
            transition_declarations("transform", MotionDuration::Fast),
        )
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("border", "0"),
                decl("border-top", "1px solid var(--fandhe-color-border-muted)"),
                decl("margin", "var(--fandhe-space-2) 0"),
            ],
        )
        // イシュー #1527: checkbox-item / radio-item は `item` と同型の
        // レイアウト（size variant の root スコープ変数 `--fandhe-menu-item-padding`
        // を共有するため、variant 定義自体の追加は不要。モジュール rustdoc
        // 「担当パートの是正（#1527）」節参照）。
        .base(
            "checkbox-item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "checkbox-item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "radio-item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "radio-item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // イシュー #1527: trigger-item は menubar `sub-trigger`（`crates/
        // pre-styled-ui/src/menubar.rs`）と `item` の合成。サブメニュー示唆
        // の子要素（インジケータ等）を右端へ寄せる `justify-content:
        // space-between` を menubar 先例から踏襲する。
        .base(
            "trigger-item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "trigger-item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // イシュー #1527: context-trigger は `trigger` と同型（1/3 の
        // `trigger` 是正を踏襲、モジュール rustdoc 参照）。
        .base(
            "context-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "padding",
                    "var(--fandhe-menu-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                hover_bg_muted(),
            ],
        )
        .base(
            "context-trigger",
            transition_declarations("border-color, background, color", MotionDuration::Fast),
        )
        // イシュー #551 受け入れ条件: `trigger`/`content` の開閉状態に応じた見た目の切り替え。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // イシュー #2033: `data-danger`（危険操作項目、`item()` の自由
        // `attrs` 経由で呼び出し側が付与する pre-styled-only の存在属性。
        // `data-variant`/`destructive` を使わない理由はモジュール rustdoc
        // 参照）。**下記の highlight 規則より前に登録する**: `data-danger`/
        // `data-highlighted` はいずれも単一属性セレクタ（specificity
        // (0,3,0) 同点）であり、両方が立つ場合は source 順で後勝ちする。
        // 両属性の AND を 1 セレクタで表現する `StateCondition` variant が
        // 現状存在しない（`AttrEqAll` へ空文字列を渡すと
        // `is_valid_identifier` が拒否し規則ごと無音に脱落する、
        // [`StateCondition::HoverExceptAttr`] rustdoc に記録済みの罠と同型）
        // ため、本規則をあえて highlight 規則より先に置くことで、
        // highlighted 中は検証済みコントラストペア（accent/accent-fg）が
        // 優先され、danger の文字色は highlighted でないときのみ反映される
        // （逆順だと `danger-fg-subtle` 文字色 + `accent` 背景という未検証の
        // コントラストペアが highlighted 状態で露出してしまう）。
        .state(
            "item",
            StateCondition::Attr("data-danger"),
            vec![decl("color", "var(--fandhe-color-danger-fg-subtle)")],
        )
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`item` は実 DOM フォーカスを受けないため `:focus-visible` ではなく
        // `data-highlighted` で表現する、モジュール rustdoc 参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // イシュー #2203: `data-danger` × `data-highlighted` の背景色合成
        // （`StateCondition::AttrAll` 追加により実装。上記 2 規則との
        // カスケード分析）。本規則の specificity は属性セレクタ 2 個分
        // (0,4,0) で `[data-danger]`・`[data-highlighted]` 単体（いずれも
        // (0,3,0)）より高いため、登録順に依存せず両属性が同時に立つ場合は
        // 常にこの規則が勝つ（上記の「danger を highlighted より前に登録」
        // という順序は golden バイト安定のため据え置くが、本規則の勝敗には
        // もはや無関係）。`item` の hover 規則は
        // `HoverExceptAttr("data-highlighted")`（highlighted を除外）の
        // ため本規則と衝突しない。色は `--fandhe-color-danger-subtle` /
        // `--fandhe-color-danger-fg-subtle`（`theme.rs` の
        // `BODY_TEXT_PAIRS` に登録済み・4.5:1 検証済みのコントラストペア）
        // を流用し、新規トークン・`color-mix`・`Theme` 変更は行わない
        // （shadcn の `bg-destructive/10` + `text-destructive` に相当）。
        // 素のポインタ hover（`data-highlighted` が付かない状態）時の
        // 赤系背景化は引き続き意図的に非対応（モジュール rustdoc参照）。
        .state(
            "item",
            StateCondition::AttrAll(&["data-danger", "data-highlighted"]),
            vec![
                decl("background", "var(--fandhe-color-danger-subtle)"),
                decl("color", "var(--fandhe-color-danger-fg-subtle)"),
            ],
        )
        // イシュー #1526: headless `item`（`crates/headless-ui/src/menu.rs`）
        // が `disabled` 引数と対で付与する `data-disabled` を消費する
        // （select 2/2・combobox 2/2 と同型）。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1526: item の hover 実適用。`StateCondition::Hover` では
        // なく `StateCondition::HoverExceptAttr("data-highlighted")` を使う
        // 理由は select 2/2（#1502）・combobox 2/2（PR #1745 codex-review
        // P1 指摘）と同じ: 素の `Hover` は selector specificity
        // （0,4,0）が `[data-highlighted]`（0,3,0）より高く、highlight 中の
        // item にポインタが重なると muted 背景が accent 背景（virtual
        // focus の視覚状態）を上書きしてコントラストが崩れるため、
        // highlight 中の item 自体を hover の対象から除外する。
        .state(
            "item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        // イシュー #1526: headless `indicator`（`crates/headless-ui/src/
        // menu.rs`）が反映する `data-state="open"` に応じてシェブロン等の
        // 開閉インジケータを反転させる（select 1/2 #1501・accordion
        // `item-indicator` と同型）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(180deg)")],
        )
        // イシュー #643 → #1525 で canonical ヘルパへ置換: `trigger` は
        // キーボード操作時のみのフォーカスリング。menu は palette 軸を
        // 持たないため `FocusRingColor::Token`（select 1/2・combobox 1/2・
        // date-picker 1/3 と同じ選択、モジュール rustdoc「担当パートの
        // 是正」節参照）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1525: headless `trigger`（`crates/headless-ui/src/
        // menu.rs`）が `disabled` 属性と対で付与する `data-disabled` を
        // 消費する（select 1/2・combobox 1/2 と同型）。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1525: trigger の hover 実適用（`--fandhe-hover-bg` の
        // 間接参照経由。`@media (hover: hover)` + `:not([data-disabled])`
        // は `Hover` 側が自動付与する、モジュール rustdoc「担当パートの
        // 是正」節参照）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #2210: `positioner` の `data-side`（wasm 層のみが書き込む、
        // headless SSR 出力には現れない）に連動して arrow-tip の回転角を
        // 切り替える。回転変数の**定義のみ**を宣言し、positioner 自体の
        // ジオメトリ（position/top/left 等）には触れない（SSR は常に
        // bottom 配置のため、popover と同じくジオメトリ宣言は不要）。
        // 値は anchor に面する辺へ先端を向ける幾何（モジュール rustdoc
        // 「arrow / arrow-tip の data-side 連動」節参照）。
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "top"),
            vec![decl("--fandhe-menu-arrow-rotate", "225deg")],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "left"),
            vec![decl("--fandhe-menu-arrow-rotate", "135deg")],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "right"),
            vec![decl("--fandhe-menu-arrow-rotate", "315deg")],
        )
        // イシュー #663: wasm 層が `data-positioned` マーカーを付与したら
        // 確定座標（viewport 座標系の `position: fixed`）へ切り替える
        // （モジュール rustdoc 参照）。base の `positioner` 規則（absolute）
        // より詳細度が高く、CSS 記述順（states は最後尾）でも上書きする。
        .state(
            "positioner",
            StateCondition::Attr("data-positioned"),
            vec![
                decl("position", "fixed"),
                decl("top", "0"),
                decl("left", "0"),
                decl("margin-top", "0"),
                decl(
                    "transform",
                    "translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0)",
                ),
            ],
        )
        // イシュー #1527: checkbox-item / radio-item の checked 表示。select/
        // listbox の選択済み表示（`--fandhe-color-bg-muted`）と同一強度にし、
        // highlight（accent）より弱い視覚的重みにする（モジュール rustdoc
        // 参照）。登録順は checked → highlighted → disabled → hover
        // （同 specificity の後勝ちで highlight が checked を上書きできる
        // 順序、2/3 の `item` と同型）。
        .state(
            "checkbox-item",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        .state(
            "checkbox-item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "checkbox-item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "checkbox-item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        .state(
            "radio-item",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        .state(
            "radio-item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "radio-item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "radio-item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        // イシュー #1527: trigger-item のサブメニュー開閉表示。menubar
        // `sub-trigger` 先例（`crates/pre-styled-ui/src/menubar.rs`）に倣い
        // open 状態を accent-subtle 背景で示す。登録順は open → highlighted
        // → disabled → hover（menubar と同じく highlight が open を後勝ちで
        // 上書きする順序）。
        .state(
            "trigger-item",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        .state(
            "trigger-item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "trigger-item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "trigger-item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        // イシュー #1527: context-trigger は `trigger` と同型の open/focus/
        // hover 状態を持つが、headless 層が disabled 引数を持たず
        // `data-disabled` が決して付かないため disabled 規則は登録しない
        // （vacuous な規則を置かない判断、モジュール rustdoc 参照）。
        .state(
            "context-trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "context-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "context-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。`--fandhe-reference-width`/
        // `--fandhe-arrow-*`/`--fandhe-x`/`--fandhe-y`（wasm positioning 契約、
        // #663/#588）には手を触れない（モジュール rustdoc 参照）。
        // イシュー #1681: Xs は Sm(1,2)→Md(2,3)→Lg(3,4) の等差進行を 1 段
        // 外挿した (0-5, 1)（`space-0`は未定義のため最小刻み `space-0-5` を
        // 使う）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-0-5) var(--fandhe-space-1)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-0-5) var(--fandhe-space-1)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-0-5)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-1)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-3)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-menu-trigger-padding",
                    "var(--fandhe-space-4) var(--fandhe-space-5)",
                ),
                decl(
                    "--fandhe-menu-item-padding",
                    "var(--fandhe-space-4) var(--fandhe-space-5)",
                ),
                decl("--fandhe-menu-content-padding", "var(--fandhe-space-4)"),
            ],
        )
        // イシュー #2033: shadcn/ui 突合で判明した `item-text`/
        // `item-indicator`（#1651 で headless anatomy へ追加済みだが、
        // pre-styled 側の再エクスポート・`SLOTS`・CSS 着装が漏れていた欠落。
        // モジュール rustdoc「担当パートの是正（イシュー #2033）」節参照）。
        .base(
            "item-text",
            vec![decl("flex", "1"), decl("min-width", "0")],
        )
        // イシュー #2033: ショートカット表示（`kbd` 合成、後続の兄弟要素）を
        // 項目右端へ押し出すための伸縮のみ（`crate::listbox` の
        // `item-text` と同型）。`item-indicator`（下記）には意図的に
        // `.base` を追加しない。`data-danger`（危険操作項目）の state 規則は
        // golden バイト安定のため、下記へ移さず `data-highlighted` 規則
        // より**前**（このコメント直前ではなくモジュール前半、
        // `item[data-highlighted]` 規則の直前）へ登録してある（イシュー
        // #2203 で `[data-danger][data-highlighted]` 合成規則
        // （`StateCondition::AttrAll`）を追加済みのため両者の勝敗自体は
        // specificity で決まり、この登録順序はもはや非本質。モジュール
        // rustdoc「担当パートの是正（イシュー #2033）」節参照）。
        // イシュー #2033: `data-inset`（アイコン/インジケータを持たない
        // 項目のテキスト位置を、持つ項目と揃えるための存在属性。値は
        // `item-indicator` の実測ではなく `item` の `gap`
        // （`--fandhe-space-2`）+ インジケータ相当幅の近似として
        // `--fandhe-space-6` を採用する）。
        .state(
            "item",
            StateCondition::Attr("data-inset"),
            vec![decl("padding-inline-start", "var(--fandhe-space-6)")],
        )
        .default_variant(Size::Md)
}

/// この styled Menu が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::menu::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = menu::root(Size::Md, OpenState::Open, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="menu" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::menu::root(state, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="menu"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolutely_positioned_for_overlay() {
        // PR #575 Bugbot 指摘対応: positioner がオーバーレイ配置になっている
        // ことを固定する（通常のフローに残ったままにならない）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        // PR #575 Bugbot 指摘 1 対応: `trigger` と `positioner` は headless
        // `root` の下の兄弟要素であり、`trigger` は `positioner` の祖先には
        // なれない。そのため `position: relative` は共通祖先である `root`
        // に付与されていることを固定する（`trigger` への誤付与への回帰防止）。
        let css = stylesheet();
        assert!(
            css.contains("[data-scope=\"menu\"][data-part=\"root\"] {\n  position: relative;\n}\n")
        );
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- イシュー #729: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                OpenState::Closed,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-menu--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_pre_729_fallback() {
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        let css = stylesheet();
        assert!(css.contains(
            "padding: var(--fandhe-menu-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(css.contains(
            "padding: var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(css.contains("padding: var(--fandhe-menu-content-padding, var(--fandhe-space-2));"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="trigger"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="content"][data-state="closed"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_menu_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // headless `Menu`（イシュー #729 により本モジュールから再エクスポート
        // しないため、エスケープハッチ経由で直接 import する。モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）経由で固定する。
        use fandhe_frontend_headless_ui::menu::Menu;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = Menu::default();
        assert_eq!(m.state(), OpenState::Closed);

        let ssr_html = render(&m.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut m, "open", ""));
        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Menu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }

    #[test]
    fn item_highlighted_attr_is_styled_and_trigger_has_focus_visible_ring() {
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`data-highlighted`）とキーボード操作系属性（`:focus-visible`）が
        // recipe 経由で反映されることを固定する。イシュー #1525 で
        // `trigger` の focus ring を canonical ヘルパ
        // （[`crate::recipe::focus_ring_declarations`]）へ置換したため、
        // 期待値をトークン参照形へ更新した（select 1/2 と同型）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    // --- イシュー #1525: trigger / content のスタイル調整 ---

    #[test]
    fn trigger_border_radius_uses_radius_md_token() {
        // radius トークン化（外観不変、select 1/2 と同じ判断）。`item` 等
        // （2/3・#1526 の担当）は本イシューの対象外のため生リテラルのまま
        // 残り得る点に注意し、`trigger` ブロックのみを切り出して検証する。
        let css = stylesheet();
        let trigger_start = css
            .find(r#"[data-scope="menu"][data-part="trigger"] {"#)
            .expect("trigger base rule must exist");
        let trigger_block_end = css[trigger_start..]
            .find(
                "}
",
            )
            .map(|idx| trigger_start + idx)
            .expect("trigger base rule must be closed");
        let trigger_block = &css[trigger_start..trigger_block_end];
        assert!(trigger_block.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(!trigger_block.contains("border-radius: 0.375rem;"));
    }

    #[test]
    fn trigger_disabled_attr_is_styled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_hover_rule_is_scoped_to_hover_capable_devices_and_excludes_disabled() {
        // `StateCondition::Hover` は `@media (hover: hover)` 配下へ集約され
        // `:not([data-disabled])` を自動付与する（`crate::recipe` 契約、
        // モジュール rustdoc「担当パートの是正」節参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(
            css.contains(r#"[data-scope="menu"][data-part="trigger"]:hover:not([data-disabled])"#)
        );
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn trigger_has_transition_declarations() {
        let css = stylesheet();
        assert!(css.contains("transition-property: border-color, background, color;"));
    }

    #[test]
    fn content_border_radius_and_shadow_use_tokens() {
        // イシュー #1525: `content` の生 `border-radius`/`box-shadow` を
        // トークン化した（select 2/2 #1775 と同型）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="content"] {"#));
        assert!(css.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-md);"));
        assert!(!css.contains("box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);"));
    }

    #[test]
    fn content_min_width_consumes_fandhe_reference_width_css_var() {
        // イシュー #643 受け入れ条件: `--fandhe-reference-width`（wasm 層
        // `crates/wasm-full/src/position.rs::reposition_one` が positioner へ
        // 書き込む変数）を CSS 継承で消費する sameWidth 相当のスタイルが
        // 反映されることを固定する（SSR 静的表示では 10rem へフォールバック）。
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, 10rem);"));
    }

    #[test]
    fn positioner_switches_to_fixed_geometry_when_data_positioned_marker_is_present() {
        // イシュー #663 受け入れ条件: wasm 層が付与する `data-positioned`
        // マーカーが立っているときのみ、positioner が確定座標（viewport
        // 座標系の `position: fixed`）へ切り替わることをゴールデンで固定する。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menu\"][data-part=\"positioner\"][data-positioned] {\n  \
             position: fixed;\n  \
             top: 0;\n  \
             left: 0;\n  \
             margin-top: 0;\n  \
             transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);\n\
             }\n"
        ));
    }

    #[test]
    fn positioner_base_rule_keeps_static_ssr_fallback_geometry() {
        // イシュー #663: `data-positioned` マーカー不在（SSR 静的表示・wasm
        // 未稼働）では従来どおり absolute + ローカル座標系のままであることの
        // 回帰固定（`positioner_is_absolutely_positioned_for_overlay` と
        // 重複しない観点として `top: 100%;` も確認する）。
        let css = stylesheet();
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn arrow_consumes_fandhe_arrow_geometry_css_vars_and_arrow_tip_is_declared() {
        // イシュー #663 受け入れ条件: arrow はマーカー切り替え不要で
        // `--fandhe-arrow-x`/`--fandhe-arrow-y` を変数フォールバックのみで
        // 消費することを固定する（モジュール rustdoc 参照）。arrow-tip は
        // 座標変数を持たない（arrow の子として相対配置される装飾要素）ため、
        // base 規則が登録されていることのみ確認する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="arrow"]"#));
        assert!(css.contains("left: var(--fandhe-arrow-x, 50%);"));
        assert!(css.contains("top: var(--fandhe-arrow-y, 0);"));
        assert!(css.contains(r#"[data-scope="menu"][data-part="arrow-tip"]"#));
    }

    #[test]
    fn arrow_tip_rotation_follows_positioner_data_side() {
        // イシュー #2210 受け入れ条件: `positioner[data-side=...]` に連動して
        // `--fandhe-menu-arrow-rotate` の値が切り替わることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="arrow-tip"]"#));
        assert!(css.contains("transform: rotate(var(--fandhe-menu-arrow-rotate, 45deg));"));
        assert!(css.contains(r#"[data-scope="menu"][data-part="positioner"][data-side="top"]"#));
        assert!(css.contains("--fandhe-menu-arrow-rotate: 225deg;"));
        assert!(css.contains(r#"[data-scope="menu"][data-part="positioner"][data-side="left"]"#));
        assert!(css.contains("--fandhe-menu-arrow-rotate: 135deg;"));
        assert!(css.contains(r#"[data-scope="menu"][data-part="positioner"][data-side="right"]"#));
        assert!(css.contains("--fandhe-menu-arrow-rotate: 315deg;"));
    }

    #[test]
    fn positioner_base_rule_resets_arrow_rotate_to_prevent_inherited_side_leaking_into_nested_menus(
    ) {
        // イシュー #2210 Bugbot 指摘の回帰固定: `--fandhe-menu-arrow-rotate`
        // は CSS custom property のため継承される。`positioner` の base
        // 規則（`data-side` 未指定 = 既定 bottom 相当）が明示的に
        // フォールバック値（45deg）を再定義していないと、ネストした
        // サブメニューの positioner が data-side 未指定（既定）でも
        // 祖先 positioner（例: `data-side="top"`）の 225deg を継承して
        // しまい、子の arrow-tip が誤った向きを指す。base 規則
        // （`[data-scope="menu"][data-part="positioner"] {`、詳細度 2）
        // に `--fandhe-menu-arrow-rotate: 45deg;` が含まれることを固定
        // する。この宣言は `positioner[data-side=...]` state（詳細度 3）
        // より低い詳細度のため、data-side 明示時の上書きは壊さない。
        let css = stylesheet();
        let base_rule_start = css
            .find("[data-scope=\"menu\"][data-part=\"positioner\"] {")
            .expect("positioner base rule must exist");
        let base_rule_end = css[base_rule_start..]
            .find('}')
            .map(|offset| base_rule_start + offset)
            .expect("positioner base rule must be closed");
        let base_rule = &css[base_rule_start..base_rule_end];
        assert!(
            base_rule.contains("--fandhe-menu-arrow-rotate: 45deg;"),
            "positioner base rule must locally reset --fandhe-menu-arrow-rotate \
             so nested default-side positioners do not inherit an ancestor's \
             non-default rotate value; base rule was: {base_rule:?}"
        );
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（イシュー #663 §5 手順 6）: 本イシューが導入する
        // 位置ジオメトリ変数（`--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`）
        // への参照はすべて明示フォールバック値を持つ（裸の `var(--x)` 禁止）。
        // 変数未定義（SSR・wasm 失敗時）でも表示が壊れないことを保証する
        // （テーマトークン系の `--fandhe-color-*` 等はフォールバック不要の
        // 常時定義済み変数のため対象外とする）。
        let css = stylesheet();
        for marker in ["var(--fandhe-x", "var(--fandhe-y", "var(--fandhe-arrow-"] {
            for (idx, _) in css.match_indices(marker) {
                let close = css[idx..]
                    .find(')')
                    .expect("every var( occurrence must be closed within the stylesheet");
                let inside = &css[idx + "var(".len()..idx + close];
                assert!(
                    inside.contains(','),
                    "var() reference without an explicit fallback found: var({inside})"
                );
            }
        }
    }

    // --- イシュー #1526: item / indicator のスタイル調整 ---

    #[test]
    fn item_border_radius_uses_radius_sm_token() {
        // radius トークン化（外観不変、select 2/2 と同じ判断）。
        let css = stylesheet();
        let item_start = css
            .find(r#"[data-scope="menu"][data-part="item"] {"#)
            .expect("item base rule must exist");
        let item_block_end = css[item_start..]
            .find(
                "}
",
            )
            .map(|idx| item_start + idx)
            .expect("item base rule must be closed");
        let item_block = &css[item_start..item_block_end];
        assert!(item_block.contains("border-radius: var(--fandhe-radius-sm);"));
        assert!(!item_block.contains("border-radius: 0.25rem;"));
    }

    #[test]
    fn item_disabled_attr_is_styled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="item"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_hover_rule_is_scoped_and_excludes_highlighted() {
        // `StateCondition::HoverExceptAttr` は `@media (hover: hover)`
        // 配下へ集約され `:not([data-highlighted])` を自動付与する
        // （select 2/2・combobox 2/2 と同型、モジュール rustdoc「担当
        // パートの是正（#1526）」節参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="menu"][data-part="item"]:hover:not([data-disabled]):not([data-highlighted])"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn item_has_transition_declarations() {
        let css = stylesheet();
        assert!(css.contains("transition-property: background, color;"));
    }

    #[test]
    fn indicator_rotates_when_open_and_has_transform_transition() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="indicator"] {"#));
        assert!(css.contains("display: inline-block;"));
        assert!(css.contains("transition-property: transform;"));
        assert!(css.contains(r#"[data-scope="menu"][data-part="indicator"][data-state="open"] {"#));
        assert!(css.contains("transform: rotate(180deg);"));
    }

    // --- イシュー #2033: shadcn/ui 突合で補完した item-text/item-indicator
    // 着装・data-danger/data-inset ---
    //
    // `data-danger`/`data-inset` は「pre-styled が出力せず、呼び出し側が
    // `item()` の自由 `attrs` 経由で付与し、recipe が `StateCondition` から
    // 参照するのみ」という役割 B 亜種のパターン（`docs/design/
    // pre-styled-ui-data-attr-vocabulary.md` §2.2 参照）であり、
    // `crates/pre-styled-ui/tests/data_attr_vocabulary.rs`（役割 A 限定の
    // スコープ）ではなく本モジュールの `#[cfg(test)]` ユニットテストとして
    // 固定する（`progress.rs`/`tour.rs` 等の類似ケースの慣習に合わせる）。

    #[test]
    fn item_text_and_item_indicator_are_reexported_and_render_anatomy_parts() {
        // #1651 で headless anatomy へ追加済みだった 2 パートが
        // `fandhe-frontend-pre-styled-ui` のみに依存する呼び出し側からも
        // 到達できることを固定する（本イシューが是正した再エクスポート
        // 漏れの回帰防止）。
        let text_html = render(&item_text(false, false, vec![], vec![]));
        assert!(text_html.contains(r#"data-scope="menu""#));
        assert!(text_html.contains(r#"data-part="item-text""#));

        let indicator_html = render(&item_indicator(true, vec![], vec![]));
        assert!(indicator_html.contains(r#"data-scope="menu""#));
        assert!(indicator_html.contains(r#"data-part="item-indicator""#));
    }

    #[test]
    fn item_text_slot_gets_flex_css_from_recipe() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="item-text"] {"#));
        assert!(css.contains("flex: 1;"));
        assert!(css.contains("min-width: 0;"));
    }

    #[test]
    fn data_danger_attr_passes_through_item_and_is_styled_by_recipe() {
        // `data-danger` は headless `ITEM_RESERVED`
        // （`crates/headless-ui/src/menu.rs`）に含まれないため `item()` の
        // `attrs` 経由でそのまま出力される（headless 側の変更不要な設計の
        // 固定）。
        let html = render(&item(
            "delete",
            false,
            false,
            vec![("data-danger", "")],
            vec![],
        ));
        assert!(html.contains(r#"data-danger="""#));

        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="item"][data-danger] {"#));
        assert!(css.contains("color: var(--fandhe-color-danger-fg-subtle);"));
    }

    #[test]
    fn data_danger_and_highlighted_item_gets_composite_rule() {
        // イシュー #2203: `data-danger` × `data-highlighted` が同時に
        // 立った item は `StateCondition::AttrAll` 由来の合成規則
        // （背景色 + 文字色）でスタイルされることを固定する。
        let html = render(&item(
            "delete",
            false,
            true,
            vec![("data-danger", "")],
            vec![],
        ));
        assert!(html.contains(r#"data-danger="""#));
        assert!(html.contains(r#"data-highlighted="""#));

        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="menu"][data-part="item"][data-danger][data-highlighted] {"#));
        assert!(css.contains("background: var(--fandhe-color-danger-subtle);"));
        assert!(css.contains("color: var(--fandhe-color-danger-fg-subtle);"));
    }

    #[test]
    fn data_danger_attr_with_forged_value_does_not_break_out_of_attribute() {
        // A03 回帰: `data-danger` は呼び出し側が任意値を渡せる `attrs`
        // 経由で出力されるため、偽装ペイロードを渡しても `render()` の
        // 既定エスケープ（REQ-1）が属性値を安全にエスケープし、
        // `<script>` タグや属性からの脱出が残らないことを固定する。
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item(
            "delete",
            false,
            true,
            vec![("data-danger", payload)],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("\"><script"));
    }

    #[test]
    fn data_inset_attr_passes_through_item_and_is_styled_by_recipe() {
        let html = render(&item(
            "settings",
            false,
            false,
            vec![("data-inset", "")],
            vec![],
        ));
        assert!(html.contains(r#"data-inset="""#));

        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="item"][data-inset] {"#));
        assert!(css.contains("padding-inline-start: var(--fandhe-space-6);"));
    }

    #[test]
    fn data_danger_attr_with_forged_value_still_passes_the_default_escape() {
        // A03 回帰: `data-danger` は値なし存在属性の契約だが、呼び出し側が
        // 万一値付きで偽装しても `render()` の既定エスケープ（REQ-1）を
        // 迂回しないことを確認する（`item()` は `raw_html()` を経由せず
        // ノード木 API のみで組み立てる契約）。
        let html = render(&item(
            "x",
            false,
            false,
            vec![("data-danger", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
    }
}
