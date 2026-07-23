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
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::tabs::{tabs, ActivationMode, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::{
    accordion, alert, badge, card, menu, popover, radio_group, select, switch, tooltip,
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
/// - dialog/menu/select/popover/tooltip の `[data-part="positioner"]` を
///   `position: static` へ中和: recipe CSS は dialog を
///   `position: fixed; inset: 0`、menu/select/popover を
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
.pre-styled-showcase [data-scope=\"accordion\"] h3 {\n  margin: 0;\n  font-size: 1rem;\n  font-weight: 400;\n  line-height: 1.5;\n  letter-spacing: normal;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  display: none;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: static;\n  padding: 0;\n  justify-content: flex-start;\n}\n\
.pre-styled-showcase [data-scope=\"menu\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"select\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"popover\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"tooltip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
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
/// accordion/dialog/menu/select/popover/tooltip/switch/radio_group/avatar）
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
    sheet.push_css(&fandhe_frontend_pre_styled_ui::popover::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tooltip::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::switch::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::radio_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::avatar::stylesheet())?;
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
        vec![accordion::root(vec![], children)],
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
            popover_section(),
            tooltip_section(),
            switch_section(),
            radio_group_section(),
            avatar_section(),
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
        ] {
            assert!(
                html.contains(&format!(r#"data-scope="{scope}""#)),
                "missing data-scope={scope}"
            );
        }
        // 静的掲示の状態固定: 選択中タブ・開いた Accordion 項目・checked
        // Switch/RadioGroup item。
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-state="checked""#));
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
        // リンクを持たない設計とし、リンク検証対象を Markdown 側へ限定する
        // （リンクを足す場合はこのテストを更新して linkcheck との整合を
        // 明示的に設計し直すこと）。
        let html = render(&showcase_body());
        assert!(!html.contains("href="));
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
