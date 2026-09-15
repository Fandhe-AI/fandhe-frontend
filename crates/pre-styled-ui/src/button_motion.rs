//! Button の Motion+ 由来 variant（opt-in、イシュー #2538、親 button の
//! Motion+ 由来 variant）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §9「ライセンス・転記
//! 制限」（判断記録 1: 参照〔閲覧・着想〕は可、転写〔コード片の逐語
//! コピー〕は禁止）に従い、Motion+（購入者限定資料）の意匠から着想した
//! 動作を Rust/CSS で独自に再実装する。§4 の分類に従うと本ファイルの
//! 4 variant は以下の 3 群に分かれる:
//!
//! - **rolling-text** / **rolling-text-stagger**: A 群（CSS のみ、hover 検知
//!   は既存 `:hover` で足りる）。stagger は既存 `recipe::stagger_index_style`
//!   （#2384/#2397）を再利用する。
//! - **hold-to-confirm**: C 群（フレームループ必須）。DOM 配線は
//!   `fandhe-frontend-wasm-full::hold_to_confirm`（#2403/#2517 の
//!   `AnimationLoop`/`RafDriver`/`DomTarget` を消費）が担う。本モジュールは
//!   マークアップ・CSS のみを供給する。
//! - **add-to-basket**: B 群（`data-state` 状態機械 + タイマー、rAF 不要）。
//!   DOM 配線は `fandhe-frontend-wasm-full::add_to_basket` が担う。
//!
//! # `to_css()` 本体を変更しない理由
//!
//! [`crate::motion`] モジュール doc「`motion` feature 配下に置く理由」節と
//! 同じ契約: [`crate::theme::Theme::to_css`] 本体・走査ループは一切変更
//! せず、[`Theme::to_css_with_button_motion`] を別 impl ブロックとして追加
//! し、`to_css()` の出力へ [`BUTTON_MOTION_CSS`] を追記するだけの opt-in
//! メソッドにする（pure append）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! `crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` は `src/` 配下の
//! 各 `.rs` ファイルを文字列走査し、`pub fn` の後ろに `css`/`stylesheet` を
//! 空引数で公開するモジュールを「styled 部品」（`all_styled_component_css`
//! 登録必須）とみなす（cfg 非対応の走査のため feature off でビルド不能に
//! なる）。本モジュールは既存 `button` 部品への追加装飾であり独立部品では
//! ないため、[`crate::motion`] と同じくその関数シグネチャ文字列を doc
//! コメント含め一切書かない。
//!
//! # `data-*` 属性・CSS カスタムプロパティの命名（他クレートとの契約）
//!
//! [`HOLD_TO_CONFIRM_ATTR`]/[`HOLD_DURATION_MS_ATTR`]/
//! [`HOLD_PROGRESS_VAR`]/[`ADD_TO_BASKET_ATTR`] のリテラル値は
//! `fandhe-frontend-wasm-full::hold_to_confirm`/`add_to_basket` の同名定数
//! と一致することが前提（本クレートは `wasm-full` に依存しないため型共有
//! はできず、リテラルの写しとして保持する）。ドリフトは
//! `crates/pre-styled-ui/tests/button_motion_attr_drift.rs` が wasm-full
//! 側ソースを読んで fail-closed に検知する（`content_height_var_drift.rs`
//! と同型のパターン）。
//!
//! # reduced-motion（WCAG 2.3.3）
//!
//! rolling-text の hover トランジションは [`BUTTON_MOTION_CSS`] 自身が
//! `@media (prefers-reduced-motion: reduce)` 内で同名規則を再定義し
//! `transition: none` へ縮退する（[`crate::motion`] の既存方針を踏襲）。
//! hold-to-confirm の進行度は wasm-full 側が毎フレーム JS で書き込む値の
//! ため、塗りつぶし自体は最初から `transition: none`（本ファイル CSS
//! 参照）であり追加の `@media` は不要。add-to-basket のアイコン切替は
//! `display` の即時切替（`transition` を持たない）のため、同様に追加の
//! `@media` は不要（[`BUTTON_MOTION_CSS`] rustdoc 各節参照）。

use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

use crate::button::{button, ButtonProps};
use crate::recipe::stagger_index_style;

