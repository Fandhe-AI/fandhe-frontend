# page-heading-avatar

`fandhe-frontend-pre-styled-ui` の `avatar` / `image` / `heading` /
`link` / `button` / `menu` 部品を合成した、アバター・ロゴ付きページ見出しの
実例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R1130、集約元は R0594・R1134。出典の固有名・ファイル名は
記載しません）。

左に円形アバター（または企業ロゴ）、右に名前と補足行（役職・所属、メール、
請求書番号など）、右端に操作ボタン列と三点メニューを横並びに配置した基本形
（profile）に加え、メールアドレスと応募関連リンク・応募日を持つ形
（invite）、media を企業ロゴへ差し替えた請求書向けの形（invoice）の 3 通り
を並べています。狭い幅では名前・補足行と操作ボタン列・三点メニューが折り
返して複数行になりますが、操作ボタンが非表示になることはなく常に到達
できます。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
三点メニューは閉じた状態の固定表示です（開閉には `fandhe-frontend-wasm-full`
の JS 配線が必要で、docs サイトは JS ハイドレーションを行いません）。
補足行のメールアドレスはリンク化せずプレーンテキストのまま扱います。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// invite インスタンスの応募関連リンクの遷移先（外部の実在 URL、
/// `href="#"` は使わない、他 block と同型の判断）。個々の応募内容・応募
/// 一覧いずれの実在ページも持たないため、遷移先の実体を特定の意味に
/// 見せる固有のラベル（「応募内容を見る」「応募一覧を見る」）は避け、
/// `careers_card_grid.rs` の「詳細を見る」と同型の汎用ラベルへ変更する
/// （イシュー #2931 codex レビュー再指摘。「応募一覧を見る」は遷移先の
/// 語感には一致していたが、実際には求人応募とは無関係な GitHub Pull
/// Requests 一覧であり、依然として意味不一致だった）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// アバター（円形）を組み立てる。氏名をアクセシブルネームとして `root` へ
/// 直接付与し、フォールバックは氏名の先頭 1 文字を表示する
/// （`list_title_meta.rs::initial_avatar` と同型。`role="img"` は role なし
/// `<div>` が `aria-label` を name computation の対象にしないため必須）。
fn profile_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Xl,
            ..AvatarProps::default()
        },
        vec![("role", "img"), ("aria-label", name)],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// 企業ロゴ（invoice インスタンスの media）。`image` 部品（avatar とは別の
/// 単純画像部品）を使い、固定 `4rem` 角の枠を `data-blocks-page-heading-
/// avatar-logo` へ CSS で与える。`image` recipe の base
/// （`[data-scope="image"][data-part="root"]`、詳細度 (0,2,0)）に
/// `height: auto`/`max-width: 100%` が乗るため、[`LAYOUT_CSS`] 側は
/// `img[data-scope="image"][data-blocks-page-heading-avatar-logo]`
/// （詳細度 (0,2,1)）で上回る（イシュー #2931 Bugbot 指摘: 単一属性
/// セレクタでは詳細度が並び、flex item として縮小し得た）。
fn logo_image(company: &str) -> Node {
    image::image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, &format!("{company} のロゴ"))
        },
        vec![("data-blocks-page-heading-avatar-logo", "")],
    )
}

/// 中黒区切り（装飾。支援技術には前後の語がそのまま連続して読み上げられる
/// ため、[`visually_hidden`] は使用部品に含めない、モジュール doc「使用
/// 部品」参照）。
fn dot_separator() -> Node {
    span(vec![], vec![text(" \u{b7} ")])
}

/// 見出し（H2、[`Heading`](heading) 部品）。docs ページ自体が H1 を持つため
/// block 内は H2 以下とする（`card_heading_toolbar.rs` と同型の判断）。
fn name_heading(name: &str) -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(name)],
    )
}

/// メタ行の外枠（素の `<p>`。モジュール doc「メタ行を素の `<p>` で組み立てる
/// 理由」参照。`fandhe_frontend_pre_styled_ui::text::text` は呼び出し側の
/// `class` を `drop_class_attr` で除去するため使わない）。
fn meta_line(children: Vec<Node>) -> Node {
    el(
        "p",
        vec![("class", "blocks-page-heading-avatar-meta")],
        children,
    )
}

/// `<time datetime>` 要素。
fn time_el(iso: &'static str, label: &'static str) -> Node {
    el("time", vec![("datetime", iso)], vec![text(label)])
}

