//! メディアプレースホルダー部品（`Media`、イシュー #2661、Phase 8
//! 「Media・Data」の 3 番目の部品（`image`・`chart` に続く）。blocks.pm
//! 上の表示名は Placeholder）。
//!
//! 動画／メディア埋め込み領域の配置イメージを示す、16:9 固定・非
//! インタラクティブなローファイ・プレースホルダー。実際のメディア再生は
//! 提供しない（`<video>`/`<iframe>`/`<source>`/`<img>` はいずれも出力
//! しない）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::media` showcase
//! （`/wireframes/media/`）から呼ばれる。子は構築済み `Node`（既定
//! エスケープ済み）で受けるため、REQ-1 の既定エスケープは core の
//! `render()` 契約に委譲される。
//!
//! # API 設計の由来（独自設計、原案 Figma プロパティは書き写さない）
//!
//! `docs/design/wireframe-ui-architecture.md` §2・§7 は blocks.pm の
//! ライセンス保留（Community Free Resource License が derivative work を
//! 禁止）を理由に、原案の Figma プロパティ構成（動画フラグの bool・
//! アイコンサイズ・再生アイコンの差し替え）をそのまま API へ翻案する
//! ことを禁じている。そのため本部品の API は §4・§6 の汎用規約から
//! 独自に設計した（avatar・counter と同じ扱い）。動画か静止画かという
//! 軸は bool ではなく [`Option<Node>`] スロットの差し替えで表す
//! （`Some(icon::image(size))` で静止画のメディア枠、`None` は動画の
//! 既定表示として扱う）。この判断は `site/wireframes/media.md` の
//! 「原案差分メモ」節にも記録する。
//!
//! # `None` のときの既定コンテンツフォールバック（§11.4 からの意図的な逸脱）
//!
//! `crate::icon`（§11.4）の `Option<Node>` アイコンスロット規約は
//! 「`None` ならスロット要素を出力しない」を原則とするが、本部品は
//! `content` が `None` のとき既定の再生グリフ [`crate::icon::play`] を
//! 差し込む。中身のないメディア枠は [`crate::frame`] と見分けがつかず、
//! ワイヤーフレームとして「ここに動画/メディアが入る」という意図を
//! 伝えられないため（[`crate::avatar`] を先例として引く逸脱、§11.4 本文
//! への例外の明文化は #2712 から続く未反映の残課題）。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::icon;
use crate::size::Size;

/// ディスク（再生グリフ等を収める円形パート）の class。
const DISC_CLASS: &str = "fw-wire-media-disc";

