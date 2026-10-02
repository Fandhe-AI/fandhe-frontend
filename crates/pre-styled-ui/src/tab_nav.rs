//! styled Tab Nav（イシュー #996、親 #520/#545、`docs/design/component-coverage-map.md`
//! §5 Part D・§9・§9.1（仮 ID 8-6）実装対象）。
//!
//! Radix Themes `Tab Nav`（`tab-nav`）相当の「見た目は tabs、意味論は素の
//! ナビゲーションリンク集合」という部品。`fandhe_frontend_headless_ui::tabs`
//! （イシュー #528）が持つ `role="tablist"`/`role="tab"` のパネル切り替え
//! 意味論をこの部品には適用しない。素の `<nav>`/`<a>` の暗黙 ARIA ロール
//! （`navigation`/`link`）のみを使い、現在ページは `aria-current="page"` で
//! 示す（[`crate::radio_card`]/[`crate::checkbox_card`] と同型に、本層で
//! 新規 anatomy `data-scope="tab-nav"` を定義し `crates/headless-ui/` へは
//! 一切手を入れない）。
//!
//! # `tabs` との差
//!
//! [`crate::tabs`] は `role="tablist"`/`role="tab"` を持つ**パネル切り替え
//! UI** であり、選択中パネルの表示/非表示を `data-state="active"`/
//! `"inactive"` で切り替える。本モジュールはページ遷移を伴う**ナビゲーション**
//! であり、パネルの概念を持たず `role` を一切出力しない。既存 `tabs` を
//! ページ遷移用途へ転用すると、スクリーンリーダー利用者へ「タブパネル」と
//! 誤って伝わる問題（`nav_list` が `role="menu"` 転用の意味論不整合を解消
//! したのと同型の問題）を、本モジュールの新設で解消する。
//!
//! # `nav_list` との差
//!
//! [`crate::nav_list`] は `nav > ul > li > a` の**縦方向の文書ナビ**
//! （サイドバー用途、`fandhe-frontend-docs-site::nav::sidebar` が消費）で
//! あり、リストマークアップと見出し（`heading`）パーツを持つ。本モジュール
//! は水平タブ外観の `root`/`link` 2 パーツのみで構成し、リストマークアップ
//! を持たない。
//!
//! # セキュリティ不変条件
//!
//! `href`/`aria-label`/`attrs`/children はすべて
//! [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] →
//! `fandhe_frontend_core::el` → `fandhe_frontend_core::render` の既定
//! エスケープ（REQ-1）を必ず経由する。`raw_html()` の新規使用なし、HTML
//! 文字列の直接組み立ても行わない。`href` の危険 URL スキーム（`javascript:`
//! 等）は core の許可リスト方式（deny-by-default）が属性ごと拒否する
//! （[`crate::link`]/`crates/headless-ui/src/link.rs` と同じ経路）。
//! `ROOT_RESERVED`/`LINK_RESERVED` は呼び出し側 `attrs` によるフレーム
//! ワーク固定キーのなりすましを fail-closed で除去する
//! （[`Anatomy::part`](fandhe_frontend_headless_ui::Anatomy::part) は
//! `data-scope`/`data-part` のみを守るため、それ以外の予約キー保護は本
//! モジュール自身の責務、[`crate::radio_card`] と同型の判断）。
//!
//! # 参考サイト基準への調整（イシュー #1541）
//!
//! 参照サイト（Radix Themes `TabNav` のみ。chakra-ui / Ark UI / Radix
//! Primitives には TabNav 相当が存在しない。Tabs は兄弟イシュー #1542 の
//! 対象）との視覚比較（issue #1541 コメントに転記した 7 軸チェック）を
//! 踏まえ、以下を是正した:
//!
//! - **`tabs.rs` 共有ヘルパからの独立**: 従来 `recipe` は
//!   `crate::tabs::shared_tab_{list,item,item_active}_declarations` を
//!   呼んでいたが、並列実行中の兄弟イシュー #1542（`tabs` のスタイル調整）
//!   が同ヘルパを変更する見込みのため、golden CSS の相互破壊を避ける目的で
//!   本イシューにて共有をやめ自前の宣言列を持つ（`tabs.rs`/
//!   `tests/tabs_css.rs` は本 PR で一切変更しない）。`tabs.rs` 側の
//!   3 ヘルパ rustdoc に残る「`tab_nav` が共有する」旨の記述は #1542 の
//!   編集範囲と重なるため本 PR では追随せず、`.claude/rules/
//!   out-of-scope-tracking.md` に従い別途記録する。
//! - **`size` 軸の新設（破壊的変更）**: [`root`] の第 1 引数へ
//!   [`crate::recipe::Size`]（Xs/Sm/Md/Lg/Xl、既定 Md）を追加した。
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 が
//!   本部品を「size 軸追加候補」として名指ししていたことに応える。padding
//!   の段進行は [`crate::tabs`] の size 進行と同一、font-size の段対応は
//!   [`crate::pagination`] と同一とし、Radix Themes TabNav の size 2 = 14px
//!   （sm）/ size 1 = 12px（xs）に整合させる。
//! - **hover**: [`crate::recipe::StateCondition::Hover`] +
//!   [`crate::recipe::hover_surface_declarations`] を追加した（`--fandhe-
//!   hover-bg` は [`crate::recipe::hover_bg_muted`]）。参照サイトは現在
//!   ページにも hover 背景を付けるため、`nav_list` の
//!   `HoverExcept`（現在リンクを hover 対象から除外する specificity 競合
//!   回避）は不要と判断した（現在リンクの `color` は既に `fg` であり
//!   hover 規則の `color: fg` と衝突しないため）。
//! - **フォーカスリング**: 直書き `outline` を
//!   [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Token`:
//!   本部品は `palette` 軸を持たない／`FocusRingOffset::Outside`）へ
//!   canonical 化した。
//! - **余白・角丸**: `link` に上側のみの角丸
//!   （`border-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0`）
//!   を追加した。下線（`border-bottom`）は直線のまま維持し、hover 面が
//!   上側だけ丸くなる参照サイトの見た目に合わせる。
//! - **現在ページの強調**: `[aria-current="page"]` へ
//!   `font-weight: var(--fandhe-font-font-weight-medium)` を追加した。
//! - **トランジション**: [`crate::recipe::transition_declarations`]
//!   （`"color, background, border-color"`、
//!   [`crate::recipe::MotionDuration::Fast`]）を追加した。
//!   `prefers-reduced-motion` は [`crate::theme::Theme::to_css`] の
//!   duration 一括 0ms 化で自動的に尊重される。
//!
//! **意図的に追随しない差分**（根拠を記録し、再評価は
//! `docs/policy/intentional-non-adoption.md` の評価軸に従う）:
//!
//! - **disabled 状態の不追加**: [`link`] は headless `data-disabled` を
//!   出力する概念を持たない（該当なし、N/A）。
//! - **inner span によるホバー面分離**: Radix の DOM 構造（`link` 内側に
//!   別要素を持ち hover 面を下線から浮かせる）は本モジュールの anatomy
//!   （`root`/`link` の 2 パーツ）を増やす変更になるため採らず、`link`
//!   全面に上側角丸の hover 面を当てる単純な構成を維持する。
//! - **現在ページの隠しテキスト幅固定**: Radix が現在ページの
//!   font-weight 変化による幅の揺れを防ぐために使う隠しテキストトリックは、
//!   tab-nav がページ遷移を伴うナビ（hover 中に太さが変わらない）である
//!   ため不要と判断した。
//!
//! # pill variant / palette 軸（イシュー #3125）
//!
//! blocks 取り込み対応表で、下線を使わず現在ページを「面（背景）」で
//! 強調する pill 形の見た目と、強調色を単一インスタンス単位で切り替える
//! 要望が現れたため、上記「意図的に追随しない差分」節の variant 軸・
//! `color-palette` 軸の不採用判断を再評価し、opt-in の純追加として
//! [`TabNavVariant`]/[`root_with`] を追加した。
//!
//! - **[`TabNavVariant::Pill`]**: [`crate::tabs::TabsVariant::Enclosed`] と
//!   同一の見た目（root を淡色の角丸コンテナにし、現在リンクを白背景 +
//!   微小な影で浮き上がらせる）。[`crate::recipe::Shape::Pill`]（形状
//!   修飾のみの軸）とは別物であり、混同しないこと。
//! - **`palette`（[`crate::recipe::ColorPalette`]）**: `Line` では現在
//!   リンクの下線色のみが palette 色になり（既存の `var(--fandhe-palette,
//!   …)` 経路のまま）、`Pill` では現在リンクの面が palette の淡色背景・
//!   文字色になる。
//! - **既存 API は不変**: [`root`] は `root_with(size, TabNavVariant::Line,
//!   None, …)` への委譲のみで、出力はバイト単位で不変（既存 golden ブロック
//!   は変更せず純追加のみ）。
//! - **フォーカスリングは palette 連動にしない**（意図的）: 既存の
//!   `FocusRingColor::Token` ブロックを書き換えると既存 golden ブロックの
//!   変更になり「純追加」の契約を破るため、本イシューでは据え置く。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-wasm-full` によるクライアント側の現在地追跡（SPA
//!   遷移時の `aria-current` 付け替え）。SSR/SSG では呼び出し側が `current`
//!   を渡す静的解決のみを提供する。
//! - `crates/headless-ui/` への `tab_nav` mod 追加（並列実行中の他イシュー
//!   との厳守事項により明示的に禁止。将来 headless 層が必要になった場合は
//!   別イシュー）。
//! - `examples/headless-pre-styled-ui` への追随（crates.io 公開後に別 PR）。
//! - `tabs.rs` 側の共有ヘルパ rustdoc（「`tab_nav` が共有する」旨の記述）の
//!   追随は #1542（tabs のスタイル調整）または後続 PR で行う。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::data_attrs::data_current;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="tab-nav"` を固定した本コンポーネントの anatomy（既存
/// `data-scope="tabs"` とは独立、モジュール冒頭 rustdoc 参照）。
const ANATOMY: Anatomy = anatomy("tab-nav");

