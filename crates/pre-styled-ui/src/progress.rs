//! styled Progress（linear + circle 対応、イシュー #763/#1564、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::progress`（イシュー #544/#600）の値状態
//! 機械 [`Progress`] が持つ Root / Label / ValueText / Track / Range（linear）
//! と Circle / CircleTrack / CircleRange（circular、SVG）の各 inherent
//! メソッドに対し、[`stylesheet`] で既定 CSS を追加提供する薄い委譲層。
//!
//! # イシュー #1564: linear（Track/Range）の styled 対応を追加
//!
//! #763 時点では対応表（`docs/design/component-coverage-map.md`）が linear の
//! pre-styled ラッパーを follow-up へ切り分けていたため、本モジュールは
//! circle 系のみに CSS を提供していた。参照 4 サイト（chakra-ui / Radix
//! Themes / Radix Primitives / ark-ui）のスクリーンショットがいずれも
//! linear を基準としていたことを受け、本イシューで [`range`] ラッパーと
//! `track`/`range` slot の recipe（サイズ別トラック高・[`ProgressVariant`]・
//! [`ColorPalette`] 軸・indeterminate/vertical の状態別 CSS）を新設した。
//!
//! # `Progress` 型を再エクスポートしない理由（`crate::dialog`/`crate::switch`
//! と同型の判断）
//!
//! [`Progress`] は `.root(...)`/`.label(...)`/`.value_text(...)`/`.track(...)`/
//! `.range(...)`/`.circle(...)`/`.circle_track(...)`/`.circle_range(...)` という
//! inherent メソッドを持つが、これらは headless 中立の未スタイル実体であり
//! `size`/`variant`/`color-palette` variant クラスを一切付与しない。本モジュール
//! が [`Progress`] を丸ごと `pub use` で再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`progress_instance.root(...)` を直接呼んでしまい、
//! variant が付与されず見た目が静かに崩れる事故を誘発する（`crate::dialog`/
//! `crate::switch` が `Dialog`/`Switch` を再エクスポートしない理由と同じ、
//! イシュー #684/PR #695 Bugbot 指摘の一般化）。[`Progress`] による状態管理・
//! hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::progress::Progress`
//! を直接 import し、Root は本モジュールの styled [`root`] を、Range は
//! 動的な `--fandhe-progress-percent` 付与のため本モジュールの styled
//! [`range`] を経由する（Track/Circle/CircleTrack/CircleRange は headless の
//! inherent メソッドをそのまま呼ぶ。CSS はクラスではなく
//! `[data-scope="progress"][data-part="..."]` セレクタで当たるため、
//! これらのパーツにクラス付与の必要がない）。[`ProgressAction`]/[`Orientation`]
//! のみ呼び出し側の利便のため選択的に再エクスポートする。
//!
//! # `size`/`variant`/`color-palette` variant（イシュー #763/#1564）
//!
//! headless [`Progress::circle`] は `--size`/`--thickness` を参照する固定
//! `style`（[`fandhe_frontend_headless_ui::progress`] 冒頭 rustdoc の
//! 「SVG ジオメトリ（CSS 変数方式、headless 中立）」節参照）を出力するのみで、
//! 実際の値は styled 層が CSS で定義する headless 中立設計になっている。
//! 本モジュールは [`ProgressProps`] を [`root`] へのみクラスとして付与し、
//! [`recipe`] が `--fandhe-progress-size`/`--fandhe-progress-thickness`/
//! `--fandhe-progress-track-height`（root スコープの CSS custom property。
//! 通常の CSS 継承で子孫の track/circle へ伝わる）を登録する。circle/track
//! 自身の base 規則には Md 相当のフォールバック値を書き、styled [`root`] を
//! 経由しない headless 直接利用マークアップでも外観を維持する（fail-safe、
//! `crate::drawer`/`crate::dialog` の `size` variant と同じ方針）。
//!
//! [`ProgressVariant`]（`Outline`/`Subtle`）は track 背景の見た目を切り替える
//! （chakra `outline`/`subtle`、Radix Themes `surface`/`soft` 相当。命名は
//! 本リポジトリ既存語彙（`ButtonVariant`/`BadgeVariant`）に合わせ、Radix 名
//! （`classic`/`soft`）は持ち込まない）。[`ColorPalette`] は root へ
//! `palette_scale_declarations` を登録し、range/circle-range の塗り色を
//! 切り替える。
//!
//! # indeterminate アニメーション（styled 層が可視表現を担う契約）
//!
//! headless の各パーツは indeterminate 時に `data-state="indeterminate"`
//! のみを出力し、進捗系の値（`--percent`/`stroke-dasharray`/
//! `stroke-dashoffset`/range の幅）を捏造しない（headless 側 rustdoc 参照）。
//! 可視表現は本モジュールが `[data-part="circle"][data-state="indeterminate"]`
//! （回転）・`[data-part="range"][data-state="indeterminate"]`（横スライド、
//! horizontal/vertical で 2 種）セレクタへ `animation` 宣言（[`SlotRecipe::state`]）
//! と `@keyframes`（[`stylesheet`] が固定文字列として追記、`crate::spinner`
//! と同型のパターン）で提供することで完成させる。
//!
//! `prefers-reduced-motion: reduce` 環境では [`stylesheet`] が
//! `[data-part="circle"][data-state="indeterminate"]`/
//! `[data-part="range"][data-state="indeterminate"]` の無限 `animation` を
//! `animation: none` で個別停止する（`crate::skeleton` と同型。`transition`
//! 側は [`MotionDuration`] トークン経由のため [`crate::theme::Theme::to_css`]
//! の duration 一括無効化に乗る）。
//!
//! # 意図的に参考サイトへ合わせない点
//!
//! - hover/focus-visible/disabled: `role="progressbar"` は表示専用・非
//!   フォーカス要素であり操作対象ではない。参照 4 サイトも持たない
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3
//!   「インタラクティブ slot のみ」）。
//! - chakra `striped`/`animated`: 4 サイト中 1 サイトのみの装飾。最小
//!   サブセット方針（`crate::badge` と同じ）で見送り、利用者要望が出た
//!   時点で再評価する。
//! - chakra `shape`（square/rounded/full）/ Radix `radius` prop: 角丸は
//!   `--fandhe-radius-full` の 1 段に固定（`crate::slider` と同じ判断）。
//!   利用者は `upsert_radius` かセレクタ上書きで変更可能。
//! - Radix `highContrast`/`duration`（自動加算）: トークン体系にない軸・
//!   実行時挙動のため見送り。
//! - `data-state="complete"` 専用の視覚差: 参照 4 サイトいずれも complete
//!   専用の見た目を持たないため付けない。
//! - circle-track は variant（`Outline`/`Subtle`）の影響を受けない（白背景
//!   上で淡色リングが視認不能になるため）。
//! - Subtle variant のトラック色は `<palette>-subtle`（`<palette>-muted` は
//!   accent/info で light 時 3:1 未達のため不採用。判断根拠は本クレート
//!   `theme.rs` テストモジュールのコントラスト表を参照）。
//!
//! # セキュリティ不変条件
//!
//! - [`recipe`] が生成する CSS は固定リテラル（[`crate::css::decl`]）のみで
//!   構成し、任意文字列が CSS 生成経路へ混入する経路はない（`crate::spinner`
//!   と同じ根拠）。
//! - [`root`] は呼び出し側 `attrs` の `class` を [`crate::class_attr::drop_class_attr`]
//!   で除去してから recipe 生成クラスと合成する（重複 `class` 属性による
//!   無効な HTML 出力・後勝ちの非決定的なスタイル適用の防止）。
//! - [`range`] は headless [`Progress::percent`] が返す `[min, max]` へ
//!   clamp 済みの有限 `f64` のみから `style` を組み立て、呼び出し側 `attrs`
//!   の `style`（大文字小文字を無視）を [`drop_style_attr`] で除去してから
//!   合成する（`crate::slider::range` と同型の dedup。文字列入力を一切
//!   含まない）。
//! - `aria_valuetext`・呼び出し側 `attrs`・children は headless
//!   [`Progress::root`]/[`Progress::range`] へそのまま委譲するため、既定
//!   エスケープ（REQ-1）は headless 側の保証をそのまま継承する（本モジュール
//!   は HTML 文字列を直接組み立てない）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, transition_declarations, ColorPalette, MotionDuration, Size,
    SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::progress::Progress;
