//! `empty-state-invite-team` block（イシュー #2970。Application / Empty
//! State カテゴリ、最初の block）。メンバー未招待の空状態と、その場から
//! メールで招待できる入力欄、おすすめメンバー候補一覧を合成する。主参照
//! R0464（代表構成）を軸に、R1394（メール招待 + 候補の縦一覧）・R1395
//! （招待 + 候補のグリッド）を集約する。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`list-title-meta`〔イシュー #2925〕・
//! `profile-detail-datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `empty-state` / `field` / `input-group` / `input` / `button` / `avatar` /
//! `item` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成 + 縦一覧）**: R0464/R1394。候補 3 名を縦一覧
//!   （`data-layout="list"`）で並べる
//! - **B（グリッド）**: R1395。候補 4 名を 2 列グリッド
//!   （`data-layout="grid"`、狭幅では 1 列へ折り返す）で並べる
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。招待ボタン・
//! 追加ボタンはいずれも [`button::button`] の既定 `type="button"` のまま
//! 用いる。
//!
//! # アイコンは自作の単純図形
//!
//! [`fandhe_frontend_pre_styled_ui::icon`] は使用部品に含めないため、
//! 空状態の装飾アイコン・追加済みの印は [`el`] による線画
//! （`stroke="currentColor"`, `fill="none"`）のみで構成する
//! （`profile_detail_datalist::geo_icon`/`hero_email_signup::play_icon` と
//! 同型の判断）。いずれも装飾用途のため `aria-hidden="true"` を付与する。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`empty_state::root`]・[`field::root`]・[`input_group::root`]・
//! [`input_group::addon`]・[`button::button`]・[`avatar::root`]・
//! [`item::root`] はいずれも `drop_class_attr` で呼び出し側 `class` を
//! 除去してから内部 variant クラスと合成するため、これらへの CSS フックは
//! `data-*` 属性で渡す（`data-blocks-empty-state-invite-team-*`）。
//! レイアウト用ラッパー（全体スタック・招待行・候補一覧）は素の `<div>` の
//! ため `class="blocks-empty-state-invite-team-*"` を使う
//! （`profile_detail_datalist` と同型の判断）。
//!
//! # 狭い幅では招待行を縦積みに、グリッド候補を 1 列にする（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-empty-state-invite-team-stack` へ `container-type: inline-size`
//! を宣言し、コンテナ幅が `36rem` 未満のとき招待行の入力グループを縦積みに
//! 折り返し、版 B の候補グリッドを 1 列へ切り替える。`input-group`
//! recipe の詳細度に勝つため `[data-scope="input-group"][data-part="root"]
//! [data-blocks-empty-state-invite-team-group]` の形で書く
//! （`error_page_centered` の Bugbot 是正 PR #3212 と同型の判断）。
//!
//! # `id` は 2 インスタンス分すべて別値にする
//!
//! A/B の 2 インスタンスは同一 Demo 内へ並記するため、`FieldProps::id`
//! （ラベル `for`/input `id` の関連付け元）を suffix で分ける
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # ダミー素材について
//!
//! 氏名・役職は `crate::blocks::dummy_assets`（架空セット）を使う。
//! メールアドレスは `example.com` ドメインとし、実在の人物・企業・PII は
//! 含まない。アバター画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::AVATAR_SRC`]）を使う（外部 URL・`data:` URI は
//! 使わない）。各版で候補 1 名を「追加済み」状態にする。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::item::{
    self, ItemMediaVariant, ItemRootProps, ItemSize, ItemVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 版ラベル（`hero_email_signup::variant_label` と同型）。素の `<p>`（core）