/// opt-in（著者が SSR 出力に静的に付与）: hold-to-confirm を有効化する
/// ルート要素マーカー。`fandhe_frontend_wasm_full::hold_to_confirm::
/// HOLD_TO_CONFIRM_ATTR` と値が一致する必要がある（モジュール冒頭「他
/// クレートとの契約」節）。
pub const HOLD_TO_CONFIRM_ATTR: &str = "data-fandhe-hold-to-confirm";
/// opt-in（任意）: 保持時間（ミリ秒）を著者が上書きする属性。
pub const HOLD_DURATION_MS_ATTR: &str = "data-fandhe-hold-duration-ms";
/// 進行度（`0.0..=1.0`）を書き込む CSS カスタムプロパティ名。
pub const HOLD_PROGRESS_VAR: &str = "--fandhe-motion-hold-progress";
/// opt-in（著者が SSR 出力に静的に付与）: add-to-basket を有効化する
/// ルート要素マーカー。
pub const ADD_TO_BASKET_ATTR: &str = "data-fandhe-add-to-basket";

/// rolling-text の current/duplicate 2 層を内包する外側ビューポート
/// レイヤーの `data-part` 値（codex-review P1 是正、下記
/// [`BUTTON_MOTION_CSS`] rustdoc「`rolling-text-viewport` を挟む理由」節
/// 参照）。
const ROLLING_TEXT_VIEWPORT_PART: &str = "rolling-text-viewport";
/// rolling-text の「現在表示中」レイヤーの `data-part` 値。
const ROLLING_TEXT_CURRENT_PART: &str = "rolling-text-current";
/// rolling-text の「複製（`aria-hidden`）」レイヤーの `data-part` 値。
const ROLLING_TEXT_DUPLICATE_PART: &str = "rolling-text-duplicate";
/// rolling-text の「1 文字（または非 stagger 版ではラベル全体）」を包む
/// `<span>` の `data-part` 値（codex-review P1 是正、下記
/// [`BUTTON_MOTION_CSS`] rustdoc「transform/transition を文字 span へ
/// 適用する理由」節参照）。`transform`/`transition`/`transition-delay` は
/// この層（`rolling-text-current`/`rolling-text-duplicate` 自身ではなく
/// その子）へ適用する。
const ROLLING_TEXT_CHAR_PART: &str = "rolling-text-char";
/// hold-to-confirm の塗りつぶしレイヤーの `data-part` 値。
const HOLD_FILL_PART: &str = "hold-fill";
/// add-to-basket の idle アイコンレイヤーの `data-part` 値。
const BASKET_ICON_IDLE_PART: &str = "basket-icon-idle";
/// add-to-basket の adding アイコンレイヤーの `data-part` 値。
const BASKET_ICON_ADDING_PART: &str = "basket-icon-adding";
/// add-to-basket の added アイコンレイヤーの `data-part` 値。
const BASKET_ICON_ADDED_PART: &str = "basket-icon-added";

