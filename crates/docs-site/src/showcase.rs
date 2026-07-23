//! pre-styled-ui コンポーネントショーケースページの Rust 生成コンテンツ。
//!
//! # 役割・呼び出し文脈
//!
//! docs サイトのビルドパイプライン（`crate::build::build_site`）は基本的に
//! 「Markdown → Node 木」の変換のみを行うが、UI コンポーネントの実レンダリング
//! 結果を掲載するショーケースページは Markdown では表現できない。本モジュールは
//! そのための「Rust 生成コンテンツページ」の最小機構であり、
//!
//! 1. [`generated_content`]: `site/nav.toml` の `page.path` をキーに、Markdown
//!    本文の**後ろへ追記する** `Node` 木を返す（該当しないページは `None`。
//!    Markdown ページ処理・linkcheck の既存パイプラインへは一切干渉しない）
//! 2. [`stylesheet`]: ショーケースが参照する CSS（テーマトークン + 使用
//!    recipe の全量 + ショーケース専用の配置スタイル）を
//!    [`StyleSheet`] として組み立てる。`build_site` がビルド成果物
//!    [`STYLESHEET_REL_PATH`] へ書き出し、ページ側は
//!    `crate::layout::docs_page_with_assets` の追加 `<link>` で参照する
//!
//! の 2 点だけを `build.rs` へ公開する。サイト骨格スタイル
//! （`site/assets/site.css`）とは分離ファイルに保ち、既存ページのカスケードへ
//! 影響させない（イシュー #520 系のショーケース統合方針）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! - マークアップはすべて `fandhe_frontend_core` / `fandhe_frontend_pre_styled_ui`
//!   のノード木 API で組み立てる。`raw_html()`・HTML 文字列の直接組み立ては
//!   使わない。headless 層の状態値（[`OpenState`] / [`Orientation`]）は
//!   pre-styled-ui のルート再エクスポート（イシュー #685）経由で使用し、
//!   headless-ui への直接依存は持たない（イシュー #693）
//! - CSS は [`StyleSheet`]（検証済み CSS のみを保持し `<` を拒否する型、
//!   `crates/pre-styled-ui/src/stylesheet.rs`）経由でのみ書き出す
//!
//! # インタラクティブ部品の扱い
//!
//! Tabs / Accordion / Dialog / Menu / Select / Popover / Tooltip 等の状態
//! 機械を持つ部品は、SSR 静的マークアップ（選択中・開いた状態を
//! `data-state="open"`/`"active"` 等で固定した掲示）のみを載せる。実際の
//! クリック挙動（dispatch 状態遷移）は wasm 層の責務であり docs サイトの
//! スコープ外（`examples/headless-pre-styled-ui` と同じ方針）。
//!
//! Dialog/Menu/Select/Popover/Tooltip は開いた状態を固定して掲示するため、
//! recipe CSS のオーバーレイ配置（`position: fixed`/`absolute` + `z-index`）
//! をそのまま反映するとページ全体を覆う・後続セクションに重なってしまう。
//! [`SHOWCASE_LAYOUT_CSS`] がショーケース内に限定してこれを中和する
//! （recipe CSS・`site/assets/site.css` はいずれも変更しない）。

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbItem, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::checkbox_card;
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode, Pagination};
use fandhe_frontend_pre_styled_ui::radio_card;
use fandhe_frontend_pre_styled_ui::rating_group::{self, RatingGroup, RatingItemFlags};
use fandhe_frontend_pre_styled_ui::segment_group;
use fandhe_frontend_pre_styled_ui::slider;
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::tabs::{tabs, ActivationMode, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::tags_input;
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::tree_view::{self, TreeNode, TreeView};
use fandhe_frontend_pre_styled_ui::{
    accordion, alert, badge, card, combobox, menu, popover, radio_group, select, switch, tooltip,
    AlertStatus, BadgeProps, BadgeVariant, CardVariant, ColorPalette, OpenState, Orientation, Size,
    StyleSheet, StylesheetError,
};

/// ショーケースページの `page.path`（`site/nav.toml` の宣言と一致させる契約。
/// 乖離すると生成コンテンツが載らない素の Markdown ページになるため、
/// `tests/site_showcase.rs` が実サイトビルドで実出力を検証する）。
pub const PAGE_PATH: &str = "/components/pre-styled-ui/";

/// ショーケース専用 CSS の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が [`stylesheet`] の内容をこのパスへ書き出し、
/// ページ `<head>` の追加 `<link>`（`docs_page_with_assets`）が参照する。
pub const STYLESHEET_REL_PATH: &str = "assets/pre-styled-ui.css";

/// ショーケース内の配置（グリッド・縦積み）専用スタイル。コンポーネント
/// 自体の見た目は pre-styled-ui の recipe が担い、ここではデモの並びのみを
/// 整える（`site/assets/site.css` のサイト骨格クラスとは名前空間を分ける）。
///
/// 末尾の見出しリセットは、showcase ページが `.docs-content` 内へ埋め込まれる
/// ことによる `site.css` 見出しルール（`.docs-content h3` の margin・
/// フォント指定）の Accordion anatomy `h3`（item trigger のラッパ）への漏れを
/// 遮断する（Bugbot 指摘）。`site.css` 側は変更せず（`site_css_contract` を
/// 壊さない）、`data-scope` 属性ベースの決定的セレクタで showcase 領域内に
/// 限定して上書きする（`.pre-styled-showcase` + 属性 + 型 = (0,2,1) が
/// `.docs-content h3` = (0,1,1) より優先される）。
///
/// Dialog/Menu/Select/Popover/Tooltip の掲示（イシュー #691）に伴い、以下の
/// オーバーレイ配置中和ルールを追加している（いずれも recipe CSS（
/// `crates/pre-styled-ui/src/{dialog,menu,select,popover,tooltip}.rs`）・
/// `site.css` は変更せず、showcase 領域内に限定した上書きのみで完結させる）:
///
/// - `[data-scope="dialog"][data-part="backdrop"]` の非表示化: dialog の
///   backdrop は `position: fixed; inset: 0` のビューポート全体暗幕であり、
///   開いた状態を固定掲示するとページ全体を覆ってしまうため掲示用にのみ隠す
///   （実際の modal 表示では backdrop は必須であり、ここでの非表示化は
///   ショーケースの掲示都合に限定する）。
/// - dialog/menu/select/combobox/popover/tooltip の `[data-part="positioner"]` を
///   `position: static` へ中和: recipe CSS は dialog を
///   `position: fixed; inset: 0`、menu/select/combobox/popover を
///   `position: absolute; top: 100%`、tooltip を
///   `position: absolute; bottom: 100%` としており、いずれも開いた content を
///   ページ内の別位置・別セクションに重ねてしまう。static 化してフロー内へ
///   インライン表示させることで、後続セクションと重ならずに掲示できる
///   （dialog はさらに `padding`/`justify-content` も中和し、中央寄せの
///   ための余白・配置指定を解除する）。
/// - dialog/popover の `title`（`h2`）見出しリセット: Accordion の `h3` と
///   同じ理由（`site.css` の `.docs-content h2` が漏れる）で、showcase 領域
///   内に限定して `border-top`/`padding-top`/`letter-spacing` を打ち消す
///   （margin/font-size/font-weight は recipe が宣言済みで自然に勝つため
///   宣言しない。recipe との二重管理を避ける最小リセット）。
const SHOWCASE_LAYOUT_CSS: &str = "\
.pre-styled-showcase {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.showcase-row {\n  display: flex;\n  flex-wrap: wrap;\n  gap: 0.75rem;\n  align-items: center;\n  margin: 1rem 0;\n}\n\
.showcase-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  margin: 1rem 0;\n  max-width: 36rem;\n}\n\
.showcase-form-field-group {\n  display: flex;\n  flex-direction: column;\n  gap: 0.25rem;\n  width: 100%;\n}\n\
.pre-styled-showcase [data-scope=\"accordion\"] h3 {\n  margin: 0;\n  font-size: 1rem;\n  font-weight: 400;\n  line-height: 1.5;\n  letter-spacing: normal;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  display: none;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: static;\n  padding: 0;\n  justify-content: flex-start;\n}\n\
.pre-styled-showcase [data-scope=\"menu\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"select\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"combobox\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"popover\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"tooltip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"] h2,\n.pre-styled-showcase [data-scope=\"popover\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n";

/// `page_path` が Rust 生成コンテンツを持つページなら、Markdown 本文の後ろへ
/// 追記する `Node` 木を返す。
///
/// 現在の登録は pre-styled-ui ショーケース（[`PAGE_PATH`]）の 1 件のみ。
/// 追加のページが必要になった場合もこの関数へ登録を足すだけで
/// `crate::build::build_site` 側の分岐は増えない（最小機構の維持）。
#[must_use]
pub fn generated_content(page_path: &str) -> Option<Node> {
    if page_path == PAGE_PATH {
        Some(showcase_body())
    } else {
        None
    }
}

/// ショーケースが参照する CSS 全量を組み立てる。
///
/// 内訳: テーマトークン（`Theme::default`、ライト/ダーク両対応）→ 掲載
/// コンポーネントの recipe CSS（button/badge/spinner/alert/card/tabs/
/// accordion/dialog/menu/select/combobox/popover/tooltip/switch/radio_group/
/// avatar/checkbox/checkbox_card/radio_card/input/textarea/native_select/
/// number_input/tags_input/rating_group/slider/segment_group/breadcrumb）
/// → ショーケース配置スタイル、の順で決定的に連結する。
///
/// # Errors
///
/// いずれかの CSS 断片が [`StyleSheet::push_css`] の検証（`<`・制御文字の
/// 拒否）に落ちた場合 [`StylesheetError`] を返す。pre-styled-ui 側の生成 CSS
/// は構造上 `<` を含み得ないため通常は到達しないが、黙って欠けた CSS を
/// 公開しない fail-closed 方針で伝播させる。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(&fandhe_frontend_pre_styled_ui::button::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::badge::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::spinner::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::alert::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::card::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tabs::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::accordion::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::dialog::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::menu::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::select::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::combobox::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::popover::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tooltip::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::switch::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::radio_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::avatar::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::checkbox::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::checkbox_card::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::radio_card::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::input::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::textarea::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::native_select::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::number_input::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tags_input::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::rating_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::slider::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::segment_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tree_view::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::pagination::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::breadcrumb::stylesheet())?;
    sheet.push_css(SHOWCASE_LAYOUT_CSS)?;
    Ok(sheet)
}

