//! `changelog-timeline-subscribe` block（イシュー #2821。親トラッキング
//! #2731「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0042 の 1 件のみを
//! 構造の参照元とする合成例。中央寄せの見出し + リード文の下に横並びの
//! 購読フォーム、その下に日付・コネクタ・本文の 3 列タイムラインでリリース
//! を並べる changelog）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と同じ
//! ライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `field` / `input` / `button` / `badge` / `timeline` /
//! `list` / `visually_hidden` の 9 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。`visually_hidden` はイシュー本文の部品一覧にはないが、
//! 可視ラベルを出さずに `<label for>` で入力へアクセシブルネームを与える
//! ために追加した（`banner_email_signup` の前例と同じ判断）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。購読ボタンは
//! `button::button`（既定 `type="button"`）のまま送信先・バリデーションを
//! 持たず、実際の購読処理は利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。文言・バージョン
//! 番号・日付・変更点はすべて架空のもの（実在の製品・企業名・PII を
//! 含まない）。
//!
//! # 購読フォームの横連結（attached）表現
//!
//! `field::root` は `drop_class_attr` により呼び出し側 `class` を除去する
//! ため、`.blocks-changelog-timeline-subscribe-form` の子として `input`/
//! `button` を横並び flex（`flex-wrap: nowrap`）で並べ、両者の隣接辺の
//! 角丸を打ち消して 1 本の連結コントロールに見せる。`input::input`
//! （scope `"field"` data-part `"input"`）・`button::button`
//! （scope `"button"` data-part `"root"`）はいずれも base 宣言（詳細度
//! 0,2,0）で `border-radius` を宣言するため、[`LAYOUT_CSS`] は
//! `.blocks-changelog-timeline-subscribe-form [data-scope=...][data-part=
//! ...][data-blocks-changelog-timeline-subscribe-*]`（詳細度 0,4,0）で
//! 確実に上書きする。狭幅でも横並びを崩さない（`banner_email_signup` の
//! ような縦積み化はしない、イシュー本文の要件）。
//!
//! # 3 列タイムラインへの recipe 上書きと詳細度
//!
//! styled `timeline::item` の既定 recipe は
//! `grid-template-columns: var(--fandhe-timeline-indicator-size, 1.5rem) 1fr`
//! の 2 列（indicator/連結線 + 本文）だが、本 block は「日付列 / connector
//! 列 / 本文列」の 3 列を要件とする。`timeline::root`/`item` は `class`
//! （item は `class` すら出力しない）を伝搬しないため、ラッパ
//! `.blocks-changelog-timeline-subscribe-timeline` からの子孫セレクタ
//! （`[data-scope="timeline"][data-part="item"]`、詳細度 0,3,0）で
//! `grid-template-columns` を 3 列へ上書きする。
//!
//! `grid-template-columns` を 3 列にするだけでは列は増えない: 既定 recipe は
//! `connector`（`grid-column: 1`）と `content`（`grid-column: 2`）にしか
//! 明示位置を持たず、日付列（`date_col`）・本文列（`release_body`）は
//! いずれも `timeline::content` を再利用するため両者とも `grid-column: 2`
//! に重なり、新設した 3 列目は空のままになる（イシュー #2821 の実装
//! バグ・レビュー指摘で発覚）。これを避けるため `date_col`/`connector`/
//! `release_body` それぞれへ `data-blocks-changelog-timeline-subscribe-
//! {date,connector,body}-col` 属性を付与し、
//! `.blocks-changelog-timeline-subscribe-timeline [data-scope="timeline"]
//! [data-part="..."][data-blocks-changelog-timeline-subscribe-*-col]`
//! （詳細度 0,4,0）で `grid-column: 1/2/3` を個別に明示する。狭幅
//! （`@media (max-width: 47.99rem)`）では `grid-template-columns` が
//! 2 列へ戻るため、`connector-col`/`body-col` の `grid-column` も
//! `1`/`2` へ揃えて戻す（日付列は次項のとおり非表示になる）。
//!
//! # 日付の二重出力と表示切り替え（狭幅対応・a11y）
//!
//! 各リリースの日付は日付列（`timeline::content`、広幅のみ表示）と本文列
//! 先頭（`<time>` インライン表示、狭幅のみ表示）の 2 箇所に出力するが、
//! [`LAYOUT_CSS`] の `@media (max-width: 47.99rem)` でどちらか一方のみが
//! 常に `display: none` になるよう切り替える。`display: none` は支援技術
//! からも隠れるため、スクリーンリーダーでの読み上げ重複は起きない。
//! `<time datetime>` は `banner_email_signup`/`changelog_accordion` と
//! 同じブレークポイント（`47.99rem`）を踏襲する。日付列を隠す狭幅側の
//! ルールは `[data-scope="timeline"][data-part="content"][data-blocks-
//! changelog-timeline-subscribe-date-col]`（詳細度 0,4,0）まで詳細度を
//! 上げて書く。素の `[data-blocks-changelog-timeline-subscribe-date-col]`
//! （詳細度 0,1,0）のままだと `content` recipe 側の `display: flex`
//! （詳細度 0,2,0）に負けて `47.99rem` 未満でも日付列が消えない
//! （イシュー #2821 レビュー指摘で発覚）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `field::root` / `input::input` /
//! `button::button` / `badge::badge` / `timeline::root` / `list::root` /
//! `visually_hidden::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-changelog-timeline-subscribe-*` 属性で渡す
//! （`timeline::item`/`connector`/`content` は `drop_class_attr` を経由
//! しないため子孫セレクタでの上書きに寄せる、前項参照）。素の `div`/`time`
//! には `class` がそのまま効くため、それらは
//! `.blocks-changelog-timeline-subscribe-*` クラスセレクタを使う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（`changelog_accordion` と同型の判断）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    changes: &'static [&'static str],
}