/// [`SlotRecipe::new`] に渡す slot 一覧（[`root`]/[`link`] の呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &["root", "link"];

/// [`root`] が固定付与する属性キー一覧（呼び出し側 `attrs` からの偽装を
/// fail-closed で除去する対象。`class` は [`drop_class_attr`] が別途処理する
/// ため含めない）。
const ROOT_RESERVED: &[&str] = &["aria-label"];

/// [`link`] が固定付与する属性キー一覧（同上）。
const LINK_RESERVED: &[&str] = &["href", "aria-current", "data-current"];

/// `root` の見た目 variant（イシュー #3125）。[`crate::tabs::TabsVariant`] と
/// 同型の「見た目だけを切り替える」軸であり、本モジュールのナビゲーション
/// 意味論（`role` 非出力）には影響しない。既定は [`TabNavVariant::Line`]
/// （従来どおり下線のみ）。[`TabNavVariant::Pill`] は [`crate::tabs::TabsVariant::Enclosed`]
/// と同じ見た目（淡色の角丸コンテナ + 現在リンクを白背景・微小な影で
/// 浮き上がらせる）にする opt-in 追加で、[`crate::recipe::Shape::Pill`]
/// （`border-radius` のみの形状修飾）とは別物（混同しないこと）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabNavVariant {
    /// 下線のみ（既定、従来の唯一の見た目）。
    #[default]
    Line,
    /// 淡色の角丸コンテナ + 現在リンクを面で強調する見た目
    /// （イシュー #3125、`tabs` の `Enclosed` と同一外観）。
    Pill,
}

