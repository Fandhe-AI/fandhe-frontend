//! styled Tabs（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size`/`color-palette` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::tabs`（イシュー #528）は Root / List /
//! Trigger / Content / Indicator（#601、opt-in）の 5 anatomy パーツを [`tabs`]
//! 単一の合成関数として組み立てる（パーツごとの自由関数を持たない、他 4
//! コンポーネントとの非対称点）。イシュー #729 以前は headless 側に root への
//! attrs 注入点自体が存在せず本モジュールは headless `tabs` をそのまま
//! 再エクスポートしていたが、`size`/`color-palette` variant クラスを root へ
//! 付与するために headless 側へ [`fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`]
//! （非破壊的な追加関数）が新設された（`crates/headless-ui/src/tabs.rs`
//! rustdoc 参照）。本モジュールはそれを呼ぶ styled [`tabs`] を新たに定義する。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、イシュー #729）
//!
//! headless 自由関数 `tabs` と名前が衝突するため、`pub use ...::*` ではなく
//! [`TabsProps`]/[`TabItem`]/[`ActivationMode`] のみを選択的に再エクスポート
//! する（[`crate::switch`]・[`crate::avatar`] と同型の判断）。headless 自由
//! 関数 `tabs`/`tabs_with_root_attrs`（未スタイル・variant クラス非付与）が
//! 必要な呼び出し側は
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::tabs` を
//! 直接 import すること。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! Tabs は `data-state` に `"open"`/`"closed"` ではなく `"active"`/`"inactive"`
//! 語彙を使う（`crates/headless-ui/src/tabs.rs` の `DATA_STATE_ACTIVE`/
//! `DATA_STATE_INACTIVE`）。選択中の `trigger` を強調する CSS を
//! [`crate::recipe::SlotRecipe::state`]（イシュー #643）経由で [`recipe`] へ
//! 登録する（`serialize_rule` を直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `trigger` は roving tabindex（`.claude/rules` 外部だが headless 層 tabs の
//! キーボードナビゲーション実装）でフォーカス移動するボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を [`recipe`]
//! へ登録する。
//!
//! # `size`/`color-palette` variant（イシュー #729）
//!
//! `size`（[`Size`]）は root へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-tabs-trigger-padding`/`-content-padding` の root スコープ CSS
//! custom property（通常の CSS 継承により `trigger`/`content` へ伝わる。
//! `root` は両パーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で寸法を切り替える。`color-palette`（[`ColorPalette`]、tabs のみが
//! 対応する第 2 軸）は既存の [`crate::recipe::palette_declarations`]
//! （chakra-ui virtual token 方式、#606）を root へ登録し、選択中 trigger の
//! 強調色（`border-bottom-color`）を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・Accent
//! パレット相当のフォールバック値を書き、styled `root`/`tabs` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の変体統一方針」節参照）。
//!
//! # 参考サイト基準への調整（イシュー #1542）
//!
//! 参照サイト（chakra-ui / Radix Themes / Radix Primitives / ark-ui）との
//! 視覚比較（issue #1542 コメントに転記した 7 軸チェック）を踏まえ、以下を
//! 是正した:
//!
//! - **サイズ**: [`recipe`] の `trigger` base へ `font-size:
//!   var(--fandhe-tabs-font-size, var(--fandhe-font-font-size-sm))` を新設
//!   し、`size` variant（Xs〜Xl）が `--fandhe-tabs-font-size` を段対応で
//!   定義するようにした（`crate::pagination`/`crate::tab_nav` と同一の段
//!   対応）。font-size が size に連動していなかった不足を解消する。
//! - **hover**: `trigger` へ [`crate::recipe::StateCondition::Hover`] +
//!   [`crate::recipe::hover_surface_declarations`] を追加した
//!   （`--fandhe-hover-bg` は [`crate::recipe::hover_bg_muted`]）。
//! - **disabled**: `trigger` へ `[data-disabled]`
//!   （[`crate::recipe::StateCondition::Attr`]）+
//!   [`crate::recipe::disabled_declarations`] を追加した。headless が
//!   `disabled=""` と併せて出力する属性であり、従来スタイル未反映だった。
//! - **フォーカスリング**: 直書き `outline: 2px solid
//!   var(--fandhe-color-accent)` を
//!   [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Palette`:
//!   本部品は `color-palette` 軸を公開するため）へ canonical 化し、
//!   `content`（`tabindex="0"` の tabpanel）にも同じリングを追加した
//!   （従来 `content` にはリングがなかった）。
//! - **トランジション**: `trigger` へ
//!   [`crate::recipe::transition_declarations`]（`"color, background,
//!   border-color"`、[`crate::recipe::MotionDuration::Fast`]）を追加した。
//!   `prefers-reduced-motion` は [`crate::theme::Theme::to_css`] の
//!   duration 一括 0ms 化で自動的に尊重される。
//! - **余白・角丸**: `trigger` に上側のみの角丸（`border-radius:
//!   var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0
//!   0`）を追加し、hover 面が上側だけ丸くなる参照サイトの見た目に合わせた
//!   （[`crate::tab_nav`] と同型）。また `margin-bottom: -1px` を追加し、
//!   選択中 trigger の 2px 下線を `list` の 1px 罫線へ重ねる（重ねないと
//!   3px に積み上がって見える不足の是正、chakra `line` variant の
//!   `--indicator-offset-y: -1px` 相当）。
//! - **`data-orientation="vertical"`**: headless が root/list/trigger/
//!   content へ出力するが視覚差がなかったため、`root`/`list`/`trigger`/
//!   `content` それぞれへ縦並び規則を追加した（下線 → 右罫線、行内 →
//!   列方向の配置転換）。選択中 trigger の強調色は
//!   [`crate::recipe::StateCondition::AttrEqAll`] で
//!   `[data-state="active"][data-orientation="vertical"]` を条件化し、
//!   縦並び時は右側の強調線へ切り替える。
//! - **`inline-flex` 化**: `trigger` を `inline-flex` + `gap` へ変更し、
//!   アイコン + ラベルの並びに対応した（chakra のアイコン付きタブ運用）。
//!
//! **意図的に合わせなかった点**（根拠を記録し、再評価は
//! `docs/policy/intentional-non-adoption.md` の評価軸に従う）:
//!
//! - **variant 軸（chakra `line`/`subtle`/`enclosed`/`outline`/`plain`）は
//!   追加しない**: [`tabs`] の公開シグネチャへ引数追加する破壊的変更に
//!   なる。Radix Themes Tabs は variant を持たず参照軸間で語彙が収斂して
//!   いない（`docs/design/pre-styled-ui-size-and-color-palette-axes.md`
//!   §7 の Forms 家族判断と同じ根拠）。`size` × `color-palette` で参照
//!   サイトの既定（line）表現は再現済み。
//! - **`indicator` パーツの装飾は追加しない**: headless は
//!   `--left`/`--top`/`--width`/`--height` を `0px` 固定で出力し、
//!   wasm-full 側に実測して更新する配線がまだない
//!   （`crates/headless-ui/src/tabs.rs` `INDICATOR_STYLE_INITIAL`
//!   rustdoc）。CSS を足しても幅 0 で不可視の dead CSS になり、将来配線
//!   された際には active trigger の下線と二重線になるため、配線実装時に
//!   あわせて設計する。
//! - **active 時の `font-weight` 変化（Radix Themes 方式）は採らない**:
//!   ページ内切り替えで幅が揺れる。代わりに全 trigger を最初から
//!   medium にする（chakra 方式、`trigger` base の `font-weight`）。
//! - **`box-shadow` によるフォーカスリング / surface 表現は採らない**:
//!   イシュー #1424 の `outline` 統一方針（`forced-colors` 対応）に従う。
//! - **Radix の内側 `span` による hover 面**: anatomy を増やすため採らず、
//!   `trigger` 全面へ上側角丸の hover 面を当てる（`tab_nav` と同型）。
//! - **`transition` の対象に `transform`/`box-shadow` を含めない**:
//!   変化させるプロパティがないため。
//!
//! # `shared_tab_*` ヘルパの廃止（イシュー #996 → #1542）
//!
//! かつて `tabs`（`list`/`trigger`パーツ）/`tab_nav`（`root`/`link` パーツ）
//! が見た目の基底宣言を `pub(crate) fn shared_tab_{list,item,item_active}_
//! declarations` として共有していた（イシュー #996）。`tab_nav` はイシュー
//! #1541 で共有をやめ自前の宣言列を持つよう独立済みであり（`tab_nav.rs`
//! 冒頭 rustdoc「参考サイト基準への調整（イシュー #1541）」節参照）、本
//! イシュー時点で `shared_tab_*` を参照するモジュールは本モジュール自身
//! のみだった（`git grep shared_tab` で確認済み）。本イシューで上記
//! ビジュアル是正に伴い宣言列自体が `tabs` 固有の内容へ発展したため、
//! 3 関数を [`recipe`] へインライン化して削除した。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `tabs`/`tabs_with_root_attrs` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体が必要な呼び出し側は
// `fandhe_frontend_headless_ui::tabs` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::tabs::{ActivationMode, TabItem, TabsProps};
// `TabsProps.orientation` フィールドの型（`data_attrs` モジュール由来のため
// 上記選択的再エクスポートでは到達しない）。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して `tabs()` を呼び出せることを
// 保証するための明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;