/// [`crate::button`] 本体への opt-in 追加装飾 CSS（4 variant 分を集約）。
///
/// セレクタは [`crate::recipe::SlotRecipe`] を経由せず
/// `[data-scope="button"][data-part="..."]` を直接手書きする（モジュール
/// 冒頭「`to_css()` 本体を変更しない理由」節参照。既存 `button` の
/// `recipe()`/`css()` 本体は一切変更しない）。
///
/// # 既知の副作用: 全 button への `position: relative; overflow: hidden;`
///
/// 先頭の `[data-part="root"]` 規則は、`Theme::to_css_with_button_motion`
/// を呼び出したテーマ配下の**すべての** `button` 部品（rolling-text/
/// hold-to-confirm/add-to-basket のいずれの variant も使っていないもの
/// を含む）に適用される（この opt-in CSS は button variant 単位ではなく
/// テーマ単位で有効化するため）。hold-to-confirm の塗りつぶし・
/// add-to-basket のアイコン切替のいずれも絶対配置レイヤーを内包の枠から
/// 溢れさせない前提で設計しているため、通常の button に対しても実害は
/// ないと判断した意図的な仕様（レイアウトへの視覚的な影響は position:
/// relative 単体では通常発生せず、overflow: hidden も内容がボックスを
/// 超えない限り無害）。
///
/// # `rolling-text-viewport` を挟む理由（codex-review P1 是正、2 件）
///
/// 当初は `current`/`duplicate` を `root` へ直接の子として置き、`root`
/// 自身の `overflow: hidden` で clip する構成だったが、2 つの不整合が
/// あった:
///
/// 1. **clip 基準のずれ**: `translateY(±100%)` は移動する要素**自身**の
///    高さ基準で計算されるのに対し、`root`（button 全体、padding 込みで
///    ラベルより高い）が clip 基準になっていたため、ラベル分の移動では
///    `root` の外まで出切らず、複製と原本が同時に見えてしまう。
/// 2. **`duplicate` の `display: flex` が文字単位 `<span>` を直接
///    flex item 化する**: [`rolling_text_stagger_button`] は 1 文字ずつ
///    `<span>` に分割するため、空白のみの `<span>` が `duplicate` の
///    flex item として独立した line box を持ち、`white-space: normal`
///    の既定挙動で単語間の空白が消える（"Add to basket" が
///    "Addtobasket" と表示される）。
///
/// 是正として `current`/`duplicate` の外側へ `rolling-text-viewport`
/// （`display: inline-block; position: relative; overflow: hidden;`）を
/// 挟む。`duplicate`（`position: absolute; inset: 0;`）は out-of-flow
/// のためサイズ計算に加わらず、`viewport` の高さは通常フローの唯一の
/// 子である `current`（ラベルそのもの）の内容だけで決まる。結果として
/// `duplicate` の box は `inset: 0` により `current` と**厳密に同じ
/// 寸法**になり、(1) `translateY(±100%)` の基準と clip 基準
/// （`viewport` の境界）が一致し、(2) `duplicate` はもはや文字を
/// 中央寄せするための `display: flex` を必要としない（box が既に
/// `current` と同一寸法のため）ので撤去できる。
///
/// なお `duplicate` の `display: flex` 撤去は上記 2 点のみを解決し、
/// 文字単位 `<span>` 自身の空白折り畳みは別途残る（下記
/// `white-space: pre` 節参照。`display: flex` 撤去だけでは
/// "Add to basket" → "Addtobasket" の崩れは解消しない、codex-review
/// 再指摘・Cursor Bugbot 指摘）。
///
/// # `rolling-text-viewport` の単一行強制（codex-review P1 指摘、複数行
/// 折り返し時の重なり）
///
/// 幅を制限したボタン（例: 固定 `width` + 長いラベル）でラベルが複数行へ
/// 折り返すと、`current`/`duplicate` の各行が**重なって**表示される。
/// 原因: 文字 span の `translateY(±100%)` は各文字**自身**の line box の
/// 高さを基準に計算される（上記「`transform`/`transition` を文字 span へ
/// 適用する理由」節）。単一行では層全体の高さと文字の line box 高さが
/// 一致するため `duplicate` の初期位置（`translateY(100%)`）は
/// `current` の**直下**（≒ 次の行の位置）に来るが、複数行では
/// `duplicate` の 1 行目がちょうど `current` の 2 行目の位置と重なって
/// しまう（`current` 自身も hover 時に 2 行目が 1 行目の位置へ
/// せり上がる）。是正として [`ROLLING_TEXT_VIEWPORT_PART`] へ
/// `white-space: nowrap` を付与し、折り返し自体を禁止して単一行を保証
/// する。既存の `overflow: hidden`（上記「`rolling-text-viewport` を
/// 挟む理由」節）と組み合わさり、幅制限ボタンで単一行に収まらない
/// ラベルは折り返さず超過分が clip される（複数行での重なりより
/// 安全な劣化——`translateY(±100%)` の基準・clip 基準を単一行に固定した
/// 上記節の前提を保つ）。
///
/// # 文字 span の `white-space: pre`（codex-review P1 是正・Cursor Bugbot
/// 指摘、空白文字が消える問題）
///
/// [`char_spans`] は空白文字も他の文字と同様に 1 個の
/// `<span data-part="rolling-text-char" style="display: inline-block">`
/// へ包む。CSS Text の空白折り畳み規則は「行（line box）の先頭・末尾の
/// 空白を除去する」対象を各インライン要素の**内部フォーマッティング
/// コンテキストの境界**にも適用するため、`display: inline-block` な
/// span の内容が空白 1 文字のみだと、その空白は span 自身の内部行の
/// 先頭かつ末尾として扱われ除去される（`duplicate` の `display: flex`
/// の有無や兄弟 span の並びとは無関係に、span 単体で発生する）。この
/// ため "Add to basket" の単語間スペースの span だけが幅 0 になり
/// "Addtobasket" と表示される。是正として文字 span へ
/// `white-space: pre` を付与し、空白の折り畳み自体を無効化して幅を
/// 保持する（`transform`/`transition` の適用対象は変わらないため、
/// stagger の見た目には影響しない）。
///
/// # `transform`/`transition` を文字 span（[`ROLLING_TEXT_CHAR_PART`]）へ
/// 適用する理由（codex-review P1 是正、2 件）
///
/// 当初は `transform`/`transition` を `rolling-text-current`/
/// `rolling-text-duplicate` 自身（層そのもの）へ適用していたが、これは
/// [`rolling_text_stagger_button`] が文字ごとに書き込む
/// `--fandhe-motion-stagger-index` を無視する: 層全体が単一の
/// `transform` で一括して動くため、[`char_spans`] が個々の `<span>` へ
/// 付与した index は見た目に一切反映されず全文字が完全に同時に動いて
/// しまい、`stagger_delay_declaration`（`animation-delay` を生成）も
/// この `transition` ベースの実装には作用しない（同一の
/// `animation`/`transition` 混同は不成立）。是正として `current`/
/// `duplicate` 自身は層の位置決め（`display: block`/
/// `position: absolute; inset: 0;`）のみを担う静止コンテナへ縮小し、
/// `transform`・`transition`・`transition-delay` は各層の**直接の子**
/// である文字 span（[`ROLLING_TEXT_CHAR_PART`]、[`char_spans`] が
/// 複数生成、非 stagger 版の [`rolling_text_button`] は 1 個のみ生成）
/// へ移す。`transition-delay` は
/// `calc(var(--fandhe-motion-stagger-index, 0) * var(--fandhe-motion-
/// duration-fast))`（[`crate::recipe::stagger_delay_declaration`] と同じ
/// 計算式を `transition-delay` プロパティで再現、`STAGGER_INDEX_VAR` 未設定
/// の非 stagger 版は `0` へフォールバックし遅延なし）。層自身が動かなく
/// なったため、`duplicate`（`position: absolute; inset: 0;` のまま）の
/// 基準は不変で `rolling-text-viewport` の clip 契約（上記節）も保たれる
/// ——文字 span 自身の `translateY(±100%)` は span 自身の行の高さ基準
/// になり、単一行ラベルでは層の高さと実質一致するため見た目は従来と
/// 同一だが、stagger 版では文字ごとに独立した遅延で動く。
pub const BUTTON_MOTION_CSS: &str = concat!(
    // rolling-text: root を相対配置・overflow hidden のコンテナ化する
    // （既存副作用、上記「既知の副作用」節）。current/duplicate の 2 層
    // は rolling-text-viewport（ラベルの内容サイズへ shrink-wrap する
    // 内側ビューポート）で包み、hover 時に current が上へ抜け duplicate
    // が下から現れる（タッチ端末の疑似 hover 貼り付き対策として
    // `@media (hover: hover)` で非タッチ限定にする、
    // `pre-styled-ui-interaction-visual-language.md` の既存方針と整合）。
    "[data-scope=\"button\"][data-part=\"root\"] {\n",
    "  position: relative;\n",
    "  overflow: hidden;\n",
    "}\n",
    // viewport は current（通常フローの唯一の子）の内容サイズへ
    // shrink-wrap し、duplicate（out-of-flow）の絶対配置基準・clip 基準
    // を兼ねる（上記「`rolling-text-viewport` を挟む理由」節参照）。
    "[data-scope=\"button\"][data-part=\"",
    "rolling-text-viewport",
    "\"] {\n",
    "  position: relative;\n",
    "  display: inline-block;\n",
    "  overflow: hidden;\n",
    // 折り返し（複数行化）を禁止し常に単一行を保証する（codex-review
    // P1 指摘、下記「単一行を強制する理由」節参照）。`overflow: hidden`
    // と組み合わせることで、幅制限ボタンで折り返す代わりに超過分を
    // clip する（`translateY(±100%)` の基準・clip 基準を単一行に固定
    // した上記「`rolling-text-viewport` を挟む理由」節の前提を保つ）。
    "  white-space: nowrap;\n",
    "}\n",
    "[data-scope=\"button\"][data-part=\"",
    "rolling-text-current",
    "\"] {\n",
    "  display: block;\n",
    "}\n",
    // duplicate は viewport（position: relative）の inset: 0 いっぱいに
    // 絶対配置される。viewport の寸法は current の内容のみで決まるため
    // （duplicate 自身は out-of-flow でサイズ計算に加わらない）、
    // duplicate の box は current と厳密に同じ寸法になり、中央寄せ用の
    // `display: flex` は不要（上記「`rolling-text-viewport` を挟む
    // 理由」節参照）。
    "[data-scope=\"button\"][data-part=\"",
    "rolling-text-duplicate",
    "\"] {\n",
    "  position: absolute;\n",
    "  inset: 0;\n",
    "}\n",
    // 文字 span（current/duplicate それぞれの直接の子）へ transform/
    // transition/transition-delay を適用する（上記「`transform`/
    // `transition` を文字 span へ適用する理由」節参照）。stagger 版は
    // `--fandhe-motion-stagger-index` を span ごとに書き込むため、
    // `transition-delay` がここで文字ごとに異なる値へ解決される。
    "[data-scope=\"button\"][data-part=\"rolling-text-current\"] > [data-part=\"",
    "rolling-text-char",
    "\"],\n",
    "[data-scope=\"button\"][data-part=\"rolling-text-duplicate\"] > [data-part=\"",
    "rolling-text-char",
    "\"] {\n",
    "  display: inline-block;\n",
    "  white-space: pre;\n",
    "  transition: transform 0.3s ease;\n",
    "  transition-delay: calc(var(--fandhe-motion-stagger-index, 0) * var(--fandhe-motion-duration-fast));\n",
    "}\n",
    "[data-scope=\"button\"][data-part=\"rolling-text-duplicate\"] > [data-part=\"",
    "rolling-text-char",
    "\"] {\n",
    "  transform: translateY(100%);\n",
    "}\n",
    "@media (hover: hover) {\n",
    "  [data-scope=\"button\"][data-part=\"root\"]:hover [data-part=\"rolling-text-current\"] > [data-part=\"",
    "rolling-text-char",
    "\"] {\n",
    "    transform: translateY(-100%);\n",
    "  }\n",
    "  [data-scope=\"button\"][data-part=\"root\"]:hover [data-part=\"rolling-text-duplicate\"] > [data-part=\"",
    "rolling-text-char",
    "\"] {\n",
    "    transform: translateY(0);\n",
    "  }\n",
    "}\n",
    "@media (prefers-reduced-motion: reduce) {\n",
    "  [data-scope=\"button\"][data-part=\"rolling-text-current\"] > [data-part=\"",
    "rolling-text-char",
    "\"],\n",
    "  [data-scope=\"button\"][data-part=\"rolling-text-duplicate\"] > [data-part=\"",
    "rolling-text-char",
    "\"] {\n",
    "    transition: none;\n",
    "  }\n",
    "}\n",
    // hold-to-confirm: `hold-fill` は wasm-full が毎フレーム
    // `--fandhe-motion-hold-progress` へ書き込む進行度に追随して幅が
    // 広がる塗りつぶしレイヤー。毎フレーム JS 書き込みで滑らかさを担保
    // するため `transition: none`（CSS 側 transition は干渉要因になる
    // ため付けない、実装計画 §2.1 参照）。
    "[data-scope=\"button\"][data-part=\"",
    "hold-fill",
    "\"] {\n",
    "  position: absolute;\n",
    "  inset: 0;\n",
    "  width: calc(var(--fandhe-motion-hold-progress, 0) * 100%);\n",
    "  background: currentColor;\n",
    "  opacity: 0.2;\n",
    "  pointer-events: none;\n",
    "  transition: none;\n",
    "}\n",
    "[data-scope=\"button\"][data-part=\"root\"][data-state=\"confirmed\"] {\n",
    "  transition: background-color 0.2s ease;\n",
    "}\n",
    // add-to-basket: 3 アイコンレイヤーは既定で非表示、`data-state` に
    // 一致するレイヤーのみ表示する（`display` の即時切替のため
    // transition を持たず、reduced-motion の追加 `@media` は不要）。
    "[data-scope=\"button\"][data-part=\"",
    "basket-icon-adding",
    "\"],\n",
    "[data-scope=\"button\"][data-part=\"",
    "basket-icon-added",
    "\"] {\n",
    "  display: none;\n",
    "}\n",
    "[data-scope=\"button\"][data-part=\"root\"][data-state=\"adding\"] [data-part=\"",
    "basket-icon-idle",
    "\"],\n",
    "[data-scope=\"button\"][data-part=\"root\"][data-state=\"added\"] [data-part=\"",
    "basket-icon-idle",
    "\"] {\n",
    "  display: none;\n",
    "}\n",
    "[data-scope=\"button\"][data-part=\"root\"][data-state=\"adding\"] [data-part=\"",
    "basket-icon-adding",
    "\"] {\n",
    "  display: inline-flex;\n",
    "}\n",
    "[data-scope=\"button\"][data-part=\"root\"][data-state=\"added\"] [data-part=\"",
    "basket-icon-added",
    "\"] {\n",
    "  display: inline-flex;\n",
    "}\n",
);

