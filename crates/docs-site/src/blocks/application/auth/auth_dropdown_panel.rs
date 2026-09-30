//! `auth-dropdown-panel` block（イシュー #2962。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、対応表 ID R0685 を主参照とする合成例。
//! 集約元は 1 件のみのため `site/blocks/auth-dropdown-panel.md` の
//! 「原案差分メモ」節に個別の差分は記載しない。
//!
//! # 構成（ナビバーのトリガーから開くサインインパネル）
//!
//! ナビバー（`header`）右端の「サインイン」ボタンを起点に開く `popover`
//! パネル内に、メール・パスワード入力欄・送信ボタン・パスワード再設定と
//! 新規登録の導線をまとめる。ヘルプ用の `menu`（閉状態）を並置し、
//! `popover`/`menu` の 2 種類のオーバーレイ trigger を区別できるようにする。
//!
//! # 使用部品
//!
//! `popover` / `menu` / `field` / `input` / `button` / `link` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、サインインパネルを開いた状態で固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、Demo はサインイン
//! パネルを開いた状態のまま固定描画する（[`super::super::header::
//! header_flyout_menu`] と同じ判断）。トリガー（サインインボタン・
//! ヘルプメニュー）は押しても何も起きないため `disabled: true`
//! （ネイティブ `disabled` 属性 + `aria-disabled="true"`）にし、
//! [`LAYOUT_CSS`] の `[data-disabled]` 複合セレクタで中和して通常状態と
//! 同じ見た目に保つ。
//!
//! # 2 レイアウト並記（広幅・狭幅）
//!
//! `wide`（既定幅）と `narrow`（`data-blocks-auth-dropdown-panel-frame=
//! "narrow"` で Demo 枠を `max-inline-size: 22rem` に固定した
//! インスタンス）を caption 付きで縦に並記する（
//! [`super::super::navbar::navbar_with_search`] と同型）。広幅では
//! パネルをトリガー右揃えで絶対配置し、狭幅ではパネルをヘッダー全幅に
//! 広げてトリガー直下へ配置する（イシュー本文の要件）。狭幅判定の
//! 閾値は `< 48rem` ではなく `< 24rem`（`@container` 節参照）。docs
//! サイトの `.docs-content` は最大幅 46rem のため、`48rem` のままでは
//! `wide` インスタンスも常に狭幅条件を満たしてしまい広幅レイアウトを
//! 実演できない（PR #3415 レビュー指摘）。
//!
//! # id と ARIA の一意性
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-auth-dropdown-panel-{email,password,trigger,panel,title,
//! menu-trigger,menu}-{variant}` の形で variant ごとに一意にする
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` 参照）。
//!
//! # `<form>` を持たない・認証処理を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。送信ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。パスワード入力欄は `value` を持たず、値の送信・検証は一切
//! 行わない。「新規登録」は実在する `signup-01` block ページへのリンク、
//! 「パスワードをお忘れですか」は遷移先を持たないため `link::root` の
//! `href="#"` ではなく `ButtonVariant::Link` の `<button type="button">`
//! を使う（[`super::login_04`] と同じ判断）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, header, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// ロゴ（ブランド名テキストのみ、リンクにしない）。
fn logo() -> Node {
    span(
        vec![("class", "blocks-auth-dropdown-panel-logo")],
        vec![text("Fandhe Console")],
    )
}

/// ヘルプメニュー（閉状態固定、`disabled: true`）。`variant` は id の
/// suffix（モジュール doc「id と ARIA の一意性」節）。
fn help_menu(variant: &str) -> Node {
    let trigger_id = format!("blocks-auth-dropdown-panel-menu-trigger-{variant}");
    let content_id = format!("blocks-auth-dropdown-panel-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-auth-dropdown-panel-menu-trigger", ""),
        ],
        vec![text("ヘルプ")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![],
        vec![
            menu::item("docs", false, false, vec![], vec![text("ドキュメント")]),
            menu::item("contact", false, false, vec![], vec![text("お問い合わせ")]),
            menu::item("status", false, false, vec![], vec![text("稼働状況")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// メール・パスワード入力欄一式（[`FieldOrientation::Vertical`] で縦積み）。
/// `variant` は id の suffix。
fn credential_fields(variant: &str) -> Vec<Node> {
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let email_id = format!("blocks-auth-dropdown-panel-email-{variant}");
    let email_field = FieldProps {
        id: &email_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let password_id = format!("blocks-auth-dropdown-panel-password-{variant}");
    let password_field = FieldProps {
        id: &password_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    vec![
        field::root(
            &orientation,
            &email_field,
            vec![("data-blocks-auth-dropdown-panel-field", "")],
            vec![
                field::label(&email_field, vec![], vec![text("メールアドレス")]),
                input::input(
                    &InputProps::default(),
                    &email_field,
                    vec![("type", "email"), ("placeholder", "you@example.com")],
                ),
            ],
        ),
        field::root(
            &orientation,
            &password_field,
            vec![("data-blocks-auth-dropdown-panel-field", "")],
            vec![
                field::label(&password_field, vec![], vec![text("パスワード")]),
                input::input(
                    &InputProps::default(),
                    &password_field,
                    vec![("type", "password")],
                ),
            ],
        ),
    ]
}

/// 補助導線行（パスワード再設定・新規登録）。
fn auxiliary_links() -> Node {
    div(
        vec![("data-blocks-auth-dropdown-panel-links", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Link,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("パスワードをお忘れですか")],
            ),
            span(
                vec![],
                vec![
                    text("アカウントをお持ちでない方は "),
                    link::root(
                        "../signup-01/",
                        &LinkProps::default(),
                        vec![],
                        vec![text("新規登録")],
                    ),
                ],
            ),
        ],
    )
}

/// サインインパネル（`popover`、開状態固定・`disabled: true`）。`variant`
/// は id の suffix。
fn sign_in_panel(variant: &str) -> Node {
    let trigger_id = format!("blocks-auth-dropdown-panel-trigger-{variant}");
    let panel_id = format!("blocks-auth-dropdown-panel-panel-{variant}");
    let title_id = format!("blocks-auth-dropdown-panel-title-{variant}");

    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(panel_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-auth-dropdown-panel-trigger", ""),
        ],
        vec![text("サインイン")],
    );

    let mut panel_children: Vec<Node> = vec![
        popover::title(Some(title_id.as_str()), vec![], vec![text("サインイン")]),
        popover::description(
            None,
            vec![],
            vec![text("登録済みのメールアドレスでサインインします")],
        ),
    ];
    panel_children.extend(credential_fields(variant));
    panel_children.push(button::button(
        &ButtonProps::default(),
        vec![("data-blocks-auth-dropdown-panel-submit", "")],
        vec![text("サインイン")],
    ));
    panel_children.push(auxiliary_links());

    let content = popover::content(
        OpenState::Open,
        Some(panel_id.as_str()),
        Some(title_id.as_str()),
        None,
        vec![("data-blocks-auth-dropdown-panel-panel", "")],
        panel_children,
    );
    let positioner = popover::positioner(
        OpenState::Open,
        vec![("data-blocks-auth-dropdown-panel-positioner", "")],
        vec![content],
    );

    popover::root(
        OpenState::Open,
        vec![("data-blocks-auth-dropdown-panel-popover", "")],
        vec![trigger, positioner],
    )
}

/// caption（並記された各レイアウトの見出し）。
fn caption(label: &'static str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-auth-dropdown-panel-caption")],
        vec![text(label)],
    )
}

/// 1 レイアウト分のナビバー本体を組み立てる。`narrow` は Demo 枠を
/// `max-inline-size: 22rem`（`< 24rem` の狭幅判定を確実に満たす幅）に
/// 固定するフラグ（[`super::super::navbar::navbar_with_search`] と同型）。
fn bar(variant: &'static str, narrow: bool) -> Node {
    let actions = div(
        vec![("data-blocks-auth-dropdown-panel-actions", "")],
        vec![help_menu(variant), sign_in_panel(variant)],
    );

    let mut frame_attrs = vec![("data-blocks-auth-dropdown-panel-shell", "")];
    if narrow {
        frame_attrs.push(("data-blocks-auth-dropdown-panel-frame", "narrow"));
    }

    div(
        frame_attrs,
        vec![header(
            vec![
                ("data-blocks-auth-dropdown-panel-root", ""),
                ("data-blocks-auth-dropdown-panel-variant", variant),
            ],
            vec![logo(), actions],
        )],
    )
}

/// `auth-dropdown-panel` の Demo 本体。広幅・狭幅の 2 レイアウトを縦に
/// 並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-auth-dropdown-panel-stack")],
        vec![
            caption("広幅（トリガー右揃えでパネルを絶対配置）"),
            bar("wide", false),
            caption("狭幅（< 24rem、パネルをトリガー直下へ全幅表示）"),
            bar("narrow", true),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-dropdown-panel/",
    title: "auth-dropdown-panel",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_dropdown_panel.rs",
    demo_class: "blocks-auth-dropdown-panel",
    parts: &[
        Part {
            label: "Popover",
            path: "/themes/popover/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_dropdown_panel` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。`--fandhe-*` トークンのみ使用し、
/// 生値は幅・rem 指定のみに限る。
///
/// # シェルの高さ確保
///
/// `.blocks-demo` は横方向のみ `overflow-x: auto` だが縦方向はブロック軸の
/// 自然な高さで確定するため、絶対配置されるサインインパネル（styled
/// popover の `positioner` recipe が `position: absolute; top: 100%` を
/// 持つ）がシェルの外へはみ出すと縦方向にクリップされる（
/// [`super::super::header::header_flyout_menu`] と同じ教訓）。
/// `[data-blocks-auth-dropdown-panel-shell]` へ `container-type:
/// inline-size`（コンテナクエリの評価基盤）と `min-block-size: 26rem`
/// （高さ予約）を宣言する。
///
/// # popover recipe を上書きする詳細度
///
/// styled popover の `root`/`positioner`/`trigger`/`content` recipe は
/// いずれも `[data-scope="popover"][data-part="..."]`（詳細度 `(0,2,0)`）
/// の無条件 base 規則、disabled state は
/// `[data-scope="popover"][data-part="trigger"][data-disabled]`（詳細度
/// `(0,3,0)`）を持つ（`crates/pre-styled-ui/src/popover.rs` 参照）。本
/// block はこれらを上書きするため、`data-scope`/`data-part` を含む複合
/// セレクタへ block 固有属性を追加して常に 1 個以上上回る詳細度にする
/// （[`super::super::header::header_flyout_menu`] と同じ判断）。
/// `menu` の disabled state も同様（`crates/pre-styled-ui/src/menu.rs`）。
///
/// # 広幅時のパネル配置
///
/// popover recipe 既定は `positioner { left: 0; top: 100%; }` でトリガー
/// から右へ伸びるため、右端に置かれたトリガーからだと水平オーバーフロー
/// する。`right: 0; left: auto;` へ上書きしトリガー右揃えにする。`content`
/// は `inline-size: 22rem` で固定幅にする。
///
/// # 狭幅時（`< 24rem`）のパネル配置
///
/// `@container blocks-auth-dropdown-panel (max-width: 23.99rem)` で
/// popover の `root` を `position: static` に切り替え、包含ブロックを
/// 共通祖先の `header`（[data-blocks-auth-dropdown-panel-root] へ常時
/// `position: relative` を宣言済み）へ移す。`positioner` を
/// `inset-inline: 0`（左右 0）にしてヘッダー幅いっぱいに広げ、`content`
/// の固定幅は `inline-size: auto` で解除する（イシュー本文の「狭幅では
/// パネルを幅いっぱいに広げトリガー直下に置く」要件）。
///
/// 閾値はイシュー本文が挙げる `48rem` ではなく `24rem` にする。docs
/// サイトの `.docs-content` は最大幅 46rem のため、`48rem` のままでは
/// `max-inline-size` を持たない `wide` インスタンスも常にこの
/// `@container` 条件を満たしてしまい、広幅レイアウト（絶対配置・
/// `content` 固定幅 22rem）を Demo 上で確認できなくなる（PR #3415
/// レビュー指摘、コメント URL:
/// <https://github.com/Fandhe-AI/fandhe-frontend/pull/3415#discussion_r4134148535>）。
/// `narrow` インスタンスは `max-inline-size: 22rem` に固定済みのため、
/// 22rem 超 46rem 以下のどの閾値でも両インスタンスを意図どおり
/// 分離できる。
///
/// # `h2` の見た目リセット
///
/// サイト本文の `h2` 装飾（罫線・letter-spacing 等）がパネル見出し
/// （`popover::title` の `h2`）へ乗るのを防ぐ（
/// [`super::super::contact::contact_dialog_form`] と同型の判断）。
const LAYOUT_CSS: &str = "\
.blocks-auth-dropdown-panel-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-auth-dropdown-panel-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-auth-dropdown-panel-shell] {\n  container-type: inline-size;\n  container-name: blocks-auth-dropdown-panel;\n  min-block-size: 26rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-blocks-auth-dropdown-panel-shell][data-blocks-auth-dropdown-panel-frame=\"narrow\"] {\n  max-inline-size: 22rem;\n}\n\
[data-blocks-auth-dropdown-panel-root] {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-subtle);\n  position: relative;\n}\n\
.blocks-auth-dropdown-panel-logo {\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-auth-dropdown-panel-actions] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-auth-dropdown-panel-menu-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"popover\"][data-part=\"trigger\"][data-blocks-auth-dropdown-panel-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"popover\"][data-part=\"positioner\"][data-blocks-auth-dropdown-panel-positioner] {\n  left: auto;\n  right: 0;\n}\n\
[data-scope=\"popover\"][data-part=\"content\"][data-blocks-auth-dropdown-panel-panel] {\n  inline-size: 22rem;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-auth-dropdown-panel [data-scope=\"popover\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
[data-blocks-auth-dropdown-panel-field] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-auth-dropdown-panel-submit] {\n  width: 100%;\n}\n\
[data-blocks-auth-dropdown-panel-links] {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
@container blocks-auth-dropdown-panel (max-width: 23.99rem) {\n  \
[data-scope=\"popover\"][data-part=\"root\"][data-blocks-auth-dropdown-panel-popover] {\n    position: static;\n  }\n  \
[data-scope=\"popover\"][data-part=\"positioner\"][data-blocks-auth-dropdown-panel-positioner] {\n    left: 0;\n    right: 0;\n    inset-inline: 0;\n  }\n  \
[data-scope=\"popover\"][data-part=\"content\"][data-blocks-auth-dropdown-panel-panel] {\n    inline-size: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 6 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src を持たないこと（`input` パーツは headless
    /// `field::input` へ委譲するため `data-scope="field"` として現れる）。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"popover\"",
            "data-scope=\"menu\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-scope="field" data-part="input""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("mailto:"));
    }

    /// `wide`/`narrow` の 2 variant が並記され、caption が 2 件あること。
    #[test]
    fn demo_renders_both_layouts() {
        let html = render(&demo());
        for variant in ["wide", "narrow"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-auth-dropdown-panel-variant=\"{variant}\""
                )),
                "variant {variant} should render"
            );
        }
        assert_eq!(
            html.matches("blocks-auth-dropdown-panel-caption").count(),
            2
        );
        assert_eq!(
            html.matches("data-blocks-auth-dropdown-panel-shell")
                .count(),
            2
        );
    }

    /// `narrow` インスタンスのみ frame 属性を持つこと。
    #[test]
    fn only_narrow_instance_has_the_frame_attribute() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-auth-dropdown-panel-frame=\"narrow\"")
                .count(),
            1
        );
    }

    /// サインインパネルが開状態、ヘルプメニューが閉状態で固定描画される
    /// こと。トリガーはいずれも disabled であること。
    #[test]
    fn sign_in_panel_is_open_and_help_menu_is_closed() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"data-state="open""#).count(), 8);
        assert_eq!(
            html.matches("data-blocks-auth-dropdown-panel-trigger")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-blocks-auth-dropdown-panel-menu-trigger")
                .count(),
            2
        );
        assert_eq!(html.matches("data-disabled").count(), 4);
        for hook in [
            "data-blocks-auth-dropdown-panel-trigger",
            "data-blocks-auth-dropdown-panel-menu-trigger",
        ] {
            for pos in html.match_indices(hook) {
                let tag_end = html[pos.0..]
                    .find('>')
                    .map(|rel| pos.0 + rel)
                    .expect("trigger tag should close");
                let tag_start = html[..pos.0]
                    .rfind("<button")
                    .expect("trigger should be a button");
                assert!(html[tag_start..tag_end].contains("data-disabled"));
            }
        }
    }

    /// メール・パスワード入力欄の `id`/`for` が variant ごとに一意である
    /// こと（headless `field::label`/`field::input` は `FieldProps::id`
    /// から `"{id}-control"` を決定的に派生させる）。
    #[test]
    fn credential_fields_have_unique_ids_per_variant() {
        let html = render(&demo());
        for variant in ["wide", "narrow"] {
            for field in ["email", "password"] {
                let control_id = format!("blocks-auth-dropdown-panel-{field}-{variant}-control");
                assert_eq!(
                    html.matches(&format!(r#"id="{control_id}""#)).count(),
                    1,
                    "control_id={control_id}"
                );
                assert_eq!(
                    html.matches(&format!(r#"for="{control_id}""#)).count(),
                    1,
                    "control_id={control_id}"
                );
            }
        }
        assert_eq!(html.matches(r#"type="email""#).count(), 2);
        assert_eq!(html.matches(r#"type="password""#).count(), 2);
    }

    /// パネルの `aria-controls`/`id`/`aria-labelledby` が variant ごとに
    /// 一意に対応すること。
    #[test]
    fn panel_references_are_unique_and_consistent() {
        let html = render(&demo());
        for variant in ["wide", "narrow"] {
            let panel_id = format!("blocks-auth-dropdown-panel-panel-{variant}");
            let title_id = format!("blocks-auth-dropdown-panel-title-{variant}");
            assert!(html.contains(&format!(r#"aria-controls="{panel_id}""#)));
            assert!(html.contains(&format!(r#"id="{panel_id}""#)));
            assert!(html.contains(&format!(r#"aria-labelledby="{title_id}""#)));
            assert!(html.contains(&format!(r#"id="{title_id}""#)));
        }
    }

    /// 「新規登録」が実在する `signup-01` block ページを指すこと。
    #[test]
    fn signup_link_points_to_existing_block_page() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"href="../signup-01/""#).count(), 2);
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ・高さ予約・disabled 中和を満たす
    /// こと。
    #[test]
    fn layout_css_has_container_query_and_guards() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("min-block-size: 26rem;"));
        assert!(LAYOUT_CSS.contains("@container blocks-auth-dropdown-panel (max-width: 23.99rem)"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("position: static;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-auth-dropdown-panel-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-auth-dropdown-panel-stack");
    }

    /// 呼び出しごとに同一の `Node` を返す決定的な関数であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }
}
