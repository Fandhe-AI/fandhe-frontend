//! Carousel の Motion+ Carousel 相当拡張（opt-in、イシュー #2541、
//! `docs/design/motion-reference-adoption-policy.md` §4 Carousel 行）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §9「ライセンス・転記
//! 制限」に従い、Motion+（購入者限定資料）の意匠から着想した表現を
//! Rust/CSS で独自に再実装する（コード片の逐語コピーはしない）。
//!
//! - **coverflow 3D**: A 群（CSS のみ）。[`COVERFLOW_ATTR`] を
//!   [`crate::carousel::root`] へ付与し、各 `item` へ
//!   [`crate::recipe::stagger_index_style`]（既存 `--fandhe-motion-
//!   stagger-index`、著者が並び順を書き込む）を付与すると 3D に並ぶ
//!   （`item`/`item-group` 自身へクラスを付ける必要はない。`root` 直下の
//!   子孫を descendant セレクタで狙う——[`CAROUSEL_DRAG_ATTR`] と同型の
//!   マーカー付与パターン）。
//! - **ドラッグ + spring スナップ**: C 群（フレームループ必須）。DOM 配線は
//!   `fandhe-frontend-wasm-full::carousel_motion`（`fandhe-frontend-
//!   animation::carousel::CarouselTrack` を消費）が担う。本モジュールは
//!   マークアップ属性・CSS のみを供給する。
//!
//! # `to_css()` 本体を変更しない理由
//!
//! [`crate::button_motion`] モジュール doc「`to_css()` 本体を変更しない
//! 理由」節と同じ契約: 既存 [`crate::carousel`] の `recipe()`/`stylesheet()`
//! 本体は一切変更せず、[`Theme::to_css_with_carousel_coverflow`] を別 impl
//! ブロックとして追加し [`carousel_motion_css()`] を追記するだけの opt-in
//! メソッドにする（pure append）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::button_motion`] モジュール doc「styled 部品の公開 CSS 関数を
//! 持たない」節と同じ理由（`stylesheet.rs` の走査対象外にするため）、
//! 本モジュールも `pub fn` の後ろに `css`/`stylesheet` を空引数で公開する
//! シグネチャを持たない（[`carousel_motion_css()`] は引数なしだが名前が
//! `css`/`stylesheet` と一致しないため走査対象にならない）。
//!
//! # `data-*` 属性の命名（他クレートとの契約）
//!
//! [`CAROUSEL_DRAG_ATTR`]/[`CAROUSEL_DRAGGING_STATE_ATTR`] のリテラル値は
//! `fandhe_frontend_wasm_full::carousel_motion` の同名定数と一致すること
//! が前提（[`crate::button_motion`] と同型のリテラルの写し）。ドリフトは
//! `crates/pre-styled-ui/tests/motion_carousel_css.rs` が wasm-full 側
//! ソースを読んで fail-closed に検知する。
//!
//! # reduced-motion（WCAG 2.3.3）
//!
//! coverflow の 3D transform は既存 `item-group`/`item` の
//! `transition-duration` トークン（`--fandhe-carousel-transition-duration`
//! → `--fandhe-motion-duration-normal`）を継承するため、
//! `Theme::to_css` 側の `prefers-reduced-motion: reduce` 一括ゼロ化
//! （既存機構）がそのまま効き、個別の `@media` は不要。ドラッグ + spring
//! スナップは `fandhe_frontend_animation::reduced_motion::
//! prefers_reduced_motion` を配線層が照会し、有効時は spring を使わず
//! 即時に整数値を書く（アニメーションさせない）。

/// opt-in（著者が SSR 出力に静的に付与）: root へ付与すると coverflow 3D
/// 表示になるマーカー属性。[`CAROUSEL_DRAG_ATTR`] と同じ「root にだけ
/// 付与し、CSS 側は descendant セレクタで `item-group`/`item` を狙う」
/// パターンを採る（codex-review/Cursor Bugbot 指摘 是正、イシュー #2541:
/// 当初はクラスとして `item-group`/`item` 自身にも付与する契約にして
/// いたが、`crate::carousel::root` は呼び出し側の `class` 属性を
/// `crate::class_attr::drop_class_attr` で破棄する契約〔`root` rustdoc
/// 「パーツ」節参照〕のため、クラスを root へ付与する経路が構造的に
/// 存在しなかった。データ属性はこの破棄経路の対象外〔`drop_class_attr`
/// は `class` キーのみを狙う〕であり、[`CAROUSEL_DRAG_ATTR`] は同じ形で
/// 既に機能しているため、そちらへ合わせる）。
pub const COVERFLOW_ATTR: &str = "data-fandhe-carousel-coverflow";