impl VariantValue for TabNavVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            TabNavVariant::Line => "line",
            TabNavVariant::Pill => "pill",
        }
    }
}

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::radio_card::drop_reserved`] と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// この styled Tab Nav の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。イシュー #1541 で `crate::tabs` の `pub(crate)` ヘルパ
/// 共有をやめ、自前の宣言列を持つ（モジュール冒頭 rustdoc「参考サイト基準
/// への調整」節参照）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("tab-nav", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .base(
            "link",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-tab-nav-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("border", "0"),
                decl("border-bottom", "2px solid transparent"),
                // イシュー #1541: 参照サイト（Radix Themes TabNav）は hover
                // 面が上側だけ丸い。下線（border-bottom）は直線のまま維持。
                decl(
                    "border-radius",
                    "var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0",
                ),
                decl("cursor", "pointer"),
                decl("text-decoration", "none"),
                // イシュー #1541: 未選択面の hover 色。current ページ
                // （aria-current="page"）にも同じ hover 面を適用する
                // （nav_list の HoverExcept は現在リンクの color が既に fg
                // で hover 規則と衝突しないため不要）。
                hover_bg_muted(),
            ],
        )
        .base(
            "link",
            transition_declarations("color, background, border-color", MotionDuration::Fast),
        )
        // イシュー #3125: Pill variant（[`TabNavVariant::Pill`]）が
        // `link` の下線・角丸・hover 背景を差し替えるための custom property
        // 間接参照。フォールバック値はいずれも既存の直書きリテラル（上記
        // base の `border-bottom`/`border-radius`/`hover_bg_muted()`）と
        // 同一のため、Line（既定）の computed style はバイト単位で不変。
        .base(
            "link",
            vec![
                decl(
                    "border-bottom",
                    "var(--fandhe-tab-nav-link-border-bottom, 2px solid transparent)",
                ),
                decl(
                    "border-radius",
                    "var(--fandhe-tab-nav-link-radius, var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0)",
                ),
                decl(
                    "--fandhe-hover-bg",
                    "var(--fandhe-tab-nav-hover-bg, var(--fandhe-color-bg-muted))",
                ),
            ],
        )
        .state(
            "link",
            StateCondition::AttrEq("aria-current", "page"),
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "border-bottom-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .state(
            "link",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state("link", StateCondition::Hover, {
            let mut decls = hover_surface_declarations();
            // イシュー #1541: 参照サイトは非現在リンクを hover 時に fg-muted
            // → fg へ強調する（`nav_list` の hover 規則と同型）。hover
            // セレクタ `:hover:not([data-disabled])`（specificity (0,4,0)）は
            // 現在ページの `[aria-current="page"]`（(0,3,0)）より高いが、
            // 両者が唯一共有するプロパティ `color` は互いに同じ
            // `var(--fandhe-color-fg)` を指すため、現在ページの見た目は
            // hover 時も変化しない（`nav_list` の `HoverExcept` のような
            // 除外は不要）。
            decls.push(decl("color", "var(--fandhe-color-fg)"));
            decls
        })
        // イシュー #1541: size 軸（Xs〜Xl、既定 Md）。padding は
        // `crate::tabs` の size 進行と同一、font-size の段対応は
        // `crate::pagination` と同一（Radix Themes TabNav size 2=14px(sm)/
        // size 1=12px(xs) に整合）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-0-5) var(--fandhe-space-2)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-1) var(--fandhe-space-3)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-2) var(--fandhe-space-4)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-3) var(--fandhe-space-5)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-4) var(--fandhe-space-6)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
            ],
        )
        // イシュー #3125: Pill variant（root）。`tabs` の `Enclosed` と同一の
        // 見た目（root 自身は淡色コンテナ、子孫の link は custom property
        // 経由で下線なし・hover 背景を現在リンクの面と衝突しない値へ）。
        .variant(
            TabNavVariant::Pill,
            "root",
            vec![
                decl("border-bottom", "0"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("--fandhe-tab-nav-link-border-bottom", "0"),
                decl(
                    "--fandhe-tab-nav-link-radius",
                    "var(--fandhe-radius-sm, 0.25rem)",
                ),
                // イシュー #2039（tabs）と同じ理由: 現在リンクの背景
                // （`--fandhe-tab-nav-current-bg`）と同じ値にしないと、
                // hover 時に現在リンクが root のコンテナ背景と同化して
                // 選択解除されたように見える。
                decl("--fandhe-tab-nav-hover-bg", "var(--fandhe-color-bg)"),
                decl(
                    "--fandhe-tab-nav-current-bg",
                    "var(--fandhe-tab-nav-pill-current-bg, var(--fandhe-color-bg))",
                ),
                decl(
                    "--fandhe-tab-nav-current-fg",
                    "var(--fandhe-tab-nav-pill-current-fg, var(--fandhe-color-fg))",
                ),
                decl(
                    "--fandhe-tab-nav-current-shadow",
                    "var(--fandhe-tab-nav-pill-current-shadow, var(--fandhe-shadow-sm))",
                ),
            ],
        )
        // イシュー #3125: 現在リンクの面（Pill のときのみ意味を持つ）。Line
        // ではすべてフォールバックへ落ちるため computed style は不変。
        .state(
            "link",
            StateCondition::AttrEq("aria-current", "page"),
            vec![
                decl("background", "var(--fandhe-tab-nav-current-bg, transparent)"),
                decl("color", "var(--fandhe-tab-nav-current-fg, var(--fandhe-color-fg))"),
                decl("box-shadow", "var(--fandhe-tab-nav-current-shadow, none)"),
                decl(
                    "--fandhe-hover-bg",
                    "var(--fandhe-tab-nav-current-bg, var(--fandhe-tab-nav-hover-bg, var(--fandhe-color-bg-muted)))",
                ),
                decl(
                    "--fandhe-tab-nav-hover-fg",
                    "var(--fandhe-tab-nav-current-fg, var(--fandhe-color-fg))",
                ),
            ],
        )
        // イシュー #3125: hover 中の現在リンクの文字色（Pill + palette で
        // 現在リンクを hover したとき fg へ戻らないようにする）。非現在
        // リンクと Line はいずれもフォールバックの fg のまま。
        .state(
            "link",
            StateCondition::Hover,
            vec![decl(
                "color",
                "var(--fandhe-tab-nav-hover-fg, var(--fandhe-color-fg))",
            )],
        );

    // イシュー #3125: `color-palette` 軸（opt-in、`default_variant` は登録
    // しない。[`root_with`] で `Some` を渡したときのみクラスが付く）。
    // `palette_scale_declarations` の 6 宣言に加え、Pill variant が参照する
    // `--fandhe-tab-nav-pill-current-*` の 3 宣言を足す: Line × palette は
    // 現在リンクの下線色（既存の `var(--fandhe-palette, …)` 経路）だけが
    // palette 色になり、Pill × palette は現在リンクの面が palette の淡色
    // 背景・文字色になる。recipe() が唯一の正であり [`root_with`]/
    // [`stylesheet`] の双方がこの登録を共有する（axis 登録が
    // `variant_classes` 呼び出し側ごとに分かれると、palette 選択時に
    // class が出ない不整合が起きるため、`tabs::recipe` と同じく単一関数に
    // 畳み込む）。
    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        let mut decls = palette_scale_declarations(palette);
        decls.push(decl(
            "--fandhe-tab-nav-pill-current-bg",
            "var(--fandhe-palette-subtle)",
        ));
        decls.push(decl(
            "--fandhe-tab-nav-pill-current-fg",
            "var(--fandhe-palette-fg-subtle)",
        ));
        decls.push(decl("--fandhe-tab-nav-pill-current-shadow", "none"));
        recipe = recipe.variant(palette, "root", decls);
    }
    recipe
}

/// この styled Tab Nav が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tabs::stylesheet`] と同じ契約）。
///
/// イシュー #3125: Pill variant の現在リンクは背景色・文字色・
/// `box-shadow`（elevation）のみで選択状態を表現するため、`tabs` の
/// Enclosed と同じ理由で Windows 強制配色モード（`forced-colors: active`）
/// 補強を追記する（root の variant クラスを起点にした子孫結合子セレクタ、
/// `tabs::stylesheet` の codex-review 是正と同型）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(
        "\n@media (forced-colors: active) {\n  [data-scope=\"tab-nav\"][data-part=\"root\"].fd-tab-nav--variant-pill [data-scope=\"tab-nav\"][data-part=\"link\"][aria-current=\"page\"] {\n    border: 1px solid CanvasText;\n  }\n}\n",
    );
    out
}

