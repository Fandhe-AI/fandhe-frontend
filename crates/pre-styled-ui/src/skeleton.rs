//! Skeleton（イシュー #764）: 単一 recipe styled 部品。データ読み込み中の
//! コンテンツ形状を模したローディングプレースホルダーを `<div>` として
//! 組み立てる。badge/spinner（イシュー #550）と同型の、headless 状態機械を
//! 要しない静的部品（`docs/design/component-coverage-map.md` skeleton 行）。
//!
//! # 参照サイトとの差分（イシュー #1566）
//!
//! chakra-ui の `Skeleton`（`bg.emphasized` 既定・`pulse`/`shine`/`none` の
//! 3 アニメーション種別）・Radix Themes の `Skeleton`（`gray-a3` 相当の
//! 中間グレー）を基準に、以下の 2 点を是正した。
//!
//! - **背景色**: 旧 `--fandhe-color-bg-subtle`（#f7f7f7）はページ既定背景
//!   （#ffffff）とほぼ同化し、占位要素として視認できなかった。
//!   `--fandhe-color-bg-emphasized`（#e2e2e2/#2e2e2e、chakra `bg.emphasized`
//!   / Radix gray 4-5 相当）へ変更した（`docs/design/color-token-system.md`）。
//! - **アニメーション種別**: 形状 `variant`（text/circle/rect）とは独立した
//!   第 2 軸 [`SkeletonAnimation`]（`pulse`（既定）/`shine`/`none`）を新設し、
//!   chakra-ui の `variant` プロップ相当を表現した。
//!
//! 以下は参照サイトに存在する要素だが、意図的に合わせていない（理由付き）。
//!
//! - **size 軸なし**: chakra-ui も Skeleton 自体に size プロップを持たない
//!   （`SkeletonCircle`/`SkeletonText` という別ヘルパが寸法を扱う）。本実装も
//!   寸法は呼び出し側が CSS custom property のフォールバック値
//!   （`--fandhe-skeleton-size`/`--fandhe-skeleton-height`）を上書きする前提を
//!   維持する（既存判断、変更なし）。
//! - **`loading` プロップなし**: chakra-ui は `loading={false}` で子コンテンツを
//!   表示するラッパーとして使える。本部品は子ノードを取らず、読み込み完了時の
//!   実コンテンツへの差し替えは呼び出し側の責務とする既存契約を維持する
//!   （`docs/policy/intentional-non-adoption.md` §3.25 の UI 部品責務境界と整合）。
//! - **colorPalette 軸なし**: 中立な占位要素のためステータス色を持たない
//!   （既存判断、変更なし。下記「variant 軸のみを持つ理由」参照）。
//! - **rect の角丸を `radius-md` のまま維持**: chakra-ui は全 variant 共通で
//!   `borderRadius: l2`（`sm` 相当）だが、rect は画像・カードの占位という
//!   意匠上の意図的差分として `radius-md` を維持する。
//! - **アニメーション duration をモーショントークン化しない**: 既存の
//!   モーショントークン（150〜300ms）のスケールは UI 操作フィードバック向けで、
//!   1〜5 秒の周期的ループアニメーションとは用途が異なるためリテラル値の
//!   まま維持する。
//! - **コントラスト比要件の対象外**: skeleton root は常時 `aria-hidden="true"`
//!   でテキスト・操作可能な UI を持たない装飾要素のため、WCAG 1.4.3/1.4.11
//!   （コントラスト比）の対象外である。
//! - **hover/focus/disabled は N/A**: 表示専用部品には付与しない
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3）。
//!
//! # aria 出力方針（受け入れ条件 1）
//!
//! skeleton root は実コンテンツを一切持たない**装飾的な占位要素**であり、
//! 常に `aria-hidden="true"`（[`fandhe_frontend_headless_ui::aria_hidden`]）
//! を出力する。スクリーンリーダーへ意味を持たない空の矩形を読み上げさせない
//! ためであり、呼び出し側がこれを外すオプションは設けない。
//!
//! `aria-busy="true"` は skeleton 自身には付与しない。読み込み中であること
//! をスクリーンリーダーへ伝える責務は、skeleton を内包する**コンテナ側**に
//! ある（[`crate::button::button`] の `loading` 時 `aria-busy` と同じ責務
//! 分担）。典型的な利用パターンは以下のとおり: 読み込み中はコンテナへ
//! `aria-busy="true"` を付与しつつ内部に [`skeleton`] を並べ、読み込み完了後は
//! `aria-busy` を外して実コンテンツへ差し替える。
//!
//! ```
//! use fandhe_frontend_core::{el, render};
//! use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps, SkeletonVariant};
//!
//! // 読み込み中: コンテナに aria-busy="true" を付与し、内部を skeleton で埋める。
//! let loading_container = el(
//!     "div",
//!     vec![("aria-busy", "true")],
//!     vec![
//!         skeleton(
//!             &SkeletonProps {
//!                 variant: SkeletonVariant::Circle,
//!                 ..Default::default()
//!             },
//!             vec![],
//!         ),
//!         skeleton(&SkeletonProps::default(), vec![]),
//!     ],
//! );
//! assert!(render(&loading_container).contains(r#"aria-busy="true""#));
//!
//! // 読み込み完了: aria-busy を外し、実コンテンツへ差し替える（呼び出し側の責務）。
//! ```
//!
//! # variant 軸のみを持つ理由
//!
//! `color-palette` 軸は付与しない。占位要素は読み込み前の中立表現であり、
//! ステータス色を持たない（[`crate::card`] が「中立コンテナのため colorPalette
//! 軸は付与しない」とした判断、`crate` 冒頭の複合部品 variant 統一方針ルール 3
//! に整合する）。`size` 軸も付与しない。chakra-ui の Skeleton も寸法は呼び出し
//! 側が `style`/CSS カスタムプロパティで指定する前提であり、本実装も
//! variant ごとの既定寸法を CSS custom property のフォールバック値
//! （例: `--fandhe-skeleton-size`）として与えるに留める。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, Anatomy};

