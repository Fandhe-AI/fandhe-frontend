//! styled Message（shadcn/ui `Message` 相当。イシュー #2106、親 #2104、
//! 祖父トラッキング参照軸 #2001。headless 側 anatomy は #2105）。
//!
//! `fandhe_frontend_headless_ui::message`（#2105）が出力する
//! `data-scope="message"` の 6 slot（`root`/`avatar`/`header`/`content`/
//! `footer`/`group`）へ、shadcn/ui `Message` の意匠（発言者の役割・整列で
//! 背景色を切り替える会話 1 発言の吹き出し）を重ねる薄い委譲層である。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::item`]/[`crate::command`] と同型。6 パーツすべてを同名再定義
//! し（呼び出し側 `class` の除去は本モジュールの責務のため）、
//! [`MessageRootProps`]/[`MessageRole`]/[`MessageAlign`] の 3 型のみを選択
//! 的に再エクスポートする。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::message`] 自身が状態機械を持た
//! ない静的な自由関数群であるため、本モジュールもその設計をそのまま継承
//! する（[`crate::item`] モジュール doc と同型の判断）。応答待ち・送信
//! 失敗の判定・再送はアプリ責務であり、本モジュールは `data-loading`/
//! `data-error` の見た目のみを切り替える。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! バリデーション・送信処理・Markdown レンダリング等のアプリケーション
//! ロジックは実装しない。headless が出力する `data-*` を CSS セレクタと
//! して参照するだけで見た目を切り替える。
//!
//! # role / align / loading / error の表現: headless の `data-*` を
//! `AttrEq`/`Attr` で参照する（[`crate::item`] と同型の意図的差分）
//!
//! headless `message::root` は `data-role`（`user`/`assistant`/`system`）・
//! `data-align`（`start`/`end`）・`data-loading`/`data-error`（存在属性）を
//! 固定出力済み（`crates/headless-ui/src/message.rs`）。本モジュールは
//! これらを [`StateCondition::AttrEq`]/[`StateCondition::Attr`] で**参照
//! するのみ**とし、class ベースの [`SlotRecipe::variant`] を持たない
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2「役割 B:
//! 参照のみ」）。したがって 6 パーツすべて見た目クラスを付与しない同名
//! 再定義であり、[`crate::item`] と同じパターンを踏襲する。
//!
//! # slot 別の意匠
//!
//! - `root`: 2 カラム grid コンテナ（1 列目 `avatar`、2 列目に `header`/
//!   `content`/`footer` を縦積み）。`avatar` は既定で `grid-row: 1 / span 3`
//!   により 2 列目の 3 パーツ分の行を縦に貫通し、`header`/`content`/
//!   `footer` はいずれも 2 列目へ配置されることで縦積みになる（headless
//!   anatomy が `avatar`/`header`/`content`/`footer` を `root` の直接の
//!   兄弟としてフラットに出力するため、grid の列固定 + 行スパンのみで
//!   「avatar + 縦積み本体」を表現する。ラッパー要素を追加しない制約は
//!   headless 側の anatomy 契約のため変更不可）。`avatar`/`header`/
//!   `footer` はいずれも省略可能なスロットのため、[`stylesheet`] が
//!   `:has()` を使った raw CSS（「raw CSS 追記の理由」節参照）で次の
//!   2 点を補正する: (1) `avatar` が省略された `root` は 2 カラム grid
//!   自体を単一カラムへ縮退させ、`column-gap` が空の 1 列目に対して
//!   常時発生する「アバターなしメッセージの余分なインデント」を防ぐ、
//!   (2) `avatar` の `grid-row` span を実際に存在する
//!   `header`/`footer` の組み合わせ（0/1/2 個）に応じて 1〜3 段へ動的に
//!   切り替え、`header`/`footer` が省略された分だけ確保され続けていた
//!   `row-gap` 分の余白（例: content のみのメッセージで下部に生じる
//!   空行分のギャップ）を防ぐ。`data-align="end"` で `grid-template-
//!   columns` を反転しつつ、[`stylesheet`] が追記する raw CSS
//!   （「raw CSS 追記の理由」節参照）で `avatar`/`header`/`content`/
//!   `footer` の `grid-column` も入れ替え、右寄せレイアウトへ切り替える。
//!   `data-role` の 3 値それぞれで `content` の背景・文字色を custom
//!   property 経由で切り替える（`user` は accent、`assistant` は muted、
//!   `system` は透明 + 斜体 + 控えめ文字色）。`data-loading` で半透明化、
//!   `data-error` で `content` の背景・文字色・枠線を危険色へ切り替える。
//! - `avatar`: 固定サイズの円形スロット（中身は呼び出し側が
//!   `avatar::root`/`avatar::image` 等を自由に組み込む、headless rustdoc
//!   「`avatar` はスロット」参照）。
//! - `header`/`footer`: 小さめ文字・控えめ色の横並びスロット。
//! - `content`: 吹き出し本体（角丸 + パディング + 背景/文字色/枠線を
//!   `root` が定義した custom property 経由で受け取る）。
//! - `group`: 縦積みコンテナ（連続発言のまとめ）。
//!
//! # raw CSS 追記の理由（[`SlotRecipe`] が子結合子を表現できないため）
//!
//! [`SlotRecipe`] はコンポーネント自身の slot にしか宣言を登録できず、
//! `root` の `data-align` に応じて**別の slot**（`avatar`/`header`/
//! `content`/`footer`）の `grid-column` を切り替える宣言や、`group`
//! 配下で 2 件目以降に連続する `root`（および、その `avatar`）を対象に
//! した宣言や、`root` に実際に存在する子スロット（`avatar`/`header`/
//! `footer` は省略可能）に応じて `root` 自身・`avatar` の grid 定義を
//! 切り替える宣言を組めない（[`crate::item`] モジュール doc「raw CSS
//! 追記の理由」と同型の制約）。[`stylesheet`] は `recipe().css()` の
//! 出力へ [`crate::css::serialize_rule`] を使った素の子結合子（`>`）+
//! 属性セレクタ + `:has()`/`:not()` を追記する:
//!
//! - `root[data-align="end"] > avatar`: `grid-column: 2` へ切り替える
//!   （既定は 1 列目）。
//! - `root[data-align="end"] > header`/`> content`/`> footer`:
//!   `grid-column: 1` へ切り替える（既定は 2 列目）。`root` 自身の
//!   `grid-template-columns` 反転（[`recipe`] の `data-align="end"` state）
//!   と対にして、右寄せ時に列の意味を丸ごと入れ替える。
//! - `root:not(:has(> avatar))`: `grid-template-columns` を単一カラム
//!   （`minmax(0, 1fr)`）へ縮退させる。2 カラム grid のまま `avatar` を
//!   省略すると、中身のない 1 列目との境界に `column-gap` が常時発生し
//!   「アバターなしメッセージの余分なインデント」になるため、
//!   `avatar` 不在時は列自体を 1 つにして境界（＝ gap）を消す。この
//!   セレクタは属性 2 つ + `:not(:has(...))`（`:has()` 内の複合セレクタ
//!   と同じ特異度を持つ）で `[data-align="end"]` state（属性 3 つ）より
//!   特異度が高いため、`data-align="end"` と同時に `avatar` が省略され
//!   ても本ルールが優先される（cascade 順に依存しない）。
//! - `root:not(:has(> avatar)) > header`/`> content`/`> footer`:
//!   `grid-column: 1` へ明示的に移す。`header`/`content`/`footer` の
//!   base 宣言は `grid-column: 2` 固定のため、`root` を単一カラムへ
//!   縮退させただけでは列 2 が依然として参照され、単一カラムの grid に
//!   暗黙の列 2（`grid-auto-columns: auto`）が生成されてしまう（列 1 の
//!   `minmax(0, 1fr)` が空のまま幅を持ち続け、`column-gap` も暗黙列との
//!   境界に残ってしまい上記の縮退が実効しない）。3 パーツとも列 1 へ
//!   明示的に移すことで、単一カラム化を実際に列 1 個分へ収束させる。
//! - `root:not(:has(> header)):not(:has(> footer)) > avatar`:
//!   `grid-row: 1 / span 1`（`header`/`footer` とも省略＝`content` のみ
//!   のメッセージ）。
//! - `root:has(> header):not(:has(> footer)) > avatar` /
//!   `root:not(:has(> header)):has(> footer) > avatar`:
//!   `grid-row: 1 / span 2`（`header`/`footer` のどちらか一方のみ存在）。
//!   `header`/`footer` の双方が存在する既定ケースは [`recipe`] の
//!   `avatar_base`（`grid-row: 1 / span 3`）のまま据え置く（3 パーツ全て
//!   存在するときのみ意味を持つ既定値のため、raw CSS での上書きが不要）。
//!   これら 3 ルールがないと、`header`/`footer` が省略されても `avatar`
//!   が常に 3 段分の高さを確保し続け、`row-gap` 分の余白が下部に空行と
//!   して残ってしまう。
//! - `group > root:not(:first-child)`: 連続発言間の余白を詰める
//!   （`margin-top` を負値にして `group` の `gap` と打ち消し合わせる）。
//! - `group > root:not(:first-child) > avatar`: `visibility: hidden` で
//!   2 件目以降の avatar を隠す。`display: none` にすると `avatar` 分の
//!   幅が消えて `content` の横位置が先頭行とずれるため、幅を残す
//!   `visibility: hidden` を採用する（[`crate::item`] の「落とし穴」節と
//!   同じ理由で rustdoc に固定する）。
//!
//! いずれも `serialize_rule` は selector 文字列を検証しないため静的
//! リテラルのみを使う。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::message`] →
//!   [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は [`drop_class_attr`] で除去してから headless
//!   関数へ委譲する（6 パーツすべて）。
//! - [`stylesheet`] が組み立てる CSS 宣言・selector 断片はすべて
//!   コンパイル時静的リテラルであり、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の検証を通る値のみを使う。
//! - `aria-live`/`aria-busy` は付与しない（headless 側の判断を継承、
//!   通知はアプリ責務。headless rustdoc「`aria-live`/`aria-busy` を
//!   付けない理由」参照）。
//!
//! # スコープ外
//!
//! - wasm-full 配線・`examples/headless-pre-styled-ui` への message 追加
//!   （[`crate::item`]/[`crate::command`] と同じ判断）。
//! - 兄弟部品 bubble の語彙追随（#2108）。attachment は #2112、marker は
//!   #2115 で Themes 化済み（いずれも `data-role`/`data-align` を持たない
//!   設計のため語彙追随は不要、`crate::attachment`/`crate::marker`
//!   モジュール doc参照）。
//! - 会話全体のスクロール・`aria-posinset`/`aria-setsize` 等は
//!   message-scroller #2121 のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{SlotRecipe, StateCondition};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// props/role/align 型のみ（`crate::item` と同型の規約）。パーツ関数 6 件
// は呼び出し側 `class` の除去を担うため同名再定義する。
pub use fandhe_frontend_headless_ui::message::{MessageAlign, MessageRole, MessageRootProps};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::message`] の anatomy
/// と 1:1、6 パーツ）。
const SLOTS: &[&str] = &["root", "avatar", "header", "content", "footer", "group"];

/// この styled Message の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let root_base = vec![
        decl("display", "grid"),
        decl("grid-template-columns", "auto minmax(0, 1fr)"),
        decl("column-gap", "var(--fandhe-space-3)"),
        decl("row-gap", "var(--fandhe-space-1)"),
        decl("align-items", "start"),
        decl("max-width", "var(--fandhe-message-max-width, 42rem)"),
        decl("min-width", "0"),
        decl("align-self", "flex-start"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl(
            "--fandhe-message-content-bg",
            "var(--fandhe-color-bg-muted)",
        ),
        decl("--fandhe-message-content-fg", "var(--fandhe-color-fg)"),
        decl("--fandhe-message-content-border", "transparent"),
    ];

    let avatar_base = vec![
        decl("grid-column", "1"),
        decl("grid-row", "1 / span 3"),
        decl("width", "var(--fandhe-space-8)"),
        decl("height", "var(--fandhe-space-8)"),
        decl("border-radius", "var(--fandhe-radius-full)"),
    ];

    let header_base = vec![
        decl("grid-column", "2"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
        decl("display", "flex"),
        decl("gap", "var(--fandhe-space-2)"),
    ];

    let content_base = vec![
        decl("grid-column", "2"),
        decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
        decl("border-radius", "var(--fandhe-radius-lg)"),
        decl("background", "var(--fandhe-message-content-bg)"),
        decl("color", "var(--fandhe-message-content-fg)"),
        decl("border", "1px solid var(--fandhe-message-content-border)"),
        decl("min-width", "0"),
        decl("overflow-wrap", "anywhere"),
    ];

    let footer_base = vec![
        decl("grid-column", "2"),
        decl("display", "flex"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("align-items", "center"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
    ];

    let group_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("list-style", "none"),
        decl("margin", "0"),
        decl("padding", "0"),
    ];

    SlotRecipe::new("message", SLOTS)
        .base("root", root_base)
        .base("avatar", avatar_base)
        .base("header", header_base)
        .base("content", content_base)
        .base("footer", footer_base)
        .base("group", group_base)
        .state(
            "root",
            StateCondition::AttrEq("data-align", "end"),
            vec![
                decl("align-self", "flex-end"),
                decl("grid-template-columns", "minmax(0, 1fr) auto"),
                decl("margin-inline-start", "auto"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-role", "user"),
            vec![decl(
                "--fandhe-message-content-bg",
                "var(--fandhe-color-accent-subtle)",
            )],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-role", "assistant"),
            vec![decl(
                "--fandhe-message-content-bg",
                "var(--fandhe-color-bg-muted)",
            )],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-role", "system"),
            vec![
                decl("--fandhe-message-content-bg", "transparent"),
                decl(
                    "--fandhe-message-content-fg",
                    "var(--fandhe-color-fg-muted)",
                ),
                decl("font-style", "italic"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-loading"),
            vec![decl("opacity", "0.7")],
        )
        .state(
            "root",
            StateCondition::Attr("data-error"),
            vec![
                decl(
                    "--fandhe-message-content-bg",
                    "var(--fandhe-color-danger-subtle)",
                ),
                decl(
                    "--fandhe-message-content-fg",
                    "var(--fandhe-color-danger-fg-subtle)",
                ),
                decl(
                    "--fandhe-message-content-border",
                    "var(--fandhe-color-danger)",
                ),
            ],
        )
}

/// この styled Message が生成する静的 CSS 全量を返す（決定的。
/// [`crate::item::stylesheet`] と同じ契約）。`group` 配下の連続発言に対する
/// raw CSS 追記を含む（モジュール doc「raw CSS 追記の理由」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const GROUP: &str = r#"[data-scope="message"][data-part="group"]"#;
    const ROOT: &str = r#"[data-scope="message"][data-part="root"]"#;
    const ROOT_ALIGN_END: &str = r#"[data-scope="message"][data-part="root"][data-align="end"]"#;
    const AVATAR: &str = r#"[data-scope="message"][data-part="avatar"]"#;
    const HEADER: &str = r#"[data-scope="message"][data-part="header"]"#;
    const CONTENT: &str = r#"[data-scope="message"][data-part="content"]"#;
    const FOOTER: &str = r#"[data-scope="message"][data-part="footer"]"#;

    // `data-align="end"` 時、root の `grid-template-columns` 反転
    // （`recipe` の state）と対にして各パーツの `grid-column` も入れ替える
    // （モジュール doc「raw CSS 追記の理由」節参照。`SlotRecipe::state` は
    // 自分自身の slot の宣言しか登録できず、`root` の属性で別 slot の
    // 宣言を切り替えられないため raw CSS で補う）。
    let align_end_avatar_selector = format!("{ROOT_ALIGN_END} > {AVATAR}");
    if let Some(rule) = serialize_rule(&align_end_avatar_selector, &[decl("grid-column", "2")]) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    for part in [HEADER, CONTENT, FOOTER] {
        let selector = format!("{ROOT_ALIGN_END} > {part}");
        if let Some(rule) = serialize_rule(&selector, &[decl("grid-column", "1")]) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&rule);
        }
    }

    // `avatar` が省略された `root` を単一カラムへ縮退させる（モジュール
    // doc「raw CSS 追記の理由」節参照）。省略時に 2 カラム grid のまま
    // だと中身のない 1 列目との境界へ `column-gap` が常時発生し、
    // アバターなしメッセージ（system 等）で余分なインデントが残る。
    let no_avatar_selector = format!("{ROOT}:not(:has(> {AVATAR}))");
    if let Some(rule) = serialize_rule(
        &no_avatar_selector,
        &[decl("grid-template-columns", "minmax(0, 1fr)")],
    ) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    // `header`/`content`/`footer` の base 宣言は `grid-column: 2` 固定
    // のため、上記で `root` を単一カラムへ縮退させただけでは列 2 が
    // 依然として参照され、単一カラムの grid に暗黙の列 2（`grid-auto-
    // columns: auto`）が生成されてしまう（列 1 の `minmax(0, 1fr)` が
    // 空のまま幅を持ち続け、`column-gap` も暗黙列との境界に残る）。
    // 3 パーツとも列 1 へ明示的に移すことで、単一カラム化を実際に列
    // 1 個分へ収束させる。
    for part in [HEADER, CONTENT, FOOTER] {
        let selector = format!("{ROOT}:not(:has(> {AVATAR})) > {part}");
        if let Some(rule) = serialize_rule(&selector, &[decl("grid-column", "1")]) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&rule);
        }
    }

    // `avatar` の `grid-row` span を実在する `header`/`footer` の組み
    // 合わせに応じて動的に切り替える（モジュール doc「raw CSS 追記の
    // 理由」節参照）。`header`/`footer` とも存在する既定ケースは
    // `avatar_base` の `grid-row: 1 / span 3` のまま据え置くため、ここ
    // では「0 個存在」「片方のみ存在」の 3 パターンのみ上書きする。
    let avatar_row_span_overrides: [(String, &str); 3] = [
        (
            format!("{ROOT}:not(:has(> {HEADER})):not(:has(> {FOOTER})) > {AVATAR}"),
            "1 / span 1",
        ),
        (
            format!("{ROOT}:has(> {HEADER}):not(:has(> {FOOTER})) > {AVATAR}"),
            "1 / span 2",
        ),
        (
            format!("{ROOT}:not(:has(> {HEADER})):has(> {FOOTER}) > {AVATAR}"),
            "1 / span 2",
        ),
    ];
    for (selector, span) in &avatar_row_span_overrides {
        if let Some(rule) = serialize_rule(selector, &[decl("grid-row", span)]) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&rule);
        }
    }

    let consecutive_root_selector = format!("{GROUP} > {ROOT}:not(:first-child)");
    if let Some(rule) = serialize_rule(
        &consecutive_root_selector,
        &[decl("margin-top", "calc(-1 * var(--fandhe-space-1))")],
    ) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    let consecutive_avatar_selector = format!("{GROUP} > {ROOT}:not(:first-child) > {AVATAR}");
    if let Some(rule) = serialize_rule(
        &consecutive_avatar_selector,
        &[decl("visibility", "hidden")],
    ) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール
