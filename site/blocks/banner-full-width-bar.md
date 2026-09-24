# banner-full-width-bar

`callout` / `button` / `link` / `badge` / `icon` / `code` を合成した、
ページ上端に置く全幅の告知バーの合成例です。集約元は対応表 ID R0007 /
R0009 / R0010 / R0011 / R0013 / R0014 / R0404 / R0405 / R0754〜R0760 の
15 件です。

`<form>` を持たず、ボタンはすべて `type="button"` のまま送信先・閉じる
処理を持ちません。実際の開閉・遷移は利用者の Rust/JS コードで実装して
ください（`docs/policy/intentional-non-adoption.md` §3.25）。

配色は淡色・暗色・アクセント色の 3 種、配置は中央寄せと左寄せです。
`< 48rem`（Demo 枠幅ではなくビューポート幅が基準）では告知文が折り返し、
ピル型リンク・右リンク群・2 個目のボタンといった補助要素は隠れます
（閉じるボタンは残ります）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, strong, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定定数（モジュール doc「リンク先を固定定数にする理由」
/// 節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 告知の抽象アイコン（自作の幾何 SVG。装飾用途のため `aria-hidden`）。
fn announce_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![
            el("circle", vec![("cx", "12"), ("cy", "12"), ("r", "3")], vec![]),
            el(
                "path",
                vec![
                    (
                        "d",
                        "M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1",
                    ),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("fill", "none"),
                ],
                vec![],
            ),
        ],
    )
}

/// 帯 1 個の共通骨格（外枠 [`callout::root`] + 内側 `.blocks-banner-full-
/// width-bar-row`）。`content` はコンテンツ領域（アイコン + 告知文 +
/// 任意の追加パーツ）、`aux` は右リンク群等の任意の補助領域、`close` は
/// 任意の閉じるボタン。`position` は `"bottom"` のときのみ下端固定の形
/// （モジュール doc「下端固定の形を通常配置で見せる理由」節）を表す。
fn bar(
    tone: &'static str,
    align: &'static str,
    position: Option<&'static str>,
    content: Vec<Node>,
    aux: Option<Node>,
    close: Option<Node>,
) -> Node {
    let mut row_children = vec![div(
        vec![("class", "blocks-banner-full-width-bar-content")],
        content,
    )];
    if let Some(aux_node) = aux {
        row_children.push(aux_node);
    }
    if let Some(close_node) = close {
        row_children.push(close_node);
    }

    let mut root_attrs = vec![
        ("data-blocks-banner-full-width-bar-bar", ""),
        ("data-blocks-banner-full-width-bar-tone", tone),
        ("data-blocks-banner-full-width-bar-align", align),
    ];
    if let Some(pos) = position {
        root_attrs.push(("data-blocks-banner-full-width-bar-position", pos));
    }

    callout::root(
        &CalloutProps::default(),
        root_attrs,
        vec![div(
            vec![("class", "blocks-banner-full-width-bar-row")],
            row_children,
        )],
    )
}

/// 閉じるボタン（各インスタンスで独立生成。`aria-label` 重複は許容する、
/// `crate::blocks` の `demo_output_has_no_dangling_aria_references_or_
/// duplicate_ids` は `id`/`aria-controls` 等の参照整合のみを検証し
/// `aria-label` の重複は対象外）。
fn dismiss_button() -> Node {
    button::close_button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        "Dismiss",
        vec![("data-blocks-banner-full-width-bar-close", "")],
    )
}

/// 1 個目: 基準形（淡色・アイコン + 告知文 + ピル型リンク + 閉じる、
/// 中央寄せ）。対応: R0754, R0007。
fn instance_basic() -> Node {
    bar(
        "light",
        "center",
        None,
        vec![
            announce_icon(),
            p(vec![], vec![text("Version 2.0 is out — see what's new")]),
            badge::link(
                REPO,
                &BadgeProps::default(),
                false,
                vec![("data-blocks-banner-full-width-bar-aux", "")],
                vec![text("Changelog")],
            ),
        ],
        None,
        Some(dismiss_button()),
    )
}

/// 2 個目: インラインコード（アクセント色・告知文中に `code`/`link` +
/// 閉じる）。対応: R0009, R0011, R0758。
fn instance_inline_code() -> Node {
    bar(
        "accent",
        "center",
        None,
        vec![p(
            vec![],
            vec![
                text("Upgrade to "),
                code::code(&CodeProps::default(), vec![], vec![text("v2.0.0")]),
                text(" — "),
                link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("read the guide")],
                ),
            ],
        )],
        None,
        Some(dismiss_button()),
    )
}

/// 3 個目: 暗色・右リンク群（左に告知文、右にリンク群、左寄せ）。
/// 対応: R0010, R0404, R0759, R0405。
fn instance_dark_link_group() -> Node {
    let links = div(
        vec![("data-blocks-banner-full-width-bar-aux", "")],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("Docs")]),
            link::root(REPO, &LinkProps::default(), vec![], vec![text("Pricing")]),
            link::root(REPO, &LinkProps::default(), vec![], vec![text("GitHub")]),
        ],
    );
    bar(
        "dark",
        "start",
        None,
        vec![
            announce_icon(),
            p(
                vec![],
                vec![text("We're hiring across engineering and design")],
            ),
        ],
        Some(links),
        None,
    )
}