/// `data-scope="skeleton"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("skeleton");

/// パルスアニメーションの `@keyframes` 名リテラル。[`spinner`] モジュールの
/// `spin_keyframes_name_lit!` と同じ理由（`decl()` の値検証は `{`/`}`/`;` を
/// 拒否するため、キーフレーム本体は宣言として表現できず、`animation` 宣言の
/// 値とキーフレームブロック名の単一情報源をマクロとして持つ必要がある）で
/// 同型のマクロを用意する。
///
/// [`spinner`]: crate::spinner
macro_rules! pulse_keyframes_name_lit {
    () => {
        "fd-skeleton-pulse"
    };
}

/// シャインアニメーション（イシュー #1566、chakra-ui `variant: "shine"`
/// 相当）の `@keyframes` 名リテラル。[`pulse_keyframes_name_lit`] と同型の
/// 理由でマクロとして単一情報源化する。
macro_rules! shine_keyframes_name_lit {
    () => {
        "fd-skeleton-shine"
    };
}

/// パルスアニメーションの `@keyframes` 名。[`SkeletonAnimation::Pulse`]
/// variant 規則（値としてのみ参照）と [`css`] が追記する `@keyframes`
/// ブロックの両方で共有する識別子（[`pulse_keyframes_name_lit`] を単一
/// 情報源として生成）。
const PULSE_KEYFRAMES_NAME: &str = pulse_keyframes_name_lit!();

/// シャインアニメーションの `@keyframes` 名（イシュー #1566）。
/// [`SkeletonAnimation::Shine`] variant 規則と [`css`] が追記する
/// `@keyframes` ブロックの両方で共有する識別子。
const SHINE_KEYFRAMES_NAME: &str = shine_keyframes_name_lit!();

/// Skeleton の見た目 variant（形状軸）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkeletonVariant {
    /// 1 行のテキストを模した占位要素（既定）。
    #[default]
    Text,
    /// 円形（アバター等の占位要素）。
    Circle,
    /// 矩形ブロック（画像・カード等の占位要素）。
    Rect,
}

impl VariantValue for SkeletonVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Circle => "circle",
            Self::Rect => "rect",
        }
    }
}

/// Skeleton のアニメーション種別（第 2 軸、イシュー #1566）。
///
/// chakra-ui の `Skeleton` `variant` プロップ（`pulse`/`shine`/`none`）に
/// 対応する。いずれも [`css`] が追記する
/// `@media (prefers-reduced-motion: reduce)` ブロックにより一括停止する
/// （`None` は元からアニメーションを持たないため影響なし）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkeletonAnimation {
    /// 不透明度を周期的に変化させる既定のアニメーション
    /// （chakra-ui `variant: "pulse"` 相当）。
    #[default]
    Pulse,
    /// 背景グラデーションが流れるアニメーション
    /// （chakra-ui `variant: "shine"` 相当）。
    Shine,
    /// アニメーションなし（chakra-ui `variant: "none"` 相当）。
    None,
}