// `Progress` 型はあえて再エクスポートしない（本モジュール冒頭 rustdoc
// 「`Progress` 型を再エクスポートしない理由」節参照）。呼び出し側の利便のため
// アクション・向き型のみ選択的に再エクスポートする。
pub use fandhe_frontend_headless_ui::{Orientation, ProgressAction};

/// headless `progress` anatomy の `data-part` 一覧（`crates/headless-ui/src/progress.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "value-text",
    "track",
    "range",
    "circle",
    "circle-track",
    "circle-range",
];

/// indeterminate 時の回転アニメーションの `@keyframes` 名リテラル。`decl()`
/// が要求する `&'static str` は実行時 `format!` で組み立てられないため、
/// リテラルの単一情報源をマクロとして持ち、[`SPIN_KEYFRAMES_NAME`]（値としての
/// 参照・`format!` 用）と [`recipe`] の `animation` 宣言（`concat!` による
/// コンパイル時連結）の両方がこのマクロ経由で同一文字列を得る
/// （`crate::spinner` と同型のパターン）。
macro_rules! spin_keyframes_name_lit {
    () => {
        "fd-progress-circle-spin"
    };
}

/// indeterminate 時の linear range 横スライドアニメーションの `@keyframes` 名
/// リテラル（horizontal 版）。[`spin_keyframes_name_lit`] と同じ単一情報源
/// パターン。
macro_rules! range_slide_keyframes_name_lit {
    () => {
        "fd-progress-range-slide"
    };
}

