# error-page-centered

`empty-state` / `heading` / `text` / `link` の 4 部品を合成した、
中央寄せの 404 ページです。上から順に「エラーコードの小ラベル → 大見出し
→ 説明文」を縦に積み、その下に「ホームへ戻るリンク」と「サポートへの
テキストリンク」を横に並べます。狭い幅でも中央寄せを保ち、アクション列は
折り返します。

文言はすべて架空のダミーです。`<form>` は使わず、ホームへ戻る導線・
サポートへの導線はいずれも固定 URL へ遷移する `link::root`（送信・状態
変更は一切行わない静的な表示例）です。当初はホームへ戻る導線を
`button::button`（`href` を持たない `<button type="button">`）としていま
したが、押しても何も起きない dead control になっていたため、実際に
遷移する `link::root` へ置き換えました（イシュー #2837 PR #3212 codex
レビュー是正）。サポートへの導線も、表示文言「Contact support」に対し
遷移先がリポジトリのトップページのままだった食い違いを、GitHub の
Issues ページへ遷移先を変えることで是正しています。

主参照は対応表 ID R1103、集約元は R1103 と R0581 です。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const ISSUES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/issues";

use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

/// `error-page-centered` の Demo 本体（エラーコードの小ラベル → 大見出し →
/// 説明文 → ホームへ戻るリンク + サポートへのリンクの 2 アクション）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let actions = empty_state::actions(
        vec![("class", "blocks-error-page-centered-actions")],
        vec![
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("data-blocks-error-page-centered-home", "")],
                vec![text("Back to home")],
            ),
            link::root(
                ISSUES,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-error-page-centered-support", "")],
                vec![
                    text("Contact support"),
                    span(vec![("aria-hidden", "true")], vec![text(" →")]),
                ],
            ),
        ],
    );

    let content = empty_state::content(
        vec![("class", "blocks-error-page-centered-content")],
        vec![
            styled_text::text(
                &TextProps {
                    weight: TextWeight::Semibold,
                    ..TextProps::default()
                },
                vec![("data-blocks-error-page-centered-code", "")],
                vec![text("404")],
            ),
            empty_state::title(
                vec![],
                vec![heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Xl3,
                        ..HeadingProps::default()
                    },
                    vec![("data-blocks-error-page-centered-title", "")],
                    vec![text("We couldn't find that page")],
                )],
            ),
            empty_state::description(
                vec![],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-error-page-centered-description", "")],
                    vec![text(
                        "The page you're looking for may have been moved or no longer exists.",
                    )],
                )],
            ),
            actions,
        ],
    );

    div(
        vec![("class", "blocks-error-page-centered-root")],
        vec![empty_state::root(
            &EmptyStateProps::default(),
            vec![("data-blocks-error-page-centered-message", "")],
            vec![content],
        )],
    )
}
```

## 原案差分メモ

主参照（対応表 ID R1103）は、小ラベル・見出し・説明の各パートに、主
アクション（ボタン）+ 矢印付きのサポートリンクの 2 アクションを並べる
構成です。集約元 R0581 は同じ小ラベル・見出し・説明の構成に CTA ボタン
1 個だけを置く最小形（`actions` からリンクを外せば同じ形になる、R1103
の部分集合）です。

- 参照元の全画面高さの表示は、Demo 枠内の `min-height`（`22rem`）へ変え
  ました。
- 参照元の主アクションはリンクであり、ホームへ戻る主アクション・サポート
  への導線ともに `link::root` にしました（当初は部品指定〔`button` /
  `link`〕に従いホームへ戻る主アクションを `button::button` にしていまし
  たが、遷移しない dead control になるため codex レビュー是正〔イシュー
  #2837 PR #3212〕で `link::root` へ変更しました）。
- インジケータ（大型装飾グリフ）に相当するスロットは持たせず、
  「小ラベル → 見出し → 説明文」のみで構成しました（要件に無いため）。
- 文言はすべて独自の英語ダミーへ書き直しました。
- 配色・余白は本リポジトリの既存トークン（`--fandhe-*`）に従わせました。
