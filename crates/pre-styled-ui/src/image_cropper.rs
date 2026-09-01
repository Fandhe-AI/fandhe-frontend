//! styled ImageCropper（headless ラッパー、イシュー #844、親トラッキング
//! #520/#546）。
//!
//! `fandhe_frontend_headless_ui::image_cropper`（イシュー #844。§3.22
//! （イシュー #735）の意図的非採用の再導入、直接の先例は AngleSlider
//! 再導入イシュー #842）の Root / Viewport / Image / Grid / Handle の
//! anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠は [`crate::slider`] の rustdoc と同じ方針に
//! 従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`ImageCropper` 型・
//! headless `selection` を再エクスポートしない理由）
//!
//! [`crate::slider`] と同型の判断: 動的な位置・寸法を伝える唯一の経路
//! （[`ImageCropper::x_percent`](fandhe_frontend_headless_ui::image_cropper::ImageCropper::x_percent)
//! 等 4 アクセサから導出する `--fandhe-image-cropper-x`/`-y`/`-w`/`-h` の 4 個の
//! CSS custom property、下記「動的な値は 4 個の custom property のみ」参照）
//! は本モジュールの styled [`selection`] が一元的に組み立てる。headless
//! 自由関数 `selection` を呼び出し側が直接使うとこの唯一の経路を経由せず
//! 選択枠が描画されない事故を誘発するため、意図的に非公開のまま
//! [`selection`] 内部からのみ委譲する。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::image_cropper::ImageCropper`] も
//! **あえて**再エクスポートしない（[`crate::slider`] の `Slider` 非再
//! エクスポートと同じ理由）。状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::image_cropper::ImageCropper` を直接 import
//! し、実際の描画は本モジュールの styled [`root`]/[`selection`]（および
//! 再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # 動的な値は 4 個の custom property のみ（chakra-ui/Zag.js 方式）
//!
//! [`selection`] の位置・寸法は、headless 中立な `x_percent`/`y_percent`/
//! `width_percent`/`height_percent`（いずれも `0.0..=100.0` の正規化済み
//! 有限 `f64`）から [`percent_style`] が組み立てる
//! `style="--fandhe-image-cropper-x: <x>%; --fandhe-image-cropper-y: <y>%; \
//! --fandhe-image-cropper-w: <w>%; --fandhe-image-cropper-h: <h>%"` の 1 属性のみで
//! 伝搬する。[`crate::slider`] と同じく [`drop_style_attr`]（本モジュール内
//! 個別実装、`crates/headless-ui/src/progress.rs` の同名ヘルパと同型の
//! 判断）で呼び出し側 `attrs` に含まれる `style`（大文字小文字を無視）を
//! 除去してからフレームワーク側の `style` を優先する（重複属性による
//! 無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、fail-closed）。
//!
//! # `size` variant のみ（`palette` は持たない）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-image-cropper-handle-size` の root スコープ custom property
//! （CSS の通常のプロパティ継承により `handle` へ伝わる）経由で寸法を切り
//! 替える（[`crate::slider`] と同型）。`ColorPalette` は持たない
//! （selection 枠・handle の配色は装飾用途で固定色のままとし、切り抜き UI
//! に配色バリアントを持ち込む必然性がないため。`crate::steps` 等の
//! `size`-only コンポーネントと同型の判断）。
//!
//! # 写真上のクローム（`selection`/`handle`/`grid` はテーマ非依存の固定色、
//! イシュー #1480）
//!
//! `selection` の枠線・`handle` の面と縁・`grid` の線色は、いずれも
//! `--fandhe-color-*` トークンを経由せず、テーマ（ライト/ダーク）に
//! 関わらず固定の白系/黒系色を直書きする（他の多くの部品が従う「配色は
//! トークン経由」の規約からの意図的な逸脱）。理由は、これらの要素が任意の
//! 利用者写真の**上に**重なって描画されるオーバーレイであり、写真の明暗は
//! テーマ設定と無関係だからである。ダークテーマで `--fandhe-color-bg` が
//! 暗色（#111 系）へ反転すると、暗い写真 + 暗幕上で枠・ハンドルが視認
//! できなくなる（本イシューで是正した実際の不具合）。参照サイト ark-ui の
//! Image Cropper も同様にテーマ非依存の固定白でクロームを描く。切り抜き外
//! の暗幕（`selection` の `box-shadow`）のみ `--fandhe-color-bg-overlay`
//! （light 0.4 / dark 0.6）トークンを使う点が例外だが、これは暗幕の濃さを
//! ダークテーマでやや強めることで写真の見やすさとのバランスを取る意図的な
//! 選択であり（`theme.rs` が部品側からの置換を申し送っていたトークン）、
//! 枠・ハンドル・グリッド線自体の固定色化とは矛盾しない。
//!
//! `grid`（三分割グリッド線）は headless 側が `data-*` 状態を発行しない
//! （常時描画）ため、ark-ui のようなドラッグ中のみの表示切り替えは行わない
//! （据え置き。本イシューのスコープ外事項として記録）。
//!
//! # variant 軸を追加しない判断（イシュー #1480）
//!
//! ark-ui の Image Cropper は `variant`/`size` の prop を持たない。既存の
//! `size`（5 段）は「配色バリアント」ではなく handle 寸法の操作性スケール
//! （タッチ/マウス操作のしやすさ）として既に存在するため、そのまま維持する
//! （上記「`size` variant のみ」節参照）。disabled・その他の視覚状態は
//! headless が `data-handle-position` 以外の `data-*` を発行しないため
//! 追加対象がない（headless への属性追加は本イシューのスコープ外）。
//!
//! # イシュー #1481（2/2）: ズーム・回転コントロールは対象外、
//! viewport/image を 7 軸是正
//!
//! 親イシュー #1479（分割 1/2 は #1480、PR #1755 マージ済み。selection/
//! handle/grid を是正）の残り分。イシュータイトルは「ズーム・回転
//! コントロールとプレビュー」だが、以下の理由でズーム・回転コントロールは
//! **対象外**と判断した:
//!
//! - ark-ui の Image Cropper anatomy は Root / Viewport / Image /
//!   Selection / Handle / Grid の 6 パーツのみであり、zoom / rotation は
//!   anatomy パーツではなく状態機械の props（`zoom`/`maxZoom`/`minZoom`/
//!   `rotation`）である（参照サイトのデモは Slider 等の別部品との合成で
//!   実現している）。
//! - headless 層（`fandhe_frontend_headless_ui::image_cropper` モジュール
//!   doc「スコープ外」節）は zoom / rotation / flip / cropShape circle を
//!   明示的にスコープ外と記録済みであり、対応する anatomy パーツが存在
//!   しない。
//! - UI コンポーネント層の責務境界（`.claude/rules/coding-rust.md` §3.25）
//!   により anatomy（構造）の新設は headless 層の責務であり、pre-styled-ui
//!   単独で新パートを発明しない。headless 側の anatomy/`data-*` 突合は
//!   open イシュー #1610 が担当する。
//!
//! 「プレビュー」は画像表示領域（viewport/image パート）に対応すると解釈
//! し、1/2 が触っていない root/viewport/image へ 7 軸チェックリスト
//! （余白・角丸・影／色／サイズ／`data-*` 状態／ダーク／フォーカス・
//! hover・トランジション）を適用した:
//!
//! - **余白・角丸・影**: `viewport` へ `border-radius: var(--fandhe-radius-lg)`
//!   を追加（`overflow: hidden` により画像もクリップされる）。ark-ui の
//!   角丸コンテナ相当。
//! - **色**: `viewport` の背景に `var(--fandhe-color-bg-muted)`（画像ロード
//!   前・アスペクト比差のレターボックス時のみ可視）を追加。生リテラルは
//!   使わない。
//! - **操作性の表示宣言**: `viewport` へ `user-select: none` を追加。
//!   `touch-action: none`（タッチでの crop 操作中にブラウザ既定のスクロール
//!   /ピンチズームが介入しないよう意図表明）は当初 `viewport` 全体へ追加
//!   し、その後ドラッグ起点である `selection`/`handle` パートへ限定
//!   適用するよう是正したが、対応する pointer/touch イベント配線（crop
//!   矩形のドラッグ操作）が headless 側スコープ外で未実装のままである限り、
//!   適用先を移すだけでは operable な要素が実際には無いのにタッチデバイス
//!   のスクロール・ピンチズームだけを恒常的に止める同じ操作性/
//!   アクセシビリティ回帰が解消されないため（codex-review 再指摘、イシュー
//!   #1481）、`touch-action: none` の宣言自体を全パートから見送った。実
//!   ドラッグ操作の実装後、当該配線と合わせて再導入を検討する。`image` へも
//!   `user-select: none`（ネイティブのゴーストドラッグ・テキスト選択
//!   抑止）を追加するが、`pointer-events: none` は付与しない（画像上から
//!   の新規ドラッグ開始を将来の DOM 配線が使えるよう、操作起点を
//!   `selection`/`handle` に限定するスタイル側の固定を避けるため）。
//! - **サイズ/`data-*` 状態/フォーカス/hover/トランジション**: `viewport`/
//!   `image` はいずれも非フォーカサブルで headless が `data-*` を発行
//!   しない（`data-handle-position` は `handle` のみ）ため追加是正なし。
//!   位置系のトランジション非付与は 1/2 の判断を踏襲する。
//!
//! `root` は 1/2 が触っていないが、`display`/`position` のみで
//! チェックリストに該当する差分がないため変更なし。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく canvas による実画像切り出し・pointer ドラッグ/
//!   キーボード操作の DOM 配線はスコープ外
//!   （`fandhe_frontend_headless_ui::image_cropper` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   ImageCropper 追加は、未公開の新バージョンを参照できないため本
//!   イシューのスコープ外とする（[`crate::slider`] 冒頭 rustdoc の先例
//!   どおり crates.io 公開後に追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// `ImageCropper` 状態機械・headless 自由関数 `selection` はあえて
// 再エクスポートしない（本モジュール冒頭の rustdoc「選択的 re-export」節
// 参照）。状態管理・hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::image_cropper::ImageCropper` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::image_cropper::ImageCropper;
pub use fandhe_frontend_headless_ui::image_cropper::{
    grid, handle, image, viewport, HandlePosition, ImageCropperAction,
};