impl VariantValue for SkeletonAnimation {
    fn axis(self) -> &'static str {
        "animation"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Pulse => "pulse",
            Self::Shine => "shine",
            Self::None => "none",
        }
    }
}

/// [`skeleton`] の設定。
#[derive(Debug, Clone, Copy, Default)]
pub struct SkeletonProps {
    /// 見た目 variant（既定 `Text`）。
    pub variant: SkeletonVariant,
    /// アニメーション種別（既定 `Pulse`、イシュー #1566）。
    pub animation: SkeletonAnimation,
}

/// Skeleton の recipe（scope `"skeleton"`、slot `"root"` のみ）。
///
/// 背景色（[`crate::theme`] の `bg-emphasized` トークン）・角丸はテーマ
/// トークンを参照する。アニメーションは形状 `variant` とは独立した第 2 軸
/// [`SkeletonAnimation`] の variant 規則として登録し（イシュー #1566）、
/// [`css`] が追記する `@keyframes`/`prefers-reduced-motion` ブロックと
/// 組み合わせて表現する（受け入れ条件 2）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("skeleton", &["root"])
        .base(
            "root",
            vec![
                decl("display", "block"),
                // イシュー #1566: 旧 `bg-subtle` はページ既定背景と同化し
                // 占位要素として視認できなかったため `bg-emphasized` へ
                // 変更（chakra-ui `bg.emphasized` / Radix gray 4-5 相当）。
                decl("background", "var(--fandhe-color-bg-emphasized)"),
            ],
        )
        .variant(
            SkeletonVariant::Text,
            "root",
            vec![
                decl("width", "100%"),
                decl("height", "1em"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .variant(
            SkeletonVariant::Circle,
            "root",
            vec![
                decl("width", "var(--fandhe-skeleton-size, 2.5rem)"),
                decl("height", "var(--fandhe-skeleton-size, 2.5rem)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                // 参照サイト（chakra-ui Skeleton pulse スクショ）のように
                // flex 行の中で circle variant が潰れないための予防的固定
                // （イシュー #1566）。Text/Rect は `width: 100%` で伸縮に
                // 依存するため flex-shrink:0 を base へ置くと兄弟要素を
                // オーバーフローさせる（PR #1837 Bugbot 指摘）。circle 限定
                // の固定サイズにのみ適用する。
                decl("flex-shrink", "0"),
            ],
        )
        .variant(
            SkeletonVariant::Rect,
            "root",
            vec![
                decl("width", "100%"),
                decl("height", "var(--fandhe-skeleton-height, 5rem)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
            ],
        )
        .default_variant(SkeletonVariant::Text)
        .variant(
            SkeletonAnimation::Pulse,
            "root",
            vec![decl(
                "animation",
                concat!(pulse_keyframes_name_lit!(), " 1.2s ease-in-out infinite"),
            )],
        )
        .variant(
            SkeletonAnimation::Shine,
            "root",
            vec![
                decl(
                    "background-image",
                    concat!(
                        "linear-gradient(270deg, ",
                        "var(--fandhe-skeleton-shine-from, var(--fandhe-color-bg-muted)), ",
                        "var(--fandhe-skeleton-shine-to, var(--fandhe-color-bg-emphasized)), ",
                        "var(--fandhe-skeleton-shine-to, var(--fandhe-color-bg-emphasized)), ",
                        "var(--fandhe-skeleton-shine-from, var(--fandhe-color-bg-muted)))"
                    ),
                ),
                decl("background-size", "400% 100%"),
                decl(
                    "animation",
                    concat!(shine_keyframes_name_lit!(), " 5s ease-in-out infinite"),
                ),
            ],
        )
        .variant(
            SkeletonAnimation::None,
            "root",
            vec![decl("animation", "none")],
        )
        .default_variant(SkeletonAnimation::Pulse)
}

/// Skeleton の静的 CSS 全文。
///
/// recipe が生成する規則群に続けて、`animation` 宣言が参照する
/// `@keyframes` ブロック（[`PULSE_KEYFRAMES_NAME`]・[`SHINE_KEYFRAMES_NAME`]、
/// イシュー #1566 でシャイン用を追加）と、`prefers-reduced-motion: reduce`
/// 環境でアニメーションを停止する `@media` ブロック（受け入れ条件 2）を
/// 固定文字列として追記する。値はソースコード中のリテラルのみで構成され、
/// 外部入力は一切混入しない（[`crate::spinner::css`] と同じ整理。
/// `.claude/rules/coding-rust.md` の HTML/CSS 文字列直接組み立て禁止規約は
/// 実行時入力の文字列結合を禁じる趣旨であり、本関数のように静的リテラルの
/// みを連結する経路は対象外）。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {PULSE_KEYFRAMES_NAME} {{\n  0%, 100% {{\n    opacity: 1;\n  }}\n  50% {{\n    opacity: 0.5;\n  }}\n}}\n"
    ));
    out.push_str(&format!(
        "\n@keyframes {SHINE_KEYFRAMES_NAME} {{\n  from {{\n    background-position: 200% 0;\n  }}\n  to {{\n    background-position: -200% 0;\n  }}\n}}\n"
    ));
    out.push_str(
        // イシュー #1566: `animation` 宣言を base から `Pulse`/`Shine`
        // variant 規則へ移したため、この停止規則もそれらと同じセレクタを
        // 列挙する必要がある。`.fd-skeleton--animation-pulse`/`--shine` は
        // 詳細度 (0,3,0) を持ち、無印セレクタ（(0,2,0)）だけを停止対象に
        // した場合は `@media` がカスケード上の詳細度を上げないため
        // variant 側の `animation` 宣言に負けてしまう（メディアクエリは
        // セレクタ詳細度を変えない）。3 セレクタを列挙することで variant
        // 側と同じ詳細度に揃え、ソース順で最後に出力される本規則が
        // 後勝ちで確実にアニメーションを止める。
        "\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"skeleton\"][data-part=\"root\"],\n  [data-scope=\"skeleton\"][data-part=\"root\"].fd-skeleton--animation-pulse,\n  [data-scope=\"skeleton\"][data-part=\"root\"].fd-skeleton--animation-shine {\n    animation: none;\n  }\n}\n",
    );
    out
}

/// Skeleton 1 個を組み立てる。
///
/// 子ノードを取らない（占位要素は実コンテンツを持たない）。呼び出し側は
/// `attrs` の `style` 属性で幅・高さを上書きできる。`class` 属性は
/// [`crate::class_attr::drop_class_attr`] により常に単一化される（呼び出し側
/// 由来のクラスは recipe 生成クラスへ合成されず破棄する、badge と同じ方針）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps};
///
/// let node = skeleton(&SkeletonProps::default(), vec![]);
/// let html = render(&node);
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn skeleton<'a>(props: &SkeletonProps, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("animation", props.animation.value()),
    ]);
    // `class` は `drop_class_attr` で除去し recipe 生成クラスへ一本化する。
    // `aria-hidden` も同様に呼び出し側の値（大文字小文字を無視）を除去する:
    // 常時 `aria-hidden="true"` を保証するという rustdoc 冒頭の契約
    // （「呼び出し側がこれを外すオプションは設けない」）は、呼び出し側が
    // `("aria-hidden", "false")` を渡せる余地を残しては成立しない
    // （`crates/headless-ui/src/checkbox.rs::control` の
    // `aria-hidden` 除去と同型の fail-closed 判断）。
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("aria-hidden"))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str()), aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn default_props_render_text_variant_and_aria_hidden() {
        let node = skeleton(&SkeletonProps::default(), vec![]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<div data-scope="skeleton" data-part="root" class="fd-skeleton--variant-text fd-skeleton--animation-pulse" aria-hidden="true"></div>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (SkeletonVariant::Text, "fd-skeleton--variant-text"),
            (SkeletonVariant::Circle, "fd-skeleton--variant-circle"),
            (SkeletonVariant::Rect, "fd-skeleton--variant-rect"),
        ] {
            let props = SkeletonProps {
                variant,
                ..Default::default()
            };
            let html = render(&skeleton(&props, vec![]));
            assert!(
                html.contains(&format!("class=\"{class} fd-skeleton--animation-pulse\"")),
                "variant={variant:?} -> {html}"
            );
            assert!(html.contains(r#"aria-hidden="true""#));
        }
    }

    /// イシュー #1566: 第 2 軸 `animation` の 3 値が期待クラスへ写ることを
    /// 固定する（chakra-ui `variant: pulse|shine|none` 相当）。
    #[test]
    fn animation_enumeration_maps_to_expected_classes() {
        for (animation, class) in [
            (SkeletonAnimation::Pulse, "fd-skeleton--animation-pulse"),
            (SkeletonAnimation::Shine, "fd-skeleton--animation-shine"),
            (SkeletonAnimation::None, "fd-skeleton--animation-none"),
        ] {
            let props = SkeletonProps {
                animation,
                ..Default::default()
            };
            let html = render(&skeleton(&props, vec![]));
            assert!(
                html.contains(&format!("class=\"fd-skeleton--variant-text {class}\"")),
                "animation={animation:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_supplied_aria_hidden_is_dropped_case_insensitively() {
        for key in ["aria-hidden", "Aria-Hidden", "ARIA-HIDDEN"] {
            let html = render(&skeleton(&SkeletonProps::default(), vec![(key, "false")]));
            assert_eq!(html.matches("aria-hidden=").count(), 1, "html={html}");
            assert!(html.contains(r#"aria-hidden="true""#), "html={html}");
            assert!(!html.contains(r#"aria-hidden="false""#), "html={html}");
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&skeleton(
            &SkeletonProps::default(),
            vec![("class", "attacker-controlled")],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&skeleton(
            &SkeletonProps::default(),
            vec![("data-testid", "\"><script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn css_output_is_deterministic_and_non_empty() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="skeleton"][data-part="root"]"#));
    }

    #[test]
    fn css_output_declares_pulse_animation_and_keyframes() {
        let out = css();
        assert!(out.contains("animation: fd-skeleton-pulse 1.2s ease-in-out infinite;"));
        assert!(out.contains("@keyframes fd-skeleton-pulse {"));
        assert!(out.contains("opacity: 1;"));
        assert!(out.contains("opacity: 0.5;"));
    }

    /// イシュー #1566: shine variant の宣言と `@keyframes` ブロックを固定する。
    #[test]
    fn shine_keyframes_present() {
        let out = css();
        assert!(out.contains("animation: fd-skeleton-shine 5s ease-in-out infinite;"));
        assert!(out.contains("background-size: 400% 100%;"));
        assert!(out.contains("@keyframes fd-skeleton-shine {"));
        assert!(out.contains("background-position: 200% 0;"));
        assert!(out.contains("background-position: -200% 0;"));
    }

    /// イシュー #1566: `none` variant が `animation: none;` を宣言することを
    /// 固定する。
    #[test]
    fn none_variant_declares_animation_none() {
        let out = css();
        assert!(out.contains(
            r#"[data-scope="skeleton"][data-part="root"].fd-skeleton--animation-none {"#
        ));
        assert!(out.contains("  animation: none;\n"));
    }

    /// イシュー #1566: 基底背景が `bg-emphasized` トークンを参照し、生の色
    /// リテラル（`#`/`rgb(`）を含まないことを固定する。
    #[test]
    fn css_uses_bg_emphasized_token_and_no_raw_color_literal() {
        let out = css();
        assert!(out.contains("background: var(--fandhe-color-bg-emphasized);"));
        assert!(!out.contains('#'));
        assert!(!out.contains("rgb("));
    }

    /// 受け入れ条件 2: `prefers-reduced-motion: reduce` でアニメーションを
    /// 停止する CSS を含むことを固定する。
    #[test]
    fn css_output_declares_reduced_motion_media_query() {
        let out = css();
        assert!(out.contains("@media (prefers-reduced-motion: reduce) {"));
        assert!(out.contains(r#"[data-scope="skeleton"][data-part="root"] {"#));
        assert!(out.contains("animation: none;"));
    }

    /// イシュー #1566: `animation` 宣言を `Pulse`/`Shine` variant 規則
    /// （詳細度 `(0,3,0)`）へ移した結果、reduced-motion の停止規則が無印
    /// セレクタ（詳細度 `(0,2,0)`）のみだと `@media` が詳細度を上げない
    /// ためカスケードで variant 側に負ける（アニメーションが止まらない）
    /// 回帰を防ぐ。停止規則が `--animation-pulse`/`--animation-shine` を
    /// 明示的に含むセレクタ列であることを固定する。
    #[test]
    fn reduced_motion_stop_selector_covers_pulse_and_shine_variant_classes() {
        let out = css();
        let media_start = out
            .find("@media (prefers-reduced-motion: reduce) {")
            .expect("reduced-motion media block must exist");
        let media_block = &out[media_start..];
        assert!(media_block.contains(
            r#"[data-scope="skeleton"][data-part="root"].fd-skeleton--animation-pulse,"#
        ));
        assert!(media_block.contains(
            r#"[data-scope="skeleton"][data-part="root"].fd-skeleton--animation-shine {"#
        ));
    }
}
