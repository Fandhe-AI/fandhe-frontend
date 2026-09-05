//! Carousel（スライド送り UI）headless コンポーネント（イシュー #754、
//! 親 #748、祖父 #520）。
//!
//! ark-ui の Carousel
//!（`.claude/skills/ark-ui/references/components/collection/carousel.md`
//! 相当）を参考に、Root / Control / PrevTrigger / NextTrigger / ItemGroup /
//! Item / IndicatorGroup / Indicator の 8 anatomy パーツと、
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する index 状態機械
//! [`Carousel`] を提供する。
//!
//! # 参考サイトとの突合（イシュー #1660）
//!
//! zag.js（ark-ui の内部実装、`carousel.connect.ts`/`carousel.anatomy.ts`）
//! を一次情報として突合した結果、以下を是正した:
//!
//! - **`data-orientation` を全パーツへ拡張**: 是正前は [`root`] のみが
//!   出力していたが、zag.js は root/item-group/item/control/prev-trigger/
//!   next-trigger/indicator-group/indicator の全パーツへ付与する。特に
//!   `item-group` への欠落は実害があった:
//!   `fandhe-frontend-pre-styled-ui` の recipe（`item-group[data-orientation=
//!   "vertical"]`）が縦方向スライドの `translateY` 切替に依存しており、
//!   呼び出し側が手動で `attrs` へ `data-orientation` を渡さない限り
//!   縦方向 carousel が機能しなかった。
//! - **`item`/`indicator` に `data-index`（0-origin、`usize` の Display
//!   整形のみから合成）を追加**: zag.js の `item`/`indicator` が持つ位置
//!   識別属性に追随する。
//! - **`item` に `data-inview`（現在表示中スライドの存在属性）を追加**:
//!   zag.js の `data-inview`（`slidesPerPage` 複数対応の可視判定）に相当し、
//!   本実装は 1 スライド表示固定のため既存の `data-current` と同値で
//!   出力する（`data-current` は既存呼び出し元・styled recipe が依存する
//!   ため互換のため維持し、`data-inview` は zag 語彙追随の別名として並存
//!   させる）。
//! - **`CarouselAction::First`/`Last`（dispatch `"first"`/`"last"`）を追加**:
//!   zag.js の `indicator-group` keydown は Home/End で先頭/末尾ページへ
//!   直接移動する。`End` は `slide_count` を要し既存の `"goto"` payload
//!   （固定 index）では表現できないため、決定的な専用 action として追加
//!   する（[`crate::slider::Slider`] の `IncrementLarge`/`DecrementLarge`
//!   と同型の先例）。
//!
//! 以下は意図的に参考サイトへ合わせていない（理由付き）:
//!
//! - **`progress-text`/`autoplay-trigger` パーツを追加しない**:
//!   `progress-text`（"X / Y" 表示のみの装飾テキスト）は呼び出し側が通常の
//!   テキストノードで代替でき、`autoplay-trigger` は下記 autoplay 自体が
//!   スコープ外のため対応する trigger も不要。パーツ追加は Themes 側
//!   `SLOTS`/golden CSS/`KNOWN_UNCOVERED` への連鎖を招くため、本イシューでは
//!   見送り Issue 化候補とする。
//! - **`aria-hidden`（非表示スライド）を付与しない**: zag.js は視覚的に隠れた
//!   スライドへ `aria-hidden="true"` を付与するが、本モジュールは CSS を
//!   前提としない SSR 静的マークアップを返すため、非 current スライドを
//!   実際に隠さない構成で `aria-hidden` だけ付けると全スライドが可視のまま
//!   支援技術からのみ隠れる不整合（WCAG 1.3.1 相当）になる。非 current を
//!   CSS で隠す呼び出し側は `attrs` へ `("aria-hidden", "true")` を明示的に
//!   渡す運用とする（原稿の自前 CSS 例参照）。
//! - **`aria-controls`（trigger → item-group）/各パーツの `id`/`dir` を
//!   付与しない**: 本モジュールに `id` 生成機構がなく（他 headless-ui
//!   コンポーネント全体の既存方針）、必要な呼び出し側は `item_group` の
//!   `attrs` に `id`、trigger の `attrs` に `aria-controls` を明示的に渡す。
//! - **`data-dragging`（pointer drag 中）/`data-readonly`（indicator）を
//!   追加しない**: pointer ドラッグ配線・indicator クリック配線がいずれも
//!   本モジュールに存在しない現状では意味を持たない状態語彙のため、DOM
//!   配線実装時に再検討する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Carousel::new`] で index を正規化してから各パーツメソッド
//! （[`Carousel::root`]/[`Carousel::control`]/[`Carousel::prev_trigger`]/
//! [`Carousel::next_trigger`]/[`Carousel::item_group`]/[`Carousel::item`]/
//! [`Carousel::indicator_group`]/[`Carousel::indicator`]）を呼んで組み立てる。
//! CSR/hydration は [`Carousel`] を経由し、dispatch（`"next"`/`"prev"`/
//! `"goto"`/`"first"`/`"last"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! が本モジュールを呼んでスタイル済み Carousel を組み立てる想定である。
//!
//! # キーボード操作（現状の対応範囲）
//!
//! zag.js の `indicator-group` keydown 契約に対応する決定的な dispatch
//! action は状態機械側に揃っている（横向き ArrowRight/ArrowLeft・縦向き
//! ArrowDown/ArrowUp → [`CarouselAction::Next`]/[`CarouselAction::Prev`]、
//! Home/End → [`CarouselAction::First`]/[`CarouselAction::Last`]）。ただし
//! 実際のキーボードイベント配線（`keydown` リスナー登録・`orientation` に
//! 応じたキー選別）は他コンポーネント同様
//! `fandhe-frontend-wasm-full`（クライアントランタイム）側の後続責務であり、
//! 本イシュー（#1660）のスコープ外とする（REQ-11 バンドル予算の再評価が
//! 前提）。trigger/indicator は native `button` のためクリック（Enter/
//! Space）は標準の DOM 挙動でカバーされる。
//!
//! # 決定的な遷移規則（受け入れ条件）
//!
//! - `slide_count == 0` のときはすべてのアクションが no-op（遷移不能な空
//!   carousel を fail-closed に扱う）。
//! - `Next`: 末尾（`index == slide_count - 1`）で `loop_ = true` なら `0` へ
//!   循環、`false` なら no-op（端で停止、[`crate::slider::Slider`] が
//!   `[min, max]` の端で clamp するのと同型の「端に留まる」判断）。
//! - `Prev`: 先頭（`index == 0`）で `loop_ = true` なら `slide_count - 1` へ
//!   循環、`false` なら no-op（[`Next`](CarouselAction::Next) と対称）。
//! - `Goto(i)`: `i >= slide_count` は改ざん入力として fail-closed に無視する
//!   （clamp して最寄りの有効値へ丸めるのではなく no-op。呼び出し側が
//!   意図しない位置へ暗黙に着地させない）。
//! - `First`: `slide_count > 0` のとき常に `0` へ移動する（既に先頭でも
//!   no-op 同然の冪等な代入）。
//! - `Last`: `slide_count > 0` のとき常に `slide_count - 1` へ移動する
//!   （[`First`](CarouselAction::First) と対称）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存不変条件
//!   をそのまま継承する）。
//! - 動的値（[`item`]/[`indicator`] が生成する `aria-label` 文字列・呼び出し側
//!   `aria_label`/`attrs`/children）は [`fandhe_frontend_core::render`] の
//!   既定エスケープを必ず経由する。`raw_html()` は使用せず、HTML 文字列を
//!   直接組み立てない。[`item`]/[`indicator`] が組み立てる `"{n} of {m}"`/
//!   `"Go to slide {n}"` 文字列、および `data-index` の整形値は `usize` の
//!   Display 整形のみから合成し、任意の呼び出し側文字列がこれらへ混入する
//!   経路はない。
//! - 呼び出し側 `attrs` の [`RESERVED`] に列挙したフレームワーク固定キー
//!   （`data-orientation`/`data-index`/`data-inview`/`data-current`/
//!   `data-disabled`/`aria-current`、ASCII 大文字小文字無視）は
//!   [`drop_reserved`] が fail-closed に除外する（[`crate::pin_input`] の
//!   `drop_reserved` と同型のなりすまし防止。偽の位置・状態を注入できない）。
//! - dispatch `"goto"` の payload はクライアント由来の信頼できない入力として
//!   扱い、厳密な `usize` パースで fail-closed（パース不能は `None`、範囲外は
//!   [`Carousel::update`] 側で no-op）。`"first"`/`"last"` は payload を
//!   使用しない。
//! - hydration 属性（`data-hydrate-index`/`-slide-count`/`-loop`/
//!   `-orientation`）はクライアント側で改ざんされうる入力として扱う。
//!   [`Carousel`] の [`fandhe_frontend_interactive::Hydrate`] 実装は panic
//!   せず `HydrateError` を返す（パース不能・範囲外 index・不正な
//!   `loop`/`orientation` 語彙をすべて拒否する。[`crate::slider::Slider`] と
//!   同型の fail-closed 契約）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **autoplay（play/pause/`aria-live` 切替/delay）**: タイマー駆動の
//!   非決定要素を初期実装から排除する。`item_group` の `aria-live` は
//!   常に `"polite"` 固定とする（autoplay 実装時に切替対象となる想定）。
//! - **pointer ドラッグ・キーボード操作（Arrow キー/スワイプ）の DOM 配線**:
//!   他コンポーネント同様、クライアントランタイム
//!   （`fandhe-frontend-wasm-full`）側の後続責務とする。本モジュールは SSR
//!   静的マークアップと dispatch 契約のみを提供する（上記「キーボード操作」
//!   節参照）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, aria_roledescription, role};
use crate::data_attrs::{data_current, data_disabled, data_orientation, Orientation};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Carousel の anatomy（`data-scope="carousel"`）。
const ANATOMY: Anatomy = anatomy("carousel");

