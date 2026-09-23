//! カウンター部品（`Counter`、イシュー #2655、Phase 7「Data display」の
//! 2 番目の部品）。
//!
//! 件数を収めた小さな丸（ピル）バッジで、通知件数・在庫数等の
//! プレースホルダー表示に使う非インタラクティブなローファイ部品。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::counter` showcase
//! （`/wireframes/counter/`）から呼ばれる。件数は
//! [`fandhe_frontend_core::text`] のみで流し込むため、既定エスケープ
//! （REQ-1）は本モジュールが独自に保証する必要はなく core 側の `render()`
//! 契約に委譲される。
//!
//! # API 設計の由来（原案からの差分）
//!
//! blocks.pm の Counter 部品の Figma プロパティ構成をそのまま転写した
//! ものではなく、`docs/design/wireframe-ui-architecture.md` §4・§6 の
//! 汎用変換規約から独自に設計した（ライセンス保留、同文書 §2）。件数は
//! `u32` ではなく `&str` で受ける。呼び出し側が `"99+"` のような省略
//! 表記を選べるようにするためで、数値の整形（桁丸め等）は
//! `docs/policy/intentional-non-adoption.md` §3.25 が述べる「UI
//! コンポーネント層の責務外」の判断軸をそのまま踏襲する。
//!
//! 配色は原案の黒塗り二値を強制せず、読みやすさ優先のグレースケール
//! （[`crate::annotation::ANNOTATION_CSS`] と同じ判断軸、
//! `docs/design/wireframe-ui-architecture.md` §3）を既定とし、黒塗り
//! （反転配色）は [`crate::props::Primary`] の opt-in へ回した。
//!
//! `crate::nav_item` が持つ内部カウンターパート（`fw-wire-nav-item-counter`）
//! とは独立した部品である。`nav_item` は「行レイアウトと一体であるため
//! 内部パートとした」というモジュール doc 上の根拠を持ち、本部品への
//! リファクタは行わない（スコープ外事項として PR 本文に記録する）。

use fandhe_frontend_core::{el_owned, text, Node};

use crate::class::class_list;
use crate::props::Primary;
use crate::size::Size;

/// カウンター CSS（ルート・`Primary` 反転の 2 セレクタ）。
/// [`crate::css::PARTS`] へ登録される。
///
/// `min-width`/`height` を等しくすることで 1 桁の件数は真円になり、
/// 桁数が増えると `padding` 分だけ幅が伸びてピル形状になる。フォント
/// サイズは [`crate::size::css`] が定義する `--fw-wire-font-size` を
/// `var()` で参照するのみで、`size::SCALE` の値そのものはここへ書き
/// 写さない（`docs/design/wireframe-ui-architecture.md` §10）。
pub const COUNTER_CSS: &str = "\
.fw-wire-counter {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 1.5em;
  height: 1.5em;
  padding: 0 0.4em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  font-variant-numeric: tabular-nums;
  line-height: 1;
  white-space: nowrap;
}
.fw-wire-counter.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
";

/// カウンターを組み立てる。
///
/// - `count`: 件数文言。`&str` で受けるため `"3"`・`"42"`・`"99+"`
///   のような呼び出し側の任意表記をそのまま流し込める。空文字列を渡すと
///   中身のない丸（ドット状のバッジ）として描画される。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、フォントサイズは [`crate::size::css`] が定義する
///   `--fw-wire-font-size` を `var()` で参照する。
/// - `primary`: `true` のとき部品固有ではなく共通型
///   [`crate::props::Primary`] による `fw-wire-primary` class を付与し、
///   反転配色（`--fw-wire-ink` 背景 + `--fw-wire-paper` 文字）にする。
///
/// ルート要素は `span`。子はテキストノード 1 つのみで、パート class は
/// 持たない。`data-*`/`role`/`aria-*`/`tabindex`/`style`/`href`/`on*`/
/// `<button>`/`<a>` は一切出力しない（表示状態の軸を持たない表示専用
/// 部品のため、`docs/design/wireframe-ui-architecture.md` §5/§7）。
/// 対話的な未読数読み上げ等が必要な場合は Themes の Badge を案内する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{counter, Primary, Size};
///
/// let node = counter("3", Size::Md, Primary(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-counter fw-wire-size-md""#));
/// assert!(html.contains(">3<"));
/// assert!(!html.contains("fw-wire-primary"));
///
/// // primary=true で反転配色の class が付く。
/// let primary = counter("99+", Size::Md, Primary(true));
/// assert!(render(&primary).contains("fw-wire-counter fw-wire-size-md fw-wire-primary"));
///
/// // XSS 回帰: count は既定エスケープを経由する。
/// let escaped = counter("<script>alert(1)</script>", Size::Md, Primary(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn counter(count: &str, size: Size, primary: Primary) -> Node {
    let class = class_list("fw-wire-counter", &[Some(size.class()), primary.class()]);

    el_owned(
        "span",
        vec![("class".to_string(), class)],
        vec![text(count)],
    )
}
