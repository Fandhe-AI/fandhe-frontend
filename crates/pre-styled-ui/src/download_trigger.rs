//! styled DownloadTrigger（headless ラッパー、イシュー #828）。
//!
//! `fandhe_frontend_headless_ui::download_trigger`（イシュー #828）の唯一の
//! anatomy パーツ `root` を薄く再利用し、[`recipe`] で既定 CSS を追加提供
//! する。
//!
//! # recipe は Button recipe の「流用」（宣言差分ゼロ）
//!
//! [`recipe`] は独自の CSS 宣言を持たず、[`crate::button::recipe_with_scope`]
//! に `"download-trigger"` scope を渡すだけの薄い委譲である。Button と
//! DownloadTrigger は「見た目は完全に同一・意味論のみ異なる」（ボタン
//! 風の外観をした `a[download]` リンク）という要件に基づく判断であり、
//! `variant`/`size`/`palette` の宣言・既定値は 1 箇所（Button）にのみ存在
//! する。本モジュールが独自に CSS 宣言を複製すると、Button 側の変更が
//! DownloadTrigger 側へ追随しない静かなドリフトを生むため、宣言の複製を
//! 避けて `recipe_with_scope` へ委譲する（`crates/pre-styled-ui/tests/download_trigger_css.rs`
//! の golden テストがこの流用契約自体を機械的に固定する）。
//!
//! `disabled`/`loading` は [`crate::button::ButtonProps`] と異なり本モジュール
//! では提供しない。DownloadTrigger の実体は `a` 要素であり、`disabled`
//! 属性・暗黙 submit 抑制は `button` 要素固有の意味論であって `a` 要素には
//! 存在しない（`disabled` は `a` に対して何の効果も持たない）。無効化が
//! 必要な場合は呼び出し側が `href` を出力しない、または `aria-disabled`/
//! `tabindex="-1"` を呼び出し側 `attrs` から明示的に付与すること。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。`href` の URL スキーム検証は headless
//!   層（`crates/headless-ui/src/download_trigger.rs` rustdoc 参照）が担う。
//! - variant クラス名は [`crate::recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//!
//! # 参考サイト基準との比較結論（イシュー #1474）
//!
//! Themes 部品の視覚比較ツリー（phase:2）の一環として、参考サイト
//! （chakra-ui / ark-ui）基準の 7 軸（サイズ・バリアント・色・`data-*`
//! 状態・ダーク・フォーカス・余白/hover/disabled/トランジション）で
//! 本部品を検証した。ark-ui の `DownloadTrigger` は unstyled headless
//! ユーティリティ（見た目を持たない）のため、視覚比較の実体は chakra-ui
//! 側の Button の見た目である。本モジュールは前述のとおり Button recipe
//! を宣言差分ゼロで流用する設計であり、この流用そのものが「参考サイト
//! 基準の見た目」を実現する手段になっている。
//!
//! - **継承により充足済み（是正不要）**: サイズ（`Size` 5 段、#1449）・
//!   バリアント（`ButtonVariant` 6 種、#1448）・色（`--fandhe-palette-*`/
//!   `--fandhe-color-*` トークン経由）・ダーク（`Theme::to_css` のトークン
//!   再定義）・フォーカス（`:focus-visible` + `focus_ring_declarations`、
//!   #1448）・hover/disabled/トランジション（#1425/#1708 の共通ビジュアル
//!   言語、`prefers-reduced-motion` 一括無効化込み）は、いずれも Button
//!   recipe の委譲経由で継承済みであり、本モジュール側での追加是正は
//!   不要と判断した。
//! - **意図的に参考サイトへ合わせない点**: `disabled`/`loading` 状態は
//!   本モジュールで提供しない（前述のとおり `a` 要素に `disabled` の
//!   意味論が存在しないため）。icon-only 修飾（IconButton 相当）も
//!   本部品には適用しない（ダウンロードリンクはラベルテキストを必須と
//!   する用途が主であり、Button 側 #830 の icon-only 拡張をそのまま
//!   輸入する動機がない）。CSS 宣言の独自追加はしない（追加すると
//!   「宣言はここでは一切追加しない」という流用契約に反し、golden
//!   テスト `tests/download_trigger_css.rs` が FAIL する設計であるため、
//!   乖離の是正が必要になった場合は Button 側イシューで行うのが本設計
//!   の正）。
//! - **是正した点**: docs サイト Demo（`crates/docs-site/src/showcase.rs`
//!   の `download_trigger_section`）の size 行が `button_section` と
//!   非対称（Sm/Md/Lg の 3 段のみ）だったため、Xs/Xl を追加して 5 段へ
//!   揃えた。CSS 実体の変更ではなく Demo 表示範囲の是正である。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - ark-ui 側のスクリーンショット取得（unstyled headless のため視覚
//!   比較には寄与しない、上記比較結論節参照）。