/// doc「role / align / loading / error の表現」節参照）、呼び出し側
/// `class` を [`drop_class_attr`] で除去してから
/// [`fandhe_frontend_headless_ui::message::root`] へそのまま委譲する。
#[must_use]
pub fn root<'a>(
    props: MessageRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::message::root(props, drop_class_attr(attrs), children)
}

/// styled `avatar` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn avatar<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::message::avatar(drop_class_attr(attrs), children)
}

/// styled `header` パーツを組み立てる。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::message::header(drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::message::content(drop_class_attr(attrs), children)
}

/// styled `footer` パーツを組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::message::footer(drop_class_attr(attrs), children)
}

/// styled `group` パーツを組み立てる。`label` は headless
/// [`fandhe_frontend_headless_ui::message::group`] が既定エスケープを経由
/// して `aria-label` へ出力する（本モジュールは再エスケープしない）。
#[must_use]
pub fn group<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::message::group(label, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="message"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_references_role_align_loading_error_via_attr_not_class() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-align="end"]"#));
        assert!(out.contains(r#"[data-role="user"]"#));
        assert!(out.contains(r#"[data-role="assistant"]"#));
        assert!(out.contains(r#"[data-role="system"]"#));
        assert!(out.contains("[data-loading]"));
        assert!(out.contains("[data-error]"));
        // class ベースの role/align クラス（`fd-message--role-...` 等）は
        // 生成しない（モジュール doc「role / align / loading / error の
        // 表現」節参照）。
        assert!(!out.contains("fd-message--"));
    }

    #[test]
    fn stylesheet_appends_consecutive_group_root_and_avatar_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-scope="message"][data-part="group"] > [data-scope="message"][data-part="root"]:not(:first-child)"#
        ));
        assert!(out.contains(
            r#"[data-scope="message"][data-part="group"] > [data-scope="message"][data-part="root"]:not(:first-child) > [data-scope="message"][data-part="avatar"]"#
        ));
        assert!(out.contains("visibility: hidden;"));
    }

    #[test]
    fn root_connects_to_headless_message_scope() {
        let html = render(&root(MessageRootProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="message" data-part="root""#));
        assert!(html.starts_with("<div"));
    }

    #[test]
    fn all_parts_connect_to_headless_message_scope() {
        let avatar_html = render(&avatar(vec![], vec![]));
        assert!(avatar_html.contains(r#"data-scope="message" data-part="avatar""#));

        let header_html = render(&header(vec![], vec![core_text("You")]));
        assert!(header_html.contains(r#"data-scope="message" data-part="header""#));

        let content_html = render(&content(vec![], vec![core_text("Hi")]));
        assert!(content_html.contains(r#"data-scope="message" data-part="content""#));

        let footer_html = render(&footer(vec![], vec![]));
        assert!(footer_html.contains(r#"data-scope="message" data-part="footer""#));

        let group_html = render(&group("Conversation", vec![], vec![]));
        assert!(group_html.contains(r#"data-scope="message" data-part="group""#));
        assert!(group_html.contains(r#"aria-label="Conversation""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let html = render(&root(
            MessageRootProps::default(),
            vec![("class", "evil")],
            vec![
                avatar(vec![("class", "evil")], vec![]),
                header(vec![("class", "evil")], vec![]),
                content(vec![("class", "evil")], vec![]),
                footer(vec![("class", "evil")], vec![]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }
}