/// indeterminate 時の linear range 縦スライドアニメーションの `@keyframes` 名
/// リテラル（vertical 版）。
macro_rules! range_slide_vertical_keyframes_name_lit {
    () => {
        "fd-progress-range-slide-vertical"
    };
}

/// indeterminate 時の回転アニメーションの `@keyframes` 名。[`recipe`] の
/// `animation` 宣言（値としてのみ参照）と [`stylesheet`] が追記する
/// `@keyframes` ブロックの両方で共有する識別子（[`spin_keyframes_name_lit`]
/// を単一情報源として生成）。
const SPIN_KEYFRAMES_NAME: &str = spin_keyframes_name_lit!();

/// indeterminate 時の linear range 横スライドアニメーションの `@keyframes` 名。
const RANGE_SLIDE_KEYFRAMES_NAME: &str = range_slide_keyframes_name_lit!();

/// indeterminate 時の linear range 縦スライドアニメーションの `@keyframes` 名。
const RANGE_SLIDE_VERTICAL_KEYFRAMES_NAME: &str = range_slide_vertical_keyframes_name_lit!();

/// track slot の見た目 variant（イシュー #1564）。chakra `outline`/`subtle`、
/// Radix Themes `surface`/`soft` に相当する軸。Radix 側の名称
/// （`classic`/`soft`）は持ち込まず、本リポジトリ既存語彙
/// （`ButtonVariant`/`BadgeVariant`）に合わせる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressVariant {
    /// 中立トラック + 淡い内側 1px 枠（既定。chakra `outline` / Radix
    /// `surface` 相当）。
    #[default]
    Outline,
    /// palette 淡色トラック（chakra `subtle` / Radix `soft` 相当）。
    Subtle,
}

impl VariantValue for ProgressVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            ProgressVariant::Outline => "outline",
            ProgressVariant::Subtle => "subtle",
        }
    }
}

/// [`root`] の設定（イシュー #1564。`crate::avatar::AvatarProps`/
/// `crate::kbd::KbdProps` と同型の Props 構造体）。
#[derive(Debug, Clone, Copy)]
pub struct ProgressProps {
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// 見た目 variant（既定 `Outline`）。
    pub variant: ProgressVariant,
    /// colorPalette 軸（既定 `Accent`。既存 circle の既定色と Radix 既定
    /// `accent` に合わせる。chakra 既定 `gray` は不採用）。
    pub palette: ColorPalette,
}

