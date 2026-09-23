//! 複数行テキスト入力欄部品（`Textarea`、イシュー #2623、Phase 3「Forms A」）。
//!
//! 画面設計図で「ここに複数行の自由記述欄がある」という配置イメージを
//! 伝えるための、非インタラクティブなローファイ・プレースホルダー。
//! blocks.pm に対応する部品は無く、wireframe-ui 独自追加部品である
//! （`site/wireframes.md` Phase 3 一覧・`docs/design/wireframe-ui-architecture.md`
//! §8 の kebab 集合参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::textarea` showcase
//! （`/wireframes/textarea/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! イシュー本文が挙げる「State」軸は共有型を新設せず、既存の
//! [`crate::props::Active`]（フォーカス中の見た目）と
//! [`crate::props::Disabled`]（無効）の 2 型へ分解した（`props.rs` は
//! 兄弟部品イシューが並行して同じファイルを触る共有ファイルであり、
//! `docs/design/wireframe-ui-architecture.md` §5「表示状態は `data-*` まで」
//! の既存型で表現できるため新設不要と判断した）。
//!
//! `rows: u32` は `style` 属性・ネイティブ `<textarea rows>` を使わず、
//! 行プレースホルダー要素（[`LINE_CLASS`]）を `rows` 個生成する構造表現に
//! した（本クレートは非インタラクティブ部品に `style` 属性・対話要素を
//! 一切出力しない方針、設計文書 §5/§7）。利用者値 1 個から無制限に
//! ノードを増やせないよう `0` は `1` へ丸め、[`MAX_ROWS`] で飽和させる
//! （A05、`crates/wireframe-ui/src/grid.rs` の `columns` clamp と同型の
//! 資源有界化）。詳細は `site/wireframes/textarea.md` の
//! 「原案差分メモ」節を参照。

use fandhe_frontend_core::{div, el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::{Active, Disabled};
use crate::size::Size;

/// `rows` の上限。これを超える値は本値へ、`0` は `1` へ丸める（[`textarea`] 参照）。
pub const MAX_ROWS: u32 = 20;

/// パート class（部品ルートなしで単独使用しない、[`textarea`] 専用）。
const LINE_CLASS: &str = "fw-wire-textarea-line";
const TEXT_CLASS: &str = "fw-wire-textarea-text";

/// 複数行テキスト入力欄 CSS（6 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// リサイズグリップ（右下）は `::after` 擬似要素で描き、ノードは増やさない。
/// `data-active`/`data-disabled` 時の見た目はいずれもグレースケールのみで
/// 表現する（設計文書 §3「黒塗り二値を強制せず読みやすさ優先」の判断軸）。
pub const TEXTAREA_CSS: &str = "\
.fw-wire-textarea {
  display: block;
  position: relative;
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  padding: 0.5em 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.5;
}
.fw-wire-textarea-line {
  min-height: 1.5em;
}
.fw-wire-textarea-text {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.fw-wire-textarea::after {
  content: \"\";
  position: absolute;
  right: 0.2em;
  bottom: 0.2em;
  width: 0.6em;
  height: 0.6em;
  border-right: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
}
.fw-wire-textarea[data-active] {
  border-color: var(--fw-wire-ink);
  box-shadow: 0 0 0 2px var(--fw-wire-fill);
}
.fw-wire-textarea[data-disabled] {
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  border-style: dashed;
}
";

/// 複数行テキスト入力欄プレースホルダーを組み立てる。
///
/// - `text`: 省略可能ではない `&str`。空文字列のときはテキストパート要素
///   自体を出力しない（空要素を残さない、[`crate::annotation::annotation`]
///   の `description: Option<&str>` と同じ判断軸だが、本部品は必須引数
///   として受け取るため空文字列を「テキストなし」の意味に用いる）。
///   非空のときは先頭の行プレースホルダー内へ流し込む。
/// - `rows`: 行数。`1..=`[`MAX_ROWS`] の範囲へ丸める（`0` は `1` へ、
///   [`MAX_ROWS`] 超過は [`MAX_ROWS`] へ）。行プレースホルダー要素
///   （`div.fw-wire-textarea-line`）をこの個数だけ生成する。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与する。
/// - `active`: [`Active`]。`true` のとき `data-active=""` を付与し、
///   フォーカス中の見た目（濃色枠 + リング）にする。
/// - `disabled`: [`Disabled`]。`true` のとき `data-disabled=""` を付与し、
///   無効の見た目（淡色地・破線枠）にする。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style` に加えネイティブ
/// `<textarea>` 要素も一切出力しない（`docs/design/wireframe-ui-architecture.md`
/// §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{textarea, Active, Disabled, Size};
///
/// let node = textarea("本文", 3, Size::Md, Active(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-textarea fw-wire-size-md""#));
/// assert_eq!(html.matches("fw-wire-textarea-line").count(), 3);
/// assert!(html.contains("本文"));
///
/// // 空文字列のときはテキストパート要素自体を出力しない。
/// let empty = textarea("", 2, Size::Md, Active(false), Disabled(false));
/// assert!(!render(&empty).contains("fw-wire-textarea-text"));
///
/// // rows は 1..=20 へ丸める。
/// let clamped_low = textarea("x", 0, Size::Md, Active(false), Disabled(false));
/// assert_eq!(render(&clamped_low).matches("fw-wire-textarea-line").count(), 1);
/// let clamped_high = textarea("x", u32::MAX, Size::Md, Active(false), Disabled(false));
/// assert_eq!(render(&clamped_high).matches("fw-wire-textarea-line").count(), 20);
///
/// // 表示状態は data-active / data-disabled のみで表す。
/// let active = textarea("x", 1, Size::Md, Active(true), Disabled(false));
/// assert!(render(&active).contains(r#"data-active="""#));
///
/// // XSS 回帰: テキストは既定エスケープを経由する。
/// let escaped = textarea("<script>alert(1)</script>", 1, Size::Md, Active(false), Disabled(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn textarea(
    text_value: &str,
    rows: u32,
    size: Size,
    active: Active,
    disabled: Disabled,
) -> Node {
    let rows = rows.clamp(1, MAX_ROWS);
    let class = class_list("fw-wire-textarea", &[Some(size.class())]);

    let attrs: Vec<(String, String)> = vec![("class".to_string(), class)]
        .into_iter()
        .chain(active.attr())
        .chain(disabled.attr())
        .collect();

    let mut lines: Vec<Node> = Vec::with_capacity(rows as usize);
    for index in 0..rows {
        let line_children: Vec<Node> = if index == 0 && !text_value.is_empty() {
            vec![span(vec![("class", TEXT_CLASS)], vec![text(text_value)])]
        } else {
            Vec::new()
        };
        lines.push(div(vec![("class", LINE_CLASS)], line_children));
    }

    el_owned("div", attrs, lines)
}
