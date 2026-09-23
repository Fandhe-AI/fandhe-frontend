//! ステッパー部品（`Stepper`、イシュー #2634、Phase 4「Forms B」）。
//!
//! 複数ステップのうち現在どこにいるかを示す、番号付きステップの横並び
//! 進捗表示だけを示す、非インタラクティブなローファイ・プレースホルダー。
//! blocks.pm に対応部品は存在せず、wireframe-ui 独自追加部品である
//! （`docs/design/wireframe-ui-architecture.md` §8 の「独自追加部品」
//! 区分、`site/wireframes/stepper.md` の「原案差分メモ」節も参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::stepper` showcase
//! （`/wireframes/stepper/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # 状態写像
//!
//! - `i < active`（完了）: `data-complete=""` を付与する。`data-active`/
//!   `data-disabled` とは異なり [`crate::props`] に型を新設せず、本
//!   モジュールが `fandhe_frontend_core::attr_if` を直接呼んで付与する
//!   （headless-ui の `steps`/`questionnaire` が使う「complete」語彙に
//!   合わせた命名。`props.rs` は Phase 4 の兄弟イシューと並行して触る
//!   共有ファイルのため、新規表示状態はここでは追加せず部品ローカルに
//!   留める）。
//! - `i == active`（現在）: 既存共通型 [`crate::props::Active`] を再利用し
//!   `data-active=""` を付与する（radio/checkbox/switch と同型の再利用）。
//! - `i > active`（未着手）: 属性を付与しない。
//!
//! `active` がステップ数以上（`active >= steps.len()`）のときは「全ステップ
//! 完了・現在ステップなし」と解釈する。`usize` の範囲外値でもパニックせず、
//! `active` を `steps.len()` へ clamp しない（比較のみで完結するため）。
//!
//! # 資源有界化
//!
//! 出力ノード数は `steps` の長さに線形で、1 つの引数から増幅する経路が
//! ないため（[`crate::textarea`] の `MAX_ROWS`・[`crate::grid`] の
//! `MAX_COLUMNS` と異なり）上限定数は設けない。
//!
//! # 番号は常に表示する（チェックアイコン差し替えを採らない）
//!
//! 完了ステップも [`crate::checkbox`] のようにグリフへ差し替えず、常に
//! 番号（`i + 1`）を表示する。アイコン差し替えは `aria-hidden` の付与を
//! 要し非対話テストの単純さを崩すため、この部品では見送った
//! （`site/wireframes/stepper.md` 参照）。
//!
//! # マークアップは `div`/`span` のみ
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `ol`/`li`（既存 19 部品で前例がない）・`role`/`aria-*`/`tabindex`/
//! `style`/`on*` は一切出力しない。ステップ間の連結線は CSS の `::before`
//! 疑似要素のみで描き、余分なノードを出さない。実際に操作可能なステップ
//! 表示が必要な利用者には Themes（`/themes/steps.md` 相当）/
//! Primitives（`/primitives/steps.md` 相当）を案内する
//! （`site/wireframes/stepper.md` 参照）。

