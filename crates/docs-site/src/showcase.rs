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
//! Tabs / Accordion 等の状態機械を持つ部品は、SSR 静的マークアップ
//! （選択中・開いた状態を `data-state` で固定した掲示）のみを載せる。実際の
//! クリック挙動（dispatch 状態遷移）は wasm 層の責務であり docs サイトの
//! スコープ外（`examples/headless-pre-styled-ui` と同じ方針）。

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::tabs::{tabs, ActivationMode, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::{
    accordion, alert, badge, card, AlertStatus, BadgeProps, BadgeVariant, CardVariant,
    ColorPalette, OpenState, Orientation, Size, StyleSheet, StylesheetError,
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
const SHOWCASE_LAYOUT_CSS: &str = "\
.pre-styled-showcase {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.showcase-row {\n  display: flex;\n  flex-wrap: wrap;\n  gap: 0.75rem;\n  align-items: center;\n  margin: 1rem 0;\n}\n\
.showcase-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  margin: 1rem 0;\n  max-width: 36rem;\n}\n\
.pre-styled-showcase [data-scope=\"accordion\"] h3 {\n  margin: 0;\n  font-size: 1rem;\n  font-weight: 400;\n  line-height: 1.5;\n  letter-spacing: normal;\n}\n";

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
/// accordion）→ ショーケース配置スタイル、の順で決定的に連結する。
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
        ] {
            assert!(
                html.contains(&format!(r#"data-scope="{scope}""#)),
                "missing data-scope={scope}"
            );
        }
        // 静的掲示の状態固定: 選択中タブ・開いた Accordion 項目。
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="open""#));
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
        // ショーケース配置スタイル。
        assert!(css.contains(".showcase-row"));
        assert!(css.contains(".showcase-stack"));
        // Accordion anatomy の h3 への `.docs-content h3`（site.css）漏れを
        // 遮断する見出しリセット（Bugbot 指摘の回帰防止）。
        assert!(css.contains(r#".pre-styled-showcase [data-scope="accordion"] h3"#));
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