/// opt-in（著者が SSR 出力に静的に付与）: root へ付与するとドラッグ +
/// spring スナップを有効化するマーカー属性。値は `""`（非 loop）または
/// `"loop"`（末尾からの折り返し）のみを許可する
/// （`fandhe_frontend_wasm_full::carousel_motion::parse_loop_opt_in` が
/// 解釈する契約）。
pub const CAROUSEL_DRAG_ATTR: &str = "data-fandhe-carousel-drag";

/// 状態属性: ドラッグ中〜spring 収束完了まで root へ付与される。この間
/// [`CAROUSEL_SPRING_SNAP_CSS`] が `item-group` の `transition` を
/// `none` にし、JS（`CarouselTrack`）が毎フレーム書く値と CSS
/// `transition` の競合を防ぐ。
pub const CAROUSEL_DRAGGING_STATE_ATTR: &str = "data-fandhe-carousel-dragging";

/// [`CAROUSEL_COVERFLOW_CSS`]/[`CAROUSEL_SPRING_SNAP_CSS`] 共通の入れ子
/// carousel 除外ガード（codex-review/Cursor Bugbot 指摘 是正 P1、イシュー
/// #2541 第 5 ラウンド）。
///
/// 素朴な子孫セレクタ `[data-fandhe-carousel-coverflow] [data-scope=
/// "carousel"][data-part="item-group"]` は、外側 carousel だけへ
/// coverflow/drag を opt-in した状態で、その `item` の中に**別の**
/// （opt-in していない）carousel が丸ごと入れ子で存在する構成において、
/// 内側 carousel の `item-group`/`item` にも一致してしまう（`root`/
/// `item-group`/`item` は `[data-scope="carousel"]` を共有する anatomy
/// のため、CSS の子孫結合子は入れ子の境界を区別できない）。内側が opt-in
/// していないのに coverflow の絶対配置・transform や drag 用
/// `touch-action`/`transition: none` を受け取ると、内側 carousel の表示が
/// 壊れる（Cursor Bugbot 指摘の症状「item-group まで absolute 配置・
/// transform 解除」・「touch-action/transition の漏れ」）。
///
/// 是正として、`[data-scope="carousel"][data-part="root"]`
/// （`crates/headless-ui/src/carousel.rs::root` が必ず出力する anatomy
/// root）のうち **opt-in 属性を持たないもの**を挟んで item-group/item に
/// 到達する経路を `:not()` で除外する。外側 root（opt-in 属性あり）から
/// 内側 root（opt-in 属性なし）を経て内側 item-group/item に至る経路は
/// この `:not()` の内側セレクタに一致するため除外され、外側自身の
/// item-group/item（間に opt-in なし root を挟まない）だけが残る。
/// 内側 carousel 自身にも同じ opt-in 属性が付与されていれば、内側は
/// 自分自身の opt-in root からの子孫として独立に一致し続けるため、
/// 深さに関わらず両者とも意図どおりに表示される。`concat!` はリテラル
/// トークンしか結合できないため、ガード文字列は各セレクタへ直接
/// リテラルとしてインライン展開する（`const` 経由の再利用はしない）。
///
/// coverflow 3D 表示の CSS（[`COVERFLOW_ATTR`] 付き root の子孫のみに
/// 閉じる。入れ子 carousel への漏れは各セレクタの `:not(...)` ガードが
/// 塞ぐ）。
///
/// `--_o`（並び順の相対オフセット、`stagger-index - carousel-index`）から
/// `translateX`/`rotateY`/`translateZ` を合成する。`abs()`/`sign()` は
/// 使わず `max(x, -x)` で絶対値を表現する（`recipe.rs` 全体の既存方針
/// ——CSS 関数の対応状況に依存しない書き方——を踏襲）。
///
/// # `item` を `position: absolute; inset: 0;` にする際の表示領域確保
/// （codex-review/Cursor Bugbot 指摘 是正）
///
/// 全 `item` を絶対配置にすると通常フローから外れ、`item-group` は
/// flex 子の内容寸法で自身の高さを決められなくなる（`flex-basis` は
/// 絶対配置要素には効かない）。是正として `item-group` へ明示 `height`
/// （縦方向の非 coverflow 状態が既に使っている `--fandhe-carousel-height`
/// トークン、既定 20rem、を再利用）を与え、`item` 側は `left: 0`/`top: 0`
/// 単独ではなく `inset: 0` で `item-group` の確定した内容領域いっぱいに
/// 広げる（寸法を `item-group` と厳密に一致させ、`overflow: hidden` と
/// 組み合わせても中身が隠れない）。
pub const CAROUSEL_COVERFLOW_CSS: &str = concat!(
    "[data-fandhe-carousel-coverflow] [data-scope=\"carousel\"][data-part=\"item-group\"]",
    ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-coverflow]) [data-scope=\"carousel\"][data-part=\"item-group\"])",
    " {\n",
    "  transform: none;\n",
    "  perspective: var(--fandhe-carousel-coverflow-perspective, 800px);\n",
    "  transform-style: preserve-3d;\n",
    "  justify-content: center;\n",
    "  position: relative;\n",
    "  height: var(--fandhe-carousel-coverflow-height, var(--fandhe-carousel-height, 20rem));\n",
    "}\n",
    "[data-fandhe-carousel-coverflow] [data-scope=\"carousel\"][data-part=\"item\"]",
    ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-coverflow]) [data-scope=\"carousel\"][data-part=\"item\"])",
    " {\n",
    "  --_o: calc(var(--fandhe-motion-stagger-index, 0) - var(--fandhe-carousel-index, 0));\n",
    "  --_abs-o: max(var(--_o), calc(-1 * var(--_o)));\n",
    "  position: absolute;\n",
    "  inset: 0;\n",
    // 水平 coverflow は移動対象が `item-group`（base）から `item` へ
    // 変わるが、base の `item`（`data-orientation` 無指定）は横方向では
    // 動かない前提のため transition を持たない（`crates/pre-styled-ui/
    // src/carousel.rs` 参照、縦方向の `item` は base 側が既に transition
    // を持つ）。coverflow では横方向でも `item` 自身が動くため、ここで
    // 既存の 3 段フォールバックトークンと同じ transition を明示しないと
    // `next`/`goto` が CSS のみの coverflow で瞬時切り替えになる
    // （codex-review 指摘 是正）。\n",
    "  transition-property: transform;\n",
    "  transition-duration: var(--fandhe-carousel-transition-duration, var(--fandhe-motion-duration-normal, 200ms));\n",
    "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
    "  transform: translateX(calc(var(--_o) * var(--fandhe-carousel-coverflow-spread, 55%)))\n",
    "    rotateY(calc(clamp(-1, var(--_o), 1) * -1 * var(--fandhe-carousel-coverflow-angle, 45deg)))\n",
    "    translateZ(calc(-1 * var(--_abs-o) * var(--fandhe-carousel-coverflow-depth, 60px)));\n",
    "}\n",
    // 縦方向は translateY/rotateX に置換した同型の 1 ブロック（`--_o`/
    // `--_abs-o` は orientation を問わない上のブロックが既に定義済みの
    // ため、ここでは `transform` の上書きのみで足りる）。
    "[data-fandhe-carousel-coverflow] [data-scope=\"carousel\"][data-part=\"item\"][data-orientation=\"vertical\"]",
    ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-coverflow]) [data-scope=\"carousel\"][data-part=\"item\"])",
    " {\n",
    "  transform: translateY(calc(var(--_o) * var(--fandhe-carousel-coverflow-spread, 55%)))\n",
    "    rotateX(calc(clamp(-1, var(--_o), 1) * var(--fandhe-carousel-coverflow-angle, 45deg)))\n",
    "    translateZ(calc(-1 * var(--_abs-o) * var(--fandhe-carousel-coverflow-depth, 60px)));\n",
    "}\n",
);

