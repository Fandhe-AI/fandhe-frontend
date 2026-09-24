# banner-announcement-pill

`link` / `icon` / `avatar` を合成した、中央寄せの角丸ピル型告知リンクの
合成例です。ピル全体が 1 個のクリック可能領域で、短い告知文と矢印アイコンを
横並びに表示します。

基準形（ピルだけの告知リンク）と、先頭に重なりアバター群を置いて利用者の
声を示す形の 2 バリエーションを、明暗 2 種の見た目とあわせて Demo に
並べています。`<form>` を持たず、リンク先はプロジェクトのリポジトリです。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先（`docs/design/docs-site-blocks-section.md` §3 と同じ判断で
/// 実在の自リポジトリへの絶対 URL を使う。モジュール doc「`href` に
/// 絶対 URL を使う理由」参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// [`avatar_stack`] が使う架空イニシャル 3 件（[`dummy_assets::PERSON_NAMES`]
/// 先頭 3 名から手書きした定数）。
const AVATAR_INITIALS: [&str; 3] = ["HF", "EV", "KB"];

/// 矢印アイコン（右向き、2 本の `path` からなる自作幾何 SVG）。装飾用途
/// のため `label: None`（既定）のまま渡し `aria-hidden="true"` を保つ。
fn arrow_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![("data-blocks-banner-announcement-pill-arrow", "")],
        vec![
            el(
                "path",
                vec![
                    ("d", "M5 12h14"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "m13 6 6 6-6 6"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 重なりアバター群（社会的証明の装飾。モジュール doc「アバター群を
/// `aria-hidden` にする a11y 判断」参照）。[`dummy_assets::AVATAR_SRC`]
/// （ビルド時生成 SVG、`data:` URI ではない）を画像に使い、
/// [`AVATAR_INITIALS`] を fallback イニシャルに使う。
fn avatar_stack() -> Node {
    let avatars: Vec<Node> = AVATAR_INITIALS
        .iter()
        .map(|initials| {
            avatar::root(
                &AvatarProps {
                    size: Size::Xs,
                    stacked: true,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-banner-announcement-pill-avatar", "")],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                    avatar::fallback(ImageStatus::Loaded, vec![], vec![text(*initials)]),
                ],
            )
        })
        .collect();
    avatar::group(
        vec![
            ("class", "blocks-banner-announcement-pill-avatars"),
            ("aria-hidden", "true"),
        ],
        avatars,
    )
}

/// ピル本体を 1 個組み立てる。`dark` はピル自身の配色反転
/// （`data-blocks-banner-announcement-pill-tone="dark"`）、`with_avatars`
/// は先頭への [`avatar_stack`] の有無（R0407/R0408 の差分）を表す。
fn pill(dark: bool, with_avatars: bool) -> Node {
    let mut attrs = vec![("data-blocks-banner-announcement-pill-pill", "")];
    if dark {
        attrs.push(("data-blocks-banner-announcement-pill-tone", "dark"));
    }

    let mut children: Vec<Node> = Vec::new();
    if with_avatars {
        children.push(avatar_stack());
    }
    let message = if with_avatars {
        "Trusted by thousands of builders"
    } else {
        "Blocks gallery just got a big update"
    };
    children.push(span(
        vec![("class", "blocks-banner-announcement-pill-label")],
        vec![
            span(
                vec![("class", "blocks-banner-announcement-pill-highlight")],
                vec![text("New")],
            ),
            span(
                vec![("class", "blocks-banner-announcement-pill-message")],
                vec![text(message)],
            ),
        ],
    ));
    children.push(arrow_icon());

    link::root(REPO, &LinkProps::default(), attrs, children)
}

/// 1 面分（Light/Dark どちらか一方 × 基準形/アバター付きどちらか一方）を
/// 組み立てる。`caption` は Demo 上で R0407/R0408・明暗の違いを読み取れる
/// ようにするキャプション文言。
fn cell(caption: &'static str, dark: bool, with_avatars: bool) -> Node {
    let mut attrs = vec![("class", "blocks-banner-announcement-pill-cell")];
    if dark {
        attrs.push(("data-blocks-banner-announcement-pill-surface", "dark"));
    }
    div(
        attrs,
        vec![
            span(
                vec![("class", "blocks-banner-announcement-pill-caption")],
                vec![text(caption)],
            ),
            pill(dark, with_avatars),
        ],
    )
}

/// `banner-announcement-pill` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。基準形（R0407）とアバター付き（R0408）を、明暗 2 種の
/// 面として計 4 パターン並べる（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-announcement-pill-grid")],
        vec![
            cell("Light", false, false),
            cell("Dark", true, false),
            cell("Light · with avatars", false, true),
            cell("Dark · with avatars", true, true),
        ],
    )
}
```

## 差分メモ

- 基準形（対応表 ID R0407）とアバター付き（対応表 ID R0408）を、明暗 2 種の
  面として Demo に並べています（差分は各面のキャプションと `with_avatars` の
  有無で読み取れます）。
- 明暗の切り替えはテーマの `dark` トークンではなく、面ごとに
  `--fandhe-color-fg`/`--fandhe-color-bg` を反転させて表現しています（`:root`
  にしか効かないテーマ切り替えでは同一ページ内に 2 面を並べられないため）。
- 参照元の文言・配色・装飾・アイコンは持ち込まず、独自の英語ダミー文言と
  自作の矢印アイコン（幾何 SVG）のみで構成しています。