/// フレームワークが固定付与する `data-*`/`aria-*` キー一覧
/// （[`drop_reserved`] が呼び出し側 `attrs` から除外する対象。ASCII 大文字
/// 小文字無視、[`crate::pin_input::ROOT_RESERVED`] と同型のパターン）。
/// パーツ間で付与するキーの組が異なる（例: `control`/`indicator-group` は
/// `data-orientation` のみ）が、[`crate::pin_input`] と異なり本モジュールは
/// 単一の合併集合で統一する（キー種別が少なく、パーツ別リストに分けるほどの
/// 誤除外リスクがないため）。
const RESERVED: &[&str] = &[
    "data-orientation",
    "data-index",
    "data-inview",
    "data-current",
    "data-disabled",
    "aria-current",
];

/// 呼び出し側 `attrs` から [`RESERVED`] キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::pin_input::drop_reserved`] と同型の重複実装。
/// モジュール間の相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !RESERVED.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Root パーツ（`div role="region"`）。WAI-ARIA carousel パターンに従い
/// `aria-roledescription="carousel"` を固定出力する。`aria_label` は
/// スクリーンリーダー利用者へ carousel の内容を説明する必須ラベルであり、
/// 呼び出し側は空文字列でなく意味のある文言を渡す責務を負う（本関数は
/// 空文字列を拒否しない。SSR 静的関数として値の妥当性検証までは行わない
/// 判断は [`crate::field`] 等の既存パーツ関数と同型）。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("region"),
        aria_roledescription("carousel"),
        aria_label(label),
        data_orientation(orientation),
    ];
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// Control パーツ（`div`）。[`prev_trigger`]/[`next_trigger`]/[`item_group`]
/// を束ねるコンテナ（装飾的、ARIA 属性を持たない）。zag.js に合わせ
/// `data-orientation` を出力する（イシュー #1660 で全パーツへ拡張）。
#[must_use]
pub fn control<'a>(
    orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("control", "div", merged, children)
}

