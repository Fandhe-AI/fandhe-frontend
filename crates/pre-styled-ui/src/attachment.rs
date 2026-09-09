//! styled Attachment（shadcn/ui `Attachment` 相当。イシュー #2112、親
//! #2110、祖父トラッキング参照軸 #2001。headless 側 anatomy は #2111）。
//!
//! `fandhe_frontend_headless_ui::attachment`（#2111）が出力する
//! `data-scope="attachment"` の 8 パーツ（`root`/`media`/`content`/`name`/
//! `meta`/`progress`/`actions`/`action`）へ、添付ファイル 1 件の意匠
//! （横並びの行カード = `file` 形態、縦積みのサムネイルカード =
//! `image` 形態、アップロード失敗時の枠色、アクション群の hover 表示）を
//! 重ねる薄い委譲層である（[`crate::bubble`]/[`crate::message`] と同型の
//! 構成）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::bubble`]/[`crate::message`]/[`crate::item`] と同型。8 パーツ
//! すべて同名再定義し（呼び出し側 `class` の除去は本モジュールの責務の
//! ため）、[`AttachmentRootProps`]/[`AttachmentVariant`]/[`AttachmentState`]
//! の 3 型のみを選択的に再エクスポートする。
//!
//! # 状態機械を持たない理由
//!
//! headless [`mod@fandhe_frontend_headless_ui::attachment`] 自身が状態機械を
//! 持たない静的な自由関数群であるため、本モジュールもその設計をそのまま
//! 継承する（[`crate::bubble`] モジュール doc と同型の判断）。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25）
//!
//! アップロード進捗の判定・エラー分類・削除処理・ファイル名/メタ情報の
//! 整形（byte → KB 変換等）はアプリケーションロジックであり実装しない
//! （規則 1）。headless が出力する `data-*` を CSS セレクタとして参照する
//! だけで見た目を切り替える。
//!
//! # `data-variant`/`data-state`/`data-disabled`: headless の `data-*` を
//! `AttrEq`/`Attr` で参照する（[`crate::bubble`] と同型の意図的差分）
//!
//! headless `attachment::root` は `data-variant`（`file`/`image`）・
//! `data-state`（`idle`/`uploading`/`error`）・`data-disabled`（存在
//! 属性）を固定出力済み（`crates/headless-ui/src/attachment.rs`）。本
//! モジュールはこれらを [`StateCondition::AttrEq`]/[`StateCondition::Attr`]
//! で**参照するのみ**とし、class ベースの [`SlotRecipe::variant`] を持たない
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2「役割 B:
//! 参照のみ」）。したがって 8 パーツすべて見た目クラスを付与しない同名
//! 再定義であり、[`crate::bubble`]/[`crate::message`]/[`crate::item`] と
//! 同じパターンを踏襲する。
//!
//! # `data-variant="file"`（既定）: 横並びの行カード
//!
//! `root` は `display: flex` の横並びで、`media`（固定サイズの角丸アイコン
//! 枠）・`content`（`name`/`meta` を縦積み）・`actions`（末尾へ
//! `margin-inline-start: auto` で押し出し、常時表示）の 3 領域から成る。
//!
//! # `data-variant="image"`: 縦積みのサムネイルカード
//!
//! `root` は `flex-direction: column` へ切り替え、`media` を正方形
//! （`aspect-ratio: 1 / 1`）のサムネイルとして子 `img` を `object-fit:
//! cover` で敷き詰める。`media` の base 宣言は固定 `height:
//! var(--fandhe-space-10)` を持つため、image 形態の上書き規則は
//! `width: 100%` に加えて **`height: auto` も明示**する（CSS 仕様上
//! `width`/`height` の両方が明示指定されていると `aspect-ratio` が
//! 無視され正方形にならないため。golden スナップショットは出力
//! バイト列を固定するのみで意味論までは検知できないため、この個別
//! 事実は `attachment_css.rs::css_positions_image_variant_media_and_actions`
//! が `height: auto;` の存在で追加固定する）。`actions` は右上へ絶対
//! 配置し、**タッチ端末対策**（下記「`actions` の hover 表示とタッチ
//! 端末対策」節参照）として `@media (hover: hover)` 配下でのみ既定
//! 非表示にする。
//!
//! # `actions` の hover 表示とタッチ端末対策
//!
//! shadcn/ui の縦積みサムネイルカードは、通常時は `actions`（削除等）を
//! 隠し、カードへの hover でのみ表示する意匠を持つ。この隠蔽を
//! `opacity: 0`（既定）→ `root:hover`/`root:focus-within` で `opacity: 1`
//! という単純な実装にすると、hover 機構を持たないタッチ端末では
//! `actions` が「見えないボタン」になり、そもそも視認できないため
//! フォーカスを当てる操作（タップ）を行えず、実質的に到達不能になる
//! （`display: none`/`visibility: hidden` を使わない、というモジュール
//! doc の意図だけでは防げない別種の到達不能を生む）。
//!
//! そのため本モジュールは `opacity: 0` の既定非表示規則そのものを
//! `@media (hover: hover)` 配下へ限定し（[`crate::recipe::StateCondition::Hover`]
//! が単一 slot のホバー規則を `@media (hover: hover)` へ集約する設計と
//! 同じ意図、[`file_upload`](crate::file_upload) の
//! `item-delete-trigger` hover 規則を参照）、hover 機構を持たない端末
//! では宣言そのものが適用されず、CSS の初期値である `opacity: 1`
//! （常時表示）のまま残る。hover 機構を持つ端末でのみ、既定で隠し
//! `root:hover`/`root:focus-within`（キーボード操作対応）で再表示する。
//! `root`（`data-part="root"`）と `actions`（`data-part="actions"`）は
//! 別 slot の組み合わせセレクタ（子結合子）のため
//! [`crate::recipe::SlotRecipe::state`] の [`StateCondition::Hover`]
//! （単一 slot 前提）では表現できず、[`stylesheet`] 内で raw CSS として
//! 手動集約する（[`crate::message`] が `raw CSS 追記の理由」で採る手法と
//! 同型）。
//!
//! # `data-state="error"`
//!
//! `root` の枠色を `var(--fandhe-color-danger)` へ、`meta` の文字色を
//! `var(--fandhe-color-danger)` へ切り替える（[`crate::field`] の
//! invalid・[`crate::message`] の `data-error` と同じトークンを再利用し、
//! 新トークンは作らない）。`root`/`meta` は別 slot のため、`meta` 側の
//! 規則も [`stylesheet`] 内で raw CSS として追記する。
//!
//! # `progress` スロット
//!
//! headless `attachment::progress` は attachment scope の単純なスロット
//! であり、[`crate::progress::Progress`] を委譲しない（headless モジュール
//! doc「`progress` は attachment scope のスロット」参照）。呼び出し側が
//! 中身へ [`crate::progress`] の styled パーツ群を入れ子にする契約を
//! そのまま継承する。`root`（`media`/`content`/`progress`/`actions` の
//! DOM 順で並ぶ、[`fandhe_frontend_headless_ui::attachment::root`] rustdoc
//! 参照）は `display: flex` の単純な横並びのため、`progress` に `width:
//! 100%` のみを与えると `flex-wrap` 既定値 `nowrap` の下では折り返さず
//! `media`/`content`/`actions` と同じ行の兄弟要素になり、アップロード中
//! カードで `name`/`meta` と同じ行へ全幅バーが割り込む見た目崩れになる
//! （下段へスタックされるべきという意匠が壊れる）。そのため `root` へ
//! `flex-wrap: wrap` を、`progress` へ `flex-basis: 100%` を追加し、
//! `progress` が単独で残り幅を占有して強制的に次の行へ折り返るように
//! する（`image` 形態は `flex-direction: column` へ切り替わり全パーツが
//! 元々縦積みのため、この 2 宣言は無害な no-op のまま残る）。
//!
//! ## `actions` を常に末尾行へ固定する（`order`、イシュー #2112 レビュー
//! 是正）
//!
//! 上記の `flex-basis: 100%` だけでは、DOM 順が `media`/`content`/
//! `progress`/`actions` であるため、アップロード中カードは `progress` が
//! 単独で 2 行目を占有した直後に続く `actions` がさらに 3 行目へ押し
//! 出され、`root` が横並びカードの**末尾行**に `actions` を置くという
//! 表示契約（本モジュール doc 冒頭「`data-variant="file"`」節参照）を
//! 満たさなくなる（アップロード中に限って削除・キャンセル操作が独立行へ
//! 孤立する見た目崩れ）。CSS の `flex-wrap` は DOM 順ではなく `order`
//! （プロパティ）で決まる**視覚順**でライン分割するため、`progress` にの
//! み `order: 1`（既定 `0` より後）を与えて視覚順を `media`/`content`/
//! `actions`/`progress` へ変える。これにより `progress` が存在しない
//! （`idle`/`error`）ときは `media`/`content`/`actions` の 1 行のみ、
//! `progress` が存在する（`uploading`）ときは 1 行目 `media`/`content`/
//! `actions` + 2 行目 `progress`（`flex-basis: 100%` により単独で折り
//! 返る）の 2 行構成に固定され、`progress` の有無によらず `actions` は
//! 常に末尾行を維持する（`image` 形態は `flex-direction: column` へ
//! 切り替わり `order` を使わない元々の縦積みのため、この宣言も無害な
//! no-op のまま残る）。
//!
//! ## `content` の `flex-basis` を `0` にする理由（長いファイル名対策、
//! 同レビュー是正）
//!
//! `content`（`flex: 1 1 auto`）の flex-basis が `auto` のままだと、子
//! `name`（`white-space: nowrap`）の min-content 幅がファイル名の全長
//! そのものになり、flex の折り返し判定はこの hypothetical main size
//! （`flex-shrink` 適用前の希望サイズ）を基準に行だけを分割する。長い
//! ファイル名では `content` の希望幅が行の残り幅を超え、`flex-shrink`
//! が効く前に `content` 自身が独立行へ折り返ってしまい、`media`/
//! `actions` との横並びが崩れる（Cursor Bugbot 指摘）。`flex: 1 1 0%`
//! （flex-basis を `0%` に固定）へ変更すると hypothetical main size が
//! `0` になり、`flex-grow`/`flex-shrink` と `min-width: 0`（既存宣言）
//! の組み合わせで `content` は常に `media`/`actions` と同じ行に留まり
//! 残り幅まで縮小できる。
//!
//! # `data-disabled`
//!
//! `root` に既存ヘルパ [`crate::recipe::disabled_declarations`]
//! （`opacity: 0.5` + `cursor: not-allowed`）を適用する。`action` 側は
//! [`file_upload::item_delete_trigger`](crate::file_upload) と同じ判断で
//! `cursor: not-allowed` のみに留める（`root`（0.5）× `action` 自身の
//! `disabled_declarations()`（0.5）で opacity が 0.25 まで二重減衰する
//! のを避けるため）。
//!
//! # `action`
//!
//! ゴーストボタン（[`file_upload::item_delete_trigger`](crate::file_upload)
//! の base をモデルにした `inline-flex` 中央寄せ・固定サイズ・
//! `border: none`・`background: transparent`・`color: inherit`・
//! [`crate::recipe::hover_bg_muted`]）+ hover（
//! [`crate::recipe::hover_surface_declarations`]）+ フォーカスリング
//! （[`crate::recipe::focus_ring_declarations`]、`Token`: 本部品は
//! `ColorPalette` 軸を持たない。`Outside`: `action` の祖先に
//! `overflow: hidden` を持つ slot がない）。
//!
//! # `ColorPalette` 軸を持たない理由
//!
//! [`crate::bubble`] と同じ判断（本イシューのスコープに含まれない）。
//! 必要になれば非破壊的に追加提案する（`.claude/rules/out-of-scope-tracking.md`
//! 対応）。
//!
//! # `prefers-reduced-motion` を書かない理由
//!
//! [`crate::theme::Theme::to_css`] が duration トークンを 0ms へ一括
//! 上書きするため本モジュール側では書かない（[`crate::bubble`]/
//! [`crate::calendar`] と同型）。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`mod@fandhe_frontend_headless_ui::attachment`] →
//!   [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は [`drop_class_attr`] で除去してから headless
//!   関数へ委譲する（8 パーツすべて）。
//! - [`stylesheet`] が組み立てる CSS 宣言はすべてコンパイル時静的
//!   リテラルであり、[`crate::css::decl`]/[`crate::css::serialize_rule`]
//!   の検証を通る値のみを使う（raw CSS 追記部分を含む）。
//!
//! # スコープ外
//!
//! - `fandhe-frontend-wasm-full` の配線（headless rustdoc「wasm-full 未
//!   配線」参照。静的部品、状態機械なし）。
//! - `ColorPalette` 軸の追加（上記「`ColorPalette` 軸を持たない理由」
//!   参照）。
//! - `examples/headless-pre-styled-ui` への attachment 追加。
//! - 兄弟部品 marker（#2114）への語彙追随。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// props/variant/state 型のみ（`crate::bubble` と同型の規約）。パーツ関数
// 8 件は呼び出し側 `class` の除去を担うため同名再定義する。
pub use fandhe_frontend_headless_ui::attachment::{
    AttachmentRootProps, AttachmentState, AttachmentVariant,
};