use crate::button::{recipe_with_scope, ButtonVariant};
use crate::class_attr::drop_class_attr;
use crate::recipe::{ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`root`] の設定。[`crate::button::ButtonProps`] の見た目軸
/// （`variant`/`size`/`palette`）のみを持ち、`disabled`/`loading` は持たない
/// （本モジュール冒頭 rustdoc「recipe は Button recipe の流用」節参照）。
#[derive(Debug, Clone, Copy)]
pub struct DownloadTriggerProps {
    /// 見た目 variant（既定 `Solid`）。[`crate::button::ButtonVariant`] を
    /// 再利用する。
    pub variant: ButtonVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`）。
    pub palette: ColorPalette,
}

impl Default for DownloadTriggerProps {
    fn default() -> Self {
        DownloadTriggerProps {
            variant: ButtonVariant::Solid,
            size: Size::Md,
            palette: ColorPalette::Accent,
        }
    }
}

/// この styled DownloadTrigger の recipe を組み立てる（内部ヘルパ、[`css`]
/// のみが呼ぶ）。[`crate::button::recipe_with_scope`] へそのまま委譲する
/// （本モジュール冒頭 rustdoc 参照、宣言はここでは一切追加しない）。
fn recipe() -> SlotRecipe {
    recipe_with_scope("download-trigger")
}

/// この styled DownloadTrigger が生成する静的 CSS 全量を返す（決定的。
/// [`crate::button::css`] の scope 違い版と同一内容、
/// `crates/pre-styled-ui/tests/download_trigger_css.rs` の golden テストが
/// この対応を固定する）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled `root` パーツ（`a[download]`）を組み立てる。`variant`/`size`/
/// `palette` に応じたクラスを付与する唯一のパーツ（[`drop_class_attr`]
/// により呼び出し側の `class` は除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::download_trigger::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::download_trigger::{root, DownloadTriggerProps};
///
/// let node = root(
///     &DownloadTriggerProps::default(),
///     "/assets/report.pdf",
///     Some("report.pdf"),
///     vec![],
///     vec![text("Download report")],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"download="report.pdf""#));
/// ```
#[must_use]
pub fn root<'a>(
    props: &DownloadTriggerProps,
    href: &'a str,
    file_name: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);

    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));

    fandhe_frontend_headless_ui::download_trigger::root(href, file_name, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_part_href_and_download() {
        let html = render(&root(
            &DownloadTriggerProps::default(),
            "/assets/report.pdf",
            Some("report.pdf"),
            vec![],
            vec![text("Download report")],
        ));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="download-trigger""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"href="/assets/report.pdf""#));
        assert!(html.contains(r#"download="report.pdf""#));
        assert!(html.contains(">Download report<"));
    }

    #[test]
    fn default_props_render_solid_md_accent_classes() {
        let html = render(&root(
            &DownloadTriggerProps::default(),
            "/assets/report.pdf",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-download-trigger--size-md"));
        assert!(html.contains("fd-download-trigger--variant-solid"));
        assert!(html.contains("fd-download-trigger--color-palette-accent"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (ButtonVariant::Solid, "fd-download-trigger--variant-solid"),
            (
                ButtonVariant::Outline,
                "fd-download-trigger--variant-outline",
            ),
            (ButtonVariant::Ghost, "fd-download-trigger--variant-ghost"),
            (ButtonVariant::Subtle, "fd-download-trigger--variant-subtle"),
            (
                ButtonVariant::Surface,
                "fd-download-trigger--variant-surface",
            ),
            (ButtonVariant::Plain, "fd-download-trigger--variant-plain"),
        ] {
            let props = DownloadTriggerProps {
                variant,
                ..DownloadTriggerProps::default()
            };
            let html = render(&root(&props, "/assets/report.pdf", None, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            &DownloadTriggerProps::default(),
            "/assets/report.pdf",
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="download-trigger""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            &DownloadTriggerProps::default(),
            "/assets/report.pdf",
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn css_is_deterministic_and_reuses_button_declarations() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="download-trigger"][data-part="root"]"#));
        assert!(a.contains("fd-download-trigger--"));
    }

    #[test]
    fn css_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    // --- URL スキーム拒否（fail-closed、headless 層 → core の render() 経由） ---

    #[test]
    fn dangerous_url_schemes_are_rejected() {
        let html = render(&root(
            &DownloadTriggerProps::default(),
            "javascript:alert(1)",
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("href="));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn href_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            &DownloadTriggerProps::default(),
            "/assets/report.pdf\" onmouseover=\"alert(1)",
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&root(
            &DownloadTriggerProps::default(),
            "/assets/report.pdf",
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
