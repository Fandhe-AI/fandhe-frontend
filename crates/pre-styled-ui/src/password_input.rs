//! styled PasswordInput（headless ラッパー、イシュー #740、親 #520/#736）。
//!
//! `fandhe_frontend_headless_ui::password_input`（イシュー #740）の Label /
//! Control / Input / VisibilityTrigger / Indicator 5 anatomy パーツ
//! （headless 側は Root を含む 6 パーツ）をそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠は
//! [`crate::switch`]/[`crate::select`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`PasswordInput` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::switch::root`]・[`crate::avatar::root`] と同型）を本モジュールで
//! 再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::password_input::PasswordInput`] は
//! **あえて**再エクスポートしない（[`crate::switch`] の `Switch` 非再
//! エクスポートと同じ理由、イシュー #708 の判断を踏襲）。`PasswordInput` は
//! `.root(props, attrs, children)` 等の inherent メソッドを持つが、これは
//! headless 自由関数 `root` へそのまま委譲するのみで `size`/`palette`
//! variant クラスを一切付与しない未スタイルの実体である。本モジュールが
//! `PasswordInput` を丸ごと再エクスポートすると、呼び出し側が（styled 層の
//! つもりで）`password_input_instance.root(...)` を呼んでしまい、`size`/
//! `palette` が付与されず見た目が静かに崩れる事故を誘発する。状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::password_input::PasswordInput` を直接
//! import し、実際の描画は本モジュールの styled [`root`]（および再
//! エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # `data-state` 語彙について
//!
//! headless 層は表示切替を `"visible"`/`"hidden"` 語彙で表現する
//! （`crates/headless-ui/src/password_input.rs` 参照）。[`recipe`] の
//! `control`/`visibility-trigger` への状態連動規則もこの語彙に合わせて
//! `data-state="visible"` を条件とする。
//!
//! # `control` の `focus-within` リング（イシュー #740、`crate::radio_group`
//! と同じ判断）
//!
//! [`crate::recipe::StateCondition::FocusWithin`] を `control` へ登録する。
//! `input`（実フォーカスを受けるネイティブ `<input>`）は `control` の子孫
//! であり、hidden-input パターン（Switch 等）と異なり実際に視覚要素が
//! フォーカスを受けるため、`:focus-within` で祖先の枠へリングを伝播できる
//! （`data-focus-visible` の付け外し配線は不要）。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-password-input-height`/`-font-size`/`-padding-x` の root
//! スコープ custom property（CSS の通常のプロパティ継承により `control`/
//! `input` へ伝わる）経由で寸法を切り替える。`palette`（[`ColorPalette`]）は
//! 既存の [`crate::recipe::palette_scale_declarations`]（chakra-ui virtual token
//! 方式、#606）を `root` へ登録し、表示中の `visibility-trigger` の色・
//! `control` のフォーカスリング色を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・Accent
//! パレット相当のフォールバック値を書き、styled `root` を経由しない headless
//! 直接利用マークアップでも現行外観を維持する（fail-safe、`crate::lib`
//! rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - クライアント側の click → dispatch 配線（`fandhe-frontend-wasm-full`）。
//! - `examples/headless-pre-styled-ui` への PasswordInput 追加（#608/#609 と
//!   同じ後続分離、crates.io 版依存のため公開後にしか追随できない）。
//!
//! # スタイル調整 (1/2): 入力枠と可視切り替えトリガー（イシュー #1487、親
//! #1486、`docs/design/pre-styled-ui-interaction-visual-language.md`）
//!
//! Phase 2 で確立済みの共通ビジュアル言語ヘルパ（[`crate::recipe`]）へ
//! `control`/`visibility-trigger` を追随させた。兄弟イシュー #1488（2/2）が
//! `indicator`（強度インジケータ）と `Size` variant 群を担当するため、本
//! イシューはそれらに一切触れない。
//!
//! - **色**: `control` の `border-radius` を生リテラル `0.375rem` から
//!   `var(--fandhe-radius-md)` トークン参照へ置換した（combobox #1744 と
//!   同形）。
//! - **フォーカス**: `control` の `:focus-within` リングを手書きの
//!   `outline: 2px solid var(--fandhe-palette, ...)` から
//!   [`crate::recipe::focus_ring_declarations`]`(FocusRingColor::Palette,
//!   FocusRingOffset::Outside)` の canonical 形へ置換した。`palette`
//!   軸を公開する部品のため `Palette` を選ぶ（[`crate::radio_group`] と
//!   同じ判断）。`input`（実フォーカスを受けるネイティブ `<input>`）の
//!   `outline: none` はあえて維持する（祖先 `control` の canonical リング
//!   と併存する許容パターン、`docs/design/
//!   pre-styled-ui-focus-ring-and-size-conventions.md` §3 参照）。
//! - **hover**: `visibility-trigger`（クリック操作を担うゴーストアイコン
//!   ボタン）へ [`crate::recipe::hover_bg_muted`] +
//!   `.state(visibility-trigger, StateCondition::Hover,
//!   hover_surface_declarations())` を追加した。参照 3 サイト（chakra-ui /
//!   ark-ui / Radix）とも toggle をゴーストアイコンボタンとして hover
//!   背景を持つため。**`control` 自体へは hover を付けない**（テキスト
//!   入力面であり参照サイトもこの面自体への hover 表現を持たない、
//!   combobox 1/2 と同一判断）。
//! - **disabled**: `control`/`visibility-trigger` の `[data-disabled]` を
//!   生の `cursor: not-allowed; opacity: 0.5` から
//!   [`crate::recipe::disabled_declarations`] 経由へ置換した（`control`
//!   のみ）。`visibility-trigger` は `control` の子孫であり、自身にも
//!   `opacity: 0.5` を持つと `control` の減光と乗算され `0.25` へ二重
//!   減光する既存不整合があったため、`visibility-trigger` 側は
//!   `cursor: not-allowed` のみへ変更し `opacity` を持たせない
//!   （date-input #1469 と同型の判断）。`input` の `[data-disabled]` へは
//!   `disabled_declarations()` を付けない（同じ二重減光回避、祖先
//!   `control` の opacity 継承のみに委ねる）。
//! - **トランジション**: `control` の生 `transition: border-color 0.15s`
//!   を除去し、別 `.base` 呼び出しで
//!   `transition_declarations("border-color, background",
//!   MotionDuration::Fast)` を純追加した。`visibility-trigger` にも
//!   `transition_declarations("background, color", MotionDuration::Fast)`
//!   を新規追加した（combobox 1/2 のパターン）。
//!
//! 意図的に合わせなかった点（親 #1486 チェックリストの担当範囲外）:
//!
//! - **variant 軸**（chakra `outline`/`subtle`/`flushed` 相当）は追加しない
//!   （`root()` シグネチャ変更を伴う破壊的変更のため、Forms 家族横断の軸
//!   語彙判断を部品単独で先行しない）。
//! - **size / palette スケール**は触らない（size は #1488 の担当。palette
//!   は既存の virtual token 方式で参照サイト水準）。
//!
//! # スタイル調整 (2/2): indicator と Size variant 群（イシュー #1488、親
//! #1486）
//!
//! 1/2（#1487）が `control`/`visibility-trigger` を担当済みのため、本
//! イシューは `indicator`（可視状態インジケータ）と `Size` variant 群
//! （root スコープ custom property 定義）のみを担当する。
//!
//! - **`strength-*`（強度メーター）は実装しない**: イシュータイトルの
//!   「強度インジケータ」について、現行の anatomy（headless-ui /
//!   pre-styled-ui とも）に強度メーター（chakra-ui の
//!   `PasswordStrengthMeter` 相当）のパートは存在しない。存在するのは
//!   可視状態インジケータの `indicator` パート（`aria-hidden="true"`
//!   固定の装飾用 span、`data-state="visible"/"hidden"` を持つ）のみで
//!   ある。強度メーター相当のパート新設は headless-ui の anatomy 変更で
//!   あり本イシューの対象ファイル（本モジュール + golden テスト）の
//!   範囲外のため、既存 open イシュー #1614（headless-ui password-input
//!   の anatomy / `data-*` 突合）へ判断を委ねる（新規イシュー起票は
//!   しない）。本イシューでは既存 `indicator` パートのスタイル是正 +
//!   Size variant のトークンスケール移行に限定する。
//! - **サイズ（Size variant 群）**: root スコープ custom property
//!   （`--fandhe-password-input-height`/`-padding-x`/`-font-size`）の値を、
//!   部品ローカルの生 rem リテラルから、イシュー #1678 で確立済みの共通
//!   size トークン（`--fandhe-size-control-height/padding-x/font-size-*`、
//!   `crate::theme::DEFAULT_SIZES`）参照へ置換した。input（#1482）・
//!   native-select（#1763）と同一のフォールバック付き参照形
//!   （`var(--fandhe-size-control-height-xs, 2rem)` 等）を用いる。
//!   xs の高さが 1.5rem → 2rem、xl が 3.5rem → 3rem 等、寸法が共通
//!   スケールへ揃う視覚変更を意図的に行う（input #1482 と同一判断）。
//!   root スコープ custom property へ書き込み CSS の通常のプロパティ
//!   継承で `control`/`input` へ伝える機構自体は password-input 固有の
//!   正当な設計のため維持し、値のみを置換した。base 規則側のフォール
//!   バックも `var(--fandhe-password-input-height, var(--fandhe-size-control-height-md,
//!   2.5rem))` の形へ、共通トークン→固定値の 2 段チェーンとして整合
//!   させた（styled `root` を経由しない headless 直接利用時の fail-safe
//!   を md 相当で維持する）。
//! - **状態（data-*）**: `indicator` に
//!   `.state("indicator", StateCondition::AttrEq("data-state", "visible"), ...)`
//!   を追加し、`visibility-trigger` の同名規則とそろえた。
//! - **トランジション**: 別 `.base` 呼び出しで
//!   `transition_declarations("color", MotionDuration::Fast)` を純追加
//!   した（combobox #1744 パターン踏襲）。
//! - **hover / フォーカス / disabled**: `indicator` は非インタラクティブ
//!   （`aria-hidden` 固定の装飾 span）のため付与しない。disabled は
//!   祖先 `control` の opacity 0.5 継承のみに委ね、二重減光を作らない
//!   （1/2 の `visibility-trigger` と同じ判断）。
//! - **variant 軸**: 1/2 と同じ理由（Forms 家族横断の語彙判断を部品
//!   単独で先行しない）で見送りを継承する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// `PasswordInput` 状態機械・headless 自由関数 `root` はあえて再エクスポート
// しない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::password_input::PasswordInput` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::password_input::{
    control, indicator, input, label, visibility_trigger, PasswordAutocomplete,
    PasswordInputAction, PasswordInputProps,
};