/// slot 一覧（headless [`mod@fandhe_frontend_headless_ui::attachment`] の
/// anatomy と 1:1、8 パーツ）。
const SLOTS: &[&str] = &[
    "root", "media", "content", "name", "meta", "progress", "actions", "action",
];

/// この styled Attachment の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let root_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("flex-wrap", "wrap"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("border-radius", "var(--fandhe-radius-lg)"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("color", "var(--fandhe-color-fg)"),
    ];

    let media_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("flex-shrink", "0"),
        decl("width", "var(--fandhe-space-10)"),
        decl("height", "var(--fandhe-space-10)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("background", "var(--fandhe-color-bg-muted)"),
        decl("overflow", "hidden"),
    ];

    let content_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-0-5)"),
        decl("flex", "1 1 0%"),
        decl("min-width", "0"),
    ];

    let name_base = vec![
        decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        decl("white-space", "nowrap"),
        decl("overflow", "hidden"),
        decl("text-overflow", "ellipsis"),
        decl("color", "var(--fandhe-color-fg)"),
    ];

    let meta_base = vec![
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
    ];

    let progress_base = vec![
        decl("width", "100%"),
        decl("flex-basis", "100%"),
        // `actions` を常に末尾行へ固定する（モジュール doc「`actions` を
        // 常に末尾行へ固定する」節参照）。DOM 順は media/content/
        // progress/actions だが、`order` で視覚順を media/content/
        // actions/progress へ変え、`progress` の有無によらず `actions` が
        // 1 行目の末尾を維持するようにする。
        decl("order", "1"),
    ];

    let actions_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("margin-inline-start", "auto"),
    ];

    let action_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("box-sizing", "border-box"),
        decl("width", "var(--fandhe-space-6)"),
        decl("height", "var(--fandhe-space-6)"),
        decl("padding", "0"),
        decl("border", "none"),
        decl("border-radius", "var(--fandhe-radius-sm)"),
        decl("background", "transparent"),
        decl("color", "inherit"),
        decl("cursor", "pointer"),
        decl("line-height", "1"),
        hover_bg_muted(),
    ];

    SlotRecipe::new("attachment", SLOTS)
        .base("root", root_base)
        .base("media", media_base)
        .base("content", content_base)
        .base("name", name_base)
        .base("meta", meta_base)
        .base("progress", progress_base)
        .base("actions", actions_base)
        .base(
            // image 形態の絶対配置 actions のフェード（`data-variant="file"`
            // では opacity が変化しないため実質 no-op、モジュール doc
            // 「`actions` の hover 表示とタッチ端末対策」参照）。
            "actions",
            transition_declarations("opacity", MotionDuration::Fast),
        )
        .base("action", action_base)
        .base(
            "action",
            transition_declarations("background", MotionDuration::Fast),
        )
        // `data-variant="image"`: 縦積みのサムネイルカード（モジュール doc
        // 参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "image"),
            vec![
                decl("flex-direction", "column"),
                decl("align-items", "stretch"),
                decl("position", "relative"),
                decl("width", "var(--fandhe-attachment-image-width, 12rem)"),
            ],
        )
        // `data-state="error"`: root の枠色（`meta` 側は別 slot のため
        // `stylesheet` 内の raw CSS で追記、モジュール doc参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-state", "error"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // `data-disabled`。
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "action",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "action",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // `action` 自身の `disabled`（root とは独立、モジュール doc
        // 「`data-disabled`」節参照）。
        .state(
            "action",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
}