/// `root`（`<nav>`）パーツを組み立てる。`size` に応じたクラスを付与する
/// 唯一のパーツ（イシュー #1541、[`crate::pagination::root`] と同型）。
/// `label` は `aria-label` として必須付与する（landmark のアクセシブル
/// ネーム欠落を型で防ぐ、[`crate::nav_list::root`] と同型の判断）。呼び出し
/// 側 `attrs` の `class` は `drop_class_attr` で除去し、`ROOT_RESERVED`
/// の偽装は `drop_reserved` で除去してから合成する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tab_nav;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = tab_nav::root(Size::Md, "Section navigation", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="tab-nav" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    root_with(size, TabNavVariant::Line, None, label, attrs, children)
}

/// `root`（`<nav>`）パーツを組み立てる（[`root`] の opt-in 拡張版、イシュー
/// #3125）。`variant`/`palette` は純追加の軸で、`variant ==
/// `[`TabNavVariant::Line`]` かつ `palette == None` のときは [`root`] と
/// バイト単位で同一の出力になる（class に `variant`/`color-palette` の
/// クラスが付かない。`recipe()` が両軸へ `default_variant` を登録しない
/// 契約のため、モジュール冒頭 rustdoc「意図的に追随しない差分」節参照）。
/// 引数の並びは「軸が先、`label`/`attrs`/`children` が後」（並行実装中の
/// `crate::select::root_with` と同型の規約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tab_nav::{self, TabNavVariant};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = tab_nav::root_with(
///     Size::Md,
///     TabNavVariant::Pill,
///     Some(ColorPalette::Accent),
///     "Section navigation",
///     vec![],
///     vec![],
/// );
/// let html = render(&node);
/// assert!(html.contains("fd-tab-nav--variant-pill"));
/// assert!(html.contains("fd-tab-nav--color-palette-accent"));
/// ```
#[must_use]
pub fn root_with<'a>(
    size: Size,
    variant: TabNavVariant,
    palette: Option<ColorPalette>,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut selection: Vec<(&str, &str)> = vec![("size", size.value())];
    if variant == TabNavVariant::Pill {
        selection.push(("variant", variant.value()));
    }
    if let Some(p) = palette {
        selection.push(("color-palette", p.value()));
    }
    let class = recipe().variant_classes(&selection);
    let mut merged: Vec<(&str, &str)> = vec![("aria-label", label), ("class", class.as_str())];
    merged.extend(drop_reserved(drop_class_attr(attrs), ROOT_RESERVED));
    ANATOMY.part("root", "nav", merged, children)
}

