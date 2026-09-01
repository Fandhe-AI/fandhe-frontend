//! Button（イシュー #830 で icon-only 修飾 variant を追加）の golden CSS
//! テスト。
//!
//! `crates/pre-styled-ui/src/button.rs` は単一 recipe styled 部品として
//! `crates/pre-styled-ui/tests/*_css.rs`（`image_icon_css.rs`・
//! `download_trigger_css.rs` 等）と同型の「CSS 全文をバイト単位で固定する」
//! golden テストを従来持っていなかった。イシュー #830 で icon-only 修飾
//! variant（非公開 `ButtonIcon` 軸）・compound variant 3 件を追加した機会に
//! `button::css()` の golden を新設し、以後の宣言変更（既存 variant の
//! 誤った書き換え・compound variant の意図しない追加削除）を機械的に検知
//! できるようにする（イシュー #830 受け入れ条件 2「golden CSS 再固定」）。

use fandhe_frontend_pre_styled_ui::button;

/// `button::css()` の期待値（バイト完全一致）。
///
/// 出力順は `SlotRecipe::css`（`crates/pre-styled-ui/src/recipe.rs`）の
/// 契約どおり「base → variants（登録順: size → variant → color-palette →
/// icon-only）→ compound variants（登録順: icon-only×size の xs〜xl）→
/// states（登録順: focus-visible → data-disabled。hover のみ
/// `@media (hover: hover)` へ集約され常に末尾）」（イシュー #1448 で
/// focus-visible state を追加）。size variant はイシュー #1449 で
/// `--fandhe-size-control-*` トークン（イシュー #1678 新設）を参照する
/// よう変更し、icon-only は 5 段の均等 padding compound variant を
/// `padding: 0` へ簡約した（`button.rs` モジュール冒頭 rustdoc「size
/// スケール・icon-only・loading」節参照）。**codex-review P1 指摘の是正**:
/// size variant の `height` は固定高さがラベル折り返し・フォント拡大で
/// あふれる問題を避けるため `min-height` へ変更し、icon-only（子ノードが
/// 常にアイコン 1 個で折り返し要因を持たない）のみ `icon×size` の
/// compound variant で確定 `height` を追加し正方形を維持する
/// （`button.rs::recipe()` rustdoc 参照）。
const EXPECTED_CSS: &str = r#"[data-scope="button"][data-part="root"] {
  display: inline-flex;
  box-sizing: border-box;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border-radius: var(--fandhe-radius-md);
  font-family: var(--fandhe-font-font-body);
  cursor: pointer;
  text-decoration: none;
}

[data-scope="button"][data-part="root"] {
  transition-property: background, border-color, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="button"][data-part="root"].fd-button--size-xs {
  min-height: var(--fandhe-size-control-height-xs, 2rem);
  padding: 0 var(--fandhe-size-control-padding-x-xs, 0.625rem);
  font-size: var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs));
}

[data-scope="button"][data-part="root"].fd-button--size-sm {
  min-height: var(--fandhe-size-control-height-sm, 2.25rem);
  padding: 0 var(--fandhe-size-control-padding-x-sm, 0.75rem);
  font-size: var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm));
}

[data-scope="button"][data-part="root"].fd-button--size-md {
  min-height: var(--fandhe-size-control-height-md, 2.5rem);
  padding: 0 var(--fandhe-size-control-padding-x-md, 1rem);
  font-size: var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md));
}

[data-scope="button"][data-part="root"].fd-button--size-lg {
  min-height: var(--fandhe-size-control-height-lg, 2.75rem);
  padding: 0 var(--fandhe-size-control-padding-x-lg, 1.25rem);
  font-size: var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg));
}

[data-scope="button"][data-part="root"].fd-button--size-xl {
  min-height: var(--fandhe-size-control-height-xl, 3rem);
  padding: 0 var(--fandhe-size-control-padding-x-xl, 1.5rem);
  font-size: var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl));
}

[data-scope="button"][data-part="root"].fd-button--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
  border: none;
  --fandhe-hover-bg: var(--fandhe-palette-emphasized);
}

[data-scope="button"][data-part="root"].fd-button--variant-outline {
  background: transparent;
  color: var(--fandhe-palette);
  border: 1px solid var(--fandhe-palette);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="button"][data-part="root"].fd-button--variant-ghost {
  background: transparent;
  color: var(--fandhe-palette);
  border: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="button"][data-part="root"].fd-button--variant-subtle {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
  border: none;
  --fandhe-hover-bg: var(--fandhe-palette-muted);
}

[data-scope="button"][data-part="root"].fd-button--variant-surface {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
  border: 1px solid var(--fandhe-palette-muted);
  --fandhe-hover-bg: var(--fandhe-palette-muted);
}

[data-scope="button"][data-part="root"].fd-button--variant-plain {
  background: transparent;
  color: var(--fandhe-palette-fg-subtle);
  border: none;
  --fandhe-hover-bg: transparent;
}

[data-scope="button"][data-part="root"].fd-button--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--icon-only {
  aspect-ratio: 1 / 1;
  padding: 0;
}

[data-scope="button"][data-part="root"].fd-button--icon-only.fd-button--size-xs {
  height: var(--fandhe-size-control-height-xs, 2rem);
}