impl Default for ProgressProps {
    fn default() -> Self {
        ProgressProps {
            size: Size::Md,
            variant: ProgressVariant::Outline,
            palette: ColorPalette::Accent,
        }
    }
}

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`range`] が `--fandhe-progress-percent` を含む `style` を組み立てた後、
/// 呼び出し側 `attrs` を連結する前に使う dedup ヘルパ（`crate::slider::
/// drop_style_attr`/`crates/headless-ui/src/progress.rs::drop_style_attr` と
/// 同型の判断。重複 `style` 属性による後勝ちの非決定的なスタイル適用を防ぐ）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `--fandhe-progress-percent` custom property を設定する `style` 属性値を
/// 組み立てる（動的値は [`Progress::percent`] が返す正規化済み有限 `f64`
/// 由来の 1 点のみ）。
fn percent_style(percent: f64) -> String {
    format!("--fandhe-progress-percent: {percent}%")
}

/// この styled Progress の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut r = SlotRecipe::new("progress", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "label",
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .base(
            "value-text",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-variant-numeric", "tabular-nums"),
                decl("margin-left", "auto"),
            ],
        )
        .base(
            "track",
            vec![
                decl("position", "relative"),
                decl("overflow", "hidden"),
                decl("flex-basis", "100%"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-progress-track-height, 0.625rem)"),
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
            ],
        )
        .base("range", {
            let mut declarations = vec![
                decl("position", "absolute"),
                decl("top", "0"),
                decl("left", "0"),
                decl("height", "100%"),
                decl("width", "var(--fandhe-progress-percent, 0%)"),
                decl("border-radius", "inherit"),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ];
            declarations.extend(transition_declarations(
                "width, height",
                MotionDuration::Normal,
            ));
            declarations
        })
        // circle は headless 中立（`--size`/`--thickness` を styled 層/呼び出し
        // 側が CSS で定義する設計、headless 側 rustdoc 参照）。root variant が
        // `--fandhe-progress-size`/`--fandhe-progress-thickness` を継承経由で
        // 上書きし、ここでは Md 相当のフォールバックのみを宣言する
        // （styled root を経由しない headless 直接利用でも外観を維持する
        // fail-safe、`crate::drawer` と同じ方針）。
        .base(
            "circle",
            vec![
                decl("--size", "var(--fandhe-progress-size, 3rem)"),
                decl("--thickness", "var(--fandhe-progress-thickness, 0.25rem)"),
                decl("transform-origin", "center"),
            ],
        )
        .base(
            "circle-track",
            vec![decl("stroke", "var(--fandhe-color-border)")],
        )
        .base("circle-range", {
            let mut declarations = vec![
                decl(
                    "stroke",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("stroke-linecap", "round"),
            ];
            declarations.extend(transition_declarations(
                "stroke-dashoffset",
                MotionDuration::Normal,
            ));
            declarations
        })
        // イシュー #1681: Xs/Xl は size 1rem 刻み・thickness 0.05rem 刻みの
        // Sm→Md→Lg 等差進行を外挿。イシュー #1564: 各段へ chakra
        // xs/sm/md/lg/xl のトラック高相当の `--fandhe-progress-track-height`
        // を追加。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-progress-size", "1rem"),
                decl("--fandhe-progress-thickness", "0.15rem"),
                decl("--fandhe-progress-track-height", "0.375rem"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-progress-size", "2rem"),
                decl("--fandhe-progress-thickness", "0.2rem"),
                decl("--fandhe-progress-track-height", "0.5rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-progress-size", "3rem"),
                decl("--fandhe-progress-thickness", "0.25rem"),
                decl("--fandhe-progress-track-height", "0.625rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-progress-size", "4rem"),
                decl("--fandhe-progress-thickness", "0.3rem"),
                decl("--fandhe-progress-track-height", "0.75rem"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-progress-size", "5rem"),
                decl("--fandhe-progress-thickness", "0.35rem"),
                decl("--fandhe-progress-track-height", "1rem"),
            ],
        )
        .default_variant(Size::Md)
        // イシュー #1564: track の見た目 variant（モジュール冒頭 rustdoc
        // 「意図的に参考サイトへ合わせない点」参照。circle-track は対象外）。
        .variant(
            ProgressVariant::Outline,
            "track",
            vec![
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl(
                    "box-shadow",
                    "inset 0 0 0 1px var(--fandhe-color-border-muted)",
                ),
            ],
        )
        .variant(
            ProgressVariant::Subtle,
            "track",
            vec![decl("background", "var(--fandhe-palette-subtle)")],
        )
        .default_variant(ProgressVariant::Outline)
        // イシュー #763: indeterminate 時のみ circle（svg コンテナ）全体を
        // 回転させる（モジュール冒頭 rustdoc「indeterminate アニメーション」
        // 節参照。headless は indeterminate 時に circle へ inline `transform`
        // を出力しないため衝突しない）。
        .state(
            "circle",
            StateCondition::AttrEq("data-state", "indeterminate"),
            vec![decl(
                "animation",
                concat!(spin_keyframes_name_lit!(), " 1s linear infinite"),
            )],
        )
        // イシュー #1564: indeterminate 時の linear range（horizontal 既定）。
        .state(
            "range",
            StateCondition::AttrEq("data-state", "indeterminate"),
            vec![
                decl("width", "40%"),
                decl(
                    "animation",
                    concat!(
                        range_slide_keyframes_name_lit!(),
                        " 1.5s var(--fandhe-motion-easing-standard) infinite"
                    ),
                ),
            ],
        )
        // イシュー #1564: vertical track/range（headless が付与する
        // `data-orientation="vertical"` を消費、`crate::slider` の
        // vertical 対応と同型）。
        .state(
            "track",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("width", "var(--fandhe-progress-track-height, 0.625rem)"),
                decl("height", "var(--fandhe-progress-track-length, 12rem)"),
                decl("flex-basis", "auto"),
            ],
        )
        .state(
            "range",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("top", "auto"),
                decl("bottom", "0"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-progress-percent, 0%)"),
            ],
        )
        .state(
            "range",
            StateCondition::AttrEqAll(&[
                ("data-state", "indeterminate"),
                ("data-orientation", "vertical"),
            ]),
            vec![
                decl("width", "100%"),
                decl("height", "40%"),
                decl(
                    "animation",
                    concat!(
                        range_slide_vertical_keyframes_name_lit!(),
                        " 1.5s var(--fandhe-motion-easing-standard) infinite"
                    ),
                ),
            ],
        );

    // イシュー #1564: colorPalette 軸（root slot）。range/circle-range が
    // 参照する `var(--fandhe-palette, ...)` の切り替え元。circle-track は
    // 対象外（モジュール冒頭 rustdoc 参照）。
    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        r = r.variant(palette, "root", palette_scale_declarations(palette));
    }
    r.default_variant(ColorPalette::Accent)
}