/// headless `password_input` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/password_input.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "visibility-trigger",
    "indicator",
];

/// この styled PasswordInput の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("password-input", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl(
                    "height",
                    "var(--fandhe-password-input-height, var(--fandhe-size-control-height-md, 2.5rem))",
                ),
                decl(
                    "padding",
                    "0 var(--fandhe-password-input-padding-x, var(--fandhe-size-control-padding-x-md, 1rem))",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        // 別 `.base` 呼び出しでの純追加（combobox #1744 の「既存 base
        // ブロックを書き換えない」パターンを踏襲する）。生の
        // `transition: border-color 0.15s` を motion トークン経由へ置換。
        .base(
            "control",
            transition_declarations("border-color, background", MotionDuration::Fast),
        )
        .state(
            "control",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "control",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "control",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .base(
            "input",
            vec![
                decl("flex", "1"),
                decl("border", "none"),
                decl("background", "transparent"),
                // 祖先 `control` の `:focus-within` canonical リング
                // （上記）と併存する許容パターン
                // （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
                // §3 参照）。実フォーカスを受けるネイティブ `<input>` 自体
                // のブラウザ既定アウトラインのみを消し、視覚的なリングは
                // 祖先 `control` 側が一枚で担う。
                decl("outline", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("padding", "0"),
                decl(
                    "font-size",
                    "var(--fandhe-password-input-font-size, var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md)))",
                ),
            ],
        )
        // `input[data-disabled]` へは `disabled_declarations()` を付けない
        // （祖先 `control` の `[data-disabled]` opacity 0.5 継承との二重
        // 減光回避、date-input #1469 と同型の判断。本モジュール冒頭
        // rustdoc「スタイル調整」節参照）。
        .base(
            "visibility-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("margin-left", "var(--fandhe-space-1)"),
                hover_bg_muted(),
            ],
        )
        // 別 `.base` 呼び出しでの純追加（combobox #1744 と同型）。
        .base(
            "visibility-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "visibility-trigger",
            StateCondition::AttrEq("data-state", "visible"),
            vec![decl(
                "color",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        .state(
            "visibility-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // `cursor: not-allowed` のみ（`opacity` を持たせない）。trigger は
        // `control`（disabled 時 opacity 0.5）の子孫であり、自身にも
        // opacity を持たせると 0.25 へ二重減光する既存不整合の是正
        // （date-input #1469 と同型、本モジュール冒頭 rustdoc「スタイル
        // 調整」節参照）。
        .state(
            "visibility-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
            ],
        )
        // 別 `.base` 呼び出しでの純追加（combobox #1744・visibility-trigger
        // 上記と同型）。`indicator` は装飾用の可視状態アイコンであり、
        // `visibility-trigger` のクリックに追随して即座に切り替わるが、
        // motion トークン経由のトランジションを与え唐突な変化を避ける
        // （イシュー #1488、本モジュール冒頭 rustdoc「スタイル調整 (2/2)」
        // 節参照）。
        .base(
            "indicator",
            transition_declarations("color", MotionDuration::Fast),
        )
        // `visibility-trigger` の同名状態規則（上記）とそろえ、可視状態を
        // palette 色で明示する。`visibility-trigger` の子孫として配置
        // される想定のため実質的には色の継承と同値になるが、`indicator`
        // を単独配置する呼び出し側にも同じ視覚効果を保証する
        // （イシュー #1488）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "visible"),
            vec![decl(
                "color",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        // `indicator` は `aria-hidden="true"` 固定の非インタラクティブな
        // 装飾要素であり、hover / フォーカス / disabled の状態規則は
        // 意図的に付与しない。disabled 時の減光は祖先 `control` の
        // `[data-disabled]`（opacity 0.5）への継承のみに委ね、
        // `visibility-trigger` と同じ二重減光回避の判断を踏襲する
        // （イシュー #1488、本モジュール冒頭 rustdoc「スタイル調整 (2/2)」
        // 節参照）。
        // size（イシュー #1678 の `--fandhe-size-control-height/padding-x/
        // font-size-*` 共通トークンへ移行。native-select #1763・input #1482
        // と同型のフォールバック付き参照形。root スコープ custom property
        // へ書き込み、CSS の通常のプロパティ継承で `control`/`input` へ
        // 伝える機構自体は password-input 固有の正当な設計のため維持し、
        // 値だけを部品ローカルの生 rem リテラルから共通スケール参照へ
        // 置換する。イシュー #1488（本モジュール冒頭 rustdoc「スタイル
        // 調整 (2/2)」節参照）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-password-input-height",
                    "var(--fandhe-size-control-height-xs, 2rem)",
                ),
                decl(
                    "--fandhe-password-input-padding-x",
                    "var(--fandhe-size-control-padding-x-xs, 0.625rem)",
                ),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs))",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-password-input-height",
                    "var(--fandhe-size-control-height-sm, 2.25rem)",
                ),
                decl(
                    "--fandhe-password-input-padding-x",
                    "var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                ),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-password-input-height",
                    "var(--fandhe-size-control-height-md, 2.5rem)",
                ),
                decl(
                    "--fandhe-password-input-padding-x",
                    "var(--fandhe-size-control-padding-x-md, 1rem)",
                ),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-password-input-height",
                    "var(--fandhe-size-control-height-lg, 2.75rem)",
                ),
                decl(
                    "--fandhe-password-input-padding-x",
                    "var(--fandhe-size-control-padding-x-lg, 1.25rem)",
                ),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg))",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-password-input-height",
                    "var(--fandhe-size-control-height-xl, 3rem)",
                ),
                decl(
                    "--fandhe-password-input-padding-x",
                    "var(--fandhe-size-control-padding-x-xl, 1.5rem)",
                ),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl))",
                ),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

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

