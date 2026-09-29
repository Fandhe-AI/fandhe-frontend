# action-panel-stacked

`fandhe-frontend-pre-styled-ui` の `card` / `heading` / `text` / `button` /
`link` 部品を合成した、縦積みのアクションパネルの実例です。カード内に
タイトル・説明文・主操作（ボタン、または矢印付きリンク）を縦に積む骨格を
共有し、影付きカードと面色のみの枠（well 相当）の 2 通りの見せ方を並記
しています。狭い幅でも縦積みのままで、主操作は内容幅を保ち全幅化しません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
リンク操作は `href="#"` を使わず自リポジトリの実在 URL へ遷移させます
（可視テキストは遷移先がわかる「GitHub で見る」、末尾の矢印は
`aria-hidden="true"` の装飾です）。文言はすべて独自に書いた架空のもので
あり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// リンク操作版の遷移先（外部の実在 URL、`href="#"` は使わない、他 block と
/// 同型の判断。モジュール doc「リンク操作のラベルと遷移先」参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 3 インスタンスが共有するカード骨格（見出し・説明・主操作を縦に積む）。
/// `variant` でカードの見せ方（影付き/面色のみ）を切り替え、`action` に
/// 呼び出し側が組み立てたボタン or リンクを渡す。
fn panel(
    variant: CardVariant,
    instance: &'static str,
    title: &str,
    description: &str,
    action: Node,
) -> Node {
    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-action-panel-stacked-panel", instance)],
        vec![
            card::header(
                vec![],
                vec![heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Lg,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![text(title)],
                )],
            ),
            card::body(
                vec![("class", "blocks-action-panel-stacked-body")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(description)],
                )],
            ),
            card::footer(
                vec![("class", "blocks-action-panel-stacked-footer")],
                vec![action],
            ),
        ],
    )
}

/// (1) 主参照: ボタン操作・影付きカード。
fn button_elevated_panel() -> Node {
    panel(
        CardVariant::Elevated,
        "button-elevated",
        "通知設定を見直す",
        "メール・アプリ内通知の頻度をまとめて見直せます。設定は後からいつでも変更できます。",
        button(
            &ButtonProps::default(),
            vec![("data-blocks-action-panel-stacked-action", "")],
            vec![text("設定を見直す")],
        ),
    )
}

/// (2) 矢印付きリンク操作・影付きカード。
fn link_elevated_panel() -> Node {
    let action = link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-action-panel-stacked-action", "")],
        vec![
            text("GitHub で見る"),
            span(vec![("aria-hidden", "true")], vec![text(" \u{2192}")]),
        ],
    );
    panel(
        CardVariant::Elevated,
        "link-elevated",
        "リポジトリを見る",
        "ソースコードや Issue をリポジトリでまとめて確認できます。",
        action,
    )
}

/// (3) ボタン操作・面色のみの枠（影なし）。
fn button_subtle_panel() -> Node {
    panel(
        CardVariant::Subtle,
        "button-subtle",
        "ワークスペースをアーカイブする",
        "アーカイブ後もデータは保持され、必要になれば復元できます。",
        button(
            &ButtonProps::default(),
            vec![("data-blocks-action-panel-stacked-action", "")],
            vec![text("アーカイブする")],
        ),
    )
}

/// `action-panel-stacked` の Demo 本体（3 インスタンスを縦に並べる。
/// [`LAYOUT_CSS`] 参照）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-action-panel-stacked-layout")],
        vec![
            button_elevated_panel(),
            link_elevated_panel(),
            button_subtle_panel(),
        ],
    )
}
```

## 原案差分メモ

- 主参照はボタン操作・影付きカード（`button-elevated`）です。矢印付き
  リンク操作版（`link-elevated`）と、面色のみの枠（影なし、`well` 相当）
  のボタン操作版（`button-subtle`）を別インスタンスとして併記しています。
- カード variant は `CardVariant::Elevated`（影付き）と `CardVariant::Subtle`
  （面色のみ、影なし）を使い分けています。境界線のみの `Outline` は
  「面色のみの枠」の要件と一致しないため使用していません。
- 参照元の画像・原稿（`_/blocks-intake/`）はこの worktree に存在せず未読の
  ため、Issue のレイアウト仕様文のみを根拠に実装しています。
- ブラウザでの実機確認（狭幅時の縦積み維持・主操作の内容幅固定、ライト/
  ダーク両テーマ）はサンドボックス制約により未実施です。cargo test による
  出力検証のみで代替しました。