/// 三点メニュー（[`Menu`](menu) 部品）。`items` は `(value, label)` の組。
/// 無 JS のため `OpenState::Closed` 固定・disabled にはしない
/// （モジュール doc「三点メニューは無 JS のため閉じた状態で固定する」
/// 節参照）。
fn overflow_menu(
    heading_name: &str,
    content_id: &'static str,
    trigger_id: &'static str,
    items: &[(&'static str, &'static str)],
) -> Node {
    let mut menu_items: Vec<Node> = Vec::new();
    for (index, (value, label)) in items.iter().enumerate() {
        if index > 0 {
            menu_items.push(menu::separator(vec![], vec![]));
        }
        menu_items.push(menu::item(value, false, false, vec![], vec![text(*label)]));
    }
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![
            ("id", trigger_id),
            ("aria-label", &format!("その他の操作、{heading_name}")),
        ],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        Some(trigger_id),
        vec![],
        menu_items,
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// 操作ボタン + 三点メニューをまとめた actions 列。狭幅では [`LAYOUT_CSS`]
/// の `flex-wrap` で折り返すのみで、`primary`/`secondary` の 2 ボタンを
/// 非表示にはしない（モジュール doc「狭幅では操作列を折り返す（非表示には
/// しない）」節参照）。
fn actions(secondary_label: &str, primary_label: &str, menu_node: Node) -> Node {
    let secondary = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-page-heading-avatar-action", "")],
        vec![text(secondary_label)],
    );
    let primary = button::button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-page-heading-avatar-action", "")],
        vec![text(primary_label)],
    );
    div(
        vec![("class", "blocks-page-heading-avatar-actions")],
        vec![secondary, primary, menu_node],
    )
}

/// header（media | body | actions）の共通骨格。
fn header(media: Node, name_node: Node, meta: Node, actions_node: Node) -> Node {
    div(
        vec![("class", "blocks-page-heading-avatar-header")],
        vec![
            media,
            div(
                vec![("class", "blocks-page-heading-avatar-body")],
                vec![name_node, meta],
            ),
            actions_node,
        ],
    )
}

/// パネル外枠。
fn panel(variant: &'static str, content: Node) -> Node {
    div(
        vec![
            ("class", "blocks-page-heading-avatar-panel"),
            ("data-blocks-page-heading-avatar-variant", variant),
        ],
        vec![content],
    )
}

/// **profile**（R1130・主参照）インスタンス。
fn profile_instance() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let menu_node = overflow_menu(
        name,
        "blocks-page-heading-avatar-menu-profile",
        "blocks-page-heading-avatar-trigger-profile",
        &[
            ("edit-profile", "プロフィールを編集"),
            ("message", "メッセージ"),
            ("suspend", "アカウントを停止"),
        ],
    );
    panel(
        "profile",
        header(
            profile_avatar(name),
            name_heading(name),
            meta_line(vec![
                text(dummy_assets::JOB_TITLES[0]),
                dot_separator(),
                text("プラットフォームチーム"),
            ]),
            actions("メッセージ", "プロフィールを編集", menu_node),
        ),
    )
}

/// **invite**（R0594）インスタンス。
fn invite_instance() -> Node {
    let name = dummy_assets::PERSON_NAMES[1];
    let menu_node = overflow_menu(
        name,
        "blocks-page-heading-avatar-menu-invite",
        "blocks-page-heading-avatar-trigger-invite",
        &[
            ("advance", "面接に進める"),
            ("reject", "却下"),
            ("note", "メモを追加"),
        ],
    );
    panel(
        "invite",
        header(
            profile_avatar(name),
            name_heading(name),
            meta_line(vec![
                text("elena.vasquez@example.com"),
                dot_separator(),
                link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("詳細を見る")],
                ),
                dot_separator(),
                time_el("2026-09-26", "9 月 26 日に応募"),
            ]),
            actions("却下", "面接に進める", menu_node),
        ),
    )
}

/// **invoice**（R1134）インスタンス。
fn invoice_instance() -> Node {
    let company = dummy_assets::COMPANY_NAMES[0];
    let menu_node = overflow_menu(
        company,
        "blocks-page-heading-avatar-menu-invoice",
        "blocks-page-heading-avatar-trigger-invoice",
        &[
            ("record-payment", "支払いを記録"),
            ("download", "PDF をダウンロード"),
            ("void", "請求書を無効化"),
        ],
    );
    panel(
        "invoice",
        header(
            logo_image(company),
            name_heading(company),
            meta_line(vec![
                text("請求書番号 INV-0000123"),
                dot_separator(),
                time_el("2026-09-30", "2026 年 9 月 30 日 発行"),
            ]),
            actions("PDF をダウンロード", "支払いを記録", menu_node),
        ),
    )
}

/// `page-heading-avatar` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。主参照（R1130・profile）を先頭に、3 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-avatar-layout")],
        vec![profile_instance(), invite_instance(), invoice_instance()],
    )
}
```

## 原案差分メモ

- 主参照は R1130（アバター + 名前 + 操作ボタン + 三点メニュー）です。
  R0594（アバター + メール + 操作列）・R1134（企業ロゴ + 請求書番号）を
  別インスタンスとして併記しています。
- 補足行のメールアドレスはリンク化せず、応募関連リンクのみ `link` 部品を
  使っています（イシュー本文の「補足行のリンク・日時は本文テキストの
  一部として扱う」要件を、`mailto:` リンク化という前例のない解釈ではなく
  最小の形で満たしました）。遷移先は特定の実在ページを持たないため、
  `careers_card_grid.rs` と同型の汎用ラベル「詳細を見る」+ リポジトリ直下
  の URL とし、遷移先の意味を過大に主張するラベルは避けています
  （イシュー #2931 codex レビュー再指摘）。
- 実データ取得・メニュー開閉・ボタン押下は行わず、静的な初期状態のみを
  示します。氏名・役職・所属・メールアドレス・社名・請求書番号・日付は
  すべて独自の架空データです。
- ブラウザでの実機確認（`40rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