/// 見出し（`h2`）+ 説明文 + デモ本体、のショーケース 1 節を組み立てる小
/// ヘルパ。見出しは `crate::layout::with_heading_anchors` が id を注入して
/// ページ内 TOC（on this page）へ自動掲載する。
fn section(heading: &str, description: &str, demos: Vec<Node>) -> Node {
    let mut children = vec![
        el("h2", vec![], vec![text(heading)]),
        el("p", vec![], vec![text(description)]),
    ];
    children.extend(demos);
    el("section", vec![], children)
}

/// 横並びのデモ行。
fn row(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-row")], children)
}

/// 縦積みのデモ列（Alert / Card 等の幅を取る部品向け）。
fn stack(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-stack")], children)
}

/// Button 節: variant / size / palette / 状態（disabled・loading）の各軸。
fn button_section() -> Node {
    let variants = [
        (ButtonVariant::Solid, "Solid"),
        (ButtonVariant::Outline, "Outline"),
        (ButtonVariant::Ghost, "Ghost"),
        (ButtonVariant::Subtle, "Subtle"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            button(
                &ButtonProps {
                    variant: *variant,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let sizes = [
        (Size::Sm, "Small"),
        (Size::Md, "Medium"),
        (Size::Lg, "Large"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            button(
                &ButtonProps {
                    size: *size,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            button(
                &ButtonProps {
                    palette: *palette,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let state_row = row(vec![
        button(
            &ButtonProps {
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Disabled")],
        ),
        button(
            &ButtonProps {
                loading: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Loading")],
        ),
    ]);

    section(
        "Button",
        "variant（solid / outline / ghost / subtle）・size・colorPalette・状態（disabled / loading）の各軸を型安全な props で切り替えます。",
        vec![variant_row, size_row, palette_row, state_row],
    )
}

/// Badge 節: variant × palette。
fn badge_section() -> Node {
    let variants = [
        (BadgeVariant::Solid, "Solid"),
        (BadgeVariant::Subtle, "Subtle"),
        (BadgeVariant::Outline, "Outline"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            badge::badge(
                &BadgeProps {
                    variant: *variant,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            badge::badge(
                &BadgeProps {
                    palette: *palette,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    section(
        "Badge",
        "ステータス表示向けの小型ラベル。variant と colorPalette を組み合わせます。",
        vec![variant_row, palette_row],
    )
}

/// Spinner 節: size バリエーション。
fn spinner_section() -> Node {
    let sizes = [
        (Size::Sm, "Loading (small)"),
        (Size::Md, "Loading (medium)"),
        (Size::Lg, "Loading (large)"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            spinner(&SpinnerProps {
                size: *size,
                label,
                ..SpinnerProps::default()
            })
        })
        .collect());
    section(
        "Spinner",
        "読み込み中表示。role=\"status\" と aria-label でスクリーンリーダーへ状態を伝えます。",
        vec![size_row],
    )
}

/// Alert 節: status（info / success / warning / error）ごとの表示。
fn alert_section() -> Node {
    let statuses = [
        (
            AlertStatus::Info,
            "Info",
            "新しいバージョンが利用可能です。",
        ),
        (AlertStatus::Success, "Success", "ビルドが完了しました。"),
        (
            AlertStatus::Warning,
            "Warning",
            "依存クレート数が上限に近づいています。",
        ),
        (
            AlertStatus::Error,
            "Error",
            "リンク切れを検出したため書き出しを中止しました。",
        ),
    ];
    let demos = stack(
        statuses
            .iter()
            .map(|(status, title, description)| {
                alert::root(
                    *status,
                    vec![],
                    vec![
                        alert::indicator(vec![], vec![text("!")]),
                        alert::content(
                            vec![],
                            vec![
                                alert::title(vec![], vec![text(*title)]),
                                alert::description(vec![], vec![text(*description)]),
                            ],
                        ),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Alert",
        "status（info / success / warning / error）で色が切り替わる通知領域。root / indicator / content / title / description の slot 構成です。",
        vec![demos],
    )
}

/// Card 節: variant（elevated / outline / subtle）ごとの表示。
fn card_section() -> Node {
    let variants = [
        (CardVariant::Elevated, "Elevated"),
        (CardVariant::Outline, "Outline"),
        (CardVariant::Subtle, "Subtle"),
    ];
    let demos = stack(
        variants
            .iter()
            .map(|(variant, label)| {
                card::root(
                    *variant,
                    vec![],
                    vec![
                        card::header(
                            vec![],
                            vec![
                                card::title(vec![], vec![text(*label)]),
                                card::description(
                                    vec![],
                                    vec![text("card variant のデモです。")],
                                ),
                            ],
                        ),
                        card::body(
                            vec![],
                            vec![el(
                                "p",
                                vec![],
                                vec![text(
                                    "header / body / footer / title / description の slot 構成を持つ汎用コンテナです。",
                                )],
                            )],
                        ),
                        card::footer(
                            vec![],
                            vec![button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    ..ButtonProps::default()
                                },
                                vec![],
                                vec![text("Action")],
                            )],
                        ),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Card",
        "variant（elevated / outline / subtle）を持つ装飾的コンテナ。",
        vec![demos],
    )
}

/// Tabs 節: 1 番目のタブが選択された静的マークアップ。
fn tabs_section() -> Node {
    let node = tabs(
        Size::Md,
        ColorPalette::Accent,
        &TabsProps {
            id: "showcase-tabs",
            selected: "overview",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        vec![
            TabItem {
                value: "overview",
                trigger: vec![text("Overview")],
                content: vec![el(
                    "p",
                    vec![],
                    vec![text(
                        "選択中のタブは data-state=\"active\" で強調されます。",
                    )],
                )],
                disabled: false,
            },
            TabItem {
                value: "usage",
                trigger: vec![text("Usage")],
                content: vec![el(
                    "p",
                    vec![],
                    vec![text("非選択タブの content は CSS で非表示になります。")],
                )],
                disabled: false,
            },
        ],
    );
    section(
        "Tabs",
        "headless-ui の Tabs（WAI-ARIA Tabs パターン）に pre-styled-ui の data-scope / data-part セレクタ CSS を適用した静的掲示です。",
        vec![node],
    )
}

/// Accordion 節: 1 項目目が開いた静的マークアップ（single モード想定）。
fn accordion_section() -> Node {
    let items: [(&str, &str, &str, OpenState); 2] = [
        (
            "showcase-acc-1",
            "pre-styled-ui とは何ですか？",
            "headless-ui の anatomy（data-scope / data-part）へテーマトークンと recipe CSS を重ねる styled 層です。",
            OpenState::Open,
        ),
        (
            "showcase-acc-2",
            "クリックで開閉できますか？",
            "この掲示は SSR 静的マークアップです。状態遷移（dispatch）は wasm 層の責務のため、docs サイトでは開いた状態を固定表示しています。",
            OpenState::Closed,
        ),
    ];
    let mut children = Vec::new();
    for (value, question, answer, state) in items {
        let trigger_id = format!("{value}-trigger");
        let content_id = format!("{value}-content");
        children.push(accordion::item(
            state,
            false,
            vec![],
            vec![
                el(
                    "h3",
                    vec![],
                    vec![accordion::item_trigger(
                        state,
                        false,
                        Some(trigger_id.as_str()),
                        Some(content_id.as_str()),
                        vec![],
                        vec![text(question)],
                    )],
                ),
                accordion::item_content(
                    state,
                    Some(content_id.as_str()),
                    Some(trigger_id.as_str()),
                    vec![],
                    vec![el("p", vec![], vec![text(answer)])],
                ),
            ],
        ));
    }
    section(
        "Accordion",
        "開閉状態（data-state=\"open\" / \"closed\"）に応じてスタイルが切り替わる開閉パネルの静的掲示です。",
        vec![accordion::root(Size::Md, vec![], children)],
    )
}

/// Dialog 節: 開いた状態の静的マークアップ（イシュー #691）。
///
/// backdrop は掲示用に非表示化し（[`SHOWCASE_LAYOUT_CSS`]）、positioner は
/// フロー内配置へ中和している。実際の modal オーバーレイ配置は recipe CSS
/// （`crates/pre-styled-ui/src/dialog.rs`）がそのまま担う。
fn dialog_section() -> Node {
    let node = div(
        vec![],
        vec![
            dialog::trigger(
                OpenState::Open,
                Some("showcase-dialog-content"),
                vec![],
                vec![text("Open dialog")],
            ),
            dialog::root(
                Size::Md,
                OpenState::Open,
                vec![],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Dialog,
                            true,
                            ContentIds {
                                id: Some("showcase-dialog-content"),
                                labelledby: Some("showcase-dialog-title"),
                                describedby: Some("showcase-dialog-desc"),
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some("showcase-dialog-title"),
                                    vec![],
                                    vec![text("Confirm action")],
                                ),
                                dialog::description(
                                    Some("showcase-dialog-desc"),
                                    vec![],
                                    vec![text("この操作は取り消せません。")],
                                ),
                                dialog::close_trigger(vec![], vec![text("Close")]),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Dialog",
        "headless-ui の Dialog（WAI-ARIA dialog パターン）に pre-styled-ui の data-scope / data-part セレクタ CSS を適用した静的掲示です。backdrop は掲示用に非表示化し、positioner はフロー内配置へ中和しています（実際の overlay 配置は recipe CSS が担います）。",
        vec![node],
    )
}

/// Menu 節: highlighted / 通常 / separator / disabled の各状態を持つ項目リスト
/// が開いた静的マークアップ（イシュー #691）。
fn menu_section() -> Node {
    let node = menu::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            menu::trigger(
                OpenState::Open,
                false,
                Some("showcase-menu-content"),
                vec![],
                vec![text("Actions")],
            ),
            menu::positioner(
                OpenState::Open,
                vec![],
                vec![menu::content(
                    OpenState::Open,
                    Some("showcase-menu-content"),
                    None,
                    vec![],
                    vec![
                        menu::item("edit", false, true, vec![], vec![text("Edit")]),
                        menu::item("duplicate", false, false, vec![], vec![text("Duplicate")]),
                        menu::separator(vec![], vec![]),
                        menu::item("delete", true, false, vec![], vec![text("Delete")]),
                    ],
                )],
            ),
        ],
    );
    section(
        "Menu",
        "headless-ui の Menu（role=\"menu\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。highlighted（キーボードフォーカス位置）・separator・disabled の各状態を含みます。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Select 節: 1 項目が選択済みの listbox が開いた静的マークアップ
/// （イシュー #691）。
fn select_section() -> Node {
    let node = select::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            select::label(
                Some("showcase-select-label"),
                vec![],
                vec![text("Framework")],
            ),
            select::control(
                OpenState::Open,
                vec![],
                vec![select::trigger(
                    OpenState::Open,
                    false,
                    Some("showcase-select-content"),
                    Some("showcase-select-label"),
                    vec![],
                    vec![
                        select::value_text(false, vec![], vec![text("fandhe-frontend")]),
                        select::indicator(OpenState::Open, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            select::positioner(
                OpenState::Open,
                vec![],
                vec![select::content(
                    OpenState::Open,
                    Some("showcase-select-content"),
                    Some("showcase-select-label"),
                    None,
                    vec![],
                    vec![
                        select::item(
                            OpenState::Open,
                            false,
                            false,
                            "fandhe-frontend",
                            Some("showcase-select-item-fandhe"),
                            vec![],
                            vec![
                                select::item_text(None, vec![], vec![text("fandhe-frontend")]),
                                select::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                        select::item(
                            OpenState::Closed,
                            false,
                            false,
                            "other",
                            None,
                            vec![],
                            vec![
                                select::item_text(None, vec![], vec![text("Other framework")]),
                                select::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    );
    section(
        "Select",
        "headless-ui の Select（role=\"listbox\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。1 項目が選択済み（data-state=\"open\"）の listbox が開いた状態を固定表示しています。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Combobox 節: 入力によるフィルタリング後の listbox が開いた静的マークアップ
/// （イシュー #749）。[`combobox::filter_options`] を実演し、入力値
/// `"re"` に対するフィルタ結果（`"React"` のみ）をそのまま候補として掲示する。
fn combobox_section() -> Node {
    let options = [("vue", "Vue"), ("react", "React"), ("svelte", "Svelte")];
    let query = "re";
    let filtered = combobox::filter_options(&options, query);

    let items = filtered
        .into_iter()
        .map(|(value, label)| {
            combobox::item(
                OpenState::Closed,
                false,
                false,
                value,
                None,
                vec![],
                vec![combobox::item_text(None, vec![], vec![text(label)])],
            )
        })
        .collect();

    let node = combobox::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            combobox::label(
                Some("showcase-combobox-label"),
                Some("showcase-combobox-input"),
                vec![],
                vec![text("Framework")],
            ),
            combobox::control(
                OpenState::Open,
                vec![],
                vec![
                    combobox::input(
                        OpenState::Open,
                        query,
                        false,
                        Some("showcase-combobox-content"),
                        None,
                        None,
                        vec![("id", "showcase-combobox-input")],
                    ),
                    combobox::trigger(
                        OpenState::Open,
                        false,
                        Some("showcase-combobox-content"),
                        vec![],
                        vec![text("▾")],
                    ),
                ],
            ),
            combobox::positioner(
                OpenState::Open,
                vec![],
                vec![combobox::content(
                    OpenState::Open,
                    Some("showcase-combobox-content"),
                    Some("showcase-combobox-label"),
                    vec![],
                    items,
                )],
            ),
        ],
    );
    section(
        "Combobox",
        &format!(
            "headless-ui の Combobox（role=\"combobox\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。入力値 \"{query}\" による filter_options の絞り込み結果を候補として表示しています。positioner はフロー内配置へ中和しています。"
        ),
        vec![node],
    )
}

/// TreeView 節: 2〜3 階層の静的コレクション（イシュー #753）。
///
/// "src" ブランチのみ展開済み（`data-state="open"`）、"src/lib.rs" を選択中
/// （`data-selected`）で固定掲示する。positioner を持たないため
/// [`SHOWCASE_LAYOUT_CSS`] の中和ルール追加は不要（[`mod@tree_view`]
/// module doc「`size`/`color-palette` variant を提供しない」節参照）。
fn tree_view_section() -> Node {
    // SSR は本来 dispatch 履歴なしの初期状態から始まるが、ショーケースは
    // 「展開・選択済みの見た目」を固定掲示する目的のため、他セクション
    // （Accordion/Combobox 等）と同じく意図的に dispatch で非初期状態を作る。
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;
    let mut tree = TreeView::default();
    dispatch(&mut tree, "expand", "src");
    dispatch(&mut tree, "select", "src/lib.rs");

    let nodes = vec![
        TreeNode::new("src", "src").with_children(vec![
            TreeNode::new("src/lib.rs", "lib.rs"),
            TreeNode::new("src/nested", "nested")
                .with_children(vec![TreeNode::new("src/nested/util.rs", "util.rs")]),
        ]),
        TreeNode::new("Cargo.toml", "Cargo.toml"),
        TreeNode::new("README.md", "README.md").disabled(true),
    ];

    let root_children = tree.render_nodes(&nodes);
    let node = tree_view::root(
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(Some("Project files"), None, vec![], root_children),
        ],
    );

    section(
        "TreeView",
        "headless-ui の TreeView（role=\"tree\"/role=\"treeitem\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。\"src\" ブランチを展開済み、\"src/lib.rs\" を選択中、\"README.md\" を disabled として固定表示しています。インデントは CSS custom property（--fandhe-tree-view-indent）で表現しています。",
        vec![node],
    )
}

/// Popover 節: 開いた状態の静的マークアップ（イシュー #691）。
///
/// [`dialog_section`] と同じく、実際の overlay 配置は recipe CSS
/// （`crates/pre-styled-ui/src/popover.rs`）が担い、掲示用にのみフロー内配置へ
/// 中和している。
fn popover_section() -> Node {
    let node = popover::root(
        OpenState::Open,
        vec![],
        vec![
            popover::trigger(
                OpenState::Open,
                false,
                Some("showcase-popover-content"),
                vec![],
                vec![text("More info")],
            ),
            popover::positioner(
                OpenState::Open,
                vec![],
                vec![popover::content(
                    OpenState::Open,
                    Some("showcase-popover-content"),
                    Some("showcase-popover-title"),
                    Some("showcase-popover-desc"),
                    vec![],
                    vec![
                        popover::title(
                            Some("showcase-popover-title"),
                            vec![],
                            vec![text("About this feature")],
                        ),
                        popover::description(
                            Some("showcase-popover-desc"),
                            vec![],
                            vec![text("必要なときだけ表示される補足情報です。")],
                        ),
                        popover::close_trigger(vec![], vec![text("Close")]),
                    ],
                )],
            ),
        ],
    );
    section(
        "Popover",
        "headless-ui の Popover（role=\"dialog\"、非モーダル）に pre-styled-ui の recipe CSS を適用した静的掲示です。positioner はフロー内配置へ中和しています（実際の overlay 配置は recipe CSS が担います）。",
        vec![node],
    )
}

/// Tooltip 節: 開いた状態の静的マークアップ（イシュー #691）。
fn tooltip_section() -> Node {
    let node = tooltip::root(
        OpenState::Open,
        vec![],
        vec![
            tooltip::trigger(
                OpenState::Open,
                false,
                Some("showcase-tooltip-content"),
                vec![],
                vec![text("Hover target")],
            ),
            tooltip::positioner(
                OpenState::Open,
                vec![],
                vec![tooltip::content(
                    OpenState::Open,
                    Some("showcase-tooltip-content"),
                    vec![],
                    vec![text("補足のヒントテキストです。")],
                )],
            ),
        ],
    );
    section(
        "Tooltip",
        "headless-ui の Tooltip（role=\"tooltip\"、WAI-ARIA tooltip パターン）に pre-styled-ui の recipe CSS を適用した静的掲示です。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Switch 節: unchecked / checked / disabled の 3 態。
///
/// headless 層は `"checked"`/`"unchecked"` の `data-state` 語彙で状態を
/// 表現する（open/closed ではない、`fandhe_frontend_pre_styled_ui::switch`
/// のモジュール doc 参照）。フォーム意味論は visually-hidden な
/// `<input type="checkbox" role="switch">`（[`switch::hidden_input`]）が
/// 担い、見た目（トラック/つまみ）は `control`/`thumb` が装飾として担う。
fn switch_section() -> Node {
    let states = [
        (false, false, "showcase-switch-unchecked", "Unchecked"),
        (true, false, "showcase-switch-checked", "Checked"),
        (false, true, "showcase-switch-disabled", "Disabled"),
    ];
    let demo_row = row(states
        .iter()
        .map(|(checked, disabled, name, label)| {
            switch::root(
                Size::Md,
                ColorPalette::Accent,
                *checked,
                *disabled,
                vec![],
                vec![
                    switch::hidden_input(name, "on", *checked, *disabled, false, vec![]),
                    switch::control(
                        *checked,
                        *disabled,
                        vec![],
                        vec![switch::thumb(*checked, vec![], vec![])],
                    ),
                    switch::label(*checked, vec![], vec![text(*label)]),
                ],
            )
        })
        .collect());
    section(
        "Switch",
        "data-state=\"checked\"/\"unchecked\" で見た目が切り替わるオン/オフ スイッチ。visually-hidden な input[type=\"checkbox\"][role=\"switch\"] がフォーム送信・キーボード操作の意味論を担います。",
        vec![demo_row],
    )
}

/// RadioGroup 節: 3 択のうち 1 件が選択済み・1 件が disabled な静的掲示。
///
/// `label` パーツの `id` を `root` の `labelled_by` に渡し、グループ全体の
/// 見出しとの関連付け（`aria-labelledby`）を成立させる（headless
/// `radio_group` モジュールの契約）。
fn radio_group_section() -> Node {
    let label_id = "showcase-radio-label";
    let items = [
        ("plan-free", "Free", true, false),
        ("plan-pro", "Pro", false, false),
        ("plan-enterprise", "Enterprise", false, true),
    ];
    let mut children = vec![radio_group::label(
        Some(label_id),
        vec![],
        vec![text("Plan")],
    )];
    children.extend(items.iter().map(|(value, label, checked, disabled)| {
        radio_group::item(
            *checked,
            *disabled,
            value,
            vec![],
            vec![
                radio_group::item_hidden_input(
                    *checked,
                    *disabled,
                    Some("showcase-radio"),
                    value,
                    vec![],
                ),
                radio_group::item_control(*checked, *disabled, vec![]),
                radio_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    let demo = radio_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        Some(label_id),
        vec![],
        children,
    );
    section(
        "RadioGroup",
        "単一選択の選択肢グループ。ネイティブ input[type=\"radio\"] による排他選択・キーボード操作を data-scope=\"radio-group\" の anatomy へ重ねます。",
        vec![demo],
    )
}

/// 空 data URI（画像フェッチを一切発生させない `src`。イシュー #692 実装計画
/// 「外部フェッチ・404 を発生させない値」参照）。Error 状態デモの `image` src
/// として使う。
const AVATAR_EMPTY_IMAGE_SRC: &str = "data:,";

/// パーセントエンコード済みインライン SVG data URI（生の `<`・引用符を含まず、
/// GitHub Pages 上で外部リクエスト・404 を発生させない。Loaded 状態デモの
/// `image` src として使う）。単色円のプレースホルダーアイコン。
const AVATAR_INLINE_SVG_SRC: &str =
    "data:image/svg+xml,%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%20viewBox%3D%270%200%2064%2064%27%3E%3Ccircle%20cx%3D%2732%27%20cy%3D%2732%27%20r%3D%2732%27%20fill%3D%27%234a90d9%27%2F%3E%3C%2Fsvg%3E";

/// Avatar 節: size（Sm/Md/Lg、いずれも `ImageStatus::Error` でフォールバック
/// 表示）と shape（Circle/Rounded/Square）の 2 軸。
///
/// `image` パーツの `src` は外部フェッチ・404 を発生させないダミー値
/// （[`AVATAR_EMPTY_IMAGE_SRC`]/[`AVATAR_INLINE_SVG_SRC`]）を使う
/// （`examples/headless-pre-styled-ui` の avatar 節と同じく実画像を同梱
/// しない方針）。`image` パーツ自体は `ImageStatus` に応じて headless 層が
/// `hidden` 存在属性を出力するため、Error 状態でも anatomy には含まれる。
fn avatar_section() -> Node {
    let size_row = row(vec![(Size::Sm, "FT"), (Size::Md, "FT"), (Size::Lg, "FT")]
        .into_iter()
        .map(|(size, initials)| {
            avatar::root(
                size,
                AvatarShape::default(),
                vec![],
                vec![
                    avatar::image(
                        ImageStatus::Error,
                        AVATAR_EMPTY_IMAGE_SRC,
                        "Fandhe Team",
                        vec![],
                    ),
                    avatar::fallback(ImageStatus::Error, vec![], vec![text(initials)]),
                ],
            )
        })
        .collect());

    let shape_row = row(vec![
        AvatarShape::Circle,
        AvatarShape::Rounded,
        AvatarShape::Square,
    ]
    .into_iter()
    .map(|shape| {
        avatar::root(
            Size::Md,
            shape,
            vec![],
            vec![
                avatar::image(
                    ImageStatus::Loaded,
                    AVATAR_INLINE_SVG_SRC,
                    "Fandhe Team",
                    vec![],
                ),
                avatar::fallback(ImageStatus::Loaded, vec![], vec![text("FT")]),
            ],
        )
    })
    .collect());

    section(
        "Avatar",
        "size（Sm/Md/Lg）・shape（Circle/Rounded/Square）の 2 軸を持つユーザー画像表示。画像読み込み状態（ImageStatus）を固定し、Error 時はイニシャルのフォールバック表示、Loaded 時は画像表示を掲示します。",
        vec![size_row, shape_row],
    )
}

/// Checkbox 節: unchecked / checked / indeterminate / disabled の 4 態
/// （イシュー #730）。
///
/// headless 層は checked/unchecked/indeterminate の 3 値 `data-state` 語彙を
/// 持つ（`fandhe_frontend_pre_styled_ui::checkbox` のモジュール doc
/// 参照）。フォーム意味論は visually-hidden な `<input type="checkbox">`
/// （[`checkbox::hidden_input`]）が担い、見た目（チェックマーク）は
/// `control`/`indicator` が装飾として担う。
fn checkbox_section() -> Node {
    let states = [
        (
            CheckedState::Unchecked,
            false,
            "showcase-checkbox-unchecked",
            "Unchecked",
        ),
        (
            CheckedState::Checked,
            false,
            "showcase-checkbox-checked",
            "Checked",
        ),
        (
            CheckedState::Indeterminate,
            false,
            "showcase-checkbox-indeterminate",
            "Indeterminate",
        ),
        (
            CheckedState::Checked,
            true,
            "showcase-checkbox-disabled",
            "Disabled",
        ),
    ];
    let demo_row = row(states
        .iter()
        .map(|(checked, disabled, name, label)| {
            let props = CheckboxProps {
                checked: *checked,
                disabled: *disabled,
                ..CheckboxProps::default()
            };
            checkbox::root(
                Size::Md,
                ColorPalette::Accent,
                &props,
                vec![],
                vec![
                    checkbox::hidden_input(&props, name, "on", vec![]),
                    checkbox::control(
                        &props,
                        vec![],
                        vec![checkbox::indicator(&props, vec![], vec![])],
                    ),
                    checkbox::label(&props, vec![], vec![text(*label)]),
                ],
            )
        })
        .collect());
    section(
        "Checkbox",
        "data-state=\"checked\"/\"unchecked\"/\"indeterminate\" の 3 態を持つチェックボックス。visually-hidden な input[type=\"checkbox\"] がフォーム送信・キーボード操作の意味論を担い、チェックマークは CSS の border 合成で描画します（画像アセット不使用）。",
        vec![demo_row],
    )
}

/// Input / Textarea / NativeSelect 節（イシュー #737）。
///
/// 状態機械を持たない静的フォーム部品 3 種。アクセシビリティ配線
/// （`id`・ネイティブ `disabled`/`required`/`readonly`・`aria-invalid`・
/// `aria-describedby`・`data-*`）は headless `field::*`（#538/#602）へ全面
/// 委譲するため、本節では invalid/disabled の 2 態と variant/size の切り替え
/// のみを掲示する（`fandhe_frontend_pre_styled_ui::input` モジュール doc
/// 参照）。`color-palette` 軸は提供しない設計のため掲示しない。
fn form_controls_section() -> Node {
    let plain_field = |id: &'static str| FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let invalid_field = |id: &'static str| FieldProps {
        invalid: true,
        ..plain_field(id)
    };
    let disabled_field = |id: &'static str| FieldProps {
        disabled: true,
        ..plain_field(id)
    };

    let input_row = row(vec![
        input::input(
            &InputProps::default(),
            &plain_field("showcase-input-default"),
            vec![("placeholder", "Outline (default)")],
        ),
        // invalid 時、headless `field::input` は `aria-describedby` に
        // `{id}-error-text` を出力する（`field.rs` の describedby 合成則）。
        // 参照先の id を持つ `field::error_text` を併設し、存在しない id への
        // 参照を残さない（Bugbot 指摘、PR #783）。
        //
        // ラッパー div には `.showcase-form-field-group`（`width: 100%`）を
        // 付与する。付与しないと `showcase-row` の直接 flex item がこの div
        // になり、兄弟 input が持つ `width: 100%`（`field` recipe base）による
        // flex-basis 解決を div 自身が持たず auto（contents 由来の縮小）に
        // なってしまい、Invalid デモだけ Default/Disabled より狭く描画される
        // （Bugbot 指摘、PR #783 review）。
        div(
            vec![("class", "showcase-form-field-group")],
            vec![
                input::input(
                    &InputProps::default(),
                    &invalid_field("showcase-input-invalid"),
                    vec![("placeholder", "Invalid")],
                ),
                input::error_text(
                    &invalid_field("showcase-input-invalid"),
                    vec![],
                    vec![text("This field is required.")],
                ),
            ],
        ),
        input::input(
            &InputProps::default(),
            &disabled_field("showcase-input-disabled"),
            vec![("placeholder", "Disabled")],
        ),
    ]);

    let textarea_row = row(vec![textarea::textarea(
        &TextareaProps::default(),
        &plain_field("showcase-textarea-default"),
        false,
        vec![("placeholder", "Outline (default)")],
        vec![],
    )]);

    let native_select_row = row(vec![native_select::native_select(
        &NativeSelectProps::default(),
        &plain_field("showcase-native-select-default"),
        vec![],
        vec![
            el("option", vec![("value", "jp")], vec![text("Japan")]),
            el("option", vec![("value", "us")], vec![text("United States")]),
        ],
    )]);

    section(
        "Input / Textarea / NativeSelect",
        "ブラウザネイティブ挙動をそのまま尊重する静的フォーム部品 3 種。invalid/disabled 状態は headless field:: へ委譲した data-* 属性・aria-invalid で表現します。",
        vec![input_row, textarea_row, native_select_row],
    )
}

/// NumberInput 節: 中間値・境界値（min 到達で decrement disabled）・
/// disabled の 3 態。
///
/// headless 層は連続量の値を扱うため `data-state` を持たず、境界到達は
/// increment/decrement トリガーの `data-disabled` 存在属性のみで表現する
/// （`fandhe_frontend_pre_styled_ui::number_input` のモジュール doc 参照）。
fn number_input_section() -> Node {
    let mid = number_input::root(
        Size::Md,
        false,
        false,
        vec![],
        vec![
            number_input::label(
                false,
                false,
                Some("showcase-number-input-mid"),
                vec![],
                vec![text("Quantity")],
            ),
            number_input::control(
                false,
                false,
                vec![],
                vec![
                    number_input::input(
                        "quantity",
                        Some("showcase-number-input-mid"),
                        Some("5"),
                        "0",
                        "10",
                        NumberInputFlags::default(),
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("showcase-number-input-mid"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                    number_input::decrement_trigger(
                        Some("showcase-number-input-mid"),
                        false,
                        vec![],
                        vec![text("-")],
                    ),
                ],
            ),
        ],
    );
    let at_min = number_input::root(
        Size::Md,
        false,
        false,
        vec![],
        vec![
            number_input::label(
                false,
                false,
                Some("showcase-number-input-min"),
                vec![],
                vec![text("At min")],
            ),
            number_input::control(
                false,
                false,
                vec![],
                vec![
                    number_input::input(
                        "quantity-min",
                        Some("showcase-number-input-min"),
                        Some("0"),
                        "0",
                        "10",
                        NumberInputFlags::default(),
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("showcase-number-input-min"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                    // 下限到達のため decrement トリガーを disabled にする
                    // （境界到達時の唯一の視覚的合図、モジュール doc 参照）。
                    number_input::decrement_trigger(
                        Some("showcase-number-input-min"),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                ],
            ),
        ],
    );
    let disabled = number_input::root(
        Size::Md,
        true,
        false,
        vec![],
        vec![
            number_input::label(
                true,
                false,
                Some("showcase-number-input-disabled"),
                vec![],
                vec![text("Disabled")],
            ),
            number_input::control(
                true,
                false,
                vec![],
                vec![
                    number_input::input(
                        "quantity-disabled",
                        Some("showcase-number-input-disabled"),
                        Some("3"),
                        "0",
                        "10",
                        NumberInputFlags {
                            disabled: true,
                            ..NumberInputFlags::default()
                        },
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("showcase-number-input-disabled"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                    number_input::decrement_trigger(
                        Some("showcase-number-input-disabled"),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                ],
            ),
        ],
    );
    let demo_row = row(vec![mid, at_min, disabled]);
    section(
        "NumberInput",
        "min/max/step でクランプされる数値入力。increment/decrement トリガーは境界到達時に data-disabled を伴い無効化されます。",
        vec![demo_row],
    )
}

/// TagsInput 節: 通常タグ数件・max 到達（`data-invalid`/`aria-invalid`）・
/// disabled の 3 態。
///
/// `control` は `role="listbox"`、各タグの `item-preview` は `role="option"`
/// （headless 層の listbox 相当 ARIA、`fandhe_frontend_pre_styled_ui::tags_input`
/// のモジュール doc 参照）。SSR 静的掲示のため編集モード
/// （`item-input`/`data-editing`）は掲載しない（wasm 層の対話が必要なため、
/// モジュール rustdoc「スコープ外」節参照）。
fn tags_input_section() -> Node {
    fn tag_item(tag: &str, disabled: bool) -> Node {
        tags_input::item(
            disabled,
            false,
            vec![],
            vec![tags_input::item_preview(
                false,
                vec![],
                vec![
                    tags_input::item_text(vec![], vec![text(tag)]),
                    tags_input::item_delete_trigger(tag, disabled, vec![], vec![text("\u{00d7}")]),
                ],
            )],
        )
    }

    let normal = tags_input::root(
        Size::Md,
        false,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("Skills")]),
            tags_input::control(
                false,
                false,
                "Skills",
                vec![],
                vec![
                    tag_item("rust", false),
                    tag_item("wasm", false),
                    tags_input::input("", false, false, vec![]),
                ],
            ),
            tags_input::hidden_input("skills", "rust,wasm", false, vec![]),
        ],
    );

    let at_max = tags_input::root(
        Size::Md,
        false,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("At max (2)")]),
            tags_input::control(
                false,
                // max 到達のため `control` へ data-invalid、`input` へ
                // data-invalid/aria-invalid を出力する（境界到達時の唯一の
                // 視覚的合図、モジュール rustdoc「セキュリティ不変条件」節参照）。
                true,
                "At max",
                vec![],
                vec![
                    tag_item("a", false),
                    tag_item("b", false),
                    tags_input::input("", false, true, vec![]),
                ],
            ),
            tags_input::hidden_input("at-max", "a,b", false, vec![]),
        ],
    );

    let disabled = tags_input::root(
        Size::Md,
        true,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("Disabled")]),
            tags_input::control(
                true,
                false,
                "Disabled",
                vec![],
                vec![
                    tag_item("readonly", true),
                    tags_input::input("", true, false, vec![]),
                ],
            ),
            tags_input::hidden_input("disabled-tags", "readonly", true, vec![]),
        ],
    );

    let demo_row = row(vec![normal, at_max, disabled]);
    section(
        "TagsInput",
        "自由入力によるタグ配列。control は role=\"listbox\"、各タグは role=\"option\" を持ち、max 到達時は input が data-invalid/aria-invalid を伴います。",
        vec![demo_row],
    )
}

/// RatingGroup 節: 選択中（value=3）・readonly（他ユーザーの平均評価想定）・
/// disabled の 3 態。星形 indicator は外部リソース非参照の `clip-path`
/// インライン表現（`fandhe_frontend_pre_styled_ui::rating_group` のモジュール
/// doc「星形 indicator」節参照）。`hidden_input` はフォーム送信用の現在値を
/// 送るネイティブ input（視覚上非表示、`display: none` の既定 CSS）。
fn rating_group_section() -> Node {
    let build = |id_prefix: &'static str, value: Option<u32>, disabled: bool, readonly: bool| {
        let g = RatingGroup::new(5, value, readonly);
        let label_id = format!("{id_prefix}-label");
        let mut children = vec![rating_group::label(
            Some(label_id.as_str()),
            vec![],
            vec![text("Rate this product")],
        )];
        let items: Vec<Node> = (1..=g.count())
            .map(|i| {
                let checked = g.is_checked(i);
                let highlighted = g.is_highlighted(i);
                rating_group::item(
                    i,
                    RatingItemFlags {
                        checked,
                        highlighted,
                        disabled,
                        readonly,
                    },
                    &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                    vec![],
                    vec![],
                )
            })
            .collect();
        children.push(rating_group::control(
            Some(label_id.as_str()),
            vec![],
            items,
        ));
        children.push(rating_group::hidden_input(
            Some("rating"),
            g.value_text().as_str(),
            disabled,
            vec![],
        ));
        rating_group::root(
            Size::Md,
            ColorPalette::Accent,
            disabled,
            readonly,
            vec![],
            children,
        )
    };

    let selected = build("showcase-rating-selected", Some(3), false, false);
    let readonly = build("showcase-rating-readonly", Some(4), false, true);
    let disabled = build("showcase-rating-disabled", Some(2), true, false);

    section(
        "RatingGroup",
        "1..=count の星評価。data-highlighted が塗り表示（hover プレビュー優先）、data-checked が確定選択を表します。星形は SVG/画像 URL を使わない clip-path によるインライン表現です。",
        vec![row(vec![selected, readonly, disabled])],
    )
}

/// Slider 節: 中間値・境界値（max 到達）・disabled の 3 態。
///
/// `range`/`thumb` の塗りつぶし・位置は headless 中立な
/// [`Slider::percent`] から導出する `--fandhe-slider-percent` CSS custom
/// property の 1 点のみで伝搬する
/// （`fandhe_frontend_pre_styled_ui::slider` のモジュール doc 参照）。
fn slider_section() -> Node {
    let mid_state = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
    let mid = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &mid_state,
        false,
        vec![],
        vec![
            slider::label(vec![], vec![text("Volume")]),
            slider::control(
                Orientation::Horizontal,
                false,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        false,
                        vec![],
                        vec![slider::range(&mid_state, false, vec![])],
                    ),
                    slider::thumb_styled(&mid_state, Some("40 percent"), false, vec![]),
                ],
            ),
            slider::hidden_input("volume", "40", false, vec![]),
        ],
    );

    let at_max_state = Slider::new(0.0, 100.0, 1.0, 100.0, Orientation::Horizontal);
    let at_max = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &at_max_state,
        false,
        vec![],
        vec![
            slider::label(vec![], vec![text("At max")]),
            slider::control(
                Orientation::Horizontal,
                false,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        false,
                        vec![],
                        vec![slider::range(&at_max_state, false, vec![])],
                    ),
                    slider::thumb_styled(&at_max_state, Some("100 percent"), false, vec![]),
                ],
            ),
            slider::hidden_input("volume-max", "100", false, vec![]),
        ],
    );

    let disabled_state = Slider::new(0.0, 100.0, 1.0, 25.0, Orientation::Horizontal);
    let disabled = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &disabled_state,
        true,
        vec![],
        vec![
            slider::label(vec![], vec![text("Disabled")]),
            slider::control(
                Orientation::Horizontal,
                true,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        true,
                        vec![],
                        vec![slider::range(&disabled_state, true, vec![])],
                    ),
                    slider::thumb_styled(&disabled_state, Some("25 percent"), true, vec![]),
                ],
            ),
            slider::hidden_input("volume-disabled", "25", true, vec![]),
        ],
    );

    let demo_row = row(vec![mid, at_max, disabled]);
    section(
        "Slider",
        "min/max/step でクランプされる連続値スライダー。塗りつぶし・つまみの位置は --fandhe-slider-percent の 1 点で伝搬します。",
        vec![demo_row],
    )
}

/// SegmentGroup 節（イシュー #743）: 既定（選択済み）・disabled・Size 3 種の
/// 静的掲示。状態機械（[`fandhe_frontend_pre_styled_ui::segment_group::SegmentGroup`]、
/// `radio_group::RadioGroup` への全委譲）は使わず、他の docs-site 節と同じく
/// SSR 静的マークアップのみを組み立てる（本モジュール冒頭「インタラクティブ
/// 部品の扱い」節参照）。indicator の位置は選択項目の `(index, count)` から
/// 手計算で `segment_group::indicator` へ渡す（headless 層の SSR 決定的な
/// 位置表現契約、`crates/headless-ui/src/segment_group.rs` module doc 参照）。
fn segment_group_demo(id_prefix: &str, size: Size, disabled: bool, selected_index: usize) -> Node {
    let items = ["List", "Grid", "Table"];
    let mut children = vec![segment_group::indicator(
        Some((selected_index, items.len())),
        None,
        vec![],
    )];
    children.extend(items.iter().enumerate().map(|(index, label)| {
        let checked = index == selected_index;
        let value = label.to_lowercase();
        segment_group::item(
            checked,
            disabled,
            &value,
            vec![],
            vec![
                segment_group::item_hidden_input(
                    checked,
                    disabled,
                    Some(id_prefix),
                    &value,
                    vec![],
                ),
                segment_group::item_control(checked, disabled, vec![]),
                segment_group::item_text(checked, disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    segment_group::root(size, disabled, None, None, vec![], children)
}

fn segment_group_section() -> Node {
    let size_row = row(vec![
        segment_group_demo("showcase-segment-sm", Size::Sm, false, 0),
        segment_group_demo("showcase-segment-md", Size::Md, false, 1),
        segment_group_demo("showcase-segment-lg", Size::Lg, false, 2),
    ]);
    let disabled_demo = segment_group_demo("showcase-segment-disabled", Size::Md, true, 0);
    section(
        "SegmentGroup",
        "単一選択のセグメント UI（segmented control）。ネイティブ input[type=\"radio\"] による排他選択を data-scope=\"segment-group\" の anatomy へ重ね、選択中の項目を indicator の CSS 変数（--fandhe-segment-group-index/-count）で示します。状態機械は RadioGroup（SingleSelect）への全委譲です。",
        vec![size_row, disabled_demo],
    )
}

/// Pagination 節（イシュー #751）: `page_entries()` から ellipsis を含む
/// ページ列を組み立てた静的掲示 + 現在ページ・prev/next の disabled 連動。
/// 状態機械は SSR 静的な現在ページの固定表示のみ（クリック挙動は wasm 層の
/// スコープ外、モジュール冒頭「インタラクティブ部品の扱い」節参照）。
fn pagination_section() -> Node {
    // 総ページ数 20（count=200, page_size=10）、page=10 で両側 ellipsis を
    // 固定掲示する（headless 層のテスト `both_ellipsis` と同じ入力）。
    let p = Pagination::new(200, 10, 1, 1, 10);
    let mut children = vec![p.prev_trigger(ItemMode::Button, vec![], vec![text("Prev")])];
    for entry in p.page_entries() {
        match entry {
            pagination::PageEntry::Page(n) => {
                children.push(p.item(
                    ItemMode::Button,
                    n,
                    false,
                    vec![],
                    vec![text(n.to_string())],
                ));
            }
            pagination::PageEntry::Ellipsis => {
                children.push(pagination::ellipsis(vec![], vec![text("…")]));
            }
        }
    }
    children.push(p.next_trigger(ItemMode::Button, vec![], vec![text("Next")]));

    let demo = pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "pagination",
        vec![],
        children,
    );
    section(
        "Pagination",
        "総件数・ページサイズ・現在ページから省略記号（ellipsis）を含むページ列を決定的に導出する headless Pagination の静的掲示。現在ページは aria-current=\"page\"/data-selected で、端到達は prev/next の disabled で表現します（クリック挙動は wasm 層のスコープ外）。",
        vec![row(vec![demo])],
    )
}

/// CheckboxCard 節: unchecked / checked / disabled の 3 態（イシュー #747）。
///
/// chakra-ui checkbox-card 相当のカード型選択 UI。状態機械は
/// [`fandhe_frontend_pre_styled_ui::checkbox`] 節と同じ headless `Checkbox`/
/// `CheckboxProps` を再利用し、`data-scope="checkbox-card"` の新規 anatomy
/// （`crates/pre-styled-ui/src/checkbox_card.rs` 参照）でカード外観を重ねる。
fn checkbox_card_section() -> Node {
    let states = [
        (
            CheckedState::Unchecked,
            false,
            "showcase-checkbox-card-unchecked",
            "Starter",
            "個人利用向けの基本プラン。",
        ),
        (
            CheckedState::Checked,
            false,
            "showcase-checkbox-card-checked",
            "Pro",
            "チームでの共同作業に対応。",
        ),
        (
            CheckedState::Checked,
            true,
            "showcase-checkbox-card-disabled",
            "Enterprise",
            "現在準備中のプランです。",
        ),
    ];
    let demo_row = row(states
        .iter()
        .map(|(checked, disabled, name, label, description)| {
            let props = CheckboxProps {
                checked: *checked,
                disabled: *disabled,
                ..CheckboxProps::default()
            };
            checkbox_card::root(
                Size::Md,
                ColorPalette::Accent,
                &props,
                vec![],
                vec![
                    checkbox_card::hidden_input(&props, name, "on", vec![]),
                    checkbox_card::control(
                        &props,
                        vec![],
                        vec![
                            checkbox_card::indicator(
                                &props,
                                vec![],
                                vec![checkbox_card::indicator_check(&props, vec![], vec![])],
                            ),
                            checkbox_card::content(
                                &props,
                                vec![],
                                vec![
                                    checkbox_card::label(&props, vec![], vec![text(*label)]),
                                    checkbox_card::description(
                                        &props,
                                        vec![],
                                        vec![text(*description)],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            )
        })
        .collect());
    section(
        "CheckboxCard",
        "chakra-ui checkbox-card 相当のカード型選択 UI。状態機械は Checkbox（headless）をそのまま再利用し、data-scope=\"checkbox-card\" の新規 anatomy でカード外観を重ねます。",
        vec![demo_row],
    )
}

/// RadioCard 節: 単一選択のカード型選択 UI（イシュー #747）。
///
/// 状態機械は [`fandhe_frontend_pre_styled_ui::radio_group`] 節と同じ headless
/// `RadioGroup`（`SingleSelect`）をそのまま再利用し、
/// `data-scope="radio-card"` の新規 anatomy（`crates/pre-styled-ui/src/radio_card.rs`
/// 参照）でカード外観を重ねる。
fn radio_card_section() -> Node {
    let label_id = "showcase-radio-card-label";
    let items = [
        (
            "plan-free-card",
            "Free",
            "基本機能のみ利用可能。",
            true,
            false,
        ),
        (
            "plan-pro-card",
            "Pro",
            "チーム機能・優先サポート付き。",
            false,
            false,
        ),
        (
            "plan-enterprise-card",
            "Enterprise",
            "SSO・監査ログに対応。",
            false,
            true,
        ),
    ];
    let mut children = vec![radio_card::label(
        Some(label_id),
        vec![],
        vec![text("Plan")],
    )];
    children.extend(
        items
            .iter()
            .map(|(value, label, description, checked, disabled)| {
                radio_card::item(
                    *checked,
                    *disabled,
                    value,
                    vec![],
                    vec![
                        radio_card::item_hidden_input(
                            *checked,
                            *disabled,
                            Some("showcase-radio-card"),
                            value,
                            vec![],
                        ),
                        radio_card::item_control(
                            *checked,
                            *disabled,
                            vec![],
                            vec![
                                radio_card::item_indicator(*checked, *disabled, vec![]),
                                radio_card::item_content(
                                    vec![],
                                    vec![
                                        radio_card::item_text(vec![], vec![text(*label)]),
                                        radio_card::item_description(
                                            vec![],
                                            vec![text(*description)],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                )
            }),
    );
    let demo = radio_card::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        Some(label_id),
        vec![],
        children,
    );
    section(
        "RadioCard",
        "chakra-ui radio-card 相当のカード型選択 UI。状態機械は RadioGroup（headless）をそのまま再利用し、data-scope=\"radio-card\" の新規 anatomy でカード外観を重ねます。",
        vec![demo],
    )
}

/// Breadcrumb 節: `size`/[`BreadcrumbVariant`] を既定値で掲示する（イシュー
/// #755）。状態機械を持たない静的意味論ナビのため、開閉等の状態掲示は不要
/// （3 階層のパンくずをそのまま組み立てる）。
fn breadcrumb_section() -> Node {
    // `href` は空文字列（`fandhe_frontend_core::render` の URL 検証上は
    // 相対 URL として許可されるが、linkcheck 対象からは除外される。
    // `crate::linkcheck::check_links` は空 href を無条件でスキップする
    // 契約であり、生成コンテンツを linkcheck の突合対象へ含めない本モジュール
    // の既存設計（`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality`
    // 参照）を壊さずに `link` パーツ（実際に `href` 属性を持つ要素）を掲示する
    // ための選択。実サイトへの導線が必要な利用は呼び出し側アプリケーションの
    // 責務（本ショーケースは recipe CSS の見た目確認が目的）。
    let items = [
        BreadcrumbItem {
            label: "Docs",
            href: "",
        },
        BreadcrumbItem {
            label: "Components",
            href: "",
        },
        BreadcrumbItem {
            label: "Breadcrumb",
            href: "",
        },
    ];
    let node = breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::Plain,
        None,
        vec![],
        vec![breadcrumb::list(
            vec![],
            items
                .iter()
                .enumerate()
                .flat_map(|(index, entry)| {
                    let inner = if index == items.len() - 1 {
                        breadcrumb::current_link(vec![], vec![text(entry.label)])
                    } else {
                        breadcrumb::link(entry.href, vec![], vec![text(entry.label)])
                    };
                    let mut parts = vec![breadcrumb::item(vec![], vec![inner])];
                    if index != items.len() - 1 {
                        parts.push(breadcrumb::separator(vec![], vec![text("/")]));
                    }
                    parts
                })
                .collect(),
        )],
    );
    section(
        "Breadcrumb",
        "headless-ui の Breadcrumb（nav[aria-label=\"breadcrumb\"] + ol/li）に pre-styled-ui の recipe CSS を適用した静的掲示です。末尾項目のみ aria-current=\"page\"/data-current を持つ非対話の現在位置表示（span）として描画します。",
        vec![node],
    )
}

/// colorPalette 軸の全値（表示ラベル付き）。Button / Badge の palette 行で
/// 共有する。
fn palettes() -> [(ColorPalette, &'static str); 5] {
    [
        (ColorPalette::Accent, "Accent"),
        (ColorPalette::Info, "Info"),
        (ColorPalette::Success, "Success"),
        (ColorPalette::Warning, "Warning"),
        (ColorPalette::Danger, "Danger"),
    ]
}

/// ショーケース本文全体（Markdown 本文の直後へ追記される `Node` 木）。
fn showcase_body() -> Node {
    div(
        vec![("class", "pre-styled-showcase")],
        vec![
            button_section(),
            badge_section(),
            spinner_section(),
            alert_section(),
            card_section(),
            tabs_section(),
            accordion_section(),
            dialog_section(),
            menu_section(),
            select_section(),
            combobox_section(),
            popover_section(),
            tooltip_section(),
            switch_section(),
            radio_group_section(),
            avatar_section(),
            checkbox_section(),
            form_controls_section(),
            number_input_section(),
            tags_input_section(),
            rating_group_section(),
            slider_section(),
            segment_group_section(),
            tree_view_section(),
            pagination_section(),
            checkbox_card_section(),
            radio_card_section(),
            breadcrumb_section(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn generated_content_matches_only_showcase_path() {
        assert!(generated_content(PAGE_PATH).is_some());
        assert!(generated_content("/").is_none());
        assert!(generated_content("/guides/embedding-guide/").is_none());
    }

    #[test]
    fn showcase_markup_contains_all_component_scopes() {
        let html = render(&showcase_body());
        for scope in [
            "button",
            "badge",
            "spinner",
            "alert",
            "card",
            "tabs",
            "accordion",
            "dialog",
            "menu",
            "select",
            "popover",
            "tooltip",
            "switch",
            "radio-group",
            "avatar",
            "checkbox",
            "field",
            "number-input",
            "tags-input",
            "rating-group",
            "slider",
            "segment-group",
            "pagination",
            "checkbox-card",
            "radio-card",
            "breadcrumb",
        ] {
            assert!(
                html.contains(&format!(r#"data-scope="{scope}""#)),
                "missing data-scope={scope}"
            );
        }
        // Input / Textarea / NativeSelect（イシュー #737）: field scope 内の
        // 3 パーツすべてが掲示されていることを固定する。
        for part in ["input", "textarea", "select"] {
            assert!(
                html.contains(&format!(r#"data-scope="field" data-part="{part}""#)),
                "missing data-scope=field data-part={part}"
            );
        }
        // 静的掲示の状態固定: 選択中タブ・開いた Accordion 項目・checked
        // Switch/RadioGroup item・indeterminate Checkbox。
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"data-state="indeterminate""#));
    }

    #[test]
    fn showcase_markup_fixes_overlay_components_open_with_wai_aria_roles() {
        // イシュー #691 受け入れ条件: Dialog/Menu/Select/Popover/Tooltip は
        // 開いた状態を固定し、対応する WAI-ARIA role/属性が出力されることを
        // 固定する（headless 層の既存保証をショーケース掲示側でも回帰させる）。
        let html = render(&showcase_body());
        assert!(html.contains(r#"aria-modal="true""#)); // dialog content
        assert!(html.contains(r#"role="menu""#));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"role="tooltip""#));
        assert!(html.contains(r#"aria-expanded="true""#)); // trigger 群（開状態）
        assert!(html.contains(r#"aria-haspopup="dialog""#)); // dialog/popover trigger
        assert!(html.contains(r#"aria-haspopup="menu""#));
        assert!(html.contains(r#"aria-haspopup="listbox""#));
    }

    #[test]
    fn showcase_markup_has_no_href_attributes_for_linkcheck_neutrality() {
        // build.rs の linkcheck は全 href を突合検証する。生成コンテンツは
        // 実ページへ解決される href を持たない設計とし、リンク検証対象を
        // Markdown 側へ限定する。イシュー #755 で Breadcrumb（`link` パーツ、
        // 実際に `href` 属性を持つ anatomy）を掲示したため、本テストは
        // 「`href=""`（空文字列、`crate::linkcheck::check_links` が無条件
        // スキップする値）以外の href が存在しないこと」へ更新した
        // （`showcase::breadcrumb_section` rustdoc 参照。空 href 以外を
        // 足す場合はこのテストを更新して linkcheck との整合を明示的に
        // 設計し直すこと）。
        let html = render(&showcase_body());
        let non_empty_hrefs: Vec<&str> = html
            .match_indices("href=\"")
            .filter(|(i, _)| !html[i + 6..].starts_with('"'))
            .map(|(i, _)| &html[i..i + 20.min(html.len() - i)])
            .collect();
        assert!(
            non_empty_hrefs.is_empty(),
            "non-empty href found: {non_empty_hrefs:?}"
        );
    }

    #[test]
    fn stylesheet_covers_theme_component_and_layout_css() {
        let sheet = stylesheet().expect("showcase stylesheet should assemble");
        let css = sheet.as_css();
        // テーマトークン（ライト/ダーク基盤）。
        assert!(css.contains("--fandhe-color-"));
        // 各コンポーネントの recipe セレクタ。
        assert!(css.contains(".fd-button--variant-solid"));
        assert!(css.contains(".fd-badge--variant-subtle"));
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
        assert!(css.contains(r#"[data-scope="accordion"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="select"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="popover"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="switch"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item-control"]"#));
        assert!(css.contains(".fd-avatar--size-md"));
        assert!(css.contains(".fd-avatar--shape-circle"));
        assert!(css.contains(r#"[data-scope="checkbox"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="checkbox-card"][data-part="indicator"]"#));
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item-indicator"]"#));
        assert!(css.contains(r#"[data-scope="field"][data-part="input"]"#));
        assert!(css.contains(r#"[data-scope="field"][data-part="textarea"]"#));
        assert!(css.contains(r#"[data-scope="field"][data-part="select"]"#));
        assert!(css.contains(r#"[data-scope="number-input"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="control"]"#));
        // ショーケース配置スタイル。
        assert!(css.contains(".showcase-row"));
        assert!(css.contains(".showcase-stack"));
        // Accordion anatomy の h3 への `.docs-content h3`（site.css）漏れを
        // 遮断する見出しリセット（Bugbot 指摘の回帰防止）。
        assert!(css.contains(r#".pre-styled-showcase [data-scope="accordion"] h3"#));
        // オーバーレイ配置中和ルール（イシュー #691）。
        assert!(css.contains(r#".pre-styled-showcase [data-scope="dialog"][data-part="backdrop"]"#));
        assert!(
            css.contains(r#".pre-styled-showcase [data-scope="dialog"][data-part="positioner"]"#)
        );
        assert!(css.contains(r#".pre-styled-showcase [data-scope="menu"][data-part="positioner"]"#));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="dialog"] h2"#));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="popover"] h2"#));
        // StyleSheet の不変条件（<style> 埋め込み・CSS ファイル双方で安全）。
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet().unwrap().as_css().to_string();
        let b = stylesheet().unwrap().as_css().to_string();
        assert_eq!(a, b);
    }
}