/// この styled PasswordInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。実体は
/// [`fandhe_frontend_headless_ui::password_input::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::password_input::{PasswordAutocomplete, PasswordInputProps};
/// use fandhe_frontend_pre_styled_ui::password_input;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let props = PasswordInputProps {
///     id: "login-password",
///     disabled: false,
///     readonly: false,
///     invalid: false,
///     required: false,
///     autocomplete: PasswordAutocomplete::CurrentPassword,
/// };
/// let node = password_input::root(Size::Md, ColorPalette::Accent, false, &props, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="password-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    visible: bool,
    props: &PasswordInputProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::password_input::root(visible, props, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::password_input::PasswordInput;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    fn default_props(id: &str) -> PasswordInputProps<'_> {
        PasswordInputProps {
            id,
            disabled: false,
            readonly: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="password-input"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_control_to_invalid_and_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="control"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="control"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}"#
        ));
    }

    #[test]
    fn stylesheet_links_visibility_trigger_to_visible_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="visibility-trigger"][data-state="visible"] {
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}"#
        ));
    }

    #[test]
    fn control_focus_within_uses_canonical_palette_focus_ring() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="control"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}"#
        ));
        assert!(!css.contains("outline: 2px solid var(--fandhe-palette"));
    }

    #[test]
    fn control_border_radius_uses_radius_token() {
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-md)"));
        assert!(!css.contains("border-radius: 0.375rem"));
    }

    #[test]
    fn visibility_trigger_hover_rule_is_wrapped_in_hover_media_query() {
        let css = stylesheet();
        assert!(css.contains(
            r#"@media (hover: hover) {
  [data-scope="password-input"][data-part="visibility-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}"#
        ));
    }

    #[test]
    fn visibility_trigger_disabled_has_cursor_only_no_double_dimming() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="visibility-trigger"][data-disabled] {
  cursor: not-allowed;
}"#
        ));
    }

    #[test]
    fn control_and_trigger_declare_motion_token_transitions() {
        let css = stylesheet();
        assert!(css.contains("transition-property: border-color, background;"));
        assert!(css.contains("transition-property: background, color;"));
        assert!(
            css.matches("transition-duration: var(--fandhe-motion-duration-fast);")
                .count()
                >= 2
        );
        assert!(!css.contains("transition: border-color 0.15s"));
    }

    #[test]
    fn stylesheet_size_variants_use_control_tokens() {
        // イシュー #1488: 各 size が #1678 の共通 control トークンへ移行した
        // ことを固定（native-select #1763・input #1482 と同型の 3 点セット）。
        let css = stylesheet();
        for suffix in ["xs", "sm", "md", "lg", "xl"] {
            assert!(
                css.contains(&format!("--fandhe-size-control-height-{suffix}")),
                "height token missing for {suffix} -> {css}"
            );
            assert!(
                css.contains(&format!("--fandhe-size-control-padding-x-{suffix}")),
                "padding-x token missing for {suffix} -> {css}"
            );
            assert!(
                css.contains(&format!("--fandhe-size-control-font-size-{suffix}")),
                "font-size token missing for {suffix} -> {css}"
            );
        }
    }

    #[test]
    fn stylesheet_size_variants_no_longer_use_raw_rem_literals() {
        // 是正前の部品ローカル生 rem リテラル（xs の高さ/padding-x 等）が
        // 一切残っていないことを固定する。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-password-input-height: 1.5rem"));
        assert!(!css.contains("--fandhe-password-input-height: 2rem;\n"));
        assert!(!css.contains("--fandhe-password-input-padding-x: 0.25rem"));
        assert!(!css.contains("--fandhe-password-input-padding-x: 0.5rem;\n"));
        assert!(!css.contains("--fandhe-password-input-height: 3.5rem"));
        assert!(!css.contains("--fandhe-password-input-padding-x: 1.25rem;\n"));
    }

    #[test]
    fn indicator_visible_state_uses_palette_color() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="indicator"][data-state="visible"] {
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}"#
        ));
    }

    #[test]
    fn indicator_declares_motion_token_transition() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="indicator"] {
  display: inline-flex;
  align-items: center;
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="indicator"] {
  transition-property: color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}"#
        ));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-password-input--size-md"));
        assert!(html.contains("fd-password-input--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let props = default_props("pw");
        for (size, class) in [
            (Size::Xs, "fd-password-input--size-xs"),
            (Size::Sm, "fd-password-input--size-sm"),
            (Size::Md, "fd-password-input--size-md"),
            (Size::Lg, "fd-password-input--size-lg"),
            (Size::Xl, "fd-password-input--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                &props,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let props = default_props("pw");
        for (palette, class) in [
            (
                ColorPalette::Accent,
                "fd-password-input--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-password-input--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-password-input--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-password-input--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-password-input--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-password-input--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, false, &props, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let props = default_props("pw");
        let html = render(&label(
            &props,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_input_never_outputs_value_attribute() {
        let props = default_props("pw");
        let html = render(&input(false, &props, vec![]));
        assert!(!html.contains("value="));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_password_input_state_machine() {
        // `PasswordInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`PasswordInput` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ検証
        // する。
        let mut p = PasswordInput::default();
        assert!(!p.is_visible());

        let props = default_props("pw");
        let ssr_html = render(&p.root(&props, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="hidden""#));

        assert!(dispatch(&mut p, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-visible="visible""#));

        let restored = PasswordInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }
}