/// opt-in API。[`crate::theme::Theme::to_css`] の出力へ
/// [`BUTTON_MOTION_CSS`] を追記して返す（pure append、[`crate::motion::
/// Theme::to_css_with_keyframes`] と同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に Button Motion+ variant 用
    /// CSS（[`BUTTON_MOTION_CSS`]）を追記して返す。
    #[must_use]
    pub fn to_css_with_button_motion(&self) -> String {
        let mut out = self.to_css();
        out.push_str(BUTTON_MOTION_CSS);
        out
    }
}

/// 1 文字ずつ `data-part="rolling-text-char"` の
/// `<span style="--fandhe-motion-stagger-index: N">` へ分割する
/// （[`rolling_text_stagger_button`] 専用）。[`BUTTON_MOTION_CSS`] の
/// `transform`/`transition`/`transition-delay` はこの `data-part`
/// （[`ROLLING_TEXT_CHAR_PART`]）を持つ子要素へ適用されるため、ここで
/// 属性を付けないと文字単位の変形が効かない（codex-review P1 是正、
/// 上記 [`BUTTON_MOTION_CSS`] rustdoc「`transform`/`transition` を文字
/// span へ適用する理由」節参照）。
///
/// `str::chars()` は Unicode grapheme cluster（結合文字・ZWJ 絵文字等）を
/// 分割し得るため、多バイト結合文字を含むラベルでは見た目が乱れ得る
/// （既知の限界。新規依存追加はユーザー承認が要るため grapheme cluster
/// 分割クレートは導入しない、実装計画 §2.1 参照）。
fn char_spans(label: &str) -> Vec<Node> {
    label
        .chars()
        .enumerate()
        .map(|(index, ch)| {
            let style = stagger_index_style(index);
            el(
                "span",
                vec![
                    ("data-part", ROLLING_TEXT_CHAR_PART),
                    ("style", style.as_str()),
                ],
                vec![text(ch)],
            )
        })
        .collect()
}