/// `link`（`<a>`）パーツを組み立てる。`current` が `true` のとき
/// `aria-current="page"` + `data-current` を付与する（[`crate::link::root`]
/// /`crates/headless-ui/src/link.rs::root` と同じ語彙）。`data-current` は
/// `fandhe_frontend_headless_ui::data_attrs::data_current` ヘルパを経由して
/// 付与する（イシュー #1063、生タプルでの再定義をしない。
/// `docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B-1）。`role` は
/// 一切出力しない（モジュール冒頭 rustdoc「`tabs` との差」節参照）。呼び出し側
/// `attrs` の `class` は `drop_class_attr` で除去し、`LINK_RESERVED`
/// の偽装は `drop_reserved` で除去してから合成する。`href` の危険 URL
/// スキームは core の既定経路が拒否する（モジュール冒頭 rustdoc「セキュリ
/// ティ不変条件」節参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::tab_nav;
///
/// let node = tab_nav::link("/docs", true, vec![], vec![text("Docs")]);
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="tab-nav" data-part="link""#));
/// assert!(html.contains(r#"aria-current="page""#));
/// ```
#[must_use]
pub fn link<'a>(
    href: &'a str,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("href", href)];
    if current {
        merged.push(("aria-current", "page"));
        // イシュー #1063: 生タプルでの再定義をやめ、headless-ui の共有ヘルパを
        // 経由する（`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約
        // B-1）。出力は従来の `("data-current", "")` と完全に同一。
        merged.extend(data_current(true));
    }
    merged.extend(drop_reserved(drop_class_attr(attrs), LINK_RESERVED));
    ANATOMY.part("link", "a", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_part_tag_and_aria_label() {
        let html = render(&root(Size::Md, "Section navigation", vec![], vec![]));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"data-scope="tab-nav""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="Section navigation""#));
    }

    #[test]
    fn link_outputs_scope_part_tag_and_href() {
        let html = render(&link("/docs", false, vec![], vec![text("Docs")]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="tab-nav""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(html.contains(r#"href="/docs""#));
        assert!(html.contains(">Docs<"));
    }

    // --- 受け入れ条件: role を一切出力しない・"tablist" という文字列を含まない ---

    #[test]
    fn root_and_link_never_output_role_or_tablist() {
        let root_html = render(&root(Size::Md, "Section navigation", vec![], vec![]));
        let link_html = render(&link("/docs", true, vec![], vec![text("Docs")]));
        assert!(!root_html.contains("role="));
        assert!(!link_html.contains("role="));
        assert!(!root_html.contains("tablist"));
        assert!(!link_html.contains("tablist"));
    }

    #[test]
    fn current_true_adds_aria_current_and_data_current() {
        let html = render(&link("/docs", true, vec![], vec![]));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("data-current"));
    }

    #[test]
    fn current_false_omits_aria_current_and_data_current() {
        let html = render(&link("/docs", false, vec![], vec![]));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            "Section navigation",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tab-nav""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_reserved_attr_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            "Section navigation",
            vec![("aria-label", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"aria-label="Section navigation""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn link_reserved_attr_spoofing_is_dropped() {
        let html = render(&link(
            "/docs",
            true,
            vec![
                ("href", "/attacker"),
                ("aria-current", "attacker"),
                ("data-current", "attacker"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"href="/docs""#));
        assert_eq!(html.matches("href=").count(), 1);
        assert!(html.contains(r#"aria-current="page""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_from_caller_is_dropped() {
        let html = render(&root(
            Size::Md,
            "Section navigation",
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
    }

    #[test]
    fn stylesheet_contains_current_state_selector() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tab-nav"][data-part="link"][aria-current="page"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- イシュー #1541: hover / フォーカスリング / size 軸 ---

    #[test]
    fn stylesheet_contains_hover_surface_declaration() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(":hover:not([data-disabled])"));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    #[test]
    fn stylesheet_contains_focus_ring_declarations() {
        let css = stylesheet();
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
        assert!(css.contains(":focus-visible"));
    }

    #[test]
    fn stylesheet_contains_all_five_size_variant_classes() {
        let css = stylesheet();
        for class in [
            "fd-tab-nav--size-xs",
            "fd-tab-nav--size-sm",
            "fd-tab-nav--size-md",
            "fd-tab-nav--size-lg",
            "fd-tab-nav--size-xl",
        ] {
            assert!(css.contains(class), "missing size class: {class}");
        }
    }

    #[test]
    fn root_applies_default_md_size_class_when_unspecified_elsewhere() {
        let html = render(&root(Size::Md, "Section navigation", vec![], vec![]));
        assert!(html.contains("fd-tab-nav--size-md"));
    }

    #[test]
    fn root_applies_requested_size_class() {
        let html = render(&root(Size::Lg, "Section navigation", vec![], vec![]));
        assert!(html.contains("fd-tab-nav--size-lg"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_label_is_escaped() {
        let html = render(&root(
            Size::Md,
            "\"><script>alert(1)</script>",
            vec![],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn link_children_script_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn link_attrs_value_breakout_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            false,
            vec![("data-note", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
    }

    // --- 危険 URL スキーム拒否（fail-closed、core の render() 経由） ---

    #[test]
    fn dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "java\tscript:alert(1)",
            "\u{0}javascript:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let html = render(&link(url, false, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    // --- イシュー #3125: pill variant / palette 軸（純追加） ---

    #[test]
    fn root_with_line_none_matches_root_byte_for_byte() {
        let via_root = render(&root(Size::Md, "Section navigation", vec![], vec![]));
        let via_root_with = render(&root_with(
            Size::Md,
            TabNavVariant::Line,
            None,
            "Section navigation",
            vec![],
            vec![],
        ));
        assert_eq!(via_root, via_root_with);
    }

    #[test]
    fn root_with_pill_outputs_variant_class() {
        let html = render(&root_with(
            Size::Md,
            TabNavVariant::Pill,
            None,
            "Section navigation",
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-tab-nav--variant-pill"));
    }

    #[test]
    fn root_with_palette_outputs_color_palette_class() {
        for palette in [
            ColorPalette::Accent,
            ColorPalette::Info,
            ColorPalette::Success,
            ColorPalette::Warning,
            ColorPalette::Danger,
            ColorPalette::Neutral,
        ] {
            let html = render(&root_with(
                Size::Md,
                TabNavVariant::Line,
                Some(palette),
                "Section navigation",
                vec![],
                vec![],
            ));
            let expected = format!("fd-tab-nav--color-palette-{}", palette.value());
            assert!(html.contains(&expected), "missing class: {expected}");
        }
    }

    #[test]
    fn root_with_line_and_none_omit_variant_and_palette_classes() {
        let html = render(&root_with(
            Size::Md,
            TabNavVariant::Line,
            None,
            "Section navigation",
            vec![],
            vec![],
        ));
        assert!(!html.contains("fd-tab-nav--variant-"));
        assert!(!html.contains("fd-tab-nav--color-palette-"));
    }

    #[test]
    fn stylesheet_contains_pill_variant_block() {
        let css = stylesheet();
        assert!(css.contains(".fd-tab-nav--variant-pill"));
    }

    #[test]
    fn stylesheet_contains_all_six_palette_variant_classes() {
        let css = stylesheet();
        for class in [
            "fd-tab-nav--color-palette-accent",
            "fd-tab-nav--color-palette-info",
            "fd-tab-nav--color-palette-success",
            "fd-tab-nav--color-palette-warning",
            "fd-tab-nav--color-palette-danger",
            "fd-tab-nav--color-palette-neutral",
        ] {
            assert!(css.contains(class), "missing palette class: {class}");
        }
    }

    #[test]
    fn stylesheet_contains_forced_colors_block() {
        let css = stylesheet();
        assert!(css.contains("@media (forced-colors: active)"));
        assert!(css.contains("fd-tab-nav--variant-pill"));
        assert!(css.contains("border: 1px solid CanvasText;"));
    }

    #[test]
    fn root_with_class_and_aria_label_spoofing_is_dropped() {
        let html = render(&root_with(
            Size::Md,
            TabNavVariant::Pill,
            Some(ColorPalette::Accent),
            "Section navigation",
            vec![("class", "attacker-controlled"), ("aria-label", "attacker")],
            vec![],
        ));
        assert!(!html.contains("attacker-controlled"));
        assert!(!html.contains(">attacker<"));
        assert!(html.contains(r#"aria-label="Section navigation""#));
    }

    #[test]
    fn root_with_label_is_escaped() {
        let html = render(&root_with(
            Size::Md,
            TabNavVariant::Pill,
            None,
            "\"><script>alert(1)</script>",
            vec![],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