/// 4 個目: タイトル + 説明 + 2 ボタン（淡色）。対応: R0013, R0014。
fn instance_title_two_buttons() -> Node {
    let actions = div(
        vec![("data-blocks-banner-full-width-bar-aux", "")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Get started")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Learn more")],
            ),
        ],
    );
    bar(
        "light",
        "start",
        None,
        vec![
            strong(vec![], vec![text("New: workspace roles.")]),
            p(
                vec![],
                vec![text("Invite teammates with fine-grained access.")],
            ),
        ],
        Some(actions),
        Some(dismiss_button()),
    )
}

/// 5 個目: 文全体がリンク（暗色）。対応: R0755。
fn instance_full_text_link() -> Node {
    bar(
        "dark",
        "center",
        None,
        vec![link::root(
            REPO,
            &LinkProps::default(),
            vec![],
            vec![text("Join the beta program — request access")],
        )],
        None,
        Some(dismiss_button()),
    )
}

/// 6 個目: 文全体がリンク・閉じるボタンなし（アクセント色）。
/// 対応: R0756, R0757。
fn instance_full_text_link_no_dismiss() -> Node {
    bar(
        "accent",
        "center",
        None,
        vec![link::root(
            REPO,
            &LinkProps::default(),
            vec![],
            vec![text("Black Friday: 30% off all annual plans")],
        )],
        None,
        None,
    )
}

/// 7 個目: 下端固定の形（淡色。Demo 枠内では通常配置、モジュール doc
/// 「下端固定の形を通常配置で見せる理由」節参照）。対応: R0760。
fn instance_bottom_fixed() -> Node {
    bar(
        "light",
        "center",
        Some("bottom"),
        vec![
            announce_icon(),
            p(
                vec![],
                vec![text("This site uses cookies to improve your experience")],
            ),
            badge::link(
                REPO,
                &BadgeProps::default(),
                false,
                vec![("data-blocks-banner-full-width-bar-aux", "")],
                vec![text("Learn more")],
            ),
        ],
        None,
        Some(dismiss_button()),
    )
}

/// 1 行のキャプション（`h2`/`h3` は使わない。右目次・折りたたみ目次が
/// 拾ってしまうため、`footer_newsletter` と同じ判断で素の `p` を使う）。
fn caption(label: &str) -> Node {
    p(
        vec![("data-blocks-banner-full-width-bar-caption", "")],
        vec![text(label)],
    )
}

/// `banner-full-width-bar` の Demo 本体（7 インスタンスを縦に並べる）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-full-width-bar-stack")],
        vec![
            caption("Light · pill link · dismissible · centered"),
            instance_basic(),
            caption("Accent · inline code + link · dismissible · centered"),
            instance_inline_code(),
            caption("Dark · right link group · left-aligned"),
            instance_dark_link_group(),
            caption("Light · title + description + 2 buttons · dismissible"),
            instance_title_two_buttons(),
            caption("Dark · whole text is a link · dismissible · centered"),
            instance_full_text_link(),
            caption("Accent · whole text is a link · no dismiss · centered"),
            instance_full_text_link_no_dismiss(),
            caption("Light · bottom-fixed variant (shown in normal flow)"),
            instance_bottom_fixed(),
        ],
    )
}
```

## 差分メモ

参照は対応表 ID R0007 / R0009 / R0010 / R0011 / R0013 / R0014 / R0404 /
R0405 / R0754〜R0760 の 15 件を集約します。参照元の文言・配色・装飾・
アイコンは持ち込まず、既存トーンでデモ文言を独自に書いています。

- **7 インスタンスに集約**: 参照元 15 件それぞれを個別 DOM にせず、
  代表的なバリエーション（配色 3 種・配置 2 種・インラインコード・
  右リンク群・2 ボタン・文全体リンク・閉じるボタン有無・下端固定）を
  網羅する 7 インスタンスへ畳んでいます。各インスタンスの直前のキャプション
  （`h2`/`h3` ではなく素の `p`。右目次・折りたたみ目次に拾われないための
  判断）に対応関係を記しています。
- **下端固定（R0760）は通常配置で表示**: `position: fixed` を使うと Demo
  枠を突き抜けて画面へ張り付いてしまうため、罫線の向き（`border-top` へ
  切替）のみで下端固定の形を表現し、配置自体は他インスタンスと同じ通常
  フローに置いています。
- **ブランド色は `--fandhe-color-accent` へ置換**: 参照元のブランド色は
  持ち込まず、既存のアクセントトークンで代替しています。
- **リンク先は固定の GitHub リポジトリ URL**: `demo()` が `base_path` を
  受け取れない制約と `linkcheck::check_links` の `href="#"` 拒否のため、
  全リンク・ピル型リンクの href は `Fandhe-AI` の実在リポジトリへの外部
  絶対 URL を使っています。
- **配色 tone の上書きは詳細度 (0,4,0) 以上**: `callout::root` の
  variant/size recipe base（詳細度 (0,3,0)）に負けないよう、tone 上書きは
  Demo ラッパクラス + `[data-scope]`/`[data-part]`/`data-*` 属性による
  詳細度 (0,4,0) 以上のセレクタで宣言しています。
