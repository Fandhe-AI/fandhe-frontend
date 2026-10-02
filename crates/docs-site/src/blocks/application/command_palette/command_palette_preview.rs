//! `command-palette-preview` block（イシュー #2968。Application / Command
//! Palette カテゴリ）。ダイアログ内の上部に検索欄、下部を
//! 左右 2 ペインに分割し、左に「最近の検索」と「候補」の一覧、右に選択中
//! 候補のプレビュー（アバター・氏名・連絡先・送信ボタン）を配置する。
//! `_/blocks-intake/` の対応ファイル R0846 は本イシュー着手時点で本
//! worktree に存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile-detail-datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `command` / `dialog` / `avatar` / `button` / `heading` / `data-list` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 集約元について
//!
//! 主参照は R0846 の 1 件のみであり、他版との並記は行わない（購入者限定
//! 素材のライセンス上の転記制限、`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じ方針）。参照から取り込むのは構造（領域の配置と部品構成）のみで、
//! 文言・配色・アイコンは持ち込まず独自に書く。
//!
//! # `trigger`/`title` を置かない理由
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 本 Demo はダイアログが**既に開いた静的な初期状態**のみを描く
//! （`contact_dialog_form`/`game_ui_modal` と同じ設計判断）。
//! [`fandhe_frontend_pre_styled_ui::dialog::trigger`] は無 JS 下では開閉を
//! 切り替えられず表示上の意味を持たないため置かない。コマンドパレットは
//! 見出しを持たない構成のため `dialog::title` も置かず、`content` の
//! `aria-label` がアクセシブルネームを担う（`aria-label` は
//! `fandhe_frontend_headless_ui::dialog::content` の固定出力属性に含まれず、
//! 呼び出し側 `attrs` からそのまま 1 回だけ出力される）。
//!
//! # `aria-modal` を false にする理由
//!
//! 静的なデモは閉じる機構を持たず、ダイアログの外側に説明・コード・
//! ナビゲーションがある。支援技術が外側を無視しないよう、表示の実態と
//! 一致させて `aria-modal` は false にする（`contact_dialog_form`/
//! `game_ui_modal` と同じ判断）。
//!
//! # 先頭候補を選択中として固定表示
//!
//! 絞り込み・選択切替は行わない（無 JS）ため、候補一覧の先頭 1 件
//! （`dummy_assets::PERSON_NAMES[0]`）のみ `command::item` の `selected` を
//! `true` にし、`command::input` の `aria-activedescendant` を同じ id へ
//! 向ける。右ペインのプレビューはこの先頭候補を表示する。
//!
//! # 狭幅ではプレビューを候補一覧の下へ移す
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` 等と同型の
//! パターン）。[`LAYOUT_CSS`] は `.blocks-command-palette-preview-panes` へ
//! `container-type: inline-size` の祖先を持たせ、コンテナ幅が `40rem`
//! 未満のとき `grid-template-columns` を 1 カラムへ畳んで右ペインを
//! 候補一覧の下へ積む（`display: none` にはしない）。右ペインにのみ
//! ある連絡先（メール・電話・所属・送信ボタン）は左候補一覧に含まれず
//! 隠すと欠落するため（イシュー #2968 レビュー指摘の是正）。
//!
//! # 固定オーバーレイの中和
//!
//! `dialog::backdrop`/`positioner` は本来 `position: fixed; inset: 0` の
//! ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
//! 収める必要がある（`contact_dialog_form` と同型の中和パターン）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::dialog`]・[`fandhe_frontend_pre_styled_ui::command`]・
//! [`fandhe_frontend_pre_styled_ui::avatar::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部クラスと合成
//! するため、これらへの CSS フックは `data-*` 属性で渡す。レイアウト用
//! ラッパー（2 ペイン分割・プレビュー枠）は素の `<div>` のため
//! `class="blocks-command-palette-preview-*"` を使う。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。送信ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! 氏名・役職・社名は `crate::blocks::dummy_assets`（架空セット）を使う。
//! メールアドレスは `example.com` ドメイン、電話番号は架空パターンとし、
//! 実在の人物・企業・PII は含まない。アバター画像はビルド時生成の同梱
//! SVG（[`dummy_assets::AVATAR_SRC`]）を使う（外部 URL・`data:` URI は
//! 使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::command::{self, OpenState};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;

const LIST_ID: &str = "blocks-command-palette-preview-list";
const RECENT_HEADING_ID: &str = "blocks-command-palette-preview-recent-heading";
const SUGGESTIONS_HEADING_ID: &str = "blocks-command-palette-preview-suggestions-heading";
const SELECTED_ITEM_ID: &str = "blocks-command-palette-preview-item-0";