/// この styled Progress が生成する静的 CSS 全量を返す（決定的。同一プロセス内
/// で複数回呼んでも常にバイト単位で同一の文字列を返す、`crate::spinner` の
/// [`css`](crate::spinner::css) と同じ契約）。
///
/// recipe が生成する規則群に続けて、`animation` 宣言が参照する `@keyframes`
/// ブロック（[`SPIN_KEYFRAMES_NAME`]/[`RANGE_SLIDE_KEYFRAMES_NAME`]/
/// [`RANGE_SLIDE_VERTICAL_KEYFRAMES_NAME`]）と、`prefers-reduced-motion: reduce`
/// 環境で無限 `animation` を停止する `@media` ブロック（`crate::skeleton` と
/// 同型）を固定文字列として追記する。値はソースコード中のリテラルのみで
/// 構成され、外部入力は一切混入しない（`.claude/rules/coding-rust.md` の
/// HTML/CSS 文字列直接組み立て禁止規約は「実行時入力を文字列結合で埋め込む
/// こと」を禁じる趣旨であり、本関数のように静的リテラルのみを連結する経路は
/// 対象外、`crate::spinner::css` と同じ根拠）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {SPIN_KEYFRAMES_NAME} {{\n  from {{\n    transform: rotate(0deg);\n  }}\n  to {{\n    transform: rotate(360deg);\n  }}\n}}\n"
    ));
    out.push_str(&format!(
        "@keyframes {RANGE_SLIDE_KEYFRAMES_NAME} {{\n  from {{\n    transform: translateX(-100%);\n  }}\n  to {{\n    transform: translateX(250%);\n  }}\n}}\n"
    ));
    out.push_str(&format!(
        "@keyframes {RANGE_SLIDE_VERTICAL_KEYFRAMES_NAME} {{\n  from {{\n    transform: translateY(100%);\n  }}\n  to {{\n    transform: translateY(-250%);\n  }}\n}}\n"
    ));
    out.push_str(
        "@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"progress\"][data-part=\"circle\"][data-state=\"indeterminate\"] {\n    animation: none;\n  }\n\n  [data-scope=\"progress\"][data-part=\"range\"][data-state=\"indeterminate\"] {\n    animation: none;\n  }\n}\n",
    );
    out
}

