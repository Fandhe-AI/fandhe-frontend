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
/// 付与し、CSS 側は root からの子コンビネータ（`>`）チェーンで
/// `item-group`/`item` を狙う」パターンを採る（codex-review/Cursor Bugbot
/// 指摘 是正、イシュー #2541: 当初はクラスとして `item-group`/`item` 自身
/// にも付与する契約にしていたが、`crate::carousel::root` は呼び出し側の
/// `class` 属性を `crate::class_attr::drop_class_attr` で破棄する契約
/// 〔`root` rustdoc「パーツ」節参照〕のため、クラスを root へ付与する経路
/// が構造的に存在しなかった。データ属性はこの破棄経路の対象外
/// 〔`drop_class_attr` は `class` キーのみを狙う〕であり、
/// [`CAROUSEL_DRAG_ATTR`] は同じ形で既に機能しているため、そちらへ合わせ
/// る）。子コンビネータ採用の経緯は [`CAROUSEL_COVERFLOW_CSS`] rustdoc
/// 「入れ子 carousel 除外の是正」節参照。
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

/// coverflow 3D 表示の CSS（[`COVERFLOW_ATTR`] 付き root 配下のみに閉じる。
/// 入れ子 carousel への漏れ対策は下記「入れ子 carousel 除外の是正」節
/// 参照）。
///
/// # 入れ子 carousel 除外の是正（codex-review/Cursor Bugbot 指摘 是正 P1、
/// イシュー #2541 第 5〜9 ラウンド）
///
/// 第 5〜8 ラウンドは `[OPT] [item-group]`（子孫セレクタ）に対して
/// `:not([OPT] [root]...[item-group])` 形の除外ガードを重ねる方針を採って
/// いたが、CSS の `:not(複合セレクタ)` は「その形に一致する経路が
/// **文書全体のどこかに 1 つでも存在すれば**除外する」という単一の
/// 真偽判定であり、「どの祖先 root を起点に一致したか」を区別できない。
/// このため、外側 opt-in → 中間非 opt-in → 内側 opt-in という 3 段の
/// 入れ子では、内側 item-group 自身が「外側 opt-in 祖先 → 中間 root
/// （非 opt-in）」という経路にも同時に一致してしまい、内側が正しく
/// opt-in しているにもかかわらず除外されてしまう欠陥が残った
/// （ガード側の条件をいくら調整しても、子孫セレクタ + `:not()` という
/// 組み合わせ自体が「最も近い祖先 root」を表現できない構造的な限界。
/// 第 9 ラウンドで確認）。
///
/// 是正として、`:not()` ガードによる除外方式を廃止し、carousel の
/// anatomy が持つ**固定の親子関係**を子コンビネータ（`>`）で直接表現する
/// 方式へ変更した。`item`（`display: flex` の `item-group` の flex 子）は
/// 仕様上 `item-group` の**直接の子**でなければならず、また `item-group`
/// は呼び出し側の組み立て（`crates/docs-site/src/showcase.rs` の実例）上
/// root の直接の子、または `control`（`item_group` を束ねるコンテナ）を
/// 挟んで root の孫のいずれかにしかならない。**入れ子にした別の carousel
/// は必ず `item` の子孫として置かれる**（`item-group` 自身の直接の子に
/// なることはない）ため、root から `item-group`/`item` へ至る経路は
/// 「root の子」または「root の子の子（control 経由）」の**高々 2 段**に
/// 構造的に収まる。子コンビネータはこの段数を厳密に要求するため、
/// 入れ子 carousel の `item`（さらに深い段）を経由する経路は**どの opt-in
/// 属性の有無に関わらず一致し得ない**（除外ガードで場合分けする必要が
/// なくなる）。各 root の opt-in 判定はその root 自身が起点の子コンビ
/// ネータ・チェーンでのみ成立するため、入れ子の深さ・opt-in 属性の
/// 組み合わせに関わらず「各パーツは自分自身の最も近い root の opt-in
/// 状態でのみスタイルされる」という当初の意図が構造的に保証される
/// （第 5〜8 ラウンドの `:not()` ガードは全廃）。
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
    "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"],\n",
    "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"]",
    " {\n",
    "  transform: none;\n",
    "  perspective: var(--fandhe-carousel-coverflow-perspective, 800px);\n",
    "  transform-style: preserve-3d;\n",
    "  justify-content: center;\n",
    "  position: relative;\n",
    "  height: var(--fandhe-carousel-coverflow-height, var(--fandhe-carousel-height, 20rem));\n",
    "}\n",
    "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"] > [data-scope=\"carousel\"][data-part=\"item\"],\n",
    "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"] > [data-scope=\"carousel\"][data-part=\"item\"]",
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
    "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"] > [data-scope=\"carousel\"][data-part=\"item\"][data-orientation=\"vertical\"],\n",
    "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"] > [data-scope=\"carousel\"][data-part=\"item\"][data-orientation=\"vertical\"]",
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
/// 書き込みだけが見た目を駆動するようにする。入れ子 carousel 除外の
/// 是正方針は [`CAROUSEL_COVERFLOW_CSS`] rustdoc 参照（root からの子
/// コンビネータ・チェーンで自分自身の item-group/item のみを狙う）。
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
    "[data-fandhe-carousel-drag][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"],\n",
    "[data-fandhe-carousel-drag][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"]",
    " {\n",
    "  touch-action: pan-y;\n",
    "  cursor: grab;\n",
    "}\n",
    "[data-fandhe-carousel-drag][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"],\n",
    "[data-fandhe-carousel-drag][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"]",
    " {\n",
    "  touch-action: pan-x;\n",
    "}\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"],\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"],\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"] > [data-scope=\"carousel\"][data-part=\"item\"],\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"] > [data-scope=\"carousel\"][data-part=\"item\"]",
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
        assert!(css.contains("[data-orientation=\"vertical\"],\n"));
        assert!(css.contains("[data-orientation=\"vertical\"] {\n  touch-action: pan-x;"));
    }

    /// 入れ子 carousel 除外の是正（codex-review/Cursor Bugbot 指摘 是正
    /// P1、イシュー #2541 第 5〜9 ラウンド）の回帰防止: `:not()` ガード方式
    /// を全廃し、root からの子コンビネータ（`>`）チェーンのみで
    /// item-group/item を狙っていることを固定する（モジュール doc
    /// 「入れ子 carousel 除外の是正」節参照）。`>` を使わない素朴な子孫
    /// セレクタが再導入されると、入れ子 carousel の子孫にも一致してしまう
    /// 元の不具合へ逆戻りする。
    #[test]
    fn coverflow_and_drag_css_scope_via_child_combinator_not_not_guard() {
        let coverflow = CAROUSEL_COVERFLOW_CSS;
        assert!(!coverflow.contains(":not("));
        assert!(coverflow.contains(
            "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"]"
        ));
        assert!(coverflow.contains(
            "[data-fandhe-carousel-coverflow][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"]"
        ));
        assert!(coverflow.contains(
            "[data-scope=\"carousel\"][data-part=\"item-group\"] > [data-scope=\"carousel\"][data-part=\"item\"]"
        ));

        let drag = CAROUSEL_SPRING_SNAP_CSS;
        assert!(!drag.contains(":not("));
        assert!(drag.contains(
            "[data-fandhe-carousel-drag][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"item-group\"]"
        ));
        assert!(drag.contains(
            "[data-fandhe-carousel-drag][data-scope=\"carousel\"][data-part=\"root\"] > [data-scope=\"carousel\"][data-part=\"control\"] > [data-scope=\"carousel\"][data-part=\"item-group\"]"
        ));
    }

    /// 3 段入れ子（外側 opt-in → 中間非 opt-in → 内側 opt-in）のような
    /// 任意深さの入れ子でも内側自身が正しく含まれ、外側の状態が内側へ
    /// 漏れないことの構造的根拠（codex-review/Cursor Bugbot 指摘 是正 P1
    /// の回帰防止、イシュー #2541 第 9 ラウンド）。子コンビネータのみを
    /// 使う本実装では「root → (control >)? item-group (> item)?」の
    /// **高々 3 個**（root→control・control→item-group・item-group→item）
    /// でしか一致しないため、入れ子 carousel（必ず `item` の子孫として
    /// 置かれる）の内部を経由する経路は 4 個以上の `>` を要し、
    /// coverflow/drag いずれの CSS セレクタにも構造的に現れ得ない
    /// （`:not()` ガードによる場合分けが不要になった理由そのもの）。
    #[test]
    fn coverflow_and_drag_css_chains_are_bounded_to_three_hops() {
        let coverflow = CAROUSEL_COVERFLOW_CSS;
        let drag = CAROUSEL_SPRING_SNAP_CSS;
        for css in [coverflow, drag] {
            for line in css.lines() {
                // 各セレクタ行が持つ `>` の個数は、root→control→
                // item-group（control 経由の最大 2 個）+ item-group→item
                // （最大 1 個）の合計で高々 3 個までに収まること。入れ子
                // carousel（`item` の子孫としてのみ置かれる）を経由する
                // 経路は 4 個以上の `>` を要するため、この上限が常に
                // 成り立つ限り入れ子への漏れは構造的に発生しない。
                let child_combinators = line.matches('>').count();
                assert!(
                    child_combinators <= 3,
                    "セレクタ行 {line:?} の子コンビネータが 3 個を超えている（入れ子 \
                     carousel への漏れが再導入された可能性）"
                );
            }
        }
    }
}
