//! styled Avatar（headless ラッパー、イシュー #684、親 #680/#681。
//! イシュー #1554 で参照サイト基準（chakra-ui/Radix Themes）へスタイル
//! 調整済み）。
//!
//! `fandhe_frontend_headless_ui::avatar`（イシュー #543/#569）の Root /
//! Image / Fallback 3 anatomy パーツと [`Avatar`] 状態機械を薄く再利用し、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::dialog`]/[`crate::tooltip`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Avatar` 型を
//! 再エクスポートしない理由）
//!
//! [`crate::tooltip`]/[`crate::popover`] は headless モジュールを
//! `pub use ...::*` で丸ごと再エクスポートするが、本モジュールは styled
//! `root`（variant クラス付与のため本モジュールで再定義、`crate::card::root`
//! と同型）と headless の自由関数 `root` が名前衝突するため、必要な識別子
//! のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::avatar::Avatar`] は**あえて**
//! 再エクスポートしない（PR #695 Bugbot 指摘、イシュー #684）。`Avatar` は
//! `.root(attrs, children)` という inherent メソッドを持つが、これは
//! headless 自由関数 `root` へそのまま委譲するのみで variant クラスを
//! 一切付与しない（[`root`] とは別の、未スタイルの実体）。本モジュールが
//! `Avatar` を丸ごと再エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `avatar_instance.root(...)` を呼んでしまい、base 属性のスタイルは効くが
//! variant クラスが付与されずレイアウトが静かに崩れる事故を誘発する
//! （Rust の可視性機構では外部型の inherent メソッドだけを選択的に隠せない
//! ため、型自体を再エクスポートしないことが唯一の fail-closed な対策）。
//! `Avatar` による状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::avatar::Avatar` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! [`image`]/[`fallback`]、`status()` は `Avatar::status()` から取得）を
//! 組み合わせて構築すること。
//!
//! # イシュー #1554 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（Avatar、`size`（`2xs`〜`2xl`）+ `variant`（`solid`(既定)/
//! `subtle`/`outline`）+ `colorPalette` 連動、既定 `gray`）・Radix Themes
//! （Avatar、`size`（1〜9）+ `variant`（`solid`(既定)/`soft`）+ `color`）と
//! スクリーンショット（`docs/design/reference-screenshots/{chakra,radixt,radixp,ark}-avatar-*.png`）
//! 比較した結果を記録する。
//!
//! - **サイズ**: chakra `xs/sm/md/lg/xl` = 24/32/40/48/56px、Radix
//!   `size="1"`〜`"4"` = 24/32/40/48px の両者が一致する段へ是正した（旧実装は
//!   #1681 の機械的外挿で Md = 48px と両参照サイトより 1 段大きかった）。
//!   font-size は旧実装の「Size と同名トークン 1:1」から、イニシャルが円内に
//!   収まる比率（chakra 実測 ≒ 35%）に合わせて「1 段下のトークン」へ変更した
//!   （下限の [`crate::recipe::Size::Xs`] は下限トークン `font-size-xs` に
//!   底打ちする）。共通 [`crate::recipe::Size`] enum の段数（5）は変更しない
//!   （chakra `2xs`/`2xl`、Radix 5〜9・`highContrast` は共通語彙の範囲外の
//!   細分化のため非採用。badge/tag/code/kbd 等、既存 styled 部品と同じ
//!   判断軸）。
//! - **バリアント**: [`AvatarVariant`]（`Subtle`(既定)/`Solid`/`Outline`）を
//!   新設した（本リポジトリ既存語彙 `BadgeVariant`/`KbdVariant` と同名）。
//!   Radix の `soft` は `Subtle` に読み替え、Radix `classic`/`solid` の
//!   ハイコントラスト指定・chakra `plain` は最小サブセット方針（badge/code
//!   と同じ判断）により見送る。既定を chakra/Radix の `solid` ではなく
//!   `Subtle` にするのは、旧実装の灰色フラット外観からの見た目乖離を最小化
//!   するため（[`ColorPalette::Neutral`] 既定と合わせ、変更なし呼び出しの
//!   既存デモの見た目を保つ）。
//! - **色**: [`ColorPalette`] 軸（6 値）を新設した。既定 palette は chakra
//!   Avatar の既定 colorPalette（`gray`）に合わせ [`ColorPalette::Neutral`]
//!   とする（kbd #1436・code #1717 と同じ判断）。
//! - **状態（hover/disabled/transition）・フォーカスリング**: 適用しない
//!   （意図的）。Avatar root は表示専用の `<div>` でインタラクティブ slot を
//!   持たず、`docs/design/pre-styled-ui-interaction-visual-language.md`
//!   （hover はインタラクティブ slot のみ）の適用対象に当たらない。参照
//!   3 サイト（chakra/Radix Themes/ark-ui）のいずれも avatar 単体に
//!   hover/focus-visible リングを持たない。image/fallback の表示切替は
//!   `display: none` の即時切り替えでありアニメーション対象がないため
//!   `transition_declarations` も付与しない。
//! - **ダーク**: 全宣言を `--fandhe-*` トークン参照へ寄せた（旧実装の生色
//!   リテラルは元々含まない）ため `write_dark_declarations` の一元機構に
//!   自動追従する。
//! - **余白・角丸・影・その他 base**: root base に `position: relative`
//!   （chakra、将来の重ね表示バッジのアンカーとして）と `box-sizing:
//!   border-box`（`Outline` variant の 1px 枠線を足してもサイズが変わらない
//!   ように）を追加した。`image` base に `border-radius: inherit`（chakra、
//!   image が root の角丸をはみ出さないように）を追加した。`fallback` base
//!   の `font-weight` を `semibold` から `medium`（chakra/Radix Themes とも
//!   `medium`）へ変更し、`text-transform: uppercase`（両参照サイト共通）を
//!   追加した。角丸トークン（`--fandhe-radius-full`/`-lg`/`0`）は変更しない
//!   （参照サイトと相当）。影は参照サイトも avatar に付与しないため追加
//!   しない。
//!
//! # `image`/`fallback` の base 規則が `display` を宣言しない理由
//!
//! headless 層（[`fandhe_frontend_headless_ui::avatar::image`]/
//! [`fandhe_frontend_headless_ui::avatar::fallback`]）は非表示側に `hidden`
//! 存在属性を付与し、UA 既定 `[hidden] { display: none }` に依存して JS
//! なし SSR の表示制御を成立させる。[`recipe`] の `image`/`fallback` base
//! 規則で `display` を宣言すると、`[data-scope][data-part]`（詳細度
//! (0,2,0)）が `[hidden]`（詳細度 (0,1,0)）に勝ってしまい表示制御が壊れる
//! （[`crate::tooltip`] の positioner 節・PR #575 Bugbot 指摘と同じ構造的な
//! 回避）。`data-state` に応じた `display: none` の明示は [`SlotRecipe::state`]
//! （[`crate::recipe::StateCondition::AttrEq`]）で `[data-state="hidden"]`
//! （詳細度 (0,3,0)）としてのみ登録し、常に `[hidden]` より詳細度で勝つ
//! ことで多層防御にする。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。
//! - variant クラス名は [`recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からの
//!   クラス偽装・重複混入を防ぐ）。
//! - styled `root` は headless [`fandhe_frontend_headless_ui::avatar::root`]
//!   へ委譲するため、呼び出し側 `attrs` の `data-scope`/`data-part` 偽装除去
//!   （headless anatomy の fail-closed 挙動）をそのまま継承する。
//! - すべての配色宣言は `--fandhe-*` トークン参照経由（[`palette_scale_declarations`]
//!   含む）で生成し、生の色リテラル（`#`/`rgb`/`hsl` 等）を一切埋め込まない。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の手書き avatar CSS 撤去・本モジュール
//!   への切り替えは #689（PR #704）で実施済み（#680 は close 済み）。ただし
//!   同 example は crates.io バージョン依存（0.40.0）のままのため、本イシュー
//!   #1554 の `root` シグネチャ破壊は未追随（公開後の別 PR で追随予定）。
//! - crates.io への公開・利用側依存追随は #686 のスコープ。
//! - headless 共通型の再エクスポート整備は #685 のスコープ。
//! - 画像 `load`/`error` イベントの wasm グルーは headless 層 doc 記載済みの
//!   既存スコープ外を継承する。
//! - `Avatar.Group`（重ね表示・attached）部品の新設可否は #1554 のスコープ外
//!   （Issue 化候補として記録）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};
// `Avatar` 状態機械はあえて再エクスポートしない（本モジュール冒頭の rustdoc
// 「`Avatar` 型を再エクスポートしない理由」参照）。状態管理・hydration が
// 必要な呼び出し側は `fandhe_frontend_headless_ui::avatar::Avatar` を直接 import する。
pub use fandhe_frontend_headless_ui::avatar::{fallback, image, AvatarAction, ImageStatus};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/avatar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &["root", "image", "fallback"];