/// リリース一覧（架空、3 件）。
const RELEASES: [Release; 3] = [
    Release {
        version: "v3.6.0",
        date_iso: "2026-09-20",
        date_label: "2026年9月20日",
        title: "購読フォーム付きタイムライン表示を追加",
        tags: &["新機能"],
        changes: &[
            "更新履歴を時系列のタイムラインとして表示するレイアウトを追加",
            "メールアドレスで新着リリースを購読できる導線を追加",
        ],
    },
    Release {
        version: "v3.5.2",
        date_iso: "2026-09-08",
        date_label: "2026年9月8日",
        title: "狭い画面での表示崩れを修正",
        tags: &["修正"],
        changes: &["狭い幅で日付列が本文と重なる表示崩れを修正"],
    },
    Release {
        version: "v3.5.0",
        date_iso: "2026-08-25",
        date_label: "2026年8月25日",
        title: "変更点の種別タグ表示に対応",
        tags: &["改善", "内部"],
        changes: &[
            "変更点ごとに新機能・改善・修正の種別タグを表示",
            "リリース一覧の内部データ構造を整理",
        ],
    },
];

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ日を
/// 指す組にする不変条件、`changelog_accordion` と同じ判断）。
fn release_date(class: &'static str, iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![("class", class), ("datetime", iso)],
        vec![text(label)],
    )
}

/// 中央寄せの見出し + リード文 + 購読フォーム（`header` 領域）。
fn header() -> Node {
    const EMAIL_FIELD_ID: &str = "blocks-changelog-timeline-subscribe-email";
    let email_field = FieldProps {
        id: EMAIL_FIELD_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    div(
        vec![("class", "blocks-changelog-timeline-subscribe-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("更新履歴")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "新しいリリースをメールでお届けします。いつでも購読解除できます。",
                )],
            ),
            div(
                vec![("class", "blocks-changelog-timeline-subscribe-form")],
                vec![
                    field::root(
                        &orientation,
                        &email_field,
                        vec![("data-blocks-changelog-timeline-subscribe-field", "")],
                        vec![
                            visually_hidden::root(
                                vec![],
                                vec![field::label(
                                    &email_field,
                                    vec![],
                                    vec![text("メールアドレス")],
                                )],
                            ),
                            input::input(
                                &InputProps::default(),
                                &email_field,
                                vec![
                                    ("data-blocks-changelog-timeline-subscribe-input", ""),
                                    ("type", "email"),
                                    ("autocomplete", "email"),
                                    ("placeholder", "you@example.com"),
                                ],
                            ),
                        ],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-changelog-timeline-subscribe-submit", "")],
                        vec![text("購読する")],
                    ),
                ],
            ),
        ],
    )
}

/// リリース 1 件分のタイトル行（version + 種別 badge 群）。
fn release_head(release: &Release) -> Node {
    let mut children = vec![span(
        vec![("class", "blocks-changelog-timeline-subscribe-version")],
        vec![text(release.version)],
    )];
    children.extend(release.tags.iter().map(|tag| {
        badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-changelog-timeline-subscribe-tag", "")],
            vec![text(*tag)],
        )
    }));
    div(
        vec![("class", "blocks-changelog-timeline-subscribe-release-head")],
        children,
    )
}