/// PrevTrigger パーツ（`button type="button"`）。`disabled` が `true` の
/// とき（`loop` 無効かつ先頭スライド）ネイティブ `disabled` +
/// `data-disabled` の対を出力する（[`crate::slider::thumb`] 等と同型の
/// 「端で操作不能」表現）。`aria_label` は既定ラベル
/// （例: `"Previous slide"`）を呼び出し側が渡す（本モジュールは固定英語
/// 文言をハードコードせず、国際化は呼び出し側に委ねる）。`data-orientation`
/// はイシュー #1660 で追加（zag.js 突合）。
#[must_use]
pub fn prev_trigger<'a>(
    orientation: Orientation,
    disabled: bool,
    aria_label_text: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_label(aria_label_text),
        data_orientation(orientation),
    ];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("prev-trigger", "button", merged, children)
}

/// NextTrigger パーツ（`button type="button"`）。[`prev_trigger`] と対称。
#[must_use]
pub fn next_trigger<'a>(
    orientation: Orientation,
    disabled: bool,
    aria_label_text: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_label(aria_label_text),
        data_orientation(orientation),
    ];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("next-trigger", "button", merged, children)
}

/// ItemGroup パーツ（`div`）。`aria-live="polite"` を固定出力する
/// （autoplay 非対応の初期実装ではスライド切替がユーザー操作起点のみで
/// あるため、常に控えめな通知で安全側に倒す。モジュール doc「スコープ外」
/// 節参照）。styled 層（`fandhe-frontend-pre-styled-ui`）が
/// `--fandhe-carousel-index` CSS カスタムプロパティ（[`Carousel::item_group`]
/// が出力する `style`）と `data-orientation`（イシュー #1660 で追加、
/// `item-group[data-orientation="vertical"]` recipe が依存する）を参照して
/// transform ベースのスライド表示を行う。
#[must_use]
pub fn item_group<'a>(
    orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("aria-live", "polite"), data_orientation(orientation)];
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("item-group", "div", merged, children)
}

/// Item パーツ（`div role="group"`）。WAI-ARIA carousel パターンに従い
/// `aria-roledescription="slide"` と `aria-label="{index+1} of {count}"`
/// （1-origin、`usize` の Display 整形のみから合成。モジュール doc
/// 「セキュリティ不変条件」参照）を固定出力する。`current` が `true` のとき
/// [`data_current`] を出力する（イシュー #1660: `data-index`〔0-origin〕・
/// `data-inview`〔本実装は 1 スライド表示固定のため `current` と同値〕・
/// `data-orientation` を追加、zag.js 突合）。
#[must_use]
pub fn item<'a>(
    orientation: Orientation,
    index: usize,
    count: usize,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let label = format!("{} of {}", index + 1, count);
    let index_str = index.to_string();
    let mut merged: Vec<(&str, &str)> = vec![role("group"), aria_roledescription("slide")];
    merged.push(aria_label(label.as_str()));
    merged.push(("data-index", index_str.as_str()));
    merged.push(data_orientation(orientation));
    if current {
        merged.push(("data-inview", ""));
    }
    merged.extend(data_current(current));
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("item", "div", merged, children)
}

/// IndicatorGroup パーツ（`div`）。[`indicator`] 群のコンテナ（装飾的）。
/// `data-orientation` はイシュー #1660 で追加（zag.js 突合）。
#[must_use]
pub fn indicator_group<'a>(
    orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("indicator-group", "div", merged, children)
}

/// Indicator パーツ（`button type="button"`）。`aria-label="Go to slide
/// {index+1}"`（1-origin、[`item`] と同じ整形方針）を固定出力し、`current`
/// が `true` のとき `aria-current="true"` + [`data_current`] を出力する
/// （`aria-current="true"` は zag.js には存在しない超集合、モジュール doc
/// 参照）。イシュー #1660: `data-index`（0-origin）・`data-orientation` を
/// 追加（zag.js 突合）。
#[must_use]
pub fn indicator<'a>(
    orientation: Orientation,
    index: usize,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let label = format!("Go to slide {}", index + 1);
    let index_str = index.to_string();
    let mut merged: Vec<(&str, &str)> = vec![("type", "button")];
    merged.push(aria_label(label.as_str()));
    merged.push(("data-index", index_str.as_str()));
    merged.push(data_orientation(orientation));
    if current {
        merged.push(("aria-current", "true"));
    }
    merged.extend(data_current(current));
    merged.extend(drop_reserved(attrs));
    ANATOMY.part("indicator", "button", merged, vec![])
}