/// headless `image-cropper` anatomy の `data-part` 一覧（
/// `crates/headless-ui/src/image_cropper.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &["root", "viewport", "image", "selection", "handle", "grid"];

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す
/// （[`crate::slider::drop_style_attr`] と同型の判断。重複属性による無効な
/// HTML 出力・後勝ちの非決定的な描画を防ぐ、fail-closed）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `x`/`y`/`width`/`height` の百分率（[`ImageCropper::x_percent`] 等が返す
/// 正規化済み有限 `f64`）から 4 個の `--fandhe-image-cropper-*` custom property を
/// 設定する `style` 属性値を組み立てる（動的値はこの 1 箇所のみ、モジュール
/// doc「動的な値は 4 個の custom property のみ」参照）。
fn percent_style(x: f64, y: f64, width: f64, height: f64) -> String {
    format!(
        "--fandhe-image-cropper-x: {x}%; --fandhe-image-cropper-y: {y}%; \
         --fandhe-image-cropper-w: {width}%; --fandhe-image-cropper-h: {height}%"
    )
}

/// この styled ImageCropper の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("image-cropper", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("position", "relative"),
            ],
        )
        .base(
            "viewport",
            vec![
                decl("position", "relative"),
                decl("overflow", "hidden"),
                decl("display", "block"),
                decl("width", "100%"),
                decl("height", "100%"),
                // 角丸（イシュー #1481）: ark-ui の Image Cropper デモは
                // viewport（プレビュー領域）を角丸コンテナで囲む。
                // `overflow: hidden` が既にあるため角丸で画像自体もクリップ
                // される。共通トークンスケール（xs〜xl）から視覚比較で
                // `lg` を選定。
                decl("border-radius", "var(--fandhe-radius-lg)"),
                // 背景色（イシュー #1481）: 画像ロード前・アスペクト比の
                // 差でレターボックスが生じたときにのみ可視になる下地。
                // `--fandhe-color-bg-muted` トークン経由（生リテラル直書き
                // をしない、モジュール既存方針）。
                decl("background", "var(--fandhe-color-bg-muted)"),
                // ユーザー選択抑止（イシュー #1481）: プレビュー領域上の
                // テキスト選択ジェスチャ（誤ドラッグ選択）を防ぐ、非操作的な
                // 表示宣言。`touch-action: none` は viewport 全体には適用
                // しない（イシュー #1481 codex-review 指摘）: 対応する
                // pointer/touch イベント配線（crop 矩形のドラッグ操作）は
                // headless 側スコープ外で未実装のため、viewport 全体へ
                // 常時適用すると、operable な要素が無いままタッチデバイスの
                // 縦横スクロール・ピンチズームだけを恒常的に止める操作性/
                // アクセシビリティ回帰になる。ドラッグ起点となる
                // `selection`/`handle` パート側にのみ限定して宣言する
                // （下記 `selection`/`handle` base 参照）。
                decl("user-select", "none"),
            ],
        )
        .base(
            "image",
            vec![
                decl("display", "block"),
                decl("max-width", "100%"),
                // テキスト/要素選択の抑止（イシュー #1481）。`user-select`
                // は CSS の選択制御のみを担い、ブラウザ既定のネイティブ画像
                // ドラッグ（HTML5 Drag and Drop・`dragstart`）は抑止しない
                // （イシュー #1481 codex-review 指摘）。ネイティブドラッグの
                // 抑止は headless 層の
                // `fandhe_frontend_headless_ui::image_cropper::image` が
                // 既定で出力する `draggable="false"` が担う（本モジュールは
                // 当該関数をそのまま再エクスポートするのみで、スタイル層は
                // 関与しない）。`pointer-events: none` は付与しない: 将来
                // DOM 配線（headless スコープ外）で画像上から crop 矩形の
                // 新規ドラッグ開始を実装する余地を残すため（`selection`/
                // `handle` のみが操作起点という制約をスタイル層で先に
                // 固定しない）。
                decl("user-select", "none"),
            ],
        )
        .base(
            "selection",
            vec![
                decl("position", "absolute"),
                decl("left", "var(--fandhe-image-cropper-x, 0%)"),
                decl("top", "var(--fandhe-image-cropper-y, 0%)"),
                decl("width", "var(--fandhe-image-cropper-w, 100%)"),
                decl("height", "var(--fandhe-image-cropper-h, 100%)"),
                decl("box-sizing", "border-box"),
                // 枠線はテーマ非依存の固定白（`--fandhe-color-*` を経由しない
                // 意図的判断、モジュール冒頭 rustdoc「写真上のクローム」節
                // 参照）。ark-ui 準拠の細枠。
                decl("border", "1px solid rgba(255, 255, 255, 0.9)"),
                // 切り抜き外の暗幕は `theme.rs` が部品側からの置換を申し送って
                // いた `--fandhe-color-bg-overlay`（light 0.4 / dark 0.6）へ
                // 移行（rgba リテラル直書きの解消）。
                decl("box-shadow", "0 0 0 9999px var(--fandhe-color-bg-overlay)"),
                decl("cursor", "move"),
                // `touch-action: none` は付与しない（イシュー #1481
                // codex-review 再指摘）: `selection` は crop 矩形の移動
                // ドラッグの起点だが、対応する pointer/touch イベント配線
                // （ドラッグ操作の実装）は headless 側スコープ外で未実装の
                // ため、`viewport` から `selection`/`handle` へ移しただけ
                // では同じ回帰（operable な要素が実際には無いのにタッチの
                // スクロール・ピンチズームだけを常時止める）が解消されない。
                // 実ドラッグ操作の実装後、当該配線と合わせて再導入を検討
                // する（モジュール冒頭 rustdoc・`viewport` base 参照）。
                // transition は付けない: left/top/width/height はドラッグ追従値
                // であり、遷移を付けると指の動きに対して視覚的な遅延が生まれる
                // （`angle_slider` の thumb `transform` 除外と同じ理由）。
            ],
        )
        .base(
            "handle",
            vec![
                decl("position", "absolute"),
                decl("width", "var(--fandhe-image-cropper-handle-size, 0.75rem)"),
                decl("height", "var(--fandhe-image-cropper-handle-size, 0.75rem)"),
                // `touch-action: none` は付与しない（イシュー #1481
                // codex-review 再指摘）: `handle` はリサイズドラッグの
                // 起点だが、`selection` と同じ理由（実ドラッグ操作が未実装）
                // で見送る。
                // 面・縁はテーマ非依存の固定色（モジュール冒頭 rustdoc「写真上
                // のクローム」節参照）。ダークテーマで `--fandhe-color-bg` が
                // 暗色へ反転すると暗い写真 + 暗幕上でハンドルが視認できなく
                // なるため、任意の写真の上でも見える白 + 淡い黒縁へ固定する
                // （ark-ui も同様に固定白）。
                decl("background", "#ffffff"),
                decl("border", "1px solid rgba(0, 0, 0, 0.25)"),
                decl("border-radius", "var(--fandhe-radius-xs)"),
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
                decl("box-sizing", "border-box"),
                decl("transform", "translate(-50%, -50%)"),
            ],
        )
        .base(
            "handle",
            transition_declarations("background, box-shadow", MotionDuration::Fast),
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "n"),
            vec![
                decl("top", "0"),
                decl("left", "50%"),
                decl("cursor", "ns-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "s"),
            vec![
                decl("top", "100%"),
                decl("left", "50%"),
                decl("cursor", "ns-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "e"),
            vec![
                decl("top", "50%"),
                decl("left", "100%"),
                decl("cursor", "ew-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "w"),
            vec![
                decl("top", "50%"),
                decl("left", "0"),
                decl("cursor", "ew-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "ne"),
            vec![
                decl("top", "0"),
                decl("left", "100%"),
                decl("cursor", "nesw-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "nw"),
            vec![
                decl("top", "0"),
                decl("left", "0"),
                decl("cursor", "nwse-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "se"),
            vec![
                decl("top", "100%"),
                decl("left", "100%"),
                decl("cursor", "nwse-resize"),
            ],
        )
        .state(
            "handle",
            StateCondition::AttrEq("data-handle-position", "sw"),
            vec![
                decl("top", "100%"),
                decl("left", "0"),
                decl("cursor", "nesw-resize"),
            ],
        )
        .state(
            "handle",
            // 直書き outline から共通トークン経由へ移行（イシュー #1424、
            // `--fandhe-focus-ring-*`）。`palette` 軸を持たない部品のため
            // `FocusRingColor::Token` を使う（モジュール冒頭 rustdoc「`size`
            // variant のみ」節参照）。
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "handle",
            // hover フィードバック（イシュー #1425）。`hover_surface_declarations`
            // は `--fandhe-hover-bg`（`palette_declarations` 前提）を参照するが
            // 本部品は palette 軸を持たずテーマ非依存の固定白クロームのため、
            // 直書きの淡いグレーで代替する（モジュール冒頭 rustdoc「写真上の
            // クローム」節と同じ判断軸）。
            StateCondition::Hover,
            vec![decl("background", "#f0f0f0")],
        )
        .base(
            "grid",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("pointer-events", "none"),
                // 三分割線は内側の 2 本（1/3・2/3 位置）のみを描く。
                // `background-size` によるタイル方式（旧実装）は 0%/33%/66% の
                // 3 本を描いてしまい、0% の線が selection の枠線（左端・上端）
                // と重なって二重線になるため、`background-position` で
                // 個別配置し `background-repeat: no-repeat` で繰り返しを止める。
                decl(
                    "background-image",
                    "linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5)), \
                     linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5)), \
                     linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5)), \
                     linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5))",
                ),
                decl(
                    "background-size",
                    "1px 100%, 1px 100%, 100% 1px, 100% 1px",
                ),
                decl(
                    "background-position",
                    "calc(100% / 3) 0, calc(100% / 3 * 2) 0, 0 calc(100% / 3), 0 calc(100% / 3 * 2)",
                ),
                decl("background-repeat", "no-repeat"),
            ],
        )
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.35rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.55rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.75rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "0.95rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-image-cropper-handle-size", "1.15rem")],
        )
        .default_variant(Size::Md)
}

/// この styled ImageCropper が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は
/// [`fandhe_frontend_headless_ui::image_cropper::ImageCropper::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::image_cropper::ImageCropper;
/// use fandhe_frontend_pre_styled_ui::image_cropper;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let c = ImageCropper::default();
/// let node = image_cropper::root(Size::Md, &c, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="image-cropper" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: &ImageCropper,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(merged, children)
}

/// styled selection パーツを組み立てる。4 個の `--fandhe-image-cropper-*` custom
/// property を含む `style` を付与する唯一のパーツ（[`drop_style_attr`] に
/// より呼び出し側の `style` は除去してから合成する。動的値はこの 1 箇所
/// のみ、モジュール doc「動的な値は 4 個の custom property のみ」参照）。
/// 実体は
/// [`fandhe_frontend_headless_ui::image_cropper::ImageCropper::selection`]
/// へ委譲する。
#[must_use]
pub fn selection<'a>(
    state: &ImageCropper,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let style = percent_style(
        state.x_percent(),
        state.y_percent(),
        state.width_percent(),
        state.height_percent(),
    );
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.selection(merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="image-cropper"][data-part="selection"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_references_cropper_custom_properties() {
        let css = stylesheet();
        for prop in [
            "--fandhe-image-cropper-x",
            "--fandhe-image-cropper-y",
            "--fandhe-image-cropper-w",
            "--fandhe-image-cropper-h",
            "--fandhe-image-cropper-handle-size",
        ] {
            assert!(css.contains(prop), "missing {prop} in css");
        }
    }

    #[test]
    fn stylesheet_links_handle_to_all_eight_positions() {
        let css = stylesheet();
        for pos in ["n", "s", "e", "w", "ne", "nw", "se", "sw"] {
            assert!(css.contains(&format!(
                r#"[data-scope="image-cropper"][data-part="handle"][data-handle-position="{pos}"] {{"#
            )));
        }
    }

    #[test]
    fn stylesheet_links_handle_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="image-cropper"][data-part="handle"]:focus-visible {"#));
    }

    #[test]
    fn handle_focus_ring_uses_common_token_not_legacy_outline() {
        // イシュー #1480: フォーカスリングの直書き `outline: 2px solid
        // var(--fandhe-color-accent)` を共通トークン
        // （`--fandhe-focus-ring-*`）経由へ移行したことを固定する。
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-focus-ring-width, 2px)"));
        assert!(css.contains("var(--fandhe-focus-ring-offset, 2px)"));
        assert!(!css.contains("outline: 2px solid var(--fandhe-color-accent)"));
    }

    #[test]
    fn selection_backdrop_uses_overlay_token_not_raw_rgba() {
        // イシュー #1480: `theme.rs` が申し送っていた rgba リテラル
        // （切り抜き外の暗幕）を `--fandhe-color-bg-overlay` トークン
        // （light 0.4 / dark 0.6）へ置換したことを固定する。
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-color-bg-overlay)"));
        assert!(!css.contains("rgba(0, 0, 0, 0.5)"));
    }

    #[test]
    fn handle_has_hover_and_background_box_shadow_transition_only() {
        // イシュー #1480: handle に hover フィードバックと transition を
        // 新設した。`top`/`left`/`transform` はリサイズドラッグの追従値
        // であり遷移を付けないため、`transition-property` に含まれないこと
        // も併せて固定する。
        let css = stylesheet();
        assert!(css.contains("background: #f0f0f0"));
        assert!(css.contains("transition-property: background, box-shadow"));
        assert!(!css.contains("transition-property: background, box-shadow, transform"));
        assert!(!css.contains("transition-property: top, left"));
    }

    #[test]
    fn viewport_has_radius_token_and_background_but_not_touch_action() {
        // イシュー #1481: viewport（プレビュー領域）へ角丸・背景（レターボックス
        // 用）を追加したことを固定する。`touch-action: none` は
        // viewport 全体には適用しない（codex-review 是正）。
        let css = stylesheet();
        let viewport_block = css
            .split("\n\n")
            .find(|b| b.contains(r#"[data-part="viewport"]"#))
            .expect("viewport block missing");
        assert!(viewport_block.contains("border-radius: var(--fandhe-radius-lg);"));
        assert!(viewport_block.contains("background: var(--fandhe-color-bg-muted);"));
        assert!(viewport_block.contains("user-select: none;"));
        assert!(!viewport_block.contains("touch-action: none;"));
    }

    #[test]
    fn no_part_declares_touch_action_none_while_drag_is_unimplemented() {
        // イシュー #1481 codex-review 再指摘: crop 矩形のドラッグ操作
        // （移動・リサイズ）に対応する pointer/touch イベント配線は
        // headless 側スコープ外で未実装のため、`selection`/`handle` へ
        // 限定適用しても operable な要素が実際には無いままタッチデバイス
        // のスクロール・ピンチズームだけを止める同じ回帰が残る。実
        // ドラッグ操作の実装まで `touch-action: none` をどのパートにも
        // 一切宣言しないことを固定する。
        let css = stylesheet();
        assert!(!css.contains("touch-action"));
    }

    #[test]
    fn image_has_user_select_none_but_not_pointer_events_none() {
        // イシュー #1481: image はネイティブドラッグ/選択を抑止する
        // `user-select: none` のみ追加し、`pointer-events: none` は付与
        // しない（画像上からの新規ドラッグ開始を将来の DOM 配線に残すため、
        // モジュール冒頭 rustdoc「イシュー #1481」節参照）。
        let css = stylesheet();
        let image_block = css
            .split("\n\n")
            .find(|b| b.contains(r#"[data-part="image"]"#))
            .expect("image block missing");
        assert!(image_block.contains("user-select: none;"));
        assert!(!image_block.contains("pointer-events: none;"));
    }

    #[test]
    fn handle_has_radius_and_shadow_tokens() {
        // イシュー #1480: handle を余白・角丸・影の共通トークンスケールへ
        // 載せたことを固定する。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-xs)"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-sm)"));
    }

    #[test]
    fn stylesheet_contains_size_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--fandhe-image-cropper-handle-size"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let c = ImageCropper::default();
        let html = render(&root(Size::Md, &c, vec![], vec![]));
        assert!(html.contains(r#"data-scope="image-cropper""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let c = ImageCropper::default();
        let html = render(&root(Size::Md, &c, vec![], vec![]));
        assert!(html.contains("fd-image-cropper--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let c = ImageCropper::default();
        for (size, class) in [
            (Size::Xs, "fd-image-cropper--size-xs"),
            (Size::Sm, "fd-image-cropper--size-sm"),
            (Size::Md, "fd-image-cropper--size-md"),
            (Size::Lg, "fd-image-cropper--size-lg"),
            (Size::Xl, "fd-image-cropper--size-xl"),
        ] {
            let html = render(&root(size, &c, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let c = ImageCropper::default();
        let html = render(&root(
            Size::Md,
            &c,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let c = ImageCropper::default();
        let html = render(&root(
            Size::Md,
            &c,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="image-cropper""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- selection: --fandhe-image-cropper-* の唯一の動的値経路 ---

    #[test]
    fn selection_outputs_percent_style() {
        let c = ImageCropper::new(200, 100, 50, 25, 100, 50, None, 1);
        let html = render(&selection(&c, vec![], vec![]));
        assert!(html.contains("--fandhe-image-cropper-x: 25%"));
        assert!(html.contains("--fandhe-image-cropper-y: 25%"));
        assert!(html.contains("--fandhe-image-cropper-w: 50%"));
        assert!(html.contains("--fandhe-image-cropper-h: 50%"));
    }

    #[test]
    fn selection_caller_style_attr_is_dropped_not_duplicated() {
        let c = ImageCropper::default();
        let html = render(&selection(&c, vec![("style", "attacker: 1")], vec![]));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let c = ImageCropper::default();
        let html = render(&root(
            Size::Md,
            &c,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_image_src_alt_are_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&image(PAYLOAD, PAYLOAD, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_grid_children_text_are_escaped_on_render() {
        let html = render(&selection(
            &ImageCropper::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_image_cropper_state_machine() {
        // `ImageCropper` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`ImageCropper` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut c = ImageCropper::new(200, 100, 0, 0, 50, 50, None, 1);
        let ssr_html = render(&root(Size::Md, &c, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut c, "move", "10,10"));
        assert_eq!(c.x(), 10);

        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains(r#"data-hydrate-x="10""#));

        let restored = ImageCropper::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }
}
