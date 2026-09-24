//! `changelog-accordion` block（イシュー #2818。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0046 の 1 件のみを構造の
//! 参照元とする合成例。左寄せの見出し + リード文の下にリリース単位の
//! アコーディオン項目を縦に並べる changelog）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! **Marketing / Changelog カテゴリで最初の block**（イシュー #2734 の
//! 雛形を本 block 追加で卒業させた、`super`（`changelog/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `accordion` / `list` / `image` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、全項目を常時 open + disabled で固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、`Accordion`（状態機械）
//! を経由せず [`fandhe_frontend_pre_styled_ui::accordion`] の自由関数を
//! 直接呼び、[`RELEASES`] の全件を [`OpenState::Open`] として固定描画する
//! （`component_specs_overlay.rs::ex_accordion_multiple` と同型の「状態機械
//! を介さない複数項目同時 open」前例に倣う）。開閉操作はできない。
//!
//! 当初は先頭 2 件のみ open・残りを `OpenState::Closed`（`hidden` 属性で
//! 本文が到達不能）としていたが、`item_trigger` が `disabled: false` の
//! フォーカス可能な `<button>` として出力されるため、クリック・
//! Enter/Space が no-op になるうえ、閉じた項目の本文が事実上読めなくなる
//! （イシュー #2818 レビュー指摘、`docs/policy/intentional-non-adoption.md`
//! の UI 部品責務境界にある「アクセシビリティ（WAI-ARIA・キーボード操作）」
//! に反する）。是正として `AccordionProps { disabled: true, .. }` を
//! `item`/`item_trigger`/`item_indicator`/`item_content` へ共有し、
//! ネイティブ `disabled` 属性・`aria-disabled="true"`（[`item_trigger`]）を
//! 出力してフォーカス不能・操作不能であることを支援技術・キーボード双方に
//! 明示する（`game_ui_modal`（`crates/docs-site/src/blocks/application/
//! dialog/game_ui_modal.rs`）が「無 JS 下では開閉を切り替えられず表示上の
//! 意味を持たない `trigger` を置かない」とした判断と同じ思想の適用で、本
//! block はトリガー自体を除去できない〔version/date/title を担う〕ため
//! 代わりに disabled 化する）。`disabled_declarations()`（既定
//! `opacity: 0.5`）は [`LAYOUT_CSS`] で中和し、通常の changelog 見出しと
//! 同じ見た目に保つ。
//!
//! # 項目ごとに枠を付けるための recipe 上書きと詳細度
//!
//! styled `accordion::root` の既定 recipe は「root 全体を 1 枠で囲む」
//! 構成（root に border、item に border-bottom、`item:last-child` で
//! 二重線を打ち消す）だが、本 block は「項目ごとに個別の枠」を要件とする
//! ため、[`LAYOUT_CSS`] で `.blocks-changelog-accordion-list [data-scope=
//! "accordion"][data-part="item"]` 系セレクタを子孫結合子付き（詳細度
//! 0,3,0 以上）で上書きする。`accordion::root` は `drop_class_attr` に
//! より呼び出し側 `attrs` の `class` を除去するため、ルート自身へ直接
//! フックを足すことはできず、レイアウト側の `.blocks-changelog-accordion-
//! list` からの子孫セレクタで上書きする必要がある（`testimonials_stack`
//! 等、他 block の recipe 上書きと同じ判断）。`item:last-child` の
//! `border-bottom: 0`（recipe 側 0,3,0）に対しては、読み込み順に依存せず
//! 確実に勝たせるため 0,4,0 の `.blocks-changelog-accordion-list
//! [data-scope="accordion"][data-part="item"]:last-child` を明示的に書く。
//!
//! # トリガーの子を 2 個に保つ理由
//!
//! recipe の item-trigger は `justify-content: space-between` を前提に
//! 「直接の子 2 個」のレイアウトを取る。そのため [`item_trigger`] の
//! children はラベル領域（`span.blocks-changelog-accordion-trigger-label`）
//! と `item_indicator` の 2 個に固定し、間に `div` 等を挟まない。
//!
//! # id 接頭辞と ARIA 対応
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-changelog-accordion-{index}-{trigger|content}` の形でリリース
//! 添字から一意に導出する（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全
//! block 横断で id 重複・宙に浮いた参照を検査する）。
//!
//! # 見出しレベル（`H3`/`h4`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする。各トリガーは WAI-ARIA APG のアコーディオン
//! パターンに合わせて `<h4>` で包む（[`fandhe_frontend_core::el`] で
//! 直接組み立て、`heading::heading` は使わない。ページ本文の見出し階層に
//! 割り込ませない部品固有の構造要素であるため）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `accordion::root` /
//! `list::root` / `image::image` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-changelog-accordion-*` 属性で渡す
//! （`accordion::item`/`item_trigger`/`item_content`/`item_indicator` は
//! `drop_class_attr` を経由しないため子孫セレクタでの上書きに寄せる、
//! 前項参照）。素の `div`/`h4`/`time`/`span` には `class` がそのまま効く
//! ため、それらは `.blocks-changelog-accordion-*` クラスセレクタを使う。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・バージョン番号・日付・変更点はすべて架空のもの（実在の
//! 製品・企業名・PII を含まない）。リンク（`a[href]`）は使用部品にリンクが
//! ないため一切出力しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