/// Carousel のアクション（WASM 境界の文字列 dispatch と
/// [`Carousel::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselAction {
    /// 次のスライドへ進む（末尾かつ `loop` 無効なら no-op）。
    Next,
    /// 前のスライドへ戻る（先頭かつ `loop` 無効なら no-op）。
    Prev,
    /// 指定した index のスライドへ直接移動する（`index >= slide_count` は
    /// no-op、モジュール doc「決定的な遷移規則」参照）。
    Goto(usize),
    /// 先頭スライドへ移動する（zag.js の Home キー相当、イシュー #1660）。
    /// `slide_count == 0` は他アクション同様 no-op。
    First,
    /// 末尾スライドへ移動する（zag.js の End キー相当、イシュー #1660）。
    /// [`First`](CarouselAction::First) と対称。
    Last,
}

/// `index >= slide_count`（または `slide_count == 0` で `index != 0`）を
/// `0` へ fail-closed に正規化する（[`Carousel::new`]/hydration 復元の共通
/// ヘルパ）。
fn normalize_index(index: usize, slide_count: usize) -> usize {
    if slide_count == 0 || index >= slide_count {
        0
    } else {
        index
    }
}

/// Carousel の index 状態機械（ark-ui 準拠）。
///
/// `Default` は `index=0, slide_count=0, loop=false,
/// orientation=Horizontal`（SSR の初期描画に対応する既定値。スライドを
/// 持たない空 carousel）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Carousel {
    index: usize,
    slide_count: usize,
    loop_: bool,
    orientation: Orientation,
}

impl Default for Carousel {
    fn default() -> Self {
        Self::new(0, 0, false, Orientation::Horizontal)
    }
}

impl Carousel {
    /// `data-hydrate-index` 属性名のフィールド部分。
    pub const FIELD_INDEX: &'static str = "index";
    /// `data-hydrate-slide-count` 属性名のフィールド部分。
    pub const FIELD_SLIDE_COUNT: &'static str = "slide-count";
    /// `data-hydrate-loop` 属性名のフィールド部分。
    pub const FIELD_LOOP: &'static str = "loop";
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";

    /// 指定した状態で [`Carousel`] を生成する（[`normalize_index`] で
    /// fail-closed 正規化する。呼び出し側の不正な `index` で panic しない）。
    #[must_use]
    pub fn new(index: usize, slide_count: usize, loop_: bool, orientation: Orientation) -> Self {
        Self {
            index: normalize_index(index, slide_count),
            slide_count,
            loop_,
            orientation,
        }
    }

    /// 現在のスライド index（`0`-origin）。
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    /// スライド総数。
    #[must_use]
    pub fn slide_count(&self) -> usize {
        self.slide_count
    }

    /// 端で循環するかどうか。
    #[must_use]
    pub fn is_loop(&self) -> bool {
        self.loop_
    }

    /// 現在の向き（`data-orientation`/hydration ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// [`prev_trigger`] を no-op にすべきかどうか（`slide_count == 0`、また
    /// `loop` 無効かつ先頭スライドのいずれか）。
    #[must_use]
    pub fn prev_disabled(&self) -> bool {
        self.slide_count == 0 || (!self.loop_ && self.index == 0)
    }

    /// [`next_trigger`] を no-op にすべきかどうか（[`Self::prev_disabled`]
    /// と対称）。
    #[must_use]
    pub fn next_disabled(&self) -> bool {
        self.slide_count == 0 || (!self.loop_ && self.index + 1 >= self.slide_count)
    }

    /// [`root`] へ現在の向きを注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.orientation, label, attrs, children)
    }

    /// [`control`] へ現在の向きを注入する利便メソッド（状態を持たない
    /// 装飾用パーツ、`data-orientation` はイシュー #1660 で追加）。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(self.orientation, attrs, children)
    }

    /// [`prev_trigger`] へ現在の向きと [`Self::prev_disabled`] の判定を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn prev_trigger<'a>(
        &self,
        aria_label_text: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        prev_trigger(
            self.orientation,
            self.prev_disabled(),
            aria_label_text,
            attrs,
            children,
        )
    }

    /// [`next_trigger`] へ現在の向きと [`Self::next_disabled`] の判定を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn next_trigger<'a>(
        &self,
        aria_label_text: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        next_trigger(
            self.orientation,
            self.next_disabled(),
            aria_label_text,
            attrs,
            children,
        )
    }

    /// [`item_group`] へ現在の index を CSS カスタムプロパティ
    /// （`--fandhe-carousel-index`）として `style` 属性へ注入する利便
    /// メソッド。styled 層（`fandhe-frontend-pre-styled-ui`）はこの変数を
    /// 参照して `transform: translateX/Y(calc(...))` を導出する
    /// （決定的、JS 計測に依存しない）。値は `usize` の Display 整形のみから
    /// 組み立て、任意の呼び出し側文字列がこの `style` 値へ混入する経路は
    /// ない。呼び出し側が `("style", ...)` を渡した場合はフレームワーク側の
    /// 固定 `style` を優先し破棄する（[`crate::progress::Progress::circle`]
    /// と同型の dedup 判断、fail-closed）。
    #[must_use]
    pub fn item_group<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let style = format!("--fandhe-carousel-index: {};", self.index);
        let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
        merged.extend(drop_style_attr(attrs));
        item_group(self.orientation, merged, children)
    }

    /// [`item`] へ現在の向きと状態（`slide_count`・当該 index が現在位置
    /// かどうか）を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.orientation,
            index,
            self.slide_count,
            index == self.index,
            attrs,
            children,
        )
    }

    /// [`indicator_group`] へ現在の向きを注入する利便メソッド（状態を
    /// 持たない装飾用パーツ、`data-orientation` はイシュー #1660 で追加）。
    #[must_use]
    pub fn indicator_group<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        indicator_group(self.orientation, attrs, children)
    }

    /// [`indicator`] へ現在の向きと当該 index が現在位置かどうかを注入する
    /// 利便メソッド。
    #[must_use]
    pub fn indicator<'a>(&self, index: usize, attrs: Vec<(&'a str, &'a str)>) -> Node {
        indicator(self.orientation, index, index == self.index, attrs)
    }
}