/// Avatar の外形（chakra-ui Avatar の `shape` variant を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarShape {
    /// 円形（既定）。
    #[default]
    Circle,
    /// 角丸四角形。
    Rounded,
    /// 直角四角形。
    Square,
}

impl VariantValue for AvatarShape {
    fn axis(self) -> &'static str {
        "shape"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::Rounded => "rounded",
            Self::Square => "square",
        }
    }
}

/// Avatar の見た目 variant（イシュー #1554 で新設。[`crate::badge::BadgeVariant`]/
/// [`crate::kbd::KbdVariant`] と同名の 3 値。本モジュール冒頭 rustdoc
/// 「イシュー #1554 の参照サイト比較」参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarVariant {
    /// 淡色背景（既定。旧実装の灰色フラット外観からの乖離を避ける）。
    #[default]
    Subtle,
    /// 濃色背景 + コントラスト文字色。
    Solid,
    /// 背景なし + 枠線。
    Outline,
}

impl VariantValue for AvatarVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Solid => "solid",
            Self::Outline => "outline",
        }
    }
}

/// [`root`] の設定（イシュー #1554 で `size`/`shape` の 2 引数から
/// `variant`/`palette` を加えた 4 軸へ拡張し、可読性のため位置引数から
/// Props 構造体へ移行した。[`crate::kbd::KbdProps`] と同型）。
#[derive(Debug, Clone, Copy)]
pub struct AvatarProps {
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// 外形（既定 `Circle`）。
    pub shape: AvatarShape,
    /// 見た目 variant（既定 `Subtle`）。
    pub variant: AvatarVariant,
    /// colorPalette 軸（既定 `Neutral`。chakra Avatar の既定 colorPalette
    /// `gray` に合わせる）。
    pub palette: ColorPalette,
}