/// メディア CSS（ルート・ディスクパート・内部グリフサイズ調整の 3
/// セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 黒塗り二値ではなくグレースケールのトークンで塗る
/// （`docs/design/wireframe-ui-architecture.md` §3）。枠は 16:9 固定・
/// 親幅いっぱいに広がり、寸法は [`crate::size::css`] が定義する
/// `--fw-wire-control-size` を `var()` で参照するのみで `size::SCALE` の
/// 値そのものはここへ書き写さない（同文書 §10）。`@keyframes`/
/// `animation` は持たない（表示専用）。
///
/// # `content` スロットの `Size` から独立させるための専用変数
///
/// ルート `.fw-wire-media` は自身の `fw-wire-size-<段階>`（`size` 引数）
/// が定める `--fw-wire-control-size` の値を、ルート自身の上で
/// `--fw-wire-media-control-size` として一度だけ確定させる。ディスク・
/// グリフは以後この専用変数のみを参照し、`--fw-wire-control-size` を
/// 直接は読まない。これは `content` に `icon::image(size)` のような
/// 別 [`Size`] のアイコンを渡した場合の対策である: アイコン自身の
/// `fw-wire-size-<段階>` class は `--fw-wire-control-size` を
/// **アイコン要素自身の上で** 再定義するため、もしグリフ側が
/// `--fw-wire-control-size` を直接参照すると、CSS カスタムプロパティの
/// 解決規則（同一要素上の宣言が祖先からの継承値より優先される）により
/// `content` 側の `Size` がルートの `size` 引数を上書きしてしまう
/// （codex-review PR #2718 指摘対応、rustdoc「size は中央のディスク・
/// グリフの大きさに効く」契約を守るための配線）。`--fw-wire-media-control-size`
/// はルート要素上でのみ確定するため、子要素側の `--fw-wire-control-size`
/// 再定義の影響を受けない。
pub const MEDIA_CSS: &str = "\
.fw-wire-media {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 100%;
  aspect-ratio: 16 / 9;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  overflow: hidden;
  --fw-wire-media-control-size: var(--fw-wire-control-size, 2rem);
}
.fw-wire-media .fw-wire-media-disc {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: calc(var(--fw-wire-media-control-size) * 1.5);
  height: calc(var(--fw-wire-media-control-size) * 1.5);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-media .fw-wire-icon-glyph {
  font-size: calc(var(--fw-wire-media-control-size) * 0.6);
}
";

/// メディアプレースホルダーを組み立てる。
///
/// - `content`: 省略可能なコンテンツスロット。`None` のときは既定の
///   再生グリフ [`crate::icon::play`] を差し込む（§11.4 からの意図的な
///   逸脱、モジュール doc 参照）。`Some(node)` を渡した場合はそのノード
///   をそのまま子要素にする（例: [`crate::icon::image`] で静止画の
///   メディア枠を表す）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、中央のディスク・グリフの大きさにのみ効く（枠自体は親の幅
///   いっぱいに広がる）。`content` に渡した `Node`（例:
///   [`crate::icon::image`]）が別の [`Size`] を持っていても、ディスク・
///   グリフの大きさはこの `size` 引数の値のみに従う（[`MEDIA_CSS`] の
///   専用変数 `--fw-wire-media-control-size` による配線、同定数の
///   rustdoc 参照）。
///
/// ルート要素は `div`。本部品自身が組み立てるマークアップ（ルート `div`・
/// ディスクパート・`content: None` 時の既定グリフ）は `role`/`aria-*`
/// （既定グリフの `aria-hidden` を除く）/`tabindex`/`style`/`href`/`src`/
/// `poster`/`controls`/`on*`/`<video>`/`<iframe>`/`<source>`/`<img>`/
/// `<a>`/`<button>`/表示状態の `data-*` を一切出力しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。メディア URL を
/// 受け取る引数も持たない（外部リソースを読み込む経路を作らない）。
///
/// # `content` スロットに渡した `Node` の内容は呼び出し側の責務
///
/// `content` は §11.4 の `Node` スロット規約に従う任意の [`Node`] を
/// 受け取り、本部品はその内容を検査・サニタイズしない（`crate::avatar`・
/// `crate::link`・`crate::tag` 等、同規約に従う他の全スロット部品と同じ
/// 扱い）。呼び出し側が [`fandhe_frontend_core::el`]/`el_owned` で
/// `<button>`/`<a href>`/`<video>` 等の対話的タグを直接組み立てて渡せば、
/// そのタグ名はそのまま出力される（core はタグ名を構文的に検証するのみで
/// 語彙を `div`/`span` 等へ制限しない）。一方、`on*` イベントハンドラ
/// 属性は content の出所によらず core の `render()` が構造的に出力を
/// 拒否する（[`fandhe_frontend_core::is_event_handler_attr`]、不変条件 9。
/// `crate::media` を含むどの部品も個別に対策する必要はない）。この crate
/// は SSR 専用・非対話プレースホルダーとして自部品の出力を設計する
/// ものであり、呼び出し側が意図的に対話要素を注入する誤用までは型で
/// 防がない（他のスロット部品と共通の既知の限界）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::{icon, media, Size};
///
/// // 既定（content: None）は再生グリフにフォールバックする。
/// let node = media(None, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-media fw-wire-size-md""#));
/// assert!(html.contains(r#"data-icon="play""#));
/// assert!(html.contains("fw-wire-media-disc"));
///
/// // スロット差し替え（静止画アイコン）。
/// let with_image = media(Some(icon::image(Size::Md)), Size::Md);
/// let with_image_html = render(&with_image);
/// assert!(with_image_html.contains(r#"data-icon="image""#));
/// assert!(!with_image_html.contains(r#"data-icon="play""#));
///
/// // スロット差し替え（テキスト）。
/// let with_text = media(Some(text("REC")), Size::Md);
/// let with_text_html = render(&with_text);
/// assert!(!with_text_html.contains("<svg"));
/// assert!(with_text_html.contains("REC"));
///
/// // XSS 回帰: スロットのテキストは既定エスケープを経由する。
/// let escaped = media(Some(text("<script>alert(1)</script>")), Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // content に渡すアイコンの Size はルートの size 引数と独立している
/// // （両者のマークアップ上の class は各要素にそのまま残るが、CSS 側は
/// // `--fw-wire-media-control-size` が size（ここでは Xl）の値だけを
/// // ディスク・グリフへ伝える。MEDIA_CSS rustdoc 参照）。
/// let mismatched = media(Some(icon::image(Size::Xs)), Size::Xl);
/// let mismatched_html = render(&mismatched);
/// assert!(mismatched_html.contains(r#"class="fw-wire-media fw-wire-size-xl""#));
/// assert!(mismatched_html.contains("fw-wire-size-xs"));
/// ```
#[must_use]
pub fn media(content: Option<Node>, size: Size) -> Node {
    let class = class_list("fw-wire-media", &[Some(size.class())]);

    let glyph = content.unwrap_or_else(|| icon::play(size));
    let disc = el_owned(
        "div",
        vec![("class".to_string(), DISC_CLASS.to_string())],
        vec![glyph],
    );

    el_owned("div", vec![("class".to_string(), class)], vec![disc])
}