/// [`rolling_text_button`]（非 stagger 版）専用: ラベル全体を 1 個の
/// `data-part="rolling-text-char"` `<span>` で包む。`--fandhe-motion-
/// stagger-index` を書かないため [`BUTTON_MOTION_CSS`] の
/// `transition-delay` 計算式は `var(--fandhe-motion-stagger-index, 0)`
/// フォールバックにより常に `0` へ解決される（[`char_spans`] rustdoc・
/// [`BUTTON_MOTION_CSS`] rustdoc 参照）。
fn whole_label_char_span(label: &str) -> Node {
    el(
        "span",
        vec![("data-part", ROLLING_TEXT_CHAR_PART)],
        vec![text(label)],
    )
}

/// rolling-text ボタン（イシュー #2538）: hover 時にラベルが上へ回転し、
/// 複製ラベルが下から現れる 2 層構成。[`crate::button::button`] の子ノード
/// を `data-part="rolling-text-viewport"` の内側ビューポート 1 個に置き
/// 換え、その中へラベル 2 層（`data-part="rolling-text-current"`/
/// `"rolling-text-duplicate"`）を格納する（[`BUTTON_MOTION_CSS`] rustdoc
/// 「`rolling-text-viewport` を挟む理由」節参照）。
///
/// # アクセシビリティ不変条件
///
/// ラベルは 2 回描画されるが、複製レイヤーは常に `aria-hidden="true"` を
/// 持つため、支援技術に伝わる名前は 1 回のみ（現在表示レイヤー由来）に
/// 保たれる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::button::ButtonProps;
/// use fandhe_frontend_pre_styled_ui::button_motion::rolling_text_button;
///
/// let node = rolling_text_button(&ButtonProps::default(), vec![], "Save");
/// let html = render(&node);
/// assert!(html.contains(r#"data-part="rolling-text-current""#));
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn rolling_text_button<'a>(
    props: &ButtonProps,
    attrs: Vec<(&'a str, &'a str)>,
    label: &'a str,
) -> Node {
    let current = el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", ROLLING_TEXT_CURRENT_PART),
        ],
        vec![whole_label_char_span(label)],
    );
    let duplicate = el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", ROLLING_TEXT_DUPLICATE_PART),
            ("aria-hidden", "true"),
        ],
        vec![whole_label_char_span(label)],
    );
    let viewport = el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", ROLLING_TEXT_VIEWPORT_PART),
        ],
        vec![current, duplicate],
    );
    button(props, attrs, vec![viewport])
}