/// ドラッグ + spring スナップ opt-in 配下の CSS（[`CAROUSEL_DRAG_ATTR`]
/// 付き root 配下のみに閉じる）。ドラッグ中〜settle 完了中
/// （[`CAROUSEL_DRAGGING_STATE_ATTR`]）は CSS `transition` を止め、
/// `fandhe_frontend_animation::carousel::CarouselTrack` の毎フレーム
/// 書き込みだけが見た目を駆動するようにする。
///
/// # `touch-action` は orientation で軸を反転する（codex-review/Cursor
/// Bugbot 指摘 是正）
///
/// `touch-action` はブラウザのネイティブパン操作へ**委譲する**軸を宣言する
/// プロパティであり、JS ドラッグ処理が奪う軸ではなく逆側の軸を指定する。
/// 横方向ドラッグ（既定）はブラウザの縦スクロールを妨げないよう
/// `pan-y`（縦方向のみブラウザへ委譲）を宣言する。縦方向ドラッグ
/// （`item-group[data-orientation="vertical"]`）でこれをそのまま使うと、
/// ユーザーが縦にドラッグするたびページの縦スクロールも同時に発火して
/// 競合し `pointercancel` を誘発する。縦方向は軸を反転し
/// `pan-x`（横方向のみ委譲、縦方向はドラッグ処理が専有）を宣言する。
pub const CAROUSEL_SPRING_SNAP_CSS: &str = concat!(
    "[data-fandhe-carousel-drag] [data-scope=\"carousel\"][data-part=\"item-group\"]",
    ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-drag]) [data-scope=\"carousel\"][data-part=\"item-group\"])",
    " {\n",
    "  touch-action: pan-y;\n",
    "  cursor: grab;\n",
    "}\n",
    "[data-fandhe-carousel-drag] [data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"]",
    ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-drag]) [data-scope=\"carousel\"][data-part=\"item-group\"])",
    " {\n",
    "  touch-action: pan-x;\n",
    "}\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging] [data-scope=\"carousel\"][data-part=\"item-group\"]",
    ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-drag]) [data-scope=\"carousel\"][data-part=\"item-group\"]),\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging] [data-scope=\"carousel\"][data-part=\"item\"]",
    ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-drag]) [data-scope=\"carousel\"][data-part=\"item\"])",
    " {\n",
    "  transition: none;\n",
    "  cursor: grabbing;\n",
    "}\n",
);