/// [`Carousel::item_group`] がフレームワーク側で固定 `style` を先頭に積んだ
/// 後、呼び出し側 `attrs` を連結する前に使う dedup ヘルパ（
/// [`crate::progress`] の同名内部ヘルパと同型の重複実装、モジュール間の
/// 相互依存を避けるため個別定義する）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

impl Component for Carousel {
    type Action = CarouselAction;

    /// `slide_count == 0` はすべてのアクションを no-op にする（モジュール
    /// doc「決定的な遷移規則」参照）。
    fn update(&mut self, action: CarouselAction) {
        if self.slide_count == 0 {
            return;
        }
        match action {
            CarouselAction::Next => {
                if self.index + 1 < self.slide_count {
                    self.index += 1;
                } else if self.loop_ {
                    self.index = 0;
                }
            }
            CarouselAction::Prev => {
                if self.index > 0 {
                    self.index -= 1;
                } else if self.loop_ {
                    self.index = self.slide_count - 1;
                }
            }
            CarouselAction::Goto(i) => {
                if i < self.slide_count {
                    self.index = i;
                }
            }
            CarouselAction::First => {
                self.index = 0;
            }
            CarouselAction::Last => {
                self.index = self.slide_count - 1;
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// control > item-group）。公開 UI としての利用は想定しない。
    fn view(&self) -> Node {
        self.root(
            "carousel",
            Vec::new(),
            vec![self.control(Vec::new(), vec![self.item_group(Vec::new(), Vec::new())])],
        )
    }

    /// `"next"`/`"prev"`/`"first"`/`"last"`: payload 不使用。`"goto"`:
    /// payload を `str::parse::<usize>()` でパースし、パース不能な場合は
    /// `None`（fail-closed、dispatch は no-op）。範囲外 index（`i >=
    /// slide_count`）はここでは弾かず [`Carousel::update`] 側の no-op に
    /// 委ねる（`decode_action` は静的関数で `slide_count` へアクセスできない
    /// ため）。`"first"`/`"last"` は zag.js の Home/End キー相当
    /// （イシュー #1660、モジュール doc「キーボード操作」節参照）。
    fn decode_action(name: &str, payload: &str) -> Option<CarouselAction> {
        match name {
            "next" => Some(CarouselAction::Next),
            "prev" => Some(CarouselAction::Prev),
            "goto" => payload.parse::<usize>().ok().map(CarouselAction::Goto),
            "first" => Some(CarouselAction::First),
            "last" => Some(CarouselAction::Last),
            _ => None,
        }
    }
}

impl Hydrate for Carousel {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INDEX),
                self.index.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SLIDE_COUNT),
                self.slide_count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_LOOP),
                self.loop_.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION),
                self.orientation.as_str().to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・範囲外 index・不正な
    /// `loop`/`orientation` 語彙は [`HydrateError::InvalidValue`]（panic
    /// しない。[`crate::slider::Slider`] と同型の fail-closed 契約）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let index_raw = find(Self::FIELD_INDEX)?;
        let slide_count_raw = find(Self::FIELD_SLIDE_COUNT)?;
        let loop_raw = find(Self::FIELD_LOOP)?;
        let orientation_raw = find(Self::FIELD_ORIENTATION)?;

        let attr_name_slide_count = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SLIDE_COUNT);
        let slide_count =
            slide_count_raw
                .parse::<usize>()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: attr_name_slide_count,
                    reason: "expected a non-negative integer".to_string(),
                })?;

        let attr_name_index = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INDEX);
        let index = index_raw
            .parse::<usize>()
            .map_err(|_| HydrateError::InvalidValue {
                attr: attr_name_index.clone(),
                reason: "expected a non-negative integer".to_string(),
            })?;
        if slide_count == 0 {
            if index != 0 {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_index,
                    reason: "expected index == 0 when slide_count == 0".to_string(),
                });
            }
        } else if index >= slide_count {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_index,
                reason: "expected index within [0, slide_count)".to_string(),
            });
        }

        let attr_name_loop = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_LOOP);
        let loop_ = match loop_raw {
            "true" => true,
            "false" => false,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_loop,
                    reason: "expected \"true\" or \"false\"".to_string(),
                })
            }
        };

        let attr_name_orientation = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION);
        let orientation = match orientation_raw {
            "horizontal" => Orientation::Horizontal,
            "vertical" => Orientation::Vertical,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_orientation,
                    reason: "expected \"horizontal\" or \"vertical\"".to_string(),
                })
            }
        };

        Ok(Self {
            index,
            slide_count,
            loop_,
            orientation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/ARIA/data-* 出力 ---

    #[test]
    fn root_outputs_region_role_and_roledescription_and_label() {
        let html = render(&root(
            Orientation::Horizontal,
            "Featured products",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="carousel""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="region""#));
        assert!(html.contains(r#"aria-roledescription="carousel""#));
        assert!(html.contains(r#"aria-label="Featured products""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn control_outputs_scope_and_part_only() {
        let html = render(&control(Orientation::Horizontal, vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn prev_trigger_not_disabled_outputs_type_and_label() {
        let html = render(&prev_trigger(
            Orientation::Horizontal,
            false,
            "Previous slide",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="prev-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Previous slide""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn prev_trigger_disabled_true_adds_disabled_and_data_disabled() {
        let html = render(&prev_trigger(
            Orientation::Horizontal,
            true,
            "Previous slide",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn next_trigger_mirrors_prev_trigger() {
        let html = render(&next_trigger(
            Orientation::Horizontal,
            false,
            "Next slide",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="next-trigger""#));
        assert!(html.contains(r#"aria-label="Next slide""#));

        let disabled = render(&next_trigger(
            Orientation::Horizontal,
            true,
            "Next slide",
            vec![],
            vec![],
        ));
        assert!(disabled.contains(r#"disabled="""#));
        assert!(disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_group_outputs_aria_live_polite() {
        let html = render(&item_group(Orientation::Horizontal, vec![], vec![]));
        assert!(html.contains(r#"data-part="item-group""#));
        assert!(html.contains(r#"aria-live="polite""#));
    }

    #[test]
    fn item_outputs_role_roledescription_and_positional_label() {
        let html = render(&item(
            Orientation::Horizontal,
            0,
            3,
            false,
            vec![],
            vec![text("Slide A")],
        ));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-roledescription="slide""#));
        assert!(html.contains(r#"aria-label="1 of 3""#));
        assert!(!html.contains("data-current"));
        assert!(html.contains("Slide A"));
    }

    #[test]
    fn item_current_true_adds_data_current() {
        let html = render(&item(Orientation::Horizontal, 1, 3, true, vec![], vec![]));
        assert!(html.contains(r#"aria-label="2 of 3""#));
        assert!(html.contains(r#"data-current="""#));
    }

    #[test]
    fn indicator_group_outputs_scope_and_part_only() {
        let html = render(&indicator_group(Orientation::Horizontal, vec![], vec![]));
        assert!(html.contains(r#"data-part="indicator-group""#));
    }

    #[test]
    fn indicator_not_current_outputs_type_and_label_without_aria_current() {
        let html = render(&indicator(Orientation::Horizontal, 0, false, vec![]));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Go to slide 1""#));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    #[test]
    fn indicator_current_true_adds_aria_current_and_data_current() {
        let html = render(&indicator(Orientation::Horizontal, 2, true, vec![]));
        assert!(html.contains(r#"aria-label="Go to slide 3""#));
        assert!(html.contains(r#"aria-current="true""#));
        assert!(html.contains(r#"data-current="""#));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            Orientation::Horizontal,
            "Products",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="carousel""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_normalizes_out_of_range_index_to_zero() {
        let c = Carousel::new(5, 3, false, Orientation::Horizontal);
        assert_eq!(c.index(), 0);
    }

    #[test]
    fn new_zero_slide_count_forces_index_zero() {
        let c = Carousel::new(2, 0, true, Orientation::Horizontal);
        assert_eq!(c.index(), 0);
        assert_eq!(c.slide_count(), 0);
    }

    #[test]
    fn default_is_empty_carousel() {
        let c = Carousel::default();
        assert_eq!(c.index(), 0);
        assert_eq!(c.slide_count(), 0);
        assert!(!c.is_loop());
        assert_eq!(c.orientation(), Orientation::Horizontal);
    }

    // --- prev/next disabled 判定 ---

    #[test]
    fn prev_next_disabled_reflect_loop_and_edges() {
        let c = Carousel::new(0, 3, false, Orientation::Horizontal);
        assert!(c.prev_disabled());
        assert!(!c.next_disabled());

        let c = Carousel::new(2, 3, false, Orientation::Horizontal);
        assert!(!c.prev_disabled());
        assert!(c.next_disabled());

        let c = Carousel::new(0, 3, true, Orientation::Horizontal);
        assert!(!c.prev_disabled());
        let c = Carousel::new(2, 3, true, Orientation::Horizontal);
        assert!(!c.next_disabled());
    }

    #[test]
    fn prev_next_disabled_true_when_slide_count_zero_regardless_of_loop() {
        let c = Carousel::new(0, 0, true, Orientation::Horizontal);
        assert!(c.prev_disabled());
        assert!(c.next_disabled());
    }

    // --- dispatch 統合: 決定的な遷移規則 ---

    #[test]
    fn dispatch_next_advances_and_stops_at_end_without_loop() {
        let mut c = Carousel::new(0, 3, false, Orientation::Horizontal);
        assert!(dispatch(&mut c, "next", ""));
        assert_eq!(c.index(), 1);
        assert!(dispatch(&mut c, "next", ""));
        assert_eq!(c.index(), 2);
        assert!(dispatch(&mut c, "next", ""));
        assert_eq!(c.index(), 2, "loop 無効時は末尾で停止する");
    }

    #[test]
    fn dispatch_next_wraps_to_zero_at_end_with_loop() {
        let mut c = Carousel::new(2, 3, true, Orientation::Horizontal);
        assert!(dispatch(&mut c, "next", ""));
        assert_eq!(c.index(), 0);
    }

    #[test]
    fn dispatch_prev_retreats_and_stops_at_start_without_loop() {
        let mut c = Carousel::new(2, 3, false, Orientation::Horizontal);
        assert!(dispatch(&mut c, "prev", ""));
        assert_eq!(c.index(), 1);
        assert!(dispatch(&mut c, "prev", ""));
        assert_eq!(c.index(), 0);
        assert!(dispatch(&mut c, "prev", ""));
        assert_eq!(c.index(), 0, "loop 無効時は先頭で停止する");
    }

    #[test]
    fn dispatch_prev_wraps_to_end_at_start_with_loop() {
        let mut c = Carousel::new(0, 3, true, Orientation::Horizontal);
        assert!(dispatch(&mut c, "prev", ""));
        assert_eq!(c.index(), 2);
    }

    #[test]
    fn dispatch_goto_moves_to_valid_index() {
        let mut c = Carousel::new(0, 5, false, Orientation::Horizontal);
        assert!(dispatch(&mut c, "goto", "3"));
        assert_eq!(c.index(), 3);
    }

    #[test]
    fn dispatch_goto_out_of_range_is_noop() {
        // `"goto"` は decode_action で有効な action として認識される
        // （dispatch は true を返す）が、`update()` 内で範囲外 index を
        // fail-closed に無視するため `index` 自体は変化しない（[`Slider`]
        // の `dispatch_increment_clamps_at_max` と同型の「認識はされるが
        // 状態は変化しない」ケース、モジュール doc「決定的な遷移規則」参照）。
        let mut c = Carousel::new(1, 5, false, Orientation::Horizontal);
        assert!(dispatch(&mut c, "goto", "5"));
        assert_eq!(c.index(), 1);
        assert!(dispatch(&mut c, "goto", "999"));
        assert_eq!(c.index(), 1);
    }

    #[test]
    fn dispatch_goto_rejects_invalid_payload() {
        let mut c = Carousel::new(1, 5, false, Orientation::Horizontal);
        for bogus in ["abc", "-1", "1.5", ""] {
            assert!(!dispatch(&mut c, "goto", bogus));
            assert_eq!(c.index(), 1);
        }
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut c = Carousel::new(1, 3, false, Orientation::Horizontal);
        assert!(!dispatch(&mut c, "no_such_action", "x"));
        assert_eq!(c.index(), 1);
    }

    #[test]
    fn slide_count_zero_makes_all_actions_noop() {
        // `"next"`/`"prev"`/`"goto"` はいずれも有効な action として認識
        // される（dispatch は true）が、`slide_count == 0` のため
        // `update()` 冒頭の早期 return で `index` は変化しない
        // （[`dispatch_goto_out_of_range_is_noop`] と同型）。
        let mut c = Carousel::default();
        assert!(dispatch(&mut c, "next", ""));
        assert!(dispatch(&mut c, "prev", ""));
        assert!(dispatch(&mut c, "goto", "0"));
        assert_eq!(c.index(), 0);
    }

    // --- 利便メソッド ---

    #[test]
    fn convenience_item_group_outputs_css_var_for_current_index() {
        let c = Carousel::new(2, 5, false, Orientation::Horizontal);
        let html = render(&c.item_group(vec![], vec![]));
        assert!(html.contains("--fandhe-carousel-index: 2;"));
    }

    #[test]
    fn convenience_item_group_drops_caller_supplied_style() {
        let c = Carousel::new(1, 3, false, Orientation::Horizontal);
        let html = render(&c.item_group(vec![("style", "color: red")], vec![]));
        assert!(!html.contains("color: red"));
        assert!(html.contains("--fandhe-carousel-index: 1;"));
        assert_eq!(html.matches("style=").count(), 1);
    }

    #[test]
    fn convenience_item_reflects_current_position() {
        let c = Carousel::new(1, 3, false, Orientation::Horizontal);
        let current = render(&c.item(1, vec![], vec![]));
        assert!(current.contains(r#"data-current="""#));

        let not_current = render(&c.item(0, vec![], vec![]));
        assert!(!not_current.contains("data-current"));
    }

    #[test]
    fn convenience_indicator_reflects_current_position() {
        let c = Carousel::new(2, 4, false, Orientation::Horizontal);
        let current = render(&c.indicator(2, vec![]));
        assert!(current.contains(r#"aria-current="true""#));

        let not_current = render(&c.indicator(0, vec![]));
        assert!(!not_current.contains("aria-current"));
    }

    #[test]
    fn convenience_prev_next_trigger_reflect_disabled_state() {
        let c = Carousel::new(0, 3, false, Orientation::Horizontal);
        let prev = render(&c.prev_trigger("Previous", vec![], vec![]));
        assert!(prev.contains(r#"disabled="""#));
        let next = render(&c.next_trigger("Next", vec![], vec![]));
        assert!(!next.contains("disabled"));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Carousel::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let c = Carousel::new(2, 5, true, Orientation::Horizontal);
        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains(r#"data-hydrate-index="2""#));
        assert!(rendered.contains(r#"data-hydrate-slide-count="5""#));
        assert!(rendered.contains(r#"data-hydrate-loop="true""#));
        assert!(rendered.contains(r#"data-hydrate-orientation="horizontal""#));

        let restored = Carousel::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    #[test]
    fn hydration_round_trip_vertical_without_loop() {
        let c = Carousel::new(0, 3, false, Orientation::Vertical);
        let restored = Carousel::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
        assert_eq!(restored.orientation(), Orientation::Vertical);
        assert!(!restored.is_loop());
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Carousel::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-index".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // index が範囲外。
            vec![
                ("data-hydrate-index".to_string(), "5".to_string()),
                ("data-hydrate-slide-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // slide_count がパース不能。
            vec![
                ("data-hydrate-index".to_string(), "0".to_string()),
                ("data-hydrate-slide-count".to_string(), "abc".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // loop が未知の値。
            vec![
                ("data-hydrate-index".to_string(), "0".to_string()),
                ("data-hydrate-slide-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "yes".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // orientation が未知の値。
            vec![
                ("data-hydrate-index".to_string(), "0".to_string()),
                ("data-hydrate-slide-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "diagonal".to_string(),
                ),
            ],
            // slide_count == 0 なのに index != 0。
            vec![
                ("data-hydrate-index".to_string(), "1".to_string()),
                ("data-hydrate-slide-count".to_string(), "0".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // index が XSS ペイロード。
            vec![
                (
                    "data-hydrate-index".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                ("data-hydrate-slide-count".to_string(), "3".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
        ];
        for attrs in bogus_sets {
            let err = Carousel::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: aria_label/attrs/children/hydration にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_aria_label_payload_is_escaped_on_render() {
        let html = render(&root(
            Orientation::Horizontal,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn prev_trigger_aria_label_payload_is_escaped_on_render() {
        let html = render(&prev_trigger(
            Orientation::Horizontal,
            false,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&control(
            Orientation::Horizontal,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item(
            Orientation::Horizontal,
            0,
            1,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_xss_payload_in_index_is_rejected_not_rendered() {
        let attrs = vec![
            (
                "data-hydrate-index".to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
            ("data-hydrate-slide-count".to_string(), "3".to_string()),
            ("data-hydrate-loop".to_string(), "false".to_string()),
            (
                "data-hydrate-orientation".to_string(),
                "horizontal".to_string(),
            ),
        ];
        let err = Carousel::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- 参照突合（イシュー #1660）: data-orientation 全パーツ ---

    #[test]
    fn all_parts_output_data_orientation() {
        let vertical = Orientation::Vertical;
        assert!(render(&root(vertical, "Products", vec![], vec![]))
            .contains(r#"data-orientation="vertical""#));
        assert!(
            render(&control(vertical, vec![], vec![])).contains(r#"data-orientation="vertical""#)
        );
        assert!(
            render(&prev_trigger(vertical, false, "Previous", vec![], vec![]))
                .contains(r#"data-orientation="vertical""#)
        );
        assert!(
            render(&next_trigger(vertical, false, "Next", vec![], vec![]))
                .contains(r#"data-orientation="vertical""#)
        );
        assert!(render(&item_group(vertical, vec![], vec![]))
            .contains(r#"data-orientation="vertical""#));
        assert!(render(&item(vertical, 0, 3, false, vec![], vec![]))
            .contains(r#"data-orientation="vertical""#));
        assert!(render(&indicator_group(vertical, vec![], vec![]))
            .contains(r#"data-orientation="vertical""#));
        assert!(render(&indicator(vertical, 0, false, vec![]))
            .contains(r#"data-orientation="vertical""#));
    }

    // --- 参照突合（イシュー #1660）: data-index / data-inview ---

    #[test]
    fn item_outputs_data_index_and_data_inview_when_current() {
        let current = render(&item(Orientation::Horizontal, 2, 5, true, vec![], vec![]));
        assert!(current.contains(r#"data-index="2""#));
        assert!(current.contains(r#"data-inview="""#));

        let not_current = render(&item(Orientation::Horizontal, 2, 5, false, vec![], vec![]));
        assert!(not_current.contains(r#"data-index="2""#));
        assert!(!not_current.contains("data-inview"));
    }

    #[test]
    fn indicator_outputs_data_index() {
        let html = render(&indicator(Orientation::Horizontal, 4, false, vec![]));
        assert!(html.contains(r#"data-index="4""#));
    }

    // --- 参照突合（イシュー #1660）: RESERVED キーのなりすまし拒否 ---

    #[test]
    fn drop_reserved_rejects_spoofed_framework_keys() {
        let html = render(&item(
            Orientation::Horizontal,
            1,
            3,
            false,
            vec![
                ("data-orientation", "vertical"),
                ("data-index", "999"),
                ("data-inview", ""),
                ("data-current", ""),
                ("aria-current", "true"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"data-index="1""#));
        assert!(!html.contains("999"));
        assert!(!html.contains("data-inview"));
        assert!(!html.contains("data-current"));
        assert!(!html.contains("aria-current"));
    }

    #[test]
    fn drop_reserved_is_case_insensitive() {
        let html = render(&control(
            Orientation::Horizontal,
            vec![("DATA-ORIENTATION", "vertical")],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(!html.contains("vertical"));
    }

    // --- 参照突合（イシュー #1660）: First/Last dispatch ---

    #[test]
    fn dispatch_first_moves_to_zero() {
        let mut c = Carousel::new(3, 5, false, Orientation::Horizontal);
        assert!(dispatch(&mut c, "first", ""));
        assert_eq!(c.index(), 0);
    }

    #[test]
    fn dispatch_last_moves_to_slide_count_minus_one() {
        let mut c = Carousel::new(0, 5, false, Orientation::Horizontal);
        assert!(dispatch(&mut c, "last", ""));
        assert_eq!(c.index(), 4);
    }

    #[test]
    fn dispatch_first_last_noop_when_slide_count_zero() {
        let mut c = Carousel::default();
        assert!(dispatch(&mut c, "first", ""));
        assert!(dispatch(&mut c, "last", ""));
        assert_eq!(c.index(), 0);
    }

    // --- 参照突合（イシュー #1660）: 意図的非追随（aria-hidden/aria-controls/id/dir） ---

    #[test]
    fn item_does_not_output_aria_hidden_id_or_dir() {
        let html = render(&item(Orientation::Horizontal, 0, 3, false, vec![], vec![]));
        assert!(!html.contains("aria-hidden"));
        assert!(!html.contains(" id="));
        assert!(!html.contains(" dir="));
    }

    #[test]
    fn prev_trigger_does_not_output_aria_controls() {
        let html = render(&prev_trigger(
            Orientation::Horizontal,
            false,
            "Previous",
            vec![],
            vec![],
        ));
        assert!(!html.contains("aria-controls"));
    }
}