[data-scope="button"][data-part="root"].fd-button--icon-only.fd-button--size-sm {
  height: var(--fandhe-size-control-height-sm, 2.25rem);
}

[data-scope="button"][data-part="root"].fd-button--icon-only.fd-button--size-md {
  height: var(--fandhe-size-control-height-md, 2.5rem);
}

[data-scope="button"][data-part="root"].fd-button--icon-only.fd-button--size-lg {
  height: var(--fandhe-size-control-height-lg, 2.75rem);
}

[data-scope="button"][data-part="root"].fd-button--icon-only.fd-button--size-xl {
  height: var(--fandhe-size-control-height-xl, 3rem);
}

[data-scope="button"][data-part="root"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="button"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (hover: hover) {
  [data-scope="button"][data-part="root"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn button_css_matches_golden_byte_for_byte() {
    assert_eq!(
        button::css(),
        EXPECTED_CSS,
        "button::css() の出力が golden と一致しない。意図した宣言変更なら \
         EXPECTED_CSS を更新すること（本ファイル冒頭 rustdoc 参照）"
    );
}

#[test]
fn button_css_is_deterministic() {
    assert_eq!(button::css(), button::css());
}

/// `css` から base ブロック（`[data-scope="button"][data-part="root"] {`
/// で始まり、宣言群を持つ最初のブロック。同一セレクタは transition-only
/// ブロック（`transition-property` 等のみを持つ 2 つ目のブロック）としても
/// 再出現するため、単純な `str::contains` では base 以外の場所へ宣言が
/// 移動していても検知できない。この関数は「セレクタ行に続く最初の
/// `{`〜`}` ブロック」を厳密に 1 つだけ切り出す）。
fn extract_button_root_base_block(css: &str) -> &str {
    const SELECTOR: &str = "[data-scope=\"button\"][data-part=\"root\"] {\n";
    let start = css
        .find(SELECTOR)
        .expect("button root セレクタが CSS 内に見つからない");
    let body_start = start + SELECTOR.len();
    let body_end = css[body_start..]
        .find("\n}")
        .expect("button root base ブロックの終端 `}` が見つからない");
    &css[body_start..body_start + body_end]
}

/// イシュー #1756: golden（バイト完全一致）とは独立に、`box-sizing:
/// border-box` が button root の **base ブロック内**に存在することを
/// 意味的に固定する回帰テスト。
///
/// `box-sizing: border-box` の下では `border`/`padding` が `height`/
/// `min-height` の内側に含まれるため、Outline variant（`border: 1px
/// solid`）と Solid variant（`border: none`）の外寸（描画高さ）が一致する
/// （`content-box` のままだと Outline のみ border 分〔上下合計 2px〕外寸が
/// 大きくなる不具合があった。是正の記録は `button.rs` モジュール冒頭
/// rustdoc「Outline / Solid の高さ一致」節参照）。**codex-review P2
/// 指摘の是正**: 当初は `css.contains("box-sizing: border-box;")` で
/// CSS 全文のどこか 1 箇所に宣言があれば成功していたため、base ブロックから
/// 宣言が削除・別の variant/state ブロックへ移動しても、他の箇所（例えば
/// 別 selector）に同一文字列が残っていれば検知できなかった。
/// `extract_button_root_base_block` で base ブロックのみを切り出し、
/// その断片に対して assert することで、base ブロックそのものからの
/// 宣言消失・移動を確実に検知する（`radio_group.rs` の
/// `item_control_has_border_box_sizing` と同型の意図を、対象ブロック限定
/// まで強化したもの）。golden テスト
/// （`button_css_matches_golden_byte_for_byte`）が将来 base ブロックの
/// 宣言順・周辺装飾を変更しても、この不変条件だけは独立に検知できる。
#[test]
fn outline_and_solid_variants_share_total_height_via_border_box() {
    let css = button::css();
    let base_block = extract_button_root_base_block(&css);
    assert!(
        base_block.contains("box-sizing: border-box;"),
        "button root の base ブロックに box-sizing: border-box が無いと、\
         UA 既定の content-box 下で Outline variant の border 1px が \
         height の外側へ積み増され、Solid variant と描画高さがずれる \
         （イシュー #1756）。実際の base ブロック: {base_block:?}"
    );
}

/// `extract_button_root_base_block` 自体の健全性を確認する回帰テスト。
///
/// base ブロックには transition-only ブロックが持つ `transition-property`
/// 宣言が含まれないこと（＝先頭の base ブロックを正しく切り出せていて、
/// 2 つ目の transition-only ブロックまで読み進めていないこと）を確認する。
#[test]
fn extract_button_root_base_block_excludes_transition_only_block() {
    let css = button::css();
    let base_block = extract_button_root_base_block(&css);
    assert!(
        !base_block.contains("transition-property"),
        "extract_button_root_base_block が transition-only ブロックまで \
         取り込んでしまっている（base ブロックの終端検出が壊れている）: \
         {base_block:?}"
    );
    assert!(
        base_block.contains("display: inline-flex;"),
        "extract_button_root_base_block が base ブロックの先頭宣言を \
         含んでいない: {base_block:?}"
    );
}