/// [`CAROUSEL_COVERFLOW_CSS`] と [`CAROUSEL_SPRING_SNAP_CSS`] を連結した
/// もの（[`Theme::to_css_with_carousel_coverflow`] が追記する CSS 全体）。
#[must_use]
pub fn carousel_motion_css() -> String {
    let mut out =
        String::with_capacity(CAROUSEL_COVERFLOW_CSS.len() + CAROUSEL_SPRING_SNAP_CSS.len());
    out.push_str(CAROUSEL_COVERFLOW_CSS);
    out.push_str(CAROUSEL_SPRING_SNAP_CSS);
    out
}

impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に Carousel の coverflow /
    /// ドラッグ + spring スナップ用 CSS（[`carousel_motion_css()`]）を
    /// 追記して返す。
    #[must_use]
    pub fn to_css_with_carousel_coverflow(&self) -> String {
        let mut out = self.to_css();
        out.push_str(&carousel_motion_css());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carousel_motion_css_has_no_forbidden_angle_bracket_or_at_rule() {
        let css = carousel_motion_css();
        assert!(!css.contains('<'));
        assert!(!css.contains("@keyframes"));
        assert!(!css.contains("@property"));
    }

    #[test]
    fn to_css_with_carousel_coverflow_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_carousel_coverflow();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + carousel_motion_css().len());
    }

    #[test]
    fn coverflow_css_scoped_to_opt_in_attr() {
        let css = carousel_motion_css();
        assert!(css.contains(COVERFLOW_ATTR));
        assert!(css.contains("perspective:"));
        assert!(css.contains("rotateY("));
        // item-group/item 自身にクラスを要求しない（root の子孫セレクタで
        // 閉じる、codex-review/Cursor Bugbot 指摘 是正の回帰防止）。
        assert!(!css.contains("fd-carousel--coverflow"));
    }

    #[test]
    fn coverflow_item_fills_item_group_via_inset() {
        let css = carousel_motion_css();
        assert!(css.contains("inset: 0;"));
        assert!(css.contains("height: var(--fandhe-carousel-coverflow-height"));
    }

    #[test]
    fn spring_snap_css_disables_transition_only_while_dragging() {
        let css = carousel_motion_css();
        assert!(css.contains(CAROUSEL_DRAG_ATTR));
        assert!(css.contains(CAROUSEL_DRAGGING_STATE_ATTR));
        assert!(css.contains("transition: none;"));
    }

    #[test]
    fn spring_snap_css_flips_touch_action_axis_for_vertical() {
        let css = carousel_motion_css();
        assert!(css.contains("touch-action: pan-y;"));
        assert!(css.contains("[data-orientation=\"vertical\"]:not("));
        assert!(css.contains(") {\n  touch-action: pan-x;"));
    }

    /// 入れ子 carousel への漏れ防止ガード（codex-review/Cursor Bugbot 指摘
    /// 是正 P1、イシュー #2541 第 5 ラウンド）の回帰防止: coverflow/drag
    /// いずれの CSS も、opt-in 属性を持たない `[data-part="root"]` を挟む
    /// item-group/item への到達経路を `:not()` で除外する。
    #[test]
    fn coverflow_and_drag_css_exclude_nested_non_opt_in_carousel() {
        let coverflow = CAROUSEL_COVERFLOW_CSS;
        assert!(coverflow.contains(
            ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-coverflow]) [data-scope=\"carousel\"][data-part=\"item-group\"])"
        ));
        assert!(coverflow.contains(
            ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-coverflow]) [data-scope=\"carousel\"][data-part=\"item\"])"
        ));

        let drag = CAROUSEL_SPRING_SNAP_CSS;
        assert!(drag.contains(
            ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-drag]) [data-scope=\"carousel\"][data-part=\"item-group\"])"
        ));
        assert!(drag.contains(
            ":not([data-scope=\"carousel\"][data-part=\"root\"]:not([data-fandhe-carousel-drag]) [data-scope=\"carousel\"][data-part=\"item\"])"
        ));
    }
}