use fandhe_frontend_core::{attr_if, el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::Active;
use crate::size::Size;

/// 各ステップのパート class（部品ルートなしで単独使用しない、[`stepper`] 専用）。
const STEP_CLASS: &str = "fw-wire-stepper-step";
/// 番号のパート class（部品ルートなしで単独使用しない、[`stepper`] 専用）。
const INDICATOR_CLASS: &str = "fw-wire-stepper-indicator";
/// ラベルのパート class（部品ルートなしで単独使用しない、[`stepper`] 専用）。
const LABEL_CLASS: &str = "fw-wire-stepper-label";

/// ステッパー CSS。[`crate::css::PARTS`] へ登録される。
///
/// 値は [`crate::size::css`] が定義するカスタムプロパティと
/// [`crate::tokens`] のトークン（`--fw-wire-*`）を `var()` で参照するのみ
/// で書き写さない。連結線は `::before` 疑似要素のみで描き、追加ノードを
/// 持たない（先頭ステップには `::before` を出さない）。
///
/// 連結線（`::before`）の完了色は「自身が `data-complete`」だけでなく
/// 「自身が `data-active`」でも点灯させる。`data-active` は「直前までの
/// 全ステップが完了済み」を意味するため、現在ステップの直前セグメントも
/// 完了扱いにしないと進捗線が実際の進捗より 1 区間遅れて見える不具合になる
/// （例: 3 ステップ中 `active=1` の既定デモで、ステップ 0 は完了済みなのに
/// ステップ 1 直前の連結線が未完了色のまま残る）。
///
/// 連結線の位置・長さは `right`/`width` を自身の padding box に対する
/// 割合で算出する（`:not(:first-child)::before` は自身の padding box 幅
/// `calc(100% - control-size)` だけから前ステップの中心までの距離を
/// 導出する）。この式は「全ステップの padding box 幅が等しく、かつ
/// インジケータが各 padding box 内で対称に中央寄せされている」ことを
/// 暗黙に前提としている。いずれかのステップだけ左右の
/// `padding-inline-*` を非対称にすると、当該ステップ自身の
/// `align-items: center` によるインジケータ中心が padding box の中心
/// からずれ、この前提が崩れる。ずれは非対称にした当該ステップ自身の
/// 連結線だけでなく、そのステップを「前ステップ」として参照する
/// **次のステップ**の連結線にも波及する（先頭ステップにのみ
/// `padding-inline-start: 0` を与えていた過去の実装は、先頭ステップ
/// 自身が `::before` を持たないことだけを見て「無害」と判断しており、
/// 次ステップの連結線がこの前提に依存していることを見落としていた）。
/// そのため全ステップの `padding-inline-*` は対称のまま揃え、個別
/// ステップの `padding-inline-*` を 0 にする特別扱いは行わない。
pub const STEPPER_CSS: &str = "\
.fw-wire-stepper {
  display: flex;
  align-items: flex-start;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-stepper-step {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.375em;
  flex: 1;
  min-width: 0;
  padding-inline-start: 0.5em;
  padding-inline-end: 0.5em;
}
.fw-wire-stepper-step:not(:first-child)::before {
  content: \"\";
  position: absolute;
  top: calc(var(--fw-wire-control-size, 2rem) / 2);
  right: calc(50% + var(--fw-wire-control-size, 2rem) / 2);
  width: calc(100% - var(--fw-wire-control-size, 2rem));
  height: var(--fw-wire-line-width);
  background: var(--fw-wire-line-subtle);
}
.fw-wire-stepper-step[data-complete]:not(:first-child)::before,
.fw-wire-stepper-step[data-active]:not(:first-child)::before {
  background: var(--fw-wire-line);
}
.fw-wire-stepper-indicator {
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--fw-wire-control-size, 2rem);
  height: var(--fw-wire-control-size, 2rem);
  border-radius: 999px;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-weight: 600;
  flex-shrink: 0;
}
.fw-wire-stepper-step[data-complete] .fw-wire-stepper-indicator {
  background: var(--fw-wire-fill);
  border-color: var(--fw-wire-line);
}
.fw-wire-stepper-step[data-active] .fw-wire-stepper-indicator {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-stepper-label {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stepper-step[data-active] .fw-wire-stepper-label {
  color: var(--fw-wire-ink);
  font-weight: 600;
}
";

/// ステッパーを組み立てる。
///
/// - `steps`: ステップ名のスライス。空スライスのときは子を持たないルート
///   要素のみを出力する（パニックしない）。
/// - `active`: 現在ステップの index（0 始まり）。`active >= steps.len()`
///   （空スライスを含む）のときは「全ステップ完了・現在ステップなし」と
///   解釈する（`i < active` が常に成立するため）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与
///   する。
///
/// 各ステップ名は [`fandhe_frontend_core::text`] のみで流し込み（REQ-1
/// 既定エスケープ）、番号（`i + 1`）も `text()` 経由で出力する。`role`/
/// `aria-*`/`tabindex`/`style`/`on*`/`href` は一切付与しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{stepper, Size};
///
/// let node = stepper(&["アカウント作成", "プラン選択", "支払い"], 1, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-stepper fw-wire-size-md""#));
/// assert_eq!(html.matches("fw-wire-stepper-step\"").count(), 3);
/// assert!(html.contains("アカウント作成"));
/// assert!(html.contains("プラン選択"));
/// assert!(html.contains("支払い"));
/// assert_eq!(html.matches("data-active").count(), 1);
/// assert_eq!(html.matches("data-complete").count(), 1);
///
/// // 範囲外の active はパニックせず「全ステップ完了」として扱う。
/// let all_done = stepper(&["a", "b"], 5, Size::Md);
/// let all_done_html = render(&all_done);
/// assert_eq!(all_done_html.matches("data-complete").count(), 2);
/// assert_eq!(all_done_html.matches("data-active").count(), 0);
///
/// // 空スライスはルート要素のみを出力する。
/// let empty = stepper(&[], 0, Size::Md);
/// let empty_html = render(&empty);
/// assert!(empty_html.contains(r#"class="fw-wire-stepper fw-wire-size-md""#));
/// assert!(!empty_html.contains("fw-wire-stepper-step"));
///
/// // XSS 回帰: ステップ名は既定エスケープを経由する。
/// let escaped = stepper(&["<script>alert(1)</script>"], 0, Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn stepper(steps: &[&str], active: usize, size: Size) -> Node {
    let class = class_list("fw-wire-stepper", &[Some(size.class())]);

    let children: Vec<Node> = steps
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let mut attrs: Vec<(String, String)> =
                vec![("class".to_string(), STEP_CLASS.to_string())];
            if i < active {
                if let Some(attr) = attr_if(true, "data-complete") {
                    attrs.push(attr);
                }
            } else if i == active {
                if let Some(attr) = Active(true).attr() {
                    attrs.push(attr);
                }
            }

            el_owned(
                "div",
                attrs,
                vec![
                    span(
                        vec![("class", INDICATOR_CLASS)],
                        vec![text((i + 1).to_string())],
                    ),
                    span(vec![("class", LABEL_CLASS)], vec![text(*label)]),
                ],
            )
        })
        .collect();

    el_owned("div", vec![("class".to_string(), class)], children)
}