/// この styled Attachment が生成する静的 CSS 全量を返す（決定的。
/// [`crate::bubble::stylesheet`] と同じ契約）。`data-variant="image"` の
/// `media`/`img`/`actions` 配置、`data-state="error"` 時の `meta` 文字色、
/// `actions` の hover 表示（タッチ端末対策込み）は `root`/`media`/`meta`/
/// `actions` にまたがる組み合わせセレクタのため
/// [`crate::recipe::SlotRecipe::state`]（単一 slot 前提）では表現できず、
/// raw CSS として追記する（モジュール doc「`actions` の hover 表示と
/// タッチ端末対策」「`data-state="error"`」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const ROOT: &str = r#"[data-scope="attachment"][data-part="root"]"#;
    const MEDIA: &str = r#"[data-scope="attachment"][data-part="media"]"#;
    const ACTIONS: &str = r#"[data-scope="attachment"][data-part="actions"]"#;
    const META: &str = r#"[data-scope="attachment"][data-part="meta"]"#;

    let image_root = format!(r#"{ROOT}[data-variant="image"]"#);

    let append = |rule: Option<String>, out: &mut String| {
        if let Some(css) = rule {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&css);
        }
    };

    // image 形態: media を正方形サムネイルへ（モジュール doc
    // 「`data-variant="image"`」節参照）。
    let media_selector = format!("{image_root} > {MEDIA}");
    append(
        serialize_rule(
            &media_selector,
            &[
                decl("aspect-ratio", "1 / 1"),
                decl("width", "100%"),
                decl("height", "auto"),
            ],
        ),
        &mut out,
    );

    let media_img_selector = format!("{image_root} > {MEDIA} img");
    append(
        serialize_rule(
            &media_img_selector,
            &[
                decl("width", "100%"),
                decl("height", "100%"),
                decl("object-fit", "cover"),
            ],
        ),
        &mut out,
    );

    // image 形態: actions を右上へ絶対配置（既定表示。hover 機構がある
    // 端末でのみ後段の `@media (hover: hover)` が非表示へ上書きする）。
    let actions_position_selector = format!("{image_root} > {ACTIONS}");
    append(
        serialize_rule(
            &actions_position_selector,
            &[
                decl("position", "absolute"),
                decl("top", "var(--fandhe-space-2)"),
                decl("right", "var(--fandhe-space-2)"),
                decl("margin-inline-start", "0"),
            ],
        ),
        &mut out,
    );

    // `data-state="error"` 時の `meta` 文字色（root とは別 slot のため raw
    // CSS、モジュール doc「`data-state="error"`」節参照）。
    let error_meta_selector = format!(r#"{ROOT}[data-state="error"] {META}"#);
    append(
        serialize_rule(
            &error_meta_selector,
            &[decl("color", "var(--fandhe-color-danger)")],
        ),
        &mut out,
    );

    // タッチ端末対策（モジュール doc「`actions` の hover 表示とタッチ
    // 端末対策」参照）: `opacity: 0` の既定非表示規則そのものを `@media
    // (hover: hover)` 配下へ限定する。hover 機構を持たない端末では規則が
    // 適用されず、CSS の初期値 `opacity: 1`（常時表示）のまま残る。
    let hidden_selector = format!("{image_root} > {ACTIONS}");
    let reveal_selector =
        format!("{image_root}:hover > {ACTIONS}, {image_root}:focus-within > {ACTIONS}");
    let mut hover_block = String::new();
    let append_hover = |rule: Option<String>, block: &mut String| {
        if let Some(css) = rule {
            if !block.is_empty() {
                block.push('\n');
            }
            block.push_str(&css);
        }
    };
    append_hover(
        serialize_rule(&hidden_selector, &[decl("opacity", "0")]),
        &mut hover_block,
    );
    append_hover(
        serialize_rule(&reveal_selector, &[decl("opacity", "1")]),
        &mut hover_block,
    );
    if !hover_block.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("@media (hover: hover) {\n");
        for line in hover_block.trim_end_matches('\n').lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                out.push_str("  ");
                out.push_str(line);
                out.push('\n');
            }
        }
        out.push_str("}\n");
    }

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール
/// doc「headless の `data-*` を参照する」節参照）、呼び出し側 `class` を
/// [`drop_class_attr`] で除去してから
/// [`fandhe_frontend_headless_ui::attachment::root`] へそのまま委譲する。
#[must_use]
pub fn root<'a>(
    props: AttachmentRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::attachment::root(props, drop_class_attr(attrs), children)
}