use crate::blocks::dummy_assets;

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    image_src: &'static str,
    changes: &'static [&'static str],
}

/// リリース一覧（架空、4 件。モジュール doc「静的表示」節のとおり全件を
/// open + disabled で固定描画する）。
const RELEASES: [Release; 4] = [
    Release {
        version: "v2.4.0",
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        title: "アコーディオン型 changelog を追加",
        tags: &["新機能"],
        image_src: dummy_assets::SCREENSHOT_SRC,
        changes: &[
            "リリース単位で区切って表示する changelog レイアウトを追加",
            "全リリースを常時展開表示するデモ表示に対応",
        ],
    },
    Release {
        version: "v2.3.0",
        date_iso: "2026-09-10",
        date_label: "2026年9月10日",
        title: "ダミー素材ヘルパを共通化",
        tags: &["改善", "内部"],
        image_src: dummy_assets::PRODUCT_SRC,
        changes: &[
            "プレースホルダー画像・文言の生成をヘルパへ一元化",
            "block ごとの個別実装によるブレを解消",
        ],
    },
    Release {
        version: "v2.2.1",
        date_iso: "2026-09-02",
        date_label: "2026年9月2日",
        title: "画像スロットの表示崩れを修正",
        tags: &["修正"],
        image_src: dummy_assets::BACKGROUND_SRC,
        changes: &["狭い幅での画像の縦横比崩れを修正"],
    },
    Release {
        version: "v2.2.0",
        date_iso: "2026-08-20",
        date_label: "2026年8月20日",
        title: "変更点リストの表示を刷新",
        tags: &["改善"],
        image_src: dummy_assets::SCREENSHOT_SRC,
        changes: &[
            "変更点を種別ごとに読みやすく整理",
            "長い項目でも折り返しが崩れないよう調整",
        ],
    },
];

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ日を
/// 指す組にする不変条件、`blog_list_image` と同じ判断）。
fn release_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-changelog-accordion-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// トリガーのラベル領域（version・日付・タイトルの 3 点。トリガーの
/// 直接の子は本要素と `item_indicator` の 2 個に保つ、モジュール doc
/// 「トリガーの子を 2 個に保つ理由」節）。
fn trigger_label(release: &Release) -> Node {
    fandhe_frontend_core::span(
        vec![("class", "blocks-changelog-accordion-trigger-label")],
        vec![
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-accordion-version")],
                vec![text(release.version)],
            ),
            release_date(release.date_iso, release.date_label),
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-accordion-title")],
                vec![text(release.title)],
            ),
        ],
    )
}

/// アコーディオン本文（タグ badge 群 + 画像 + 変更点リスト）。
fn release_body(release: &Release) -> Node {
    let tags = div(
        vec![("class", "blocks-changelog-accordion-tags")],
        release
            .tags
            .iter()
            .map(|tag| {
                badge::badge(
                    &BadgeProps::default(),
                    vec![("data-blocks-changelog-accordion-tag", "")],
                    vec![text(*tag)],
                )
            })
            .collect(),
    );

    let figure = div(
        vec![("class", "blocks-changelog-accordion-figure")],
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Rounded,
                ..ImageProps::new(release.image_src, "")
            },
            vec![("data-blocks-changelog-accordion-image", "")],
        )],
    );

    let changes = list::root(
        ListType::Unordered,
        ListVariant::Marker,
        vec![("data-blocks-changelog-accordion-changes", "")],
        release
            .changes
            .iter()
            .map(|change| list::item(vec![], vec![text(*change)]))
            .collect(),
    );

    div(
        vec![("class", "blocks-changelog-accordion-body")],
        vec![tags, figure, changes],
    )
}

/// リリース 1 件分の accordion item（トリガー + 本文）。
///
/// 全件を [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「静的表示」節）。`disabled` はネイティブ `disabled` 属性・
/// `aria-disabled="true"` を [`item_trigger`] へ反映させ、無 JS のため
/// クリック・キーボードでは開閉できないことを支援技術・キーボード操作の
/// 双方に明示する。
fn release_item(index: usize, release: &Release) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-changelog-accordion-{index}-trigger");
    let content_id = format!("blocks-changelog-accordion-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![("class", "blocks-changelog-accordion-trigger-heading")],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    release.version,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        trigger_label(release),
                        item_indicator(state, false, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![release_body(release)],
            ),
        ],
    )
}