/// で出し、使用部品を Issue 指定の 7 部品のまま維持する。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。装飾のため `aria-hidden="true"` を付与する。
fn geo_icon(view_box: &'static str, path_d: &'static str) -> Node {
    el(
        "svg",
        vec![
            ("viewBox", view_box),
            ("width", "40"),
            ("height", "40"),
            ("aria-hidden", "true"),
        ],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 空状態の装飾アイコン（人物シルエット + プラス記号）。
fn invite_icon() -> Node {
    geo_icon(
        "0 0 24 24",
        "M9 11a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M3 20c0-3.3 2.7-6 6-6s6 2.7 6 6 M18 8v6 M15 11h6",
    )
}

/// 「追加済み」の印に添える小さいチェックマーク。
fn check_icon() -> Node {
    el(
        "svg",
        vec![
            ("viewBox", "0 0 16 16"),
            ("width", "14"),
            ("height", "14"),
            ("aria-hidden", "true"),
        ],
        vec![el(
            "path",
            vec![
                ("d", "M3 8.5l3 3 7-7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// メールアドレス入力 + 招待送信ボタンを一体化した入力グループ
/// （`hero_email_signup::signup_group` と同型）。`instance` はインスタンス
/// ごとに異なる `id` の suffix（モジュール doc「`id` は 2 インスタンス分
/// すべて別値にする」節）。
fn invite_field(instance: &'static str) -> Node {
    let field_id = format!("blocks-empty-state-invite-team-email-{instance}");
    let email_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &email_field,
        vec![("data-blocks-empty-state-invite-team-field", "")],
        vec![
            field::label(&email_field, vec![], vec![text("メールアドレス")]),
            input_group::root(
                &group_props,
                vec![("data-blocks-empty-state-invite-team-group", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "member@example.com"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![("data-blocks-empty-state-invite-team-addon", "")],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-empty-state-invite-team-submit", "")],
                            vec![text("招待を送る")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// おすすめメンバー候補 1 件（アバター + 氏名・役職 + 追加操作）。`added`
/// が `true` のときは追加ボタンの代わりに「追加済み」の印を出す。
fn candidate_item(name: &'static str, job_title: &'static str, added: bool) -> Node {
    let action = if added {
        span(
            vec![("class", "blocks-empty-state-invite-team-added")],
            vec![check_icon(), text("追加済み")],
        )
    } else {
        button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Sm,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("追加")],
        )
    };

    item::root(
        ItemRootProps {
            variant: ItemVariant::Outline,
            size: ItemSize::Sm,
            ..ItemRootProps::default()
        },
        vec![("data-blocks-empty-state-invite-team-candidate", "")],
        vec![
            item::media(
                ItemMediaVariant::Default,
                vec![],
                vec![avatar::root(
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
                )],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(name)]),
                    item::description(vec![], vec![text(job_title)]),
                ],
            ),
            item::actions(vec![], vec![action]),
        ],
    )
}

/// 候補一覧（`layout` は `"list"`/`"grid"`、[`LAYOUT_CSS`] 側の
/// `[data-layout]` セレクタで列数を切り替える）。
fn candidates(layout: &'static str, people: &[(usize, bool)]) -> Node {
    let items = people
        .iter()
        .map(|&(index, added)| {
            let name = dummy_assets::PERSON_NAMES[index];
            let job_title = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
            candidate_item(name, job_title, added)
        })
        .collect();
    div(
        vec![
            ("class", "blocks-empty-state-invite-team-candidates"),
            ("data-layout", layout),
        ],
        items,
    )
}

/// 空状態 + 招待行 + 候補一覧の 1 インスタンス分を組み立てる。
fn invite_block(instance: &'static str, layout: &'static str, people: &[(usize, bool)]) -> Node {
    div(
        vec![("class", "blocks-empty-state-invite-team-instance")],
        vec![
            empty_state::root(
                &EmptyStateProps::default(),
                vec![("data-blocks-empty-state-invite-team-empty", "")],
                vec![
                    empty_state::indicator(vec![], vec![invite_icon()]),
                    empty_state::title(vec![], vec![text("まだメンバーがいません")]),
                    empty_state::description(
                        vec![],
                        vec![text(
                            "チームメンバーをメールで招待するか、おすすめの候補から追加してください。",
                        )],
                    ),
                    empty_state::actions(
                        vec![("data-blocks-empty-state-invite-team-field-wrap", "")],
                        vec![invite_field(instance)],
                    ),
                ],
            ),
            candidates(layout, people),
        ],
    )
}

/// `empty-state-invite-team` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 版と集約元の対応」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-empty-state-invite-team-stack")],
        vec![
            variant_label("代表構成 + 縦一覧（R0464/R1394）"),
            invite_block("list", "list", &[(0, false), (1, true), (2, false)]),
            variant_label("グリッド候補（R1395）"),
            invite_block(
                "grid",
                "grid",
                &[(3, false), (4, true), (5, false), (6, false)],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/empty-state-invite-team/",
    title: "empty-state-invite-team",
    category: BlockCategory::EmptyState,
    rust_source: "crates/docs-site/src/blocks/application/empty_state/empty_state_invite_team.rs",
    demo_class: "blocks-empty-state-invite-team",
    parts: &[
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
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
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Item",
            path: "/themes/item/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `empty_state_invite_team` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-empty-state-invite-team-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-empty-state-invite-team;\n}\n\
.blocks-empty-state-invite-team-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-empty-state-invite-team-empty] {\n  width: 100%;\n  max-width: 40rem;\n  margin-inline: auto;\n  text-align: center;\n}\n\
[data-blocks-empty-state-invite-team-field-wrap] {\n  width: 100%;\n}\n\
[data-scope=\"field\"][data-part=\"root\"][data-blocks-empty-state-invite-team-field] {\n  width: 100%;\n  max-width: 28rem;\n  margin-inline: auto;\n  text-align: start;\n}\n\
.blocks-empty-state-invite-team-candidates {\n  display: grid;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
.blocks-empty-state-invite-team-candidates[data-layout=\"grid\"] {\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n}\n\
.blocks-empty-state-invite-team-added {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  color: var(--fandhe-color-fg-subtle);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
@container blocks-empty-state-invite-team (max-width: 36rem) {\n  \
[data-scope=\"input-group\"][data-part=\"root\"][data-blocks-empty-state-invite-team-group] {\n    flex-direction: column;\n    align-items: stretch;\n  }\n  \
[data-scope=\"button\"][data-part=\"root\"][data-blocks-empty-state-invite-team-submit] {\n    width: 100%;\n  }\n  \
.blocks-empty-state-invite-team-candidates[data-layout=\"grid\"] {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"empty-state\"",
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"button\"",
            "data-scope=\"avatar\"",
            "data-scope=\"item\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        // 招待ボタン 2 個（A/B）+ 追加ボタン 5 個（追加済みでない候補の数）。
        assert_eq!(html.matches("<button").count(), 7);
        assert_eq!(html.matches("追加済み").count(), 2);
        assert_eq!(
            html.matches("data-scope=\"empty-state\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-scope=\"item\" data-part=\"root\"")
                .count(),
            7
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains(dummy_assets::AVATAR_SRC));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-empty-state-invite-team (max-width: 36rem)"));
    }

    #[test]
    fn email_inputs_are_required_and_typed() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="email""#).count(), 2);
        assert_eq!(html.matches(r#" required="""#).count(), 2);
    }
}