/// 候補 1 件（アバター Sm + 氏名 + 役職）を `command::item` として組み立てる。
fn candidate_item(
    selected: bool,
    id: &'static str,
    value: &'static str,
    name: &'static str,
    title: &'static str,
) -> Node {
    command::item(
        selected,
        false,
        value,
        Some(id),
        vec![],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-command-palette-preview-item-body")],
                vec![
                    el("span", vec![], vec![text(name)]),
                    el(
                        "span",
                        vec![("class", "blocks-command-palette-preview-item-title")],
                        vec![text(title)],
                    ),
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// `command-palette-preview` の Demo 本体（既に開いた静的な初期状態のみ
/// 描く純関数）。
pub fn demo() -> Node {
    let recent_names = [dummy_assets::PERSON_NAMES[4], dummy_assets::PERSON_NAMES[5]];
    let recent_titles = [dummy_assets::JOB_TITLES[4], dummy_assets::JOB_TITLES[5]];
    let recent_group = command::group(
        Some(RECENT_HEADING_ID),
        vec![],
        vec![
            command::group_heading(Some(RECENT_HEADING_ID), vec![], vec![text("最近の検索")]),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-recent-0",
                "noor-al-sayed",
                recent_names[0],
                recent_titles[0],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-recent-1",
                "ola-bergstrom",
                recent_names[1],
                recent_titles[1],
            ),
        ],
    );

    let suggestion_names = [
        dummy_assets::PERSON_NAMES[0],
        dummy_assets::PERSON_NAMES[1],
        dummy_assets::PERSON_NAMES[2],
        dummy_assets::PERSON_NAMES[3],
    ];
    let suggestion_titles = [
        dummy_assets::JOB_TITLES[0],
        dummy_assets::JOB_TITLES[1],
        dummy_assets::JOB_TITLES[2],
        dummy_assets::JOB_TITLES[3],
    ];
    let suggestions_group = command::group(
        Some(SUGGESTIONS_HEADING_ID),
        vec![],
        vec![
            command::group_heading(Some(SUGGESTIONS_HEADING_ID), vec![], vec![text("候補")]),
            candidate_item(
                true,
                SELECTED_ITEM_ID,
                "haruto-fujimaki",
                suggestion_names[0],
                suggestion_titles[0],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-1",
                "elena-vasquez",
                suggestion_names[1],
                suggestion_titles[1],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-2",
                "kwame-boateng",
                suggestion_names[2],
                suggestion_titles[2],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-3",
                "mei-lindqvist",
                suggestion_names[3],
                suggestion_titles[3],
            ),
        ],
    );

    let list = command::list(
        LIST_ID,
        "People",
        false,
        vec![],
        vec![
            recent_group,
            command::separator(vec![], vec![]),
            suggestions_group,
        ],
    );

    let preview_name = dummy_assets::PERSON_NAMES[0];
    let preview = div(
        vec![("class", "blocks-command-palette-preview-pane")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xl,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(
                        ImageStatus::Loaded,
                        dummy_assets::AVATAR_SRC,
                        preview_name,
                        vec![],
                    ),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(preview_name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(preview_name)],
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![("data-blocks-command-palette-preview-preview-list", "")],
                vec![
                    row("役職", dummy_assets::JOB_TITLES[0]),
                    row("メール", "haruto.fujimaki@example.com"),
                    row("電話", "090-1234-5678"),
                    row("所属", dummy_assets::COMPANY_NAMES[0]),
                ],
            ),
            button(
                &ButtonProps::default(),
                vec![],
                vec![text("メッセージを送る")],
            ),
        ],
    );

    let panes = div(
        vec![("class", "blocks-command-palette-preview-panes")],
        vec![list, preview],
    );

    let input = command::input(
        OpenState::Open,
        "",
        LIST_ID,
        Some(SELECTED_ITEM_ID),
        vec![
            ("aria-label", "Search people"),
            ("placeholder", "名前で検索…"),
        ],
    );

    let command_root = command::root(
        OpenState::Open,
        false,
        vec![("data-blocks-command-palette-preview-command", "")],
        vec![input, panes],
    );

    dialog::root(
        Size::Lg,
        OpenState::Open,
        vec![("data-blocks-command-palette-preview-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    // 静的デモは閉じる機構を持たず外側に説明・コード・
                    // ナビゲーションがあるため、表示実態と一致させ
                    // aria-modal は false にする（`contact_dialog_form`/
                    // `game_ui_modal` と同じ判断）。
                    false,
                    ContentIds {
                        id: Some("blocks-command-palette-preview-content"),
                        labelledby: None,
                        describedby: None,
                    },
                    vec![
                        ("aria-label", "People search"),
                        ("data-blocks-command-palette-preview-content", ""),
                    ],
                    vec![command_root],
                )],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/command-palette-preview/",
    title: "command-palette-preview",
    category: BlockCategory::CommandPalette,
    rust_source:
        "crates/docs-site/src/blocks/application/command_palette/command_palette_preview.rs",
    demo_class: "blocks-command-palette-preview",
    parts: &[
        Part {
            label: "Command",
            path: "/themes/command/",
        },
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `command_palette_preview` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節、`contact_dialog_form` と同型）。
///
/// # 固定オーバーレイの中和
///
/// `dialog::backdrop`/`positioner` の `position: fixed; inset: 0` を
/// `.blocks-demo` 枠内へ収めるため、本 block スコープに限定した属性
/// セレクタで `position: relative`/`absolute`・`inset: auto` へ差し替える
/// （`contact_dialog_form` と同型）。
///
/// # content の幅・`command` の枠二重化の解除
///
/// `--fandhe-dialog-content-max-width` を 2 ペイン分の幅（48rem）へ広げ、
/// `content` の `padding: 0` + `overflow: hidden` で `command::root` の
/// 角丸をそのまま content の輪郭として見せる。`command::root` 自身の枠は
/// `border: 0` で消し、二重枠を避ける。
///
/// # 2 ペイン分割・狭幅ではプレビューを下へ積む
///
/// `.blocks-command-palette-preview-panes` を 2 カラムの grid にし、右
/// ペイン（`.blocks-command-palette-preview-pane`）は左境界線・中央寄せの
/// 縦積みで構成する。`container-type: inline-size` を持つ祖先
/// `[data-blocks-command-palette-preview-root]` を基準にした `@container`
/// （幅 40rem 未満）で `grid-template-columns` を 1 カラムへ畳み、右ペインを
/// `display: none` にはせず候補一覧の下へ積む（左境界線も上境界線へ
/// 差し替える）。連絡先（メール・電話・所属・送信ボタン）は右ペインにしか
/// なく隠すと欠落するため（モジュール doc「狭幅ではプレビューを候補
/// 一覧の下へ移す」節参照）。
const LAYOUT_CSS: &str = "\
[data-blocks-command-palette-preview-root] {\n  position: relative;\n  container-type: inline-size;\n  container-name: blocks-command-palette-preview;\n}\n\
.blocks-command-palette-preview [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-command-palette-preview [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  min-height: 28rem;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-command-palette-preview [data-scope=\"dialog\"][data-part=\"content\"] {\n  --fandhe-dialog-content-max-width: 48rem;\n  padding: 0;\n  overflow: hidden;\n}\n\
[data-blocks-command-palette-preview-command] {\n  border: 0;\n  border-radius: inherit;\n}\n\
.blocks-command-palette-preview-panes {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n}\n\
[data-blocks-command-palette-preview-command] [data-scope=\"command\"][data-part=\"list\"] {\n  --fandhe-command-list-max-height: 22rem;\n}\n\
.blocks-command-palette-preview-pane {\n  border-inline-start: 1px solid var(--fandhe-color-border);\n  padding: var(--fandhe-space-6);\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  text-align: center;\n}\n\
.blocks-command-palette-preview [data-scope=\"heading\"] {\n  margin: 0;\n}\n\
.blocks-command-palette-preview-item-body {\n  display: flex;\n  flex-direction: column;\n  min-width: 0;\n}\n\
.blocks-command-palette-preview-item-title {\n  font-size: var(--fandhe-font-font-size-xs);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@container blocks-command-palette-preview (max-width: 40rem) {\n  \
.blocks-command-palette-preview-panes {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
.blocks-command-palette-preview-pane {\n    border-inline-start: 0;\n    border-block-start: 1px solid var(--fandhe-color-border);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, SELECTED_ITEM_ID};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"command\"",
            "data-scope=\"dialog\"",
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
            "data-scope=\"data-list\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("role=\"combobox\"").count(), 1);
        assert_eq!(html.matches("role=\"option\"").count(), 6);
        assert_eq!(html.matches("data-selected").count(), 1);
        assert!(html.contains(&format!("aria-activedescendant=\"{SELECTED_ITEM_ID}\"")));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_preview_below_list_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-command-palette-preview (max-width: 40rem)"));
        // 右ペイン（連絡先を含むプレビュー）を display: none にはせず、
        // 1 カラムへ畳んで候補一覧の下へ積む（イシュー #2968 レビュー指摘の是正）。
        assert!(!LAYOUT_CSS.contains("display: none;"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr);"));
        assert!(LAYOUT_CSS.contains("border-block-start: 1px solid var(--fandhe-color-border);"));
    }

    #[test]
    fn dialog_content_is_non_modal_and_labelled() {
        let html = demo_html();
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains("aria-label=\"People search\""));
    }
}