/// styled root パーツを組み立てる。`size`/`variant`/`color-palette` に応じた
/// クラスを付与する唯一のパーツ（[`drop_class_attr`] により呼び出し側の
/// `class` は除去してから合成する）。実体は [`Progress::root`] へ委譲する。
///
/// `progress` は状態（`min`/`max`/`value`/`orientation`）の単一情報源であり、
/// 状態管理・hydration が必要な呼び出し側は
/// `fandhe_frontend_headless_ui::progress::Progress` を直接 import して
/// 構築・更新した上で本関数へ渡す（モジュール冒頭 rustdoc 参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::progress::Progress;
/// use fandhe_frontend_headless_ui::Orientation;
/// use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
///
/// let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
/// let node = progress::root(&p, &ProgressProps::default(), None, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="progress" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    progress: &Progress,
    props: &ProgressProps,
    aria_valuetext: Option<&str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("size", props.size.value()),
        ("variant", props.variant.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    progress.root(aria_valuetext, merged, children)
}

/// styled range パーツを組み立てる。determinate（[`Progress::percent`] が
/// `Some`）のときのみ `--fandhe-progress-percent` を含む `style` を付与する
/// 唯一のパーツ（[`drop_style_attr`] により呼び出し側の `style` は除去して
/// から合成する）。indeterminate では `style` を一切出力しない（進捗値を
/// 捏造しない headless 側の契約と整合、モジュール冒頭 rustdoc参照）。
/// 実体は [`Progress::range`] へ委譲する。
#[must_use]
pub fn range<'a>(progress: &Progress, attrs: Vec<(&'a str, &'a str)>) -> Node {
    match progress.percent() {
        Some(percent) => {
            let style = percent_style(percent);
            let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
            merged.extend(drop_style_attr(attrs));
            progress.range(merged, vec![])
        }
        None => progress.range(drop_style_attr(attrs), vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn determinate() -> Progress {
        Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal)
    }

    fn indeterminate() -> Progress {
        Progress::new(0.0, 100.0, None, Orientation::Horizontal)
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="progress"][data-part="track"]"#));
        assert!(a.contains(r#"[data-scope="progress"][data-part="range"]"#));
        assert!(a.contains(r#"[data-scope="progress"][data-part="circle"]"#));
        assert!(a.contains(r#"[data-scope="progress"][data-part="circle-track"]"#));
        assert!(a.contains(r#"[data-scope="progress"][data-part="circle-range"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        let p = determinate();
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let props = ProgressProps {
                size,
                ..ProgressProps::default()
            };
            let html = render(&root(&p, &props, None, vec![("class", "attacker")], vec![]));
            let expected_class = format!("fd-progress--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn variant_and_palette_append_classes_to_root() {
        let p = determinate();
        let props = ProgressProps {
            variant: ProgressVariant::Subtle,
            palette: ColorPalette::Success,
            ..ProgressProps::default()
        };
        let html = render(&root(&p, &props, None, vec![], vec![]));
        assert!(html.contains("fd-progress--variant-subtle"), "html={html}");
        assert!(
            html.contains("fd-progress--color-palette-success"),
            "html={html}"
        );
    }

    #[test]
    fn default_variant_is_md_outline_accent_and_matches_fallback() {
        let css = stylesheet();
        assert!(css.contains("--size: var(--fandhe-progress-size, 3rem);"));
        assert!(css.contains("--fandhe-progress-size: 3rem;"));
        assert!(css.contains("--fandhe-progress-track-height: 0.625rem;"));
    }

    #[test]
    fn circle_indeterminate_state_declares_spin_animation_and_keyframes() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="progress"][data-part="circle"][data-state="indeterminate"] {"#
        ));
        assert!(css.contains(&format!(
            "animation: {SPIN_KEYFRAMES_NAME} 1s linear infinite;"
        )));
        assert!(css.contains(&format!("@keyframes {SPIN_KEYFRAMES_NAME} {{")));
        assert!(css.contains("transform: rotate(0deg);"));
        assert!(css.contains("transform: rotate(360deg);"));
    }

    #[test]
    fn range_indeterminate_state_declares_slide_animations_and_keyframes() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="progress"][data-part="range"][data-state="indeterminate"] {"#
        ));
        assert!(css.contains(&format!("@keyframes {RANGE_SLIDE_KEYFRAMES_NAME} {{")));
        assert!(css.contains(&format!(
            "@keyframes {RANGE_SLIDE_VERTICAL_KEYFRAMES_NAME} {{"
        )));
    }

    #[test]
    fn reduced_motion_media_query_stops_indeterminate_animations() {
        let css = stylesheet();
        assert!(css.contains("@media (prefers-reduced-motion: reduce) {"));
        assert!(css.contains(
            r#"[data-scope="progress"][data-part="circle"][data-state="indeterminate"] {"#
        ));
        assert!(css.matches("animation: none;").count() >= 2);
    }

    #[test]
    fn vertical_orientation_states_are_declared() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="progress"][data-part="track"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="progress"][data-part="range"][data-orientation="vertical"] {"#
        ));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let p = indeterminate();
        let html = render(&root(&p, &ProgressProps::default(), None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="progress""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="indeterminate""#));
    }

    #[test]
    fn styled_range_sets_percent_style_for_determinate_and_omits_for_indeterminate() {
        let determinate_html = render(&range(&determinate(), vec![]));
        assert!(
            determinate_html.contains(r#"style="--fandhe-progress-percent: 40%""#),
            "html={determinate_html}"
        );

        let indeterminate_html = render(&range(&indeterminate(), vec![]));
        assert!(
            !indeterminate_html.contains("style="),
            "html={indeterminate_html}"
        );
    }

    #[test]
    fn styled_range_drops_caller_style_and_keeps_percent() {
        let html = render(&range(
            &determinate(),
            vec![("style", "attacker: 1"), ("STYLE", "attacker: 2")],
        ));
        assert!(
            html.contains(r#"style="--fandhe-progress-percent: 40%""#),
            "html={html}"
        );
        assert!(!html.contains("attacker"));
        assert_eq!(html.matches("style=\"").count(), 1);
    }

    #[test]
    fn caller_headless_track_and_circle_parts_render_without_wrapper() {
        // track/circle/circle-track/circle-range は headless の inherent
        // メソッドをそのまま呼ばせる契約（モジュール冒頭 rustdoc 参照）。
        // styled 層の独自ラッパーを持たないことを回帰として固定する。
        let p = determinate();
        let track_html = render(&p.track(vec![], vec![range(&p, vec![])]));
        assert!(track_html.contains(r#"data-part="track""#));
        assert!(track_html.contains(r#"data-part="range""#));

        let circle_html = render(&p.circle(
            vec![],
            vec![
                p.circle_track(vec![], vec![]),
                p.circle_range(vec![], vec![]),
            ],
        ));
        assert!(circle_html.starts_with("<svg"));
        assert!(circle_html.contains(r#"data-part="circle""#));
        assert!(circle_html.contains(r#"data-part="circle-track""#));
        assert!(circle_html.contains(r#"data-part="circle-range""#));
    }

    #[test]
    fn aria_valuetext_and_caller_attrs_are_escaped_on_render() {
        let p = determinate();
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(
            &p,
            &ProgressProps::default(),
            Some(PAYLOAD),
            vec![("data-testid", PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }
}