/// rolling-text ボタンの文字単位 stagger 版（イシュー #2538）。
/// [`rolling_text_button`] と同じ 2 層構成だが、各層のラベルを 1 文字ずつ
/// `<span>` に分割し [`crate::recipe::stagger_index_style`]（#2384）で
/// `--fandhe-motion-stagger-index` を書き込む。実際の遅延宣言（`animation-
/// delay` 計算式）を有効にするには、呼び出し側が
/// [`crate::recipe::SlotRecipe::stagger_delay`] 等で対象アニメーションへ
/// [`crate::recipe::stagger_delay_declaration`] を組み込む必要がある
/// （本関数はマークアップの `style` 属性書き込みのみを担う）。
///
/// [`char_spans`] rustdoc の既知の限界（Unicode grapheme cluster）を継承
/// する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::button::ButtonProps;
/// use fandhe_frontend_pre_styled_ui::button_motion::rolling_text_stagger_button;
///
/// let node = rolling_text_stagger_button(&ButtonProps::default(), vec![], "Go");
/// let html = render(&node);
/// assert!(html.contains("--fandhe-motion-stagger-index: 0"));
/// assert!(html.contains("--fandhe-motion-stagger-index: 1"));
/// ```
#[must_use]
pub fn rolling_text_stagger_button<'a>(
    props: &ButtonProps,
    attrs: Vec<(&'a str, &'a str)>,
    label: &'a str,
) -> Node {
    let current = el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", ROLLING_TEXT_CURRENT_PART),
        ],
        char_spans(label),
    );
    let duplicate = el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", ROLLING_TEXT_DUPLICATE_PART),
            ("aria-hidden", "true"),
        ],
        char_spans(label),
    );
    let viewport = el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", ROLLING_TEXT_VIEWPORT_PART),
        ],
        vec![current, duplicate],
    );
    button(props, attrs, vec![viewport])
}

