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
//! # `state` 未使用の理由
//!
//! [`crate::state::Disclosure`]/[`crate::state::SingleSelect`] 等の既存共通
//! 状態機械はいずれも「開閉」「単一/複数選択」を表現するものであり、
//! Carousel の「`0..slide_count` 上を循環し得る現在位置」という値状態を
//! 表現できない。[`crate::slider::Slider`]/[`crate::number_input::NumberInput`]
//! と同じ判断で、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を本モジュール内で直接実装する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Carousel::new`] で index を正規化してから各パーツメソッド
//! （[`Carousel::root`]/[`Carousel::control`]/[`Carousel::prev_trigger`]/
//! [`Carousel::next_trigger`]/[`Carousel::item_group`]/[`Carousel::item`]/
//! [`Carousel::indicator_group`]/[`Carousel::indicator`]）を呼んで組み立てる。
//! CSR/hydration は [`Carousel`] を経由し、dispatch（`"next"`/`"prev"`/
//! `"goto"`）で状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを
//! 呼んでスタイル済み Carousel を組み立てる想定である。
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
//!   `"Go to slide {n}"` 文字列は `usize` の Display 整形のみから合成し、
//!   任意の呼び出し側文字列がこの `aria-label` へ混入する経路はない。
//! - dispatch `"goto"` の payload はクライアント由来の信頼できない入力として
//!   扱い、厳密な `usize` パースで fail-closed（パース不能は `None`、範囲外は
//!   [`Carousel::update`] 側で no-op）。
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
//!   静的マークアップと dispatch 契約のみを提供する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, aria_roledescription, role};
use crate::data_attrs::{data_current, data_disabled, data_orientation, Orientation};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Carousel の anatomy（`data-scope="carousel"`）。
const ANATOMY: Anatomy = anatomy("carousel");

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
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Control パーツ（`div`）。[`prev_trigger`]/[`next_trigger`]/[`item_group`]
/// を束ねるコンテナ（装飾的、ARIA 属性を持たない）。
#[must_use]
pub fn control<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("control", "div", attrs, children)
}

/// PrevTrigger パーツ（`button type="button"`）。`disabled` が `true` の
/// とき（`loop` 無効かつ先頭スライド）ネイティブ `disabled` +
/// `data-disabled` の対を出力する（[`crate::slider::thumb`] 等と同型の
/// 「端で操作不能」表現）。`aria_label` は既定ラベル
/// （例: `"Previous slide"`）を呼び出し側が渡す（本モジュールは固定英語
/// 文言をハードコードせず、国際化は呼び出し側に委ねる）。
#[must_use]
pub fn prev_trigger<'a>(
    disabled: bool,
    aria_label_text: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button"), aria_label(aria_label_text)];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("prev-trigger", "button", merged, children)
}

/// NextTrigger パーツ（`button type="button"`）。[`prev_trigger`] と対称。
#[must_use]
pub fn next_trigger<'a>(
    disabled: bool,
    aria_label_text: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button"), aria_label(aria_label_text)];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("next-trigger", "button", merged, children)
}

/// ItemGroup パーツ（`div`）。`aria-live="polite"` を固定出力する
/// （autoplay 非対応の初期実装ではスライド切替がユーザー操作起点のみで
/// あるため、常に控えめな通知で安全側に倒す。モジュール doc「スコープ外」
/// 節参照）。styled 層（`fandhe-frontend-pre-styled-ui`）が
/// `--fandhe-carousel-index` CSS カスタムプロパティ（[`Carousel::item_group`]
/// が出力する `style`）を参照して transform ベースのスライド表示を行う。
#[must_use]
pub fn item_group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("aria-live", "polite")];
    merged.extend(attrs);
    ANATOMY.part("item-group", "div", merged, children)
}

/// Item パーツ（`div role="group"`）。WAI-ARIA carousel パターンに従い
/// `aria-roledescription="slide"` と `aria-label="{index+1} of {count}"`
/// （1-origin、`usize` の Display 整形のみから合成。モジュール doc
/// 「セキュリティ不変条件」参照）を固定出力する。`current` が `true` のとき
/// [`data_current`] を出力する。
#[must_use]
pub fn item<'a>(
    index: usize,
    count: usize,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let label = format!("{} of {}", index + 1, count);
    let mut merged: Vec<(&str, &str)> = vec![role("group"), aria_roledescription("slide")];
    merged.push(aria_label(label.as_str()));
    merged.extend(data_current(current));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// IndicatorGroup パーツ（`div`）。[`indicator`] 群のコンテナ（装飾的）。
#[must_use]
pub fn indicator_group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("indicator-group", "div", attrs, children)
}