/// `changelog-accordion` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-changelog-accordion-header")],
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
                vec![text("各リリースの変更点をまとめています。")],
            ),
        ],
    );

    let items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| release_item(index, release))
        .collect();

    let list = div(
        vec![("class", "blocks-changelog-accordion-list")],
        vec![accordion::root(
            Size::Md,
            &AccordionProps::default(),
            vec![("data-blocks-changelog-accordion-root", "")],
            items,
        )],
    );

    div(
        vec![("class", "blocks-changelog-accordion-layout")],
        vec![header, list],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/changelog-accordion/",
    title: "changelog-accordion",
    category: BlockCategory::Changelog,
    rust_source: "crates/docs-site/src/blocks/marketing/changelog/changelog_accordion.rs",
    demo_class: "blocks-changelog-accordion",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Accordion",
            path: "/themes/accordion/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `changelog_accordion` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-changelog-accordion-*` と
/// `[data-blocks-changelog-accordion-*]`、および styled accordion の
/// `[data-scope="accordion"]` 系セレクタへの子孫結合子付き上書き
/// （モジュール doc「項目ごとに枠を付けるための recipe 上書きと詳細度」
/// 節）のみを用い、他 block や部品の素のセレクタへ影響させない。
///
/// `[data-part="item-trigger"][data-disabled]` の中和（モジュール doc
/// 「静的表示」節の disabled 化に伴う追加）: `accordion::stylesheet` の
/// `disabled_declarations()`（`opacity: 0.5` + `cursor: not-allowed`）は
/// 「操作できない要素」の既定表現だが、本 block は開閉操作自体を提供
/// しない常時展開の changelog 見出しであり、薄く見せる必要がないため
/// `opacity: 1`・`cursor: default` へ上書きする。詳細度は `:last-child`
/// 上書きと同じ考え方で 0,4,0（recipe 側 0,3,0 に対して子孫結合子 1 段
/// 追加分）にして読み込み順に依存せず確実に勝たせる。
const LAYOUT_CSS: &str = "\
.blocks-changelog-accordion-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  width: 100%;\n}\n\
.blocks-changelog-accordion-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-changelog-accordion-list [data-scope=\"accordion\"][data-part=\"root\"] {\n  border: 0;\n  border-radius: 0;\n  overflow: visible;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-changelog-accordion-list [data-scope=\"accordion\"][data-part=\"item\"] {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
.blocks-changelog-accordion-list [data-scope=\"accordion\"][data-part=\"item\"]:last-child {\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-changelog-accordion-list [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-changelog-accordion-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
.blocks-changelog-accordion-trigger-label {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n  flex: 1;\n}\n\
.blocks-changelog-accordion-version {\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
.blocks-changelog-accordion-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-changelog-accordion-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-changelog-accordion-tags {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-changelog-accordion-figure {\n  width: 100%;\n  max-width: 40rem;\n}\n\
[data-blocks-changelog-accordion-image] {\n  display: block;\n  width: 100%;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, RELEASES};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品・アコーディオン状態・非対話制約を満たす
    /// ことの単体回帰（`crates/docs-site/tests/blocks_contract.rs` の横断
    /// 検査と重複し過ぎない範囲での個別固定）。
    ///
    /// 全件 open・全件 disabled（`hidden`/`data-state="closed"` を一切
    /// 出力しない）ことを固定する。これは無 JS の docs サイトで
    /// `item_trigger` が no-op のフォーカス可能なボタンとなり、閉じた
    /// 項目の本文が事実上到達不能になっていた指摘（イシュー #2818 レビュー
    /// 指摘）の是正を回帰させる。
    #[test]
    fn demo_composes_expected_parts_and_states() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"accordion\"",
            "data-scope=\"list\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            RELEASES.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches("item-content\" data-state=\"closed\"").count(),
            0,
            "html={html}"
        );
        assert_eq!(html.matches(" hidden=\"\"").count(), 0, "html={html}");
        // 全トリガーがネイティブ disabled + aria-disabled="true" を持つこと
        // （モジュール doc「静的表示」節。フォーカス不能・操作不能を支援
        // 技術・キーボード双方に明示する）。
        assert_eq!(
            html.matches(r#"data-part="item-trigger""#).count(),
            RELEASES.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            RELEASES.len(),
            "html={html}"
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-changelog-accordion-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-changelog-accordion-layout");
    }

    /// [`LAYOUT_CSS`] が recipe の `:last-child` 規則より詳細度で勝つ
    /// 上書きセレクタを持つこと。
    #[test]
    fn layout_css_overrides_item_frame_and_last_child() {
        assert!(LAYOUT_CSS.contains(
            r#".blocks-changelog-accordion-list [data-scope="accordion"][data-part="item"] {"#
        ));
        assert!(LAYOUT_CSS.contains(
            r#".blocks-changelog-accordion-list [data-scope="accordion"][data-part="item"]:last-child {"#
        ));
    }

    /// [`LAYOUT_CSS`] が `item-trigger[data-disabled]` の既定
    /// `disabled_declarations()`（`opacity: 0.5`）を中和する上書きを
    /// 持つこと（モジュール doc「静的表示」節）。
    #[test]
    fn layout_css_neutralizes_disabled_trigger_opacity() {
        assert!(LAYOUT_CSS.contains(
            r#".blocks-changelog-accordion-list [data-scope="accordion"][data-part="item-trigger"][data-disabled] {"#
        ));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
    }
}