/// styled `media` パーツを組み立てる。
#[must_use]
pub fn media<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::attachment::media(drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::attachment::content(drop_class_attr(attrs), children)
}

/// styled `name` パーツを組み立てる。
#[must_use]
pub fn name<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::attachment::name(drop_class_attr(attrs), children)
}

/// styled `meta` パーツを組み立てる。
#[must_use]
pub fn meta<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::attachment::meta(drop_class_attr(attrs), children)
}

/// styled `progress` パーツを組み立てる。呼び出し側が中身へ
/// [`crate::progress`] の styled パーツ群を入れ子にする契約（モジュール
/// doc「`progress` スロット」節参照）。
#[must_use]
pub fn progress<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::attachment::progress(drop_class_attr(attrs), children)
}

/// styled `actions` パーツを組み立てる。
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::attachment::actions(drop_class_attr(attrs), children)
}

/// styled `action` パーツを組み立てる。`label` は headless
/// [`fandhe_frontend_headless_ui::attachment::action`] が既定エスケープを
/// 経由して `aria-label` へ出力する（本モジュールは再エスケープしない）。
#[must_use]
pub fn action<'a>(
    label: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::attachment::action(
        label,
        disabled,
        drop_class_attr(attrs),
        children,
    )
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
        assert!(a.contains(r#"[data-scope="attachment"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_declares_variant_state_disabled_and_hover_rules() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-variant="image"]"#));
        assert!(out.contains(r#"[data-state="error"]"#));
        assert!(out.contains("[data-disabled]"));
        assert!(out.contains(":hover"));
        assert!(out.contains(":focus-within"));
        assert!(out.contains("object-fit: cover;"));
        assert!(out.contains("@media (hover: hover)"));
        // class ベースの variant/state クラス（`fd-attachment--` 等）は
        // 生成しない（モジュール doc参照）。
        assert!(!out.contains("fd-attachment--"));
    }

    #[test]
    fn stylesheet_hides_image_actions_only_inside_hover_media() {
        // モジュール doc「`actions` の hover 表示とタッチ端末対策」節の
        // 回帰: `opacity: 0` は `@media (hover: hover)` 配下にのみ現れ、
        // hover 機構を持たない端末では `opacity: 1`（CSS 初期値）のまま
        // 残ることを固定する。
        let out = stylesheet();
        let media_start = out
            .find("@media (hover: hover)")
            .expect("hover media block");
        let before_media = &out[..media_start];
        assert!(!before_media.contains("opacity: 0;"));
        assert!(out[media_start..].contains("opacity: 0;"));
        assert!(out[media_start..].contains("opacity: 1;"));
    }

    #[test]
    fn root_connects_to_headless_attachment_scope() {
        let html = render(&root(AttachmentRootProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="attachment" data-part="root""#));
        assert!(html.starts_with("<div"));
    }

    #[test]
    fn all_parts_connect_to_headless_attachment_scope() {
        let media_html = render(&media(vec![], vec![core_text("icon")]));
        assert!(media_html.contains(r#"data-scope="attachment" data-part="media""#));

        let content_html = render(&content(vec![], vec![]));
        assert!(content_html.contains(r#"data-scope="attachment" data-part="content""#));

        let name_html = render(&name(vec![], vec![core_text("report.pdf")]));
        assert!(name_html.contains(r#"data-scope="attachment" data-part="name""#));

        let meta_html = render(&meta(vec![], vec![core_text("PDF · 128 KB")]));
        assert!(meta_html.contains(r#"data-scope="attachment" data-part="meta""#));

        let progress_html = render(&progress(vec![], vec![core_text("64%")]));
        assert!(progress_html.contains(r#"data-scope="attachment" data-part="progress""#));

        let actions_html = render(&actions(vec![], vec![]));
        assert!(actions_html.contains(r#"data-scope="attachment" data-part="actions""#));

        let action_html = render(&action("Delete", false, vec![], vec![]));
        assert!(action_html.contains(r#"data-scope="attachment" data-part="action""#));
        assert!(action_html.contains(r#"aria-label="Delete""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let html = render(&root(
            AttachmentRootProps::default(),
            vec![("class", "evil")],
            vec![
                media(vec![("class", "evil")], vec![]),
                content(
                    vec![("class", "evil")],
                    vec![
                        name(vec![("class", "evil")], vec![]),
                        meta(vec![("class", "evil")], vec![]),
                    ],
                ),
                progress(vec![("class", "evil")], vec![]),
                actions(
                    vec![("class", "evil")],
                    vec![action("Delete", false, vec![("class", "evil")], vec![])],
                ),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }
}