impl Default for AvatarProps {
    fn default() -> Self {
        AvatarProps {
            size: Size::Md,
            shape: AvatarShape::Circle,
            variant: AvatarVariant::Subtle,
            palette: ColorPalette::Neutral,
        }
    }
}

/// この styled Avatar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("avatar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("position", "relative"),
                decl("box-sizing", "border-box"),
                decl("overflow", "hidden"),
                decl("flex-shrink", "0"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "image",
            vec![
                decl("width", "100%"),
                decl("height", "100%"),
                decl("object-fit", "cover"),
                decl("border-radius", "inherit"),
            ],
        )
        .base(
            "fallback",
            vec![
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "1"),
                decl("text-transform", "uppercase"),
            ],
        )
        // headless 層の `hidden` 存在属性（UA 既定 `[hidden] { display: none }`）
        // による JS なし SSR の表示制御を、`data-state="hidden"` 一致時の
        // 明示的な `display: none` で多層防御する（本モジュール冒頭の rustdoc
        // 「`image`/`fallback` の base 規則が `display` を宣言しない理由」参照）。
        .state(
            "image",
            StateCondition::AttrEq("data-state", "hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "fallback",
            StateCondition::AttrEq("data-state", "hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #1554: chakra `xs/sm/md/lg/xl`（24/32/40/48/56px）と Radix
        // Themes `size="1"`〜`"4"`（24/32/40/48px）が一致する段へ是正
        // （旧 #1681 の機械的外挿は Md = 48px で両参照サイトより 1 段大きかった）。
        // font-size は「1 段下のトークン」（chakra 実測のイニシャル/円比 ≒ 35%）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("width", "1.5rem"),
                decl("height", "1.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("width", "2rem"),
                decl("height", "2rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("width", "2.5rem"),
                decl("height", "2.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("width", "3rem"),
                decl("height", "3rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("width", "3.5rem"),
                decl("height", "3.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .default_variant(Size::Md)
        .variant(
            AvatarShape::Circle,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-full)")],
        )
        .variant(
            AvatarShape::Rounded,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-lg)")],
        )
        .variant(
            AvatarShape::Square,
            "root",
            vec![decl("border-radius", "0")],
        )
        .default_variant(AvatarShape::Circle)
        // イシュー #1554: variant/palette 軸を新設。Subtle は Neutral の
        // `-subtle`（#f7f7f7 相当、白背景との区別がつかない）ではなく
        // `-muted` を使う（chakra のスクショが示す灰色円と一致させるため）。
        .variant(
            AvatarVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-muted)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .variant(
            AvatarVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            AvatarVariant::Outline,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
            ],
        )
        .default_variant(AvatarVariant::Subtle)
        .default_variant(ColorPalette::Neutral);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled Avatar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`shape`/`variant`/`palette` に
/// 応じたクラスを付与する唯一のパーツ（[`drop_class_attr`] により呼び出し側
/// の `class` は除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::avatar::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::avatar::{root, AvatarProps};
///
/// let node = root(&AvatarProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="avatar" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(props: &AvatarProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("size", props.size.value()),
        ("shape", props.shape.value()),
        ("variant", props.variant.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::avatar::root(merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    // --- anatomy ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(&AvatarProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        // headless anatomy の fail-closed 偽装除去（`Anatomy::part`）を
        // styled root 経由でも継承していることの回帰。
        let html = render(&root(
            &AvatarProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- data-state 連動 ---

    #[test]
    fn stylesheet_links_hidden_state_to_display_none_for_image_and_fallback() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="avatar"][data-part="image"][data-state="hidden"] {
  display: none;
}
"#
        ));
        assert!(css.contains(
            r#"[data-scope="avatar"][data-part="fallback"][data-state="hidden"] {
  display: none;
}
"#
        ));
    }

    #[test]
    fn image_and_fallback_base_rules_do_not_declare_display() {
        // `[hidden]`（詳細度 (0,1,0)）に対し `[data-scope][data-part]`
        // （詳細度 (0,2,0)）が勝ってしまう回帰を防ぐ（本モジュール冒頭の rustdoc
        // 「`image`/`fallback` の base 規則が `display` を宣言しない理由」）。
        let css = stylesheet();
        let image_base_start = css
            .find(r#"[data-scope="avatar"][data-part="image"] {"#)
            .expect("image base rule must exist");
        let image_base_end = css[image_base_start..]
            .find('}')
            .map(|i| image_base_start + i)
            .unwrap();
        assert!(!css[image_base_start..image_base_end].contains("display"));

        let fallback_base_start = css
            .find(r#"[data-scope="avatar"][data-part="fallback"] {"#)
            .expect("fallback base rule must exist");
        let fallback_base_end = css[fallback_base_start..]
            .find('}')
            .map(|i| fallback_base_start + i)
            .unwrap();
        assert!(!css[fallback_base_start..fallback_base_end].contains("display"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_avatar_state_machine() {
        // `Avatar` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Avatar` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::avatar::Avatar;

        let mut a = Avatar::default();
        assert_eq!(a.status(), ImageStatus::Loading);

        let ssr_html = render(&a.fallback(vec![], vec![text("NM")]));
        assert!(ssr_html.contains(r#"data-state="visible""#));

        assert!(dispatch(&mut a, "loaded", ""));
        let hydrate_html = render(&render_for_hydration(&a));
        assert!(hydrate_html.contains(r#"data-hydrate-status="loaded""#));

        let restored = Avatar::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored.status(), ImageStatus::Loaded);
    }

    // --- variant クラス ---

    #[test]
    fn default_variant_is_md_circle_subtle_neutral() {
        let html = render(&root(&AvatarProps::default(), vec![], vec![]));
        assert!(html.contains("fd-avatar--size-md"));
        assert!(html.contains("fd-avatar--shape-circle"));
        assert!(html.contains("fd-avatar--variant-subtle"));
        assert!(html.contains("fd-avatar--color-palette-neutral"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-avatar--size-xs"),
            (Size::Sm, "fd-avatar--size-sm"),
            (Size::Md, "fd-avatar--size-md"),
            (Size::Lg, "fd-avatar--size-lg"),
            (Size::Xl, "fd-avatar--size-xl"),
        ] {
            let props = AvatarProps {
                size,
                ..AvatarProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn shape_enumeration_maps_to_expected_classes() {
        for (shape, class) in [
            (AvatarShape::Circle, "fd-avatar--shape-circle"),
            (AvatarShape::Rounded, "fd-avatar--shape-rounded"),
            (AvatarShape::Square, "fd-avatar--shape-square"),
        ] {
            let props = AvatarProps {
                shape,
                ..AvatarProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(html.contains(class), "shape={shape:?} -> {html}");
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (AvatarVariant::Subtle, "fd-avatar--variant-subtle"),
            (AvatarVariant::Solid, "fd-avatar--variant-solid"),
            (AvatarVariant::Outline, "fd-avatar--variant-outline"),
        ] {
            let props = AvatarProps {
                variant,
                ..AvatarProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-avatar--color-palette-accent"),
            (ColorPalette::Info, "fd-avatar--color-palette-info"),
            (ColorPalette::Success, "fd-avatar--color-palette-success"),
            (ColorPalette::Warning, "fd-avatar--color-palette-warning"),
            (ColorPalette::Danger, "fd-avatar--color-palette-danger"),
            (ColorPalette::Neutral, "fd-avatar--color-palette-neutral"),
        ] {
            let props = AvatarProps {
                palette,
                ..AvatarProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            &AvatarProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_variant_selectors_and_radius_token() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("--size-"));
        assert!(a.contains("--shape-"));
        assert!(a.contains("--variant-"));
        assert!(a.contains("--color-palette-"));
        assert!(a.contains("var(--fandhe-radius-full)"));
    }

    #[test]
    fn stylesheet_contains_no_raw_color_literals() {
        // イシュー #1554: 配色はすべて `--fandhe-*` トークン参照経由とし、
        // 生の色リテラル（hex 等）を埋め込まない不変条件。
        let css = stylesheet();
        assert!(!css.contains('#'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            &AvatarProps::default(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn fallback_children_script_payload_is_escaped() {
        let html = render(&fallback(
            ImageStatus::Loading,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
