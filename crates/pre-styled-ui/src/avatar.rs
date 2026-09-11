//! styled Avatar（headless ラッパー、イシュー #684、親 #680/#681。
//! イシュー #1554 で参照サイト基準（chakra-ui/Radix Themes）へスタイル
//! 調整済み）。
//!
//! `fandhe_frontend_headless_ui::avatar`（イシュー #543/#569）の Root /
//! Image / Fallback 3 anatomy パーツと `Avatar` 状態機械を薄く再利用し、
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
//! なし SSR の表示制御を成立させる。`recipe` の `image`/`fallback` base
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
//!   `fandhe_frontend_core::render` の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。
//! - variant クラス名は [`recipe::SlotRecipe::variant_classes`](crate::recipe::SlotRecipe::variant_classes) が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   `crate::class_attr::drop_class_attr` で除去してから recipe 生成
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
//! - `Avatar.Group`（重ね表示・attached）部品は #1554 時点ではスコープ外
//!   だったが、イシュー #2044 で `group`/`badge` パートとして実装した
//!   （下記「イシュー #2044 の shadcn/ui 突合」節参照）。
//!
//! # イシュー #2044 の shadcn/ui 突合（badge / group の補完）
//!
//! shadcn/ui（Base UI 版 Avatar）と突合した結果、`AvatarBadge`（右下の状態
//! ドット）と `AvatarGroup`（重なり表示 + `+N`）の 2 合成パターンが欠落して
//! いたため、pre-styled-only の anatomy パート（[`group`]/[`badge`]）として
//! 補完した（[`crate::dialog::footer`]/[`crate::dialog::body`] と同型。
//! headless-ui（Primitives 層）は不変、`docs/policy/intentional-non-adoption.md`
//! §3.25 規則 2 により装飾・レイアウトは pre-styled-ui の責務とする）。
//!
//! - **badge**: [`AvatarProps::with_badge`]（既定 `false`）を `true` にした
//!   root のみ `overflow: visible` を解除し、[`badge`] パート（`<span>`、
//!   [`AvatarBadgeProps`] で `size`/`palette` を指定）を右下に絶対配置する。
//!   既定 palette は shadcn `bg-primary` に合わせ [`ColorPalette::Accent`]
//!   とする。
//! - **group**: [`group`] パート（`<div>`）が `-space-x-2` 相当の重なり間隔
//!   を確保し、[`AvatarProps::stacked`]（既定 `false`）を `true` にした root
//!   へ負のマージンと `box-shadow` によるリング（`--fandhe-color-bg`）を
//!   付与する。`+N` の残数表示は独立パートを新設せず、`root(Subtle/Neutral,
//!   stacked: true) + fallback("+3")` の既存パーツの組み合わせで表現する
//!   （視覚的に通常の Avatar と同一のため、パート増を避ける）。
//! - **意図的に合わせない点**: root 既定の `overflow: hidden` は
//!   [`AvatarProps::with_badge`] が `false` のとき変更しない（純追加原則）。
//!   `data-slot` 等 shadcn 固有の語彙・独自 `data-*` は持ち込まない
//!   （`badge`/`group` は headless anatomy 由来の `data-scope`/`data-part`
//!   以外の `data-*` を出力しない）。参照競合の判定（重なり量・リング幅・
//!   badge 既定色・badge サイズ写像）は該当実装イシューの PR 本文へ記録する。

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
// イシュー #2044: pre-styled-only `group`/`badge` パート（[`group`]/[`badge`]
// 関数）が headless の `Anatomy::part` を直接呼び出すために必要
// （[`crate::dialog::footer`] と同型のパターン）。
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="avatar"` を固定した本モジュール独自パート（`group`/`badge`）
/// 用の anatomy（イシュー #2044）。headless-ui 側の `avatar::ANATOMY`
/// （`crates/headless-ui/src/avatar.rs`）とは別のインスタンスだが `scope`
/// 文字列は同一値であり、出力される `data-scope` 属性値は一致する
/// （[`crate::dialog::ANATOMY`] と同型）。
const ANATOMY: Anatomy = anatomy("avatar");

/// [`SlotRecipe::new`] に渡す slot 一覧。先頭 3 件（`root`/`image`/
/// `fallback`）は `crates/headless-ui/src/avatar.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約（ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。イシュー #2044 で pre-styled-only 2 パート
/// （`group`/`badge`。headless-ui の anatomy には存在せず、本モジュールだけ
/// が出力する）を末尾へ追加し、計 5 件になった（[`crate::dialog`] の
/// `footer`/`body` と同型）。
const SLOTS: &[&str] = &["root", "image", "fallback", "group", "badge"];

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