/// hold-to-confirm ボタン（イシュー #2538）: 一定時間押し続けたときのみ
/// 確定する。ルート要素へ [`HOLD_TO_CONFIRM_ATTR`]（+ 任意で
/// [`HOLD_DURATION_MS_ATTR`]）を付与し、先頭子ノードへ塗りつぶしレイヤー
/// （`data-part="hold-fill"`、`aria-hidden="true"`）を追加する。実際の
/// 保持検知・進行度計算・確定処理は
/// `fandhe_frontend_wasm_full::hold_to_confirm::wire_hold_to_confirm`
/// が担う（本関数はマークアップ・CSS のみ）。
///
/// [`crate::button::button`] は常に `type="button"` を強制する既存契約
/// （`assemble` 内部実装、`fandhe_frontend_wasm_full::hold_to_confirm`
/// モジュール doc「`type="button"` の前提」節が要求する不変条件）を持つ
/// ため、本関数はこれを追加で強制する必要はない。
///
/// `hold_duration_ms` に `Some(ms)` を渡すと保持時間（ミリ秒、10 進数
/// 文字列）を上書きする。`None` なら wasm-full 側の既定値
/// （`DEFAULT_HOLD_DURATION_MS`）が使われる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::button::ButtonProps;
/// use fandhe_frontend_pre_styled_ui::button_motion::hold_to_confirm_button;
///
/// let node = hold_to_confirm_button(
///     &ButtonProps::default(),
///     vec![],
///     None,
///     vec![text("Hold to delete")],
/// );
/// let html = render(&node);
/// assert!(html.contains("data-fandhe-hold-to-confirm"));
/// assert!(html.contains(r#"data-part="hold-fill""#));
/// assert!(html.contains(r#"type="button""#));
/// ```
#[must_use]
pub fn hold_to_confirm_button<'a>(
    props: &ButtonProps,
    attrs: Vec<(&'a str, &'a str)>,
    hold_duration_ms: Option<&'a str>,
    children: Vec<Node>,
) -> Node {
    let mut merged_attrs = attrs;
    merged_attrs.push((HOLD_TO_CONFIRM_ATTR, ""));
    if let Some(ms) = hold_duration_ms {
        merged_attrs.push((HOLD_DURATION_MS_ATTR, ms));
    }
    let fill = el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", HOLD_FILL_PART),
            ("aria-hidden", "true"),
        ],
        vec![],
    );
    let mut node_children = Vec::with_capacity(children.len() + 1);
    node_children.push(fill);
    node_children.extend(children);
    button(props, merged_attrs, node_children)
}