/// Indicator パーツ（`button type="button"`）。`aria-label="Go to slide
/// {index+1}"`（1-origin、[`item`] と同じ整形方針）を固定出力し、`current`
/// が `true` のとき `aria-current="true"` + [`data_current`] を出力する。
#[must_use]
pub fn indicator<'a>(index: usize, current: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let label = format!("Go to slide {}", index + 1);
    let mut merged: Vec<(&str, &str)> = vec![("type", "button")];
    merged.push(aria_label(label.as_str()));
    if current {
        merged.push(("aria-current", "true"));
    }
    merged.extend(data_current(current));
    merged.extend(attrs);
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

    /// [`control`] へ委譲する利便メソッド（状態を持たない装飾用パーツ）。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(attrs, children)
    }

    /// [`prev_trigger`] へ [`Self::prev_disabled`] の判定を注入する利便
    /// メソッド。
    #[must_use]
    pub fn prev_trigger<'a>(
        &self,
        aria_label_text: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        prev_trigger(self.prev_disabled(), aria_label_text, attrs, children)
    }

    /// [`next_trigger`] へ [`Self::next_disabled`] の判定を注入する利便
    /// メソッド。
    #[must_use]
    pub fn next_trigger<'a>(
        &self,
        aria_label_text: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        next_trigger(self.next_disabled(), aria_label_text, attrs, children)
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
        item_group(merged, children)
    }

    /// [`item`] へ現在の状態（`slide_count`・当該 index が現在位置かどうか）
    /// を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            index,
            self.slide_count,
            index == self.index,
            attrs,
            children,
        )
    }

    /// [`indicator_group`] へ委譲する利便メソッド（状態を持たない装飾用
    /// パーツ）。
    #[must_use]
    pub fn indicator_group<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        indicator_group(attrs, children)
    }

    /// [`indicator`] へ当該 index が現在位置かどうかを注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(&self, index: usize, attrs: Vec<(&'a str, &'a str)>) -> Node {
        indicator(index, index == self.index, attrs)
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

    /// `"next"`/`"prev"`: payload 不使用。`"goto"`: payload を
    /// `str::parse::<usize>()` でパースし、パース不能な場合は `None`
    /// （fail-closed、dispatch は no-op）。範囲外 index（`i >=
    /// slide_count`）はここでは弾かず [`Carousel::update`] 側の no-op に
    /// 委ねる（`decode_action` は静的関数で `slide_count` へアクセスできない
    /// ため）。
    fn decode_action(name: &str, payload: &str) -> Option<CarouselAction> {
        match name {
            "next" => Some(CarouselAction::Next),
            "prev" => Some(CarouselAction::Prev),
            "goto" => payload.parse::<usize>().ok().map(CarouselAction::Goto),
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
        let html = render(&control(vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn prev_trigger_not_disabled_outputs_type_and_label() {
        let html = render(&prev_trigger(false, "Previous slide", vec![], vec![]));
        assert!(html.contains(r#"data-part="prev-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Previous slide""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn prev_trigger_disabled_true_adds_disabled_and_data_disabled() {
        let html = render(&prev_trigger(true, "Previous slide", vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn next_trigger_mirrors_prev_trigger() {
        let html = render(&next_trigger(false, "Next slide", vec![], vec![]));
        assert!(html.contains(r#"data-part="next-trigger""#));
        assert!(html.contains(r#"aria-label="Next slide""#));

        let disabled = render(&next_trigger(true, "Next slide", vec![], vec![]));
        assert!(disabled.contains(r#"disabled="""#));
        assert!(disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_group_outputs_aria_live_polite() {
        let html = render(&item_group(vec![], vec![]));
        assert!(html.contains(r#"data-part="item-group""#));
        assert!(html.contains(r#"aria-live="polite""#));
    }

    #[test]
    fn item_outputs_role_roledescription_and_positional_label() {
        let html = render(&item(0, 3, false, vec![], vec![text("Slide A")]));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-roledescription="slide""#));
        assert!(html.contains(r#"aria-label="1 of 3""#));
        assert!(!html.contains("data-current"));
        assert!(html.contains("Slide A"));
    }

    #[test]
    fn item_current_true_adds_data_current() {
        let html = render(&item(1, 3, true, vec![], vec![]));
        assert!(html.contains(r#"aria-label="2 of 3""#));
        assert!(html.contains(r#"data-current="""#));
    }

    #[test]
    fn indicator_group_outputs_scope_and_part_only() {
        let html = render(&indicator_group(vec![], vec![]));
        assert!(html.contains(r#"data-part="indicator-group""#));
    }

    #[test]
    fn indicator_not_current_outputs_type_and_label_without_aria_current() {
        let html = render(&indicator(0, false, vec![]));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Go to slide 1""#));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    #[test]
    fn indicator_current_true_adds_aria_current_and_data_current() {
        let html = render(&indicator(2, true, vec![]));
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
        let html = render(&prev_trigger(false, ATTR_BREAK_PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&control(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item(
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
}