/// `stacked` 修飾 variant（axis `"stack"` / value `"stacked"`）。イシュー
/// #2044 で追加。[`AvatarProps::stacked`] が `true` のときのみ [`root`] の
/// `selection` へ渡す非公開 enum で、呼び出し側の公開 API
/// （[`AvatarProps`]）には axis/value 文字列を露出しない
/// （`crate::button::ButtonIcon` と同型のパターン）。`default_variant` を
/// 登録しないため、`stacked: false`（既定）の class 出力・golden CSS は
/// 不変のまま保たれる（純追加原則）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AvatarStack {
    /// [`crate::avatar::group`] 内で重なり表示する root。
    Stacked,
}

impl VariantValue for AvatarStack {
    fn axis(self) -> &'static str {
        "stack"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Stacked => "stacked",
        }
    }
}

/// `with_badge` 修飾 variant（axis `"overlay"` / value `"badge"`）。イシュー
/// #2044 で追加。[`AvatarProps::with_badge`] が `true` のときのみ [`root`] の
/// `selection` へ渡す非公開 enum（[`AvatarStack`] と同型）。`default_variant`
/// を登録しないため、`with_badge: false`（既定）の class 出力・golden CSS は
/// 不変のまま保たれる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AvatarBadgeOverlay {
    /// [`crate::avatar::badge`] を子に持つ root（`overflow: visible` を要求）。
    Badge,
}

impl VariantValue for AvatarBadgeOverlay {
    fn axis(self) -> &'static str {
        "overlay"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Badge => "badge",
        }
    }
}

/// [`root`] の設定（イシュー #1554 で `size`/`shape` の 2 引数から
/// `variant`/`palette` を加えた 4 軸へ拡張し、可読性のため位置引数から
/// Props 構造体へ移行した。[`crate::kbd::KbdProps`] と同型）。イシュー
/// #2044 で `stacked`/`with_badge` を追加した（いずれも既定 `false`。
/// `AvatarProps::default()` の `class` 属性・golden CSS は変更前と完全一致
/// のまま保たれる、本モジュール冒頭 rustdoc「イシュー #2044 の shadcn/ui
/// 突合」節参照）。
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
    /// [`group`] 内で重なり表示するか（既定 `false`、イシュー #2044）。
    /// `true` のとき負のマージンと `box-shadow` によるリングを付与する。
    pub stacked: bool,
    /// [`badge`] を子に持つか（既定 `false`、イシュー #2044）。`true` のとき
    /// のみ `overflow: visible` を解除し、右下に絶対配置された badge が
    /// 円のクリップで欠けないようにする。
    pub with_badge: bool,
}

impl Default for AvatarProps {
    fn default() -> Self {
        AvatarProps {
            size: Size::Md,
            shape: AvatarShape::Circle,
            variant: AvatarVariant::Subtle,
            palette: ColorPalette::Neutral,
            stacked: false,
            with_badge: false,
        }
    }
}

/// [`badge`] の設定（イシュー #2044、[`crate::kbd::KbdProps`] と同型）。
#[derive(Debug, Clone, Copy)]
pub struct AvatarBadgeProps {
    /// サイズ（既定 `Md` = 12px。shadcn の 8/10/12px を Xs/Sm/Md へ写像し、
    /// Lg/Xl は既存 space スケール段へ外挿する）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`。shadcn `bg-primary` に合わせる）。
    pub palette: ColorPalette,
}