/// リリース 1 件分の本文（`timeline::content`。狭幅用インライン日付 +
/// バージョン行 + タイトル + 変更点リスト）。
fn release_body(release: &Release) -> Node {
    let changes = list::root(
        ListType::Unordered,
        ListVariant::Marker,
        vec![("data-blocks-changelog-timeline-subscribe-changes", "")],
        release
            .changes
            .iter()
            .map(|change| list::item(vec![], vec![text(*change)]))
            .collect(),
    );

    timeline::content(
        vec![("data-blocks-changelog-timeline-subscribe-body-col", "")],
        vec![
            release_date(
                "blocks-changelog-timeline-subscribe-inline-date",
                release.date_iso,
                release.date_label,
            ),
            release_head(release),
            timeline::title(vec![], vec![text(release.title)]),
            changes,
        ],
    )
}

/// リリース 1 件分の item（日付列 + connector + 本文列）。
fn release_item(index: usize, release: &Release) -> Node {
    let is_last = index + 1 == RELEASES.len();

    let date_col = timeline::content(
        vec![("data-blocks-changelog-timeline-subscribe-date-col", "")],
        vec![release_date(
            "blocks-changelog-timeline-subscribe-date",
            release.date_iso,
            release.date_label,
        )],
    );

    let mut connector_children = vec![timeline::indicator(
        vec![("data-state", "complete")],
        vec![],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }

    timeline::item(
        vec![("data-blocks-changelog-timeline-subscribe-item", "")],
        vec![
            date_col,
            timeline::connector(
                vec![("data-blocks-changelog-timeline-subscribe-connector-col", "")],
                connector_children,
            ),
            release_body(release),
        ],
    )
}

/// `changelog-timeline-subscribe` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| release_item(index, release))
        .collect();

    let timeline_node = div(
        vec![("class", "blocks-changelog-timeline-subscribe-timeline")],
        vec![timeline::root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![("data-blocks-changelog-timeline-subscribe-root", "")],
            items,
        )],
    );

    div(
        vec![("class", "blocks-changelog-timeline-subscribe-layout")],
        vec![header(), timeline_node],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/changelog-timeline-subscribe/",
    title: "changelog-timeline-subscribe",
    category: BlockCategory::Changelog,
    rust_source: "crates/docs-site/src/blocks/marketing/changelog/changelog_timeline_subscribe.rs",
    demo_class: "blocks-changelog-timeline-subscribe",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Timeline",
            path: "/themes/timeline/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `changelog_timeline_subscribe` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-changelog-timeline-subscribe-*` と
/// `[data-blocks-changelog-timeline-subscribe-*]`、および styled
/// `timeline`/`field`/`input`/`button` の `[data-scope=...]` 系セレクタへの
/// 子孫結合子付き上書き（モジュール doc「購読フォームの横連結」「3 列
/// タイムラインへの recipe 上書き」節）のみを用い、他 block や部品の素の
/// セレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-changelog-timeline-subscribe-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-changelog-timeline-subscribe-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-3);\n  max-width: 36rem;\n  margin-inline: auto;\n}\n\
.blocks-changelog-timeline-subscribe-form {\n  display: flex;\n  flex-wrap: nowrap;\n  align-items: stretch;\n  width: 100%;\n  max-width: 28rem;\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-changelog-timeline-subscribe-form [data-blocks-changelog-timeline-subscribe-field] {\n  flex: 1;\n  min-width: 0;\n}\n\
.blocks-changelog-timeline-subscribe-form [data-scope=\"field\"][data-part=\"input\"][data-blocks-changelog-timeline-subscribe-input] {\n  border-top-right-radius: 0;\n  border-bottom-right-radius: 0;\n}\n\
.blocks-changelog-timeline-subscribe-form [data-scope=\"button\"][data-part=\"root\"][data-blocks-changelog-timeline-subscribe-submit] {\n  border-top-left-radius: 0;\n  border-bottom-left-radius: 0;\n  flex-shrink: 0;\n}\n\
.blocks-changelog-timeline-subscribe-form [data-scope=\"button\"][data-part=\"root\"][data-blocks-changelog-timeline-subscribe-submit]:focus-visible {\n  position: relative;\n  z-index: 1;\n}\n\
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"item\"] {\n  grid-template-columns: 7rem var(--fandhe-timeline-indicator-size, 1.5rem) 1fr;\n}\n\
.blocks-changelog-timeline-subscribe-inline-date {\n  display: none;\n}\n\
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"content\"][data-blocks-changelog-timeline-subscribe-date-col] {\n  grid-column: 1;\n  display: flex;\n  align-items: flex-start;\n  justify-content: flex-end;\n  padding-top: var(--fandhe-space-1);\n}\n\
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"connector\"][data-blocks-changelog-timeline-subscribe-connector-col] {\n  grid-column: 2;\n}\n\
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"content\"][data-blocks-changelog-timeline-subscribe-body-col] {\n  grid-column: 3;\n}\n\
.blocks-changelog-timeline-subscribe-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n  white-space: nowrap;\n}\n\
.blocks-changelog-timeline-subscribe-release-head {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-changelog-timeline-subscribe-version {\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
[data-blocks-changelog-timeline-subscribe-changes] {\n  margin-top: var(--fandhe-space-2);\n}\n\
@media (max-width: 47.99rem) {\n  \
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"item\"] {\n    grid-template-columns: var(--fandhe-timeline-indicator-size, 1.5rem) 1fr;\n  }\n  \
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"content\"][data-blocks-changelog-timeline-subscribe-date-col] {\n    display: none;\n  }\n  \
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"connector\"][data-blocks-changelog-timeline-subscribe-connector-col] {\n    grid-column: 1;\n  }\n  \
.blocks-changelog-timeline-subscribe-timeline [data-scope=\"timeline\"][data-part=\"content\"][data-blocks-changelog-timeline-subscribe-body-col] {\n    grid-column: 2;\n  }\n  \
.blocks-changelog-timeline-subscribe-inline-date {\n    display: block;\n    color: var(--fandhe-color-fg-muted);\n    font-size: var(--fandhe-font-font-size-sm);\n    margin-bottom: var(--fandhe-space-1);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, RELEASES};
    use fandhe_frontend_core::render;

    /// Demo が期待する 9 種の部品を含み、`<form>`/送信先/`data:` URI を
    /// 持たず `type="button"` がちょうど 1 個であることを固定する
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\"",
            "data-scope=\"badge\"",
            "data-scope=\"timeline\"",
            "data-scope=\"list\"",
            "data-scope=\"button\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// `<form>` を出力しない・`type="submit"`/送信先を持たない静的表示。
    #[test]
    fn demo_has_exactly_one_type_button_and_no_form() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 1);
        assert!(!html.contains("<form"));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains(r#"type="submit""#));
    }

    /// item 数・separator 数（最終 item を除く）・indicator の
    /// `data-state="complete"` 数がリリース件数と一致することを固定する。
    #[test]
    fn timeline_items_and_separators() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-scope="timeline" data-part="item""#)
                .count(),
            RELEASES.len()
        );
        assert_eq!(
            html.matches(r#"data-part="separator""#).count(),
            RELEASES.len() - 1
        );
        assert_eq!(
            html.matches(r#"data-state="complete""#).count(),
            // indicator 分 + separator 分
            RELEASES.len() + (RELEASES.len() - 1)
        );
    }

    /// 日付が日付列・本文列インラインの 2 箇所に出力される（CSS で片方の
    /// みが常に表示される、モジュール doc「日付の二重出力」節参照）。
    #[test]
    fn dates_are_rendered_twice_with_matching_datetime() {
        let html = render(&demo());
        assert_eq!(html.matches("<time").count(), 2 * RELEASES.len());
        for release in RELEASES {
            let occurrences = html
                .matches(&format!(r#"datetime="{}""#, release.date_iso))
                .count();
            assert_eq!(
                occurrences, 2,
                "expected date {} to appear twice in {html}",
                release.date_iso
            );
        }
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-changelog-timeline-subscribe-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-changelog-timeline-subscribe-layout"
        );
    }

    /// [`LAYOUT_CSS`] が 3 列上書き・attached の角丸打ち消し・狭幅
    /// ブレークポイントを持つことを固定する。
    #[test]
    fn layout_css_declares_three_column_override_and_narrow_breakpoint() {
        assert!(LAYOUT_CSS.contains(
            r#".blocks-changelog-timeline-subscribe-timeline [data-scope="timeline"][data-part="item"] {"#
        ));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains(
            r#"[data-scope="field"][data-part="input"][data-blocks-changelog-timeline-subscribe-input] {"#
        ));
        assert!(LAYOUT_CSS.contains(
            r#"[data-scope="button"][data-part="root"][data-blocks-changelog-timeline-subscribe-submit] {"#
        ));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// メールアドレス入力の `<label for>` が visually-hidden ラベルの id と
    /// 一致すること（アクセシブル名の関連付け固定、`banner_email_signup`
    /// と同型）。
    #[test]
    fn email_label_for_matches_input_id() {
        let html = render(&demo());
        assert!(html.contains(r#"for="blocks-changelog-timeline-subscribe-email-control""#));
        assert!(html.contains(r#"id="blocks-changelog-timeline-subscribe-email-control""#));
        assert!(html.contains(r#"type="email""#));
    }
}