/// headless `tabs` anatomy の `data-part` 一覧（`crates/headless-ui/src/tabs.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "list", "trigger", "content", "indicator"];

/// この styled Tabs の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
///
/// イシュー #1542: 旧 `shared_tab_*` ヘルパ（モジュール冒頭 rustdoc「`shared_tab_*`
/// ヘルパの廃止」節参照）をインライン化した上で、参考サイト基準の是正
/// （hover・disabled・フォーカスリング canonical 化・トランジション・
/// vertical 対応）を追加した。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("tabs", SLOTS)
        .base(
            "list",
            vec![
                decl("display", "flex"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .base(
            "trigger",
            vec![
                // イシュー #1542: アイコン + ラベルの並びに対応する
                // （chakra のアイコン付きタブ運用）。
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4))",
                ),
                // イシュー #1542: font-size が size variant に連動していな
                // かった不足を是正（`crate::pagination`/`crate::tab_nav` と
                // 同型）。
                decl(
                    "font-size",
                    "var(--fandhe-tabs-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // イシュー #1542: 選択切り替えで幅が揺れないよう、全 trigger
                // を最初から medium にする（chakra 方式、モジュール冒頭
                // rustdoc「意図的に合わせなかった点」節参照）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("white-space", "nowrap"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("border", "0"),
                decl("border-bottom", "2px solid transparent"),
                // イシュー #1542: 選択中 trigger の 2px 下線を `list` の 1px
                // 罫線へ重ねる（重ねないと 3px に積み上がって見える不足の
                // 是正、chakra `line` variant の `--indicator-offset-y: -1px`
                // 相当）。
                decl("margin-bottom", "-1px"),
                // イシュー #1542: hover 面が上側だけ丸くなる参照サイトの
                // 見た目に合わせる（`crate::tab_nav` と同型）。
                decl(
                    "border-radius",
                    "var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0",
                ),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ],
        )
        .base(
            "trigger",
            transition_declarations("color, background, border-color", MotionDuration::Fast),
        )
        .base(
            "content",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0)",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #551 受け入れ条件: 選択中の `trigger` を強調する。
        // イシュー #729: 強調色は `color-palette` variant（root へ登録される
        // `--fandhe-palette`）経由で切り替わる。フォールバックは Accent 相当。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "active"),
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "border-bottom-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "inactive"),
            vec![decl("display", "none")],
        )
        // イシュー #1542: headless が `disabled=""` と併せて出力する
        // `data-disabled` に視覚差がなかった不足を是正する。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #643: キーボード操作時のみのフォーカスリング。
        // イシュー #1542: 直書き `outline` を canonical 化し
        // （`FocusRingColor::Palette`: `color-palette` 軸を公開する部品の
        // ため）、`content`（`tabindex="0"` の tabpanel）にも同じリングを
        // 追加した（従来 `content` にはリングがなかった不足の是正）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .state(
            "content",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // イシュー #1542: `data-orientation="vertical"`（headless が
        // root/list/trigger/content へ出力するが視覚差がなかった不足）。
        // `align-items` は既定値（`stretch`）のまま明示しない: `flex-start`
        // を指定すると `list`/`content` が root の高さへストレッチされず
        // 自身の内容量分の高さしか持たなくなり、`list` に付けた
        // `border-inline-end`（区切り線）が `content`（パネル）全体の高さに
        // 沿わずタブトリガー分の高さで止まってしまう（レビュー指摘、
        // Bugbot「Vertical divider does not span content」）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("display", "flex")],
        )
        .state(
            "list",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "column"),
                decl("border-bottom", "0"),
                decl(
                    "border-inline-end",
                    "1px solid var(--fandhe-color-border)",
                ),
            ],
        )
        .state(
            "trigger",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("justify-content", "flex-start"),
                decl("border-bottom", "0"),
                decl("margin-bottom", "0"),
                decl("border-inline-end", "2px solid transparent"),
                decl("margin-inline-end", "-1px"),
                // イシュー #1542 codex-review 指摘（P2）: 物理方向の
                // `border-radius` 短縮記法（TL/TR/BR/BL）は RTL でも
                // 左側が丸まったままになり、inline-start 側へ追随しない。
                // `crate::toggle_group` と同型の論理プロパティ
                // （`border-start-start-radius`/`border-end-start-radius`）
                // へ置き換え、inline-end 側は明示的に角丸なしとする。
                decl("border-start-start-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                decl("border-end-start-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                decl("border-start-end-radius", "0"),
                decl("border-end-end-radius", "0"),
            ],
        )
        .state(
            "trigger",
            StateCondition::AttrEqAll(&[
                ("data-state", "active"),
                ("data-orientation", "vertical"),
            ]),
            vec![decl(
                "border-inline-end-color",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex", "1"),
                decl(
                    "padding",
                    "0 var(--fandhe-tabs-content-padding-inline, var(--fandhe-space-4))",
                ),
            ],
        )
        // イシュー #1542: hover 背景・文字色変化（参照 3 サイト共通）。
        .state("trigger", StateCondition::Hover, {
            let mut decls = hover_surface_declarations();
            decls.push(decl("color", "var(--fandhe-color-fg)"));
            decls
        })
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。
        // イシュー #1681: Xs は Sm(1,3)→Md(2,4)→Lg(3,5) の等差進行を 1 段
        // 外挿した (0-5, 2)（`space-0`は未定義のため最小刻み `space-0-5`）。
        // イシュー #1542: `--fandhe-tabs-font-size`（`crate::pagination`/
        // `crate::tab_nav` と同一の段対応）・`--fandhe-tabs-content-padding-
        // inline`（vertical 時の content 横 padding。既存
        // `--fandhe-tabs-content-padding` は `<block> 0` 形式で意味を変え
        // られないため別変数を足した）を純追加した。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-0-5) var(--fandhe-space-2)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-2) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-3)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-3) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-3)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-4)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-4) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-4)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-5)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-5) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-5)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-4) var(--fandhe-space-6)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-6) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-6)"),
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

/// この styled Tabs が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled Tabs を組み立てる。`size`/`color-palette` に応じたクラスを root へ
/// 付与する唯一のパーツ。`tabs` は headless 層に呼び出し側 attrs を受け取る
/// 引数を持たない（[`TabsProps`]/`items` のみ、モジュール冒頭 rustdoc「root
/// への attrs 注入点」節参照）ため、他の styled 部品の `root`
/// （[`crate::class_attr::drop_class_attr`] で呼び出し側 `class` を除去して
/// から合成）とは異なり、生成した variant クラスをそのまま root の `class`
/// として渡す（`drop_class_attr` は不要）。実体は
/// [`fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`] へ委譲する
/// （選択状態の決定則・roving tabindex・XSS 不変条件は headless 層と完全に
/// 同一）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tabs::{self, ActivationMode, Orientation, TabItem, TabsProps};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = tabs::tabs(
///     Size::Md,
///     ColorPalette::Accent,
///     &TabsProps {
///         id: "t",
///         selected: "a",
///         orientation: Orientation::Horizontal,
///         activation_mode: ActivationMode::Automatic,
///         loop_focus: true,
///         indicator: false,
///     },
///     vec![TabItem {
///         value: "a",
///         trigger: vec![],
///         content: vec![],
///         disabled: false,
///     }],
/// );
/// assert!(render(&node).contains(r#"data-scope="tabs" data-part="root""#));
/// ```
#[must_use]
pub fn tabs(
    size: Size,
    palette: ColorPalette,
    props: &TabsProps<'_>,
    items: Vec<TabItem<'_>>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    // `tabs` は headless 層に呼び出し側 attrs を受け取る引数を持たない
    // （headless `tabs_with_root_attrs` の rustdoc 参照）ため、ここで
    // `drop_class_attr` を通す対象（呼び出し側 attrs）は存在しない。生成した
    // variant クラスをそのまま root_attrs として渡す。
    let root_attrs: Vec<(&str, &str)> = vec![("class", class.as_str())];
    fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs(props, root_attrs, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn item<'a>(value: &'a str) -> TabItem<'a> {
        TabItem {
            value,
            trigger: vec![],
            content: vec![],
            disabled: false,
        }
    }

    fn default_props<'a>(id: &'a str, selected: &'a str) -> TabsProps<'a> {
        TabsProps {
            id,
            selected,
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn styled_tabs_renders_with_headless_anatomy_attrs() {
        let props = default_props("t1", "one");
        let items = vec![item("one")];
        let html = render(&tabs(Size::Md, ColorPalette::Accent, &props, items));
        assert!(html.contains(r#"data-scope="tabs""#));
        assert!(html.contains(r#"data-part="list""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_active_and_inactive() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する（Tabs は
        // open/closed ではなく active/inactive 語彙を使う）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"][data-state="active"]"#));
        assert!(css.contains(r#"[data-scope="tabs"][data-part="content"][data-state="inactive"]"#));
    }

    #[test]
    fn ssr_selected_tab_reflects_active_data_state() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」。
        // Tabs は状態機械を持たないため（headless 側スコープ外）、SSR 側の
        // 静的選択状態が data-state="active"/"inactive" として決定的に
        // 描画されることを固定する。
        let props = default_props("t1", "one");
        let items = vec![item("one"), item("two")];
        let html = render(&tabs(Size::Md, ColorPalette::Accent, &props, items));
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="inactive""#));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。
        // イシュー #1542: 直書き outline を `focus_ring_declarations`
        // （`FocusRingColor::Palette`）へ canonical 化した。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));"
        ));
    }

    #[test]
    fn content_declares_focus_visible_ring() {
        // イシュー #1542: `content`（tabindex="0" の tabpanel）にもフォーカス
        // リングを追加した（従来は `trigger` のみだった不足の是正）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="content"]:focus-visible {"#));
    }

    #[test]
    fn trigger_hover_rule_is_collected_under_single_media_hover_block() {
        // イシュー #1542: hover 規則は `@media (hover: hover)` 配下へ集約
        // 出力される（タッチ端末の貼り付き対策）。末尾に 1 つだけ出ること
        // を固定する。
        let css = stylesheet();
        assert_eq!(css.matches("@media (hover: hover)").count(), 1);
        assert!(css
            .contains(r#"[data-scope="tabs"][data-part="trigger"]:hover:not([data-disabled]) {"#));
        assert!(css.trim_end().ends_with('}'));
    }

    #[test]
    fn disabled_trigger_declares_opacity_and_cursor() {
        // イシュー #1542: headless が `disabled=""` と併せて出力する
        // `data-disabled` に視覚差がなかった不足を是正する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_declares_transition_with_fast_duration() {
        // イシュー #1542: hover/active の色・背景・境界変化にトランジションを
        // 付ける（`prefers-reduced-motion` は `Theme::to_css` の duration 0ms
        // 化で自動対応）。
        let css = stylesheet();
        assert!(css.contains("transition-property: color, background, border-color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn size_variant_defines_font_size_custom_property() {
        // イシュー #1542: font-size が size に連動していなかった不足を是正
        // する。5 段すべてが `--fandhe-tabs-font-size` を定義すること。
        let css = stylesheet();
        assert_eq!(css.matches("--fandhe-tabs-font-size:").count(), 5);
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-xs);"));
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-sm);"));
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-md);"));
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-lg);"));
    }

    #[test]
    fn vertical_orientation_rules_are_registered_for_all_slots() {
        // イシュー #1542: `data-orientation="vertical"`（headless が
        // root/list/trigger/content へ出力するが視覚差がなかった不足）。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tabs"][data-part="root"][data-orientation="vertical"] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="tabs"][data-part="list"][data-orientation="vertical"] {"#)
        );
        assert!(css.contains(
            r#"[data-scope="tabs"][data-part="trigger"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="tabs"][data-part="content"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="tabs"][data-part="trigger"][data-state="active"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            "border-inline-end-color: var(--fandhe-palette, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn stylesheet_contains_no_raw_color_literals() {
        // イシュー #1542: 全ての色はトークン（`var(--fandhe-...)`）経由で
        // 参照し、生の色リテラル（16進・rgb()）を混入させない。
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
    }

    // --- イシュー #729: size/color-palette variant ---

    #[test]
    fn root_outputs_scope_and_part() {
        let props = default_props("t1", "one");
        let html = render(&tabs(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![item("one")],
        ));
        assert!(html.contains(r#"data-scope="tabs""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let props = default_props("t1", "one");
            let html = render(&tabs(size, ColorPalette::Accent, &props, vec![item("one")]));
            let expected_class = format!("fd-tabs--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn color_palette_variant_appends_class_to_root() {
        for palette in [
            ColorPalette::Accent,
            ColorPalette::Info,
            ColorPalette::Success,
            ColorPalette::Warning,
            ColorPalette::Danger,
            ColorPalette::Neutral,
        ] {
            let props = default_props("t1", "one");
            let html = render(&tabs(Size::Md, palette, &props, vec![item("one")]));
            let expected_class = format!("fd-tabs--color-palette-{}", palette.value());
            assert!(html.contains(&expected_class), "html={html}");
        }
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-tabs-trigger-padding"));
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        assert!(
            css.contains("padding: var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4));")
        );
        assert!(
            css.contains("padding: var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0);")
        );
    }

    #[test]
    fn active_trigger_border_color_consumes_fandhe_palette_with_accent_fallback() {
        let css = stylesheet();
        assert!(
            css.contains("border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));")
        );
    }
}