impl Default for AvatarBadgeProps {
    fn default() -> Self {
        AvatarBadgeProps {
            size: Size::Md,
            palette: ColorPalette::Accent,
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

    // イシュー #2044（shadcn/ui 突合）: group/badge の補完。既存登録の後ろに
    // 追加するのみで、上記の base/variant/default_variant は一切変更しない
    // （純追加原則。`AvatarProps::default()` の `class` 属性は不変のまま）。
    recipe = recipe
        // `group`: shadcn `-space-x-2` 相当の重なり間隔。先頭子の負マージン
        // （下記 `AvatarStack::Stacked`）は本 `padding-left` が打ち消す。
        .base(
            "group",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("padding-left", "var(--fandhe-space-2)"),
            ],
        )
        // `stacked`（[`AvatarStack::Stacked`]）: group 内で重なり表示する
        // root。`box-shadow` は自要素の描画のためリングは root の
        // `overflow: hidden`（既定）にクリップされない。
        .variant(
            AvatarStack::Stacked,
            "root",
            vec![
                decl("margin-left", "calc(-1 * var(--fandhe-space-2))"),
                decl("box-shadow", "0 0 0 2px var(--fandhe-color-bg)"),
            ],
        )
        // `with_badge`（[`AvatarBadgeOverlay::Badge`]）: badge を子に持つ
        // root のみ `overflow: hidden`（既定）を解除する（本モジュール冒頭
        // rustdoc「イシュー #2044 の shadcn/ui 突合」節参照）。`isolation:
        // isolate` は badge の `z-index: 1`（下記 `badge` base）による
        // 重ね順をこの root 配下に閉じ込めるための stacking context
        // 生成である。root は既定で `position: relative` のみを持ち
        // `z-index` を持たないため stacking context を作らず、
        // `AvatarStack::Group` 内で `stacked` と `with_badge` を併用すると
        // 先行 Avatar の badge が後続 Avatar の画像より手前に描画され、
        // 所属する Avatar の重なり順から逸脱していた（PR #2222
        // codex-review P1 指摘）。
        .variant(
            AvatarBadgeOverlay::Badge,
            "root",
            vec![decl("overflow", "visible"), decl("isolation", "isolate")],
        )
        // `badge` base: 右下の絶対配置ドット。`color`/`background` は
        // [`AvatarBadgeProps::palette`] の palette variant（下記）が
        // `--fandhe-palette`/`--fandhe-palette-fg` を供給する前提。
        .base(
            "badge",
            vec![
                decl("position", "absolute"),
                decl("right", "0"),
                decl("bottom", "0"),
                // root からの局所的な重ね順（`--fandhe-z-index-*` の
                // ページ全体スケールは持たない小さな整数値。
                // `color_picker.rs`/`segment_group.rs` 等の局所 z-index と
                // 同じ慣例、本モジュール冒頭 rustdoc「イシュー #2044」節）。
                decl("z-index", "1"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
                decl("box-shadow", "0 0 0 2px var(--fandhe-color-bg)"),
                decl("user-select", "none"),
            ],
        )
        // badge size 5 段（shadcn 8/10/12px を Xs/Sm/Md へ写像、Lg/Xl は
        // 既存 space スケール段へ外挿。`--fandhe-space-3-5` が存在しない
        // ための判断、本モジュール冒頭 rustdoc 参照）。root の `size` 軸と
        // 同じ (axis, value) を別 slot（`badge`）へ再登録するだけであり、
        // `default_variant(Size::Md)`（root 用、既存登録済み）はそのまま
        // badge にも適用される。
        .variant(
            Size::Xs,
            "badge",
            vec![
                decl("width", "var(--fandhe-space-2)"),
                decl("height", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Sm,
            "badge",
            vec![
                decl("width", "var(--fandhe-space-2-5)"),
                decl("height", "var(--fandhe-space-2-5)"),
            ],
        )
        .variant(
            Size::Md,
            "badge",
            vec![
                decl("width", "var(--fandhe-space-3)"),
                decl("height", "var(--fandhe-space-3)"),
            ],
        )
        .variant(
            Size::Lg,
            "badge",
            vec![
                decl("width", "var(--fandhe-space-4)"),
                decl("height", "var(--fandhe-space-4)"),
            ],
        )
        .variant(
            Size::Xl,
            "badge",
            vec![
                decl("width", "var(--fandhe-space-5)"),
                decl("height", "var(--fandhe-space-5)"),
            ],
        );

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        // badge は root の子要素のため root の `--fandhe-palette*` custom
        // property が継承される（既定 Neutral のまま灰色ドットになる）のを
        // 避けるため、badge slot 自身にも palette variant を登録する
        // （本モジュール冒頭 rustdoc「イシュー #2044 の shadcn/ui 突合」節）。
        recipe = recipe.variant(palette, "badge", palette_scale_declarations(palette));
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
/// 応じたクラスを付与する唯一のパーツ（`drop_class_attr` により呼び出し側
/// の `class` は除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::avatar::root`] へ委譲する。
///
/// イシュー #2044: `props.stacked`/`props.with_badge` が `true` のときのみ
/// `selection` へ `("stack", "stacked")`/`("overlay", "badge")` を追加する
/// （`crate::button::assemble` の `icon_only` 条件付き push と同型）。両方
/// `false`（既定）のときは選択列・`class` 出力が変更前と完全に一致する。
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
    let mut selection: Vec<(&str, &str)> = vec![
        ("size", props.size.value()),
        ("shape", props.shape.value()),
        ("variant", props.variant.value()),
        ("color-palette", props.palette.value()),
    ];
    if props.stacked {
        selection.push((AvatarStack::Stacked.axis(), AvatarStack::Stacked.value()));
    }
    if props.with_badge {
        selection.push((
            AvatarBadgeOverlay::Badge.axis(),
            AvatarBadgeOverlay::Badge.value(),
        ));
    }
    let class = recipe.variant_classes(&selection);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::avatar::root(merged, children)
}

/// pre-styled-only `group` パート（`<div>`、イシュー #2044）を組み立てる。
/// 複数の [`root`]（`stacked: true`）を重ねて表示するためのレイアウト専用
/// パートであり、headless-ui の anatomy には存在しない（本モジュール冒頭
/// rustdoc「イシュー #2044 の shadcn/ui 突合」節参照）。アプリケーション
/// ロジック（選択・展開などのイベント配線）は持たない。
///
/// [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] を直接呼び出す
/// （[`crate::dialog::footer`] と同型）ため、呼び出し側 `attrs` に含まれる
/// `data-scope`/`data-part` の偽装は headless 層が fail-closed に除去する。
///
/// `+N` の残数表示は独立パートを新設せず、`root(Subtle/Neutral, stacked:
/// true)` + [`fallback`]（例: `"+3"`）の組み合わせで表現する（本モジュール
/// 冒頭 rustdoc 参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::avatar;
///
/// let node = avatar::group(vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="avatar" data-part="group""#));
/// ```
#[must_use]
pub fn group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("group", "div", attrs, children)
}

/// pre-styled-only `badge` パート（`<span>`、イシュー #2044）を組み立てる。
/// [`root`]（`with_badge: true`）の右下に絶対配置される状態ドット
/// （shadcn `AvatarBadge` 相当）で、headless-ui の anatomy には存在しない。
/// `size`/`palette` に応じたクラスを付与する（`drop_class_attr` により
/// 呼び出し側の `class` は除去してから合成する）。
///
/// クラス組み立てには [`SlotRecipe::variant_classes`] を使わない
/// （`default_variant` を持つ `shape`/`variant`/`color-palette` 軸まで既定値
/// 補完されてしまい `fd-avatar--shape-circle`/`fd-avatar--variant-subtle`
/// が誤って付与されるため）。[`SlotRecipe::variant_class`] を `size`/
/// `palette` の 2 回だけ呼んで連結し、`class` をこの 2 クラスのみに限定
/// する（本モジュール冒頭 rustdoc「イシュー #2044 の shadcn/ui 突合」節）。
///
/// 子要素（アイコン等）は任意。badge 自体は装飾であるため、アクセシブル
/// ネームが必要な場合は呼び出し側が [`root`] の `aria-label` 等で供給する
/// こと（headless 層に用意された `role`/`aria-*` の自動付与はない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarBadgeProps};
///
/// let node = avatar::badge(&AvatarBadgeProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="avatar" data-part="badge""#));
/// ```
#[must_use]
pub fn badge<'a>(
    props: &AvatarBadgeProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let size_class = recipe.variant_class(props.size);
    let palette_class = recipe.variant_class(props.palette);
    let class = format!("{size_class} {palette_class}");
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("badge", "span", merged, children)
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

    // --- イシュー #2044: group / badge ---

    #[test]
    fn group_outputs_scope_and_part() {
        let html = render(&group(vec![], vec![]));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="group""#));
    }

    #[test]
    fn group_drops_caller_supplied_scope_and_part_spoofing() {
        let html = render(&group(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="group""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn badge_outputs_scope_and_part() {
        let html = render(&badge(&AvatarBadgeProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="badge""#));
    }

    #[test]
    fn badge_drops_caller_supplied_scope_and_part_spoofing() {
        let html = render(&badge(
            &AvatarBadgeProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="badge""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn default_avatar_props_class_is_unchanged_by_new_fields() {
        // イシュー #2044 の `stacked`/`with_badge` フィールド追加が既存の
        // `class` 出力（golden CSS 前提）を壊さないことの回帰。
        let html = render(&root(&AvatarProps::default(), vec![], vec![]));
        let class_start = html.find("class=\"").expect("class attr must exist") + "class=\"".len();
        let class_end = html[class_start..]
            .find('"')
            .map(|i| class_start + i)
            .unwrap();
        assert_eq!(
            &html[class_start..class_end],
            "fd-avatar--size-md fd-avatar--shape-circle fd-avatar--variant-subtle fd-avatar--color-palette-neutral"
        );
    }

    #[test]
    fn stacked_true_adds_stack_class() {
        let props = AvatarProps {
            stacked: true,
            ..AvatarProps::default()
        };
        let html = render(&root(&props, vec![], vec![]));
        assert!(html.contains("fd-avatar--stack-stacked"));
    }

    #[test]
    fn with_badge_true_adds_overlay_class() {
        let props = AvatarProps {
            with_badge: true,
            ..AvatarProps::default()
        };
        let html = render(&root(&props, vec![], vec![]));
        assert!(html.contains("fd-avatar--overlay-badge"));
    }

    #[test]
    fn badge_size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-avatar--size-xs"),
            (Size::Sm, "fd-avatar--size-sm"),
            (Size::Md, "fd-avatar--size-md"),
            (Size::Lg, "fd-avatar--size-lg"),
            (Size::Xl, "fd-avatar--size-xl"),
        ] {
            let props = AvatarBadgeProps {
                size,
                ..AvatarBadgeProps::default()
            };
            let html = render(&badge(&props, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn badge_palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-avatar--color-palette-accent"),
            (ColorPalette::Info, "fd-avatar--color-palette-info"),
            (ColorPalette::Success, "fd-avatar--color-palette-success"),
            (ColorPalette::Warning, "fd-avatar--color-palette-warning"),
            (ColorPalette::Danger, "fd-avatar--color-palette-danger"),
            (ColorPalette::Neutral, "fd-avatar--color-palette-neutral"),
        ] {
            let props = AvatarBadgeProps {
                palette,
                ..AvatarBadgeProps::default()
            };
            let html = render(&badge(&props, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn badge_class_contains_only_size_and_palette_classes() {
        // `variant_classes` を経由しないため `fd-avatar--shape-`/
        // `fd-avatar--variant-` の既定値補完が付与されないことを固定する
        // （本モジュール冒頭 rustdoc「イシュー #2044 の shadcn/ui 突合」節）。
        let html = render(&badge(&AvatarBadgeProps::default(), vec![], vec![]));
        let class_start = html.find("class=\"").expect("class attr must exist") + "class=\"".len();
        let class_end = html[class_start..]
            .find('"')
            .map(|i| class_start + i)
            .unwrap();
        assert_eq!(
            &html[class_start..class_end],
            "fd-avatar--size-md fd-avatar--color-palette-accent"
        );
    }

    #[test]
    fn badge_class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&badge(
            &AvatarBadgeProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_declares_group_and_badge_rules() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="avatar"][data-part="group"] {"#));
        assert!(css.contains(r#"[data-scope="avatar"][data-part="badge"] {"#));
        assert!(css.contains("fd-avatar--stack-stacked"));
        assert!(css.contains("fd-avatar--overlay-badge"));
        assert!(css.contains("overflow: visible;"));
    }

    #[test]
    fn group_children_script_payload_is_escaped() {
        let html = render(&group(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn badge_children_script_payload_is_escaped() {
        let html = render(&badge(
            &AvatarBadgeProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
