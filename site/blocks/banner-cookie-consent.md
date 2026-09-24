# banner-cookie-consent

`callout` / `heading` / `text` / `button` / `link` / `icon` を合成した、
Cookie 同意の通知（バナー）の合成例です。新しい部品ではなく、既存 6 部品を
組み合わせただけの静的な表示例です。

Cookie の読み書き・同意状態の保存・送信処理は一切行いません。`<form>` を
持たず、「Decline」「Accept all」ボタンはどちらも `type="button"` のまま
送信先を持ちません。同意の記録・保存は利用者自身のアプリケーションコード
の責務です（`docs/policy/intentional-non-adoption.md` §3.25）。

実際の画面では `position: fixed` + `inset` で画面に固定表示しますが、本
Demo では docs サイトが無 JS の静的ページであるため、枠内の相対配置に
置き換えて表示しています。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps, CalloutVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as body_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 実在の遷移先（`href="#"` を使わないための外部絶対 URL、
/// `footer_newsletter`/`footer_sticky_reveal` と同じ判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の抽象 Cookie アイコン（円の輪郭 + 小さな点 3 個）。実在サービスの
/// アイコン・ロゴは写さず独自に描く。
fn cookie_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![("data-blocks-banner-cookie-consent-icon", "")],
        vec![
            el(
                "circle",
                vec![
                    ("cx", "12"),
                    ("cy", "12"),
                    ("r", "9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "9"),
                    ("cy", "10"),
                    ("r", "1.3"),
                    ("fill", "currentColor"),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "14"),
                    ("cy", "9"),
                    ("r", "1"),
                    ("fill", "currentColor"),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "13"),
                    ("cy", "14"),
                    ("r", "1.2"),
                    ("fill", "currentColor"),
                ],
                vec![],
            ),
        ],
    )
}

/// 1 形態ぶんの Cookie 同意通知本体を組み立てる。`layout` は
/// `"card"`/`"bar"`（[`LAYOUT_CSS`] の属性セレクタ値）、`label` は
/// ランドマークの名前（形態ごとに一意にし、支援技術が複数形態を区別
/// できるようにする）。
fn consent_banner(layout: &'static str, label: &'static str) -> Node {
    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![("data-blocks-banner-cookie-consent-title", "")],
        vec![text("We value your privacy")],
    );

    let description = body_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-banner-cookie-consent-description", "")],
        vec![
            text(
                "We use cookies to improve your experience and remember your \
                 preferences at Northwind Labs. See our ",
            ),
            link::root(
                REPO,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("project repository")],
            ),
            text(" for details."),
        ],
    );

    let actions = div(
        vec![("class", "blocks-banner-cookie-consent-actions")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-banner-cookie-consent-decline", "")],
                vec![text("Decline")],
            ),
            button::button(
                &ButtonProps {
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-banner-cookie-consent-accept", "")],
                vec![text("Accept all")],
            ),
        ],
    );

    section(
        vec![
            ("aria-label", label),
            ("data-blocks-banner-cookie-consent-banner", layout),
        ],
        vec![callout::root(
            &CalloutProps {
                variant: CalloutVariant::Surface,
                size: Size::Md,
                palette: ColorPalette::Neutral,
            },
            vec![("data-blocks-banner-cookie-consent-callout", layout)],
            vec![
                callout::icon(vec![], vec![cookie_icon()]),
                callout::text(
                    vec![("class", "blocks-banner-cookie-consent-body")],
                    vec![title, description, actions],
                ),
            ],
        )],
    )
}

/// 1 形態ぶんの「ビューポート枠」。`align` は [`LAYOUT_CSS`] の
/// `data-blocks-banner-cookie-consent-align` 値、`caption` は形態名。
fn stage(align: &'static str, caption: &'static str, banner: Node) -> Node {
    div(
        vec![
            ("class", "blocks-banner-cookie-consent-stage"),
            ("data-blocks-banner-cookie-consent-align", align),
        ],
        vec![
            p(
                vec![("class", "blocks-banner-cookie-consent-caption")],
                vec![text(caption)],
            ),
            banner,
        ],
    )
}

/// `banner-cookie-consent` の Demo 本体（モジュール doc「4 形態を並べて
/// 見せる」節参照）。呼び出しごとに決定的な `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-cookie-consent-stack")],
        vec![
            stage(
                "end",
                "Bottom-right card",
                consent_banner("card", "Cookie consent (bottom-right card)"),
            ),
            stage(
                "center",
                "Centered card",
                consent_banner("card", "Cookie consent (centered card)"),
            ),
            stage(
                "start",
                "Bottom-left card",
                consent_banner("card", "Cookie consent (bottom-left card)"),
            ),
            stage(
                "bar",
                "Full-width bar",
                consent_banner("bar", "Cookie consent (full-width bar)"),
            ),
        ],
    )
}
```

## 集約元との差分メモ

- 集約元カタログの R0763（画面右下に浮く小さなカード、基準形）・
  R0764（カード中央寄せ）・R0765（カード寄せなし＝左寄せ）・
  R0766（カードではなく全幅の上罫線バー）の 4 形態を、Demo では上から
  「Bottom-right card」「Centered card」「Bottom-left card」
  「Full-width bar」の順にキャプション付きで併記しています。
- 基準形（R0763）の中身の構成（アイコン + 説明 + ポリシーリンク +
  2 ボタン）は R0008 を、「Decline」「Accept all」の 2 ボタン（拒否 → 同意
  の順）は R0406 を兼ねています。4 形態とも中身の構成は共通です。
- カード形態（R0763〜R0765）はパネル幅を最大 24rem に制限した
  `callout` の Surface variant、バー形態（R0766）は幅いっぱい・上罫線のみ
  の `callout` に切り替えています（角丸を 0 にし、上辺以外の枠線を消して
  「バー」の見た目にしています）。
- 幅 640px 未満（sm 未満）では「Decline」「Accept all」ボタンを縦積み・
  全幅表示に切り替えます。
- 本 Demo では実際の固定配置（`position: fixed` + `inset`）の代わりに、
  ビューポート枠内の相対配置（`flex` + `justify-content`）で位置関係だけを
  再現しています。実際に組み込む際は `position: fixed` を使ってください。
- 同意/拒否後に通知を閉じる処理（表示・非表示の切り替え）は本 Demo には
  含まれません。利用者側の Rust コード（wasm-full 等）で実装してください。