/// add-to-basket ボタン（イシュー #2538）: click で `idle` → `adding` →
/// `added` → `idle` と遷移し、対応するアイコンレイヤーのみを表示する。
/// ラベル文言自体は動的に差し替えない（実装計画 §1「スコープ外・
/// ラベル文言固定」参照。動的テキスト書き換えは新たな文字列注入経路を
/// 増やすため、視覚状態はアイコンの表示切替のみに限定する）。
///
/// 実際の状態機械（`data-state` 遷移・タイマー）は
/// `fandhe_frontend_wasm_full::add_to_basket::wire_add_to_basket` が担う
/// （本関数はマークアップ・CSS のみ）。
///
/// `idle_icon`/`adding_icon`/`added_icon` はそれぞれの状態で表示するアイコン
/// ノード（[`crate::icon::icon`] 等）を呼び出し側が組み立てて渡す。3 層とも
/// `aria-hidden="true"` の装飾レイヤーとして包む（アクセシブルネームは
/// `label` のテキストが担う）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::button::ButtonProps;
/// use fandhe_frontend_pre_styled_ui::button_motion::add_to_basket_button;
///
/// let node = add_to_basket_button(
///     &ButtonProps::default(),
///     vec![],
///     "Add to basket",
///     text("+"),
///     text("…"),
///     text("✓"),
/// );
/// let html = render(&node);
/// assert!(html.contains("data-fandhe-add-to-basket"));
/// assert!(html.contains(r#"data-part="basket-icon-idle""#));
/// assert!(html.contains("Add to basket"));
/// ```
#[must_use]
pub fn add_to_basket_button<'a>(
    props: &ButtonProps,
    attrs: Vec<(&'a str, &'a str)>,
    label: &'a str,
    idle_icon: Node,
    adding_icon: Node,
    added_icon: Node,
) -> Node {
    let mut merged_attrs = attrs;
    merged_attrs.push((ADD_TO_BASKET_ATTR, ""));
    let mut children = vec![
        wrap_basket_icon_layer(BASKET_ICON_IDLE_PART, idle_icon),
        wrap_basket_icon_layer(BASKET_ICON_ADDING_PART, adding_icon),
        wrap_basket_icon_layer(BASKET_ICON_ADDED_PART, added_icon),
    ];
    children.push(text(label));
    button(props, merged_attrs, children)
}

/// [`add_to_basket_button`] の 1 アイコンレイヤーを組み立てる
/// （`data-scope="button"` + 指定 `data-part` + `aria-hidden="true"`）。
fn wrap_basket_icon_layer(part: &'static str, icon_node: Node) -> Node {
    el(
        "span",
        vec![
            ("data-scope", "button"),
            ("data-part", part),
            ("aria-hidden", "true"),
        ],
        vec![icon_node],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn button_motion_css_has_no_forbidden_angle_bracket() {
        assert!(!BUTTON_MOTION_CSS.contains('<'));
    }

    #[test]
    fn to_css_with_button_motion_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_button_motion();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + BUTTON_MOTION_CSS.len());
    }

    #[test]
    fn rolling_text_button_has_two_layers_with_single_accessible_name() {
        let node = rolling_text_button(&ButtonProps::default(), vec![], "Save");
        let html = render(&node);
        assert!(html.contains(r#"data-part="rolling-text-current""#));
        assert!(html.contains(r#"data-part="rolling-text-duplicate""#));
        assert_eq!(html.matches("Save").count(), 2);
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 1);
    }

    #[test]
    fn rolling_text_stagger_button_writes_incrementing_indices() {
        let node = rolling_text_stagger_button(&ButtonProps::default(), vec![], "ab");
        let html = render(&node);
        assert!(html.contains("--fandhe-motion-stagger-index: 0"));
        assert!(html.contains("--fandhe-motion-stagger-index: 1"));
    }

    #[test]
    fn hold_to_confirm_button_carries_opt_in_attribute_and_fill_layer() {
        let node = hold_to_confirm_button(&ButtonProps::default(), vec![], Some("1200"), vec![]);
        let html = render(&node);
        assert!(html.contains(HOLD_TO_CONFIRM_ATTR));
        assert!(html.contains(r#"data-fandhe-hold-duration-ms="1200""#));
        assert!(html.contains(r#"data-part="hold-fill""#));
        assert!(html.contains(r#"type="button""#));
    }

    #[test]
    fn hold_to_confirm_button_omits_duration_attr_when_not_given() {
        let node = hold_to_confirm_button(&ButtonProps::default(), vec![], None, vec![]);
        let html = render(&node);
        assert!(!html.contains(HOLD_DURATION_MS_ATTR));
    }

    #[test]
    fn add_to_basket_button_carries_three_icon_layers_and_label() {
        let node = add_to_basket_button(
            &ButtonProps::default(),
            vec![],
            "Add to basket",
            text("+"),
            text("..."),
            text("v"),
        );
        let html = render(&node);
        assert!(html.contains(ADD_TO_BASKET_ATTR));
        assert!(html.contains(r#"data-part="basket-icon-idle""#));
        assert!(html.contains(r#"data-part="basket-icon-adding""#));
        assert!(html.contains(r#"data-part="basket-icon-added""#));
        assert!(html.contains("Add to basket"));
    }
}
