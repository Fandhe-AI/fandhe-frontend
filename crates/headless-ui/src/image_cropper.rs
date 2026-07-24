//! ImageCropper（crop 矩形の決定的状態機械）headless コンポーネント
//! （イシュー #844: `docs/policy/intentional-non-adoption.md` §3.22
//! （イシュー #735）の意図的非採用の再導入。§4 再導入手続き準拠、直接の
//! 先例は AngleSlider 再導入、イシュー #842）。
//!
//! # 非採用理由を設計で無効化する（§4 再導入手続き §2 の再評価）
//!
//! §3.22 は ImageCropper を「canvas ピクセル状態の暗黙性・ポインタ座標の
//! 非決定性・機械検証困難・`web-sys` API サーフェス増」を理由に非採用と
//! 確定した。本モジュールはそれらの理由そのものを設計から排除する形で
//! 再導入する。
//!
//! - **明示性**: 本モジュールは **crop 矩形（`x`/`y`/`width`/`height` の
//!   `u32`）の純粋状態機械のみ**を扱う。canvas 描画命令・変換行列・ピクセル
//!   バッファは一切保持しない。クランプ・アスペクト比丸め・resize 端の
//!   規則はすべて整数演算のみで本 rustdoc に明文化する（下記「決定的な
//!   整数演算」参照）。
//! - **決定性**: 同一の dispatch 列（`"move"`/`"resize"`/`"set"`/`"reset"`）
//!   からは常に同一の矩形が再現される。浮動小数点・デバイス依存の座標
//!   （ポインタ位置・DPI）を一切扱わない。
//! - **機械検証可能性**: クランプ表・アスペクト比丸めの網羅表・8 方位
//!   resize の網羅表を `cargo test` の決定的アサーションで固定する
//!   （`#[cfg(test)]` 参照）。canvas ピクセル出力・ポインタ座標ストリーム
//!   を検証する視覚回帰基盤は本設計では不要になる。
//! - **コンテキスト消費**: 本イシューは `web-sys` の Canvas/Pointer API を
//!   一切追加しない（`fandhe-frontend-headless-ui` の外部依存は
//!   `fandhe-frontend-core`/`fandhe-frontend-interactive` のみのまま）。
//!
//! 「切り抜き結果」は crop 矩形の値そのものとして呼び出し側へ返す。canvas
//! による実画像切り出し（ピクセルデータの生成）は本モジュールのスコープ外
//! （下記「スコープ外」参照、`docs/design/component-coverage-map.md` に
//! 注記）。表示は CSS（`object-position`/`clip-path`/inset + custom
//! property）で決定的に表現する想定であり、本モジュールは百分率アクセサ
//! （[`ImageCropper::x_percent`] 等）でその入力のみを提供する。
//!
//! ark-ui の ImageCropper
//! （`.agents/skills/ark-ui/references/components/form/image-cropper.md`）の
//! anatomy を参考に、Root / Viewport / Image / Selection / Handle / Grid の
//! 6 anatomy パーツと、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する crop 矩形状態機械
//! [`ImageCropper`] を提供する（[`crate::slider::Slider`] の実装様式を踏襲）。
//!
//! # 決定的な整数演算（受け入れ条件）
//!
//! - 座標系は画像の自然寸法（`image_width`/`image_height`、共に
//!   `u32`・`>= 1`）を基準とする整数ピクセル空間である。
//! - crop 矩形の不変条件: `1 <= width`（`min_size` 以上）、
//!   `x + width <= image_width`（`y`/`height`/`image_height` も同様）。
//! - アスペクト比固定時（`aspect = Some((aw, ah))`、`aw`/`ah` は正の整数）は
//!   `height` を `width` から決定的な整数丸め
//!   `(width * ah + aw / 2) / aw`（四捨五入相当の整数演算、浮動小数点を
//!   一切使わない）で導出する。導出結果が `image_height` の範囲へ収まらない
//!   場合は `width` 側を [`ImageCropper::max_width_for_aspect`] まで縮小して
//!   から再導出する（fail-closed な再クランプ、`width` が主導で `height` が
//!   従属という規則を固定する）。
//! - 中間演算はすべて `i64` で行い、`u32::MAX` 近傍の入力でも overflow を
//!   起こさない（`as i64` で取り込み、最終的に `u32`/`i64::clamp` で
//!   確定させる）。
//! - 正規化は [`ImageCropper::new`] に一元化する（[`crate::slider::Slider`]
//!   の `snap_to_step_and_clamp` と同じ「正規化の単一入口」方針）。
//!
//! # resize のアンカー規則（受け入れ条件「resize 端の決定性」）
//!
//! [`HandlePosition`] の 8 方位ごとに、反対側の辺・角をアンカー（不動点）
//! として固定し、対応する辺のみを移動してから境界クランプする。
//!
//! - `E`（東）: 左辺・`x` を固定し `width` を `dx` だけ増減。
//! - `W`（西）: 右辺（`x + width`）を固定し `x`/`width` を `dx` の符号反転で
//!   増減。
//! - `S`（南）: 上辺・`y` を固定し `height` を `dy` だけ増減。
//! - `N`（北）: 下辺（`y + height`）を固定し `y`/`height` を `dy` の符号
//!   反転で増減。
//! - 角（`Ne`/`Nw`/`Se`/`Sw`）: 対応する 2 軸（東西 + 南北）を合成する。
//! - アスペクト比固定時の従属軸規則: 角・`E`/`W` は `width` が主軸で
//!   `height` を導出する。`N`/`S` は `height` が主軸で `width` を
//!   [`ImageCropper::width_from_height_for_aspect`] から導出する（本 rustdoc
//!   「決定的な整数演算」と対称の丸め規則）。
//!
//! # dispatch アクション
//!
//! - `"move"`（payload `"dx,dy"`）: 矩形の寸法を維持したまま平行移動し、
//!   境界（`[0, image_width - width]`/`[0, image_height - height]`）へ
//!   クランプする。
//! - `"resize"`（payload `"<handle>,dx,dy"`、`<handle>` は
//!   [`HandlePosition::as_str`] の 8 値のいずれか）: 上記アンカー規則で
//!   辺を移動する。
//! - `"set"`（payload `"x,y,w,h"`）: 矩形を直接設定する（[`ImageCropper::new`]
//!   相当の正規化を経由）。
//! - `"reset"`（payload 不使用）: 初期矩形（生成時に渡した値）へ復元する。
//! - payload はクライアント由来の信頼できない入力として扱い、厳密パース +
//!   fail-closed（パース不能・不正値は no-op）。パース後は必ず [`ImageCropper::new`]
//!   相当の正規化を経由する。
//!
//! # hydration
//!
//! `data-hydrate-image-width`/`-image-height`/`-x`/`-y`/`-width`/`-height`/
//! `-aspect-w`/`-aspect-h`（`0` は `aspect = None` を表す）/`-min-size`/
//! `-initial-x`/`-initial-y`/`-initial-width`/`-initial-height`（`"reset"` の
//! 復元先）を出力する。パース不能・寸法 0・矩形はみ出し・アスペクト比
//! 不整合は [`fandhe_frontend_interactive::HydrateError`] で拒否する（panic
//! しない）。受理した値にも [`ImageCropper::new`] 相当の正規化を再適用する
//! （多層防御、[`crate::slider::Slider`] と同型）。
//!
//! # ARIA
//!
//! [`root`] は `role="group"` + `aria-roledescription="image cropper"`。
//! [`handle`] は focusable（`tabindex="0"`）+ 方位別の静的 `aria-label`
//! （[`HandlePosition::aria_label`]）を持つ。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`tabindex`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存
//!   不変条件をそのまま継承する）。
//! - 動的値（整形済み数値文字列/呼び出し側 `attrs`/`children`/`alt`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `img` の `src` に対する URL スキーム検証は行わない
//!   （[`crate::avatar`] と同じ整理。既定エスケープが属性破りを防ぎ、現代の
//!   ブラウザは `img src` の `javascript:` を実行しない。URL の妥当性検証が
//!   必要な場合はアプリ側の責務とする）。
//! - 数値属性値（hydration 属性・CSS custom property 用の百分率）は
//!   サーバー側で正規化済みの整数/`f64` の決定的文字列表現のみを出力する。
//!   任意の呼び出し側文字列をこれらの数値スロットへ直接通す経路は持たない
//!   （fail-closed 正規化は [`ImageCropper::new`] が一元的に担う）。
//! - dispatch の payload はクライアント由来の信頼できない入力として扱い、
//!   厳密パース + fail-closed（不正値は no-op）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **canvas による実画像切り出し（ピクセルデータの生成）**: 本モジュール
//!   は crop 矩形の値を返すのみで、画像バッファの実際の切り出しは対象外
//!   （`docs/design/component-coverage-map.md` に注記）。
//! - **pointer ドラッグ・キーボード nudge の DOM 配線**: 他コンポーネント
//!   同様、クライアントランタイム（`fandhe-frontend-wasm-full`）側の
//!   後続責務とする。本モジュールは SSR 静的マークアップと dispatch 契約
//!   のみを提供する。
//! - **zoom / rotation / flip / cropShape circle**（ark-ui のオプション群）。

use crate::anatomy::{anatomy, Anatomy};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// ImageCropper の anatomy（`data-scope="image-cropper"`）。
const ANATOMY: Anatomy = anatomy("image-cropper");

/// [`handle`] の 8 方位。`data-handle-position` の属性値・resize
/// アンカー規則（モジュール doc「resize のアンカー規則」参照）の両方の
/// 元になる、本モジュールで唯一の方位表現。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlePosition {
    /// 北（上辺中央）。
    N,
    /// 南（下辺中央）。
    S,
    /// 東（右辺中央）。
    E,
    /// 西（左辺中央）。
    W,
    /// 北東（右上角）。
    Ne,
    /// 北西（左上角）。
    Nw,
    /// 南東（右下角）。
    Se,
    /// 南西（左下角）。
    Sw,
}

impl HandlePosition {
    /// `data-handle-position` 属性値文字列。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::N => "n",
            Self::S => "s",
            Self::E => "e",
            Self::W => "w",
            Self::Ne => "ne",
            Self::Nw => "nw",
            Self::Se => "se",
            Self::Sw => "sw",
        }
    }

    /// [`Self::as_str`] の逆変換（dispatch payload のパース用）。未知の値は
    /// `None`（fail-closed）。
    #[must_use]
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "n" => Some(Self::N),
            "s" => Some(Self::S),
            "e" => Some(Self::E),
            "w" => Some(Self::W),
            "ne" => Some(Self::Ne),
            "nw" => Some(Self::Nw),
            "se" => Some(Self::Se),
            "sw" => Some(Self::Sw),
            _ => None,
        }
    }

    /// [`handle`] の静的 `aria-label`（方位別、`&'static str` 固定）。
    #[must_use]
    pub const fn aria_label(self) -> &'static str {
        match self {
            Self::N => "Resize from top",
            Self::S => "Resize from bottom",
            Self::E => "Resize from right",
            Self::W => "Resize from left",
            Self::Ne => "Resize from top right",
            Self::Nw => "Resize from top left",
            Self::Se => "Resize from bottom right",
            Self::Sw => "Resize from bottom left",
        }
    }

    /// 全 8 方位（テスト・網羅処理向け）。
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::N,
            Self::S,
            Self::E,
            Self::W,
            Self::Ne,
            Self::Nw,
            Self::Se,
            Self::Sw,
        ]
    }
}

/// `u32` の数値属性値を一元的に文字列化するヘルパ（[`crate::slider::fmt_num`]
/// と同型の重複実装。モジュール間の相互依存を避ける意図的な重複）。
fn fmt_u32(value: u32) -> String {
    format!("{value}")
}

/// `width` からアスペクト比 `(aw, ah)` に基づく `height` を決定的な整数
/// 丸めで導出する（モジュール doc「決定的な整数演算」参照）。
///
/// `(width * ah + aw / 2) / aw` は四捨五入相当の整数演算であり、浮動小数点
/// を一切使わない。`i64` 中間値で overflow を排除する。
fn height_from_width_for_aspect(width: u32, aw: u32, ah: u32) -> u32 {
    let width = i64::from(width);
    let aw = i64::from(aw);
    let ah = i64::from(ah);
    let derived = (width * ah + aw / 2) / aw;
    derived.clamp(0, i64::from(u32::MAX)) as u32
}

/// [`height_from_width_for_aspect`] と対称の `height` 主軸版
/// （`N`/`S` handle の resize 従属軸規則、モジュール doc「resize の
/// アンカー規則」参照）。
fn width_from_height_for_aspect(height: u32, aw: u32, ah: u32) -> u32 {
    let height = i64::from(height);
    let aw = i64::from(aw);
    let ah = i64::from(ah);
    let derived = (height * aw + ah / 2) / ah;
    derived.clamp(0, i64::from(u32::MAX)) as u32
}

/// `[low, high]` の範囲で `height_from_width_for_aspect(w, aw, ah) <= cap`
/// を満たす最大の `w` を二分探索で求める（[`normalize`] の再クランプ経路
/// 専用ヘルパ）。
///
/// `height_from_width_for_aspect` は `w` に対して単調非減少なので二分探索
/// が成立する。[`width_from_height_for_aspect`] による一発導出は四捨五入の
/// 丸め誤差により逆算結果を [`height_from_width_for_aspect`] へ通し直すと
/// 再び `cap` を超えうる（イシュー #844 レビュー指摘の実バグの根本原因）。
/// 二分探索で「実際に導出した height が cap 以下」という条件そのものを
/// 満たす `w` を直接探すことで、呼び出し側は必ず
/// `height_from_width_for_aspect(戻り値, aw, ah) <= cap`（`low` において
/// も満たせない極端なアスペクト比の場合を除く）を得られ、
/// [`Hydrate::from_hydration_attrs`] の等値検証と自己無矛盾になる。
/// `low` ですら `cap` を満たせない場合は `low` を返す（呼び出し側は
/// `low` から実際に導出される height をそのまま採用する）。
fn max_width_with_height_at_most(low: u32, high: u32, cap: u32, aw: u32, ah: u32) -> u32 {
    if height_from_width_for_aspect(low, aw, ah) > cap {
        return low;
    }
    let mut lo = low;
    let mut hi = high;
    while lo < hi {
        // オーバーフロー回避のため u32 のまま `lo + (hi - lo + 1) / 2` で
        // 中点を計算する（`lo <= hi` 不変条件下では加算オーバーフローしない）。
        let mid = lo + (hi - lo).div_ceil(2);
        if height_from_width_for_aspect(mid, aw, ah) <= cap {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

/// [`max_width_with_height_at_most`] と対称の下限側版。`[low, high]` の
/// 範囲で `height_from_width_for_aspect(w, aw, ah) >= floor` を満たす
/// 最小の `w` を二分探索で求める（[`normalize`] の再クランプ経路専用
/// ヘルパ。導出結果の自己無矛盾性を保証する理由は
/// [`max_width_with_height_at_most`] 参照）。
fn min_width_with_height_at_least(low: u32, high: u32, floor: u32, aw: u32, ah: u32) -> u32 {
    if height_from_width_for_aspect(high, aw, ah) < floor {
        return high;
    }
    let mut lo = low;
    let mut hi = high;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if height_from_width_for_aspect(mid, aw, ah) >= floor {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

/// crop 矩形の正規化済みフィールド。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// `image_width`/`image_height`/`min_size`/`aspect` の制約下で
/// `(x, y, width, height)` を fail-closed に正規化する
/// （[`ImageCropper::new`] の実体。モジュール doc「決定的な整数演算」参照）。
///
/// - `width`/`height` は `[min_size, image_width]`/`[min_size, image_height]`
///   へクランプする（`min_size` が `image_width`/`image_height` を超える
///   異常入力は `image_width`/`image_height` 側を優先し矩形全体を採用する）。
/// - アスペクト比固定時は `width` から `height` を導出し、`image_height` を
///   超える場合は `width` を [`max_width_for_aspect`] まで縮めて再導出する
///   （fail-closed な再クランプ）。
/// - `x`/`y` は `[0, image_dim - size]` へクランプする。
#[allow(clippy::too_many_arguments)]
fn normalize(
    image_width: u32,
    image_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    aspect: Option<(u32, u32)>,
    min_size: u32,
) -> (u32, u32, u32, Rect) {
    let image_width = image_width.max(1);
    let image_height = image_height.max(1);
    let min_size = min_size.max(1);

    let aspect = aspect.filter(|(aw, ah)| *aw > 0 && *ah > 0);

    let max_width = image_width;
    let min_width = min_size.min(max_width);
    let mut width = width.clamp(min_width, max_width);

    let height = if let Some((aw, ah)) = aspect {
        let min_height_bound = min_size.min(image_height);
        let mut derived = height_from_width_for_aspect(width, aw, ah);
        if derived > image_height {
            // 導出結果が image_height を超える場合、height <= image_height を
            // 満たす最大の width を二分探索で直接求める（fail-closed。width
            // が主軸という規則をここでも維持する）。
            //
            // 以前は width_from_height_for_aspect(image_height, ..) の一発
            // 逆算のみで再クランプしていたが、四捨五入の丸め誤差により
            // 逆算した width から height_from_width_for_aspect を再適用する
            // と再び image_height を超えることがあり（例:
            // image_width=2000, image_height=1000, width=800, aspect=2:3 で
            // width=667 へ再クランプ後 height=1001 → 最終 clamp で 1000 に
            // 潰され、height_from_width_for_aspect(667,2,3)=1001 != 1000 と
            // いう自己矛盾状態が生成されていた）、その状態を
            // `hydration_attrs()` で SSR 出力すると
            // `Hydrate::from_hydration_attrs` の厳密等値検査
            // （height == height_from_width_for_aspect(width, aw, ah)）に
            // 自身が違反し `InvalidValue` で拒否される実バグがあった
            // （イシュー #844 レビュー指摘）。二分探索で「実際に導出した
            // height が image_height 以下」という条件そのものを満たす
            // width を求めることで、最終的に採用する (width, height) は
            // 常に height == height_from_width_for_aspect(width, aw, ah) を
            // 満たし、from_hydration_attrs の等値検証と自己無矛盾になる。
            width = max_width_with_height_at_most(min_width, max_width, image_height, aw, ah);
            derived = height_from_width_for_aspect(width, aw, ah);
        } else if derived < min_height_bound {
            // 対称のケース: 導出結果が下限（min_size/image_height の小さい
            // 方）を下回る場合、height >= min_height_bound を満たす最小の
            // width を二分探索で求める（上記と同じ理由で自己無矛盾性を
            // 保証するため、一発逆算 + 事後 clamp を避ける）。
            width = min_width_with_height_at_least(min_width, max_width, min_height_bound, aw, ah);
            derived = height_from_width_for_aspect(width, aw, ah);
        }
        derived
    } else {
        let max_height = image_height;
        let min_height = min_size.min(max_height);
        height.clamp(min_height, max_height)
    };

    let x = x.min(image_width.saturating_sub(width));
    let y = y.min(image_height.saturating_sub(height));

    (
        image_width,
        image_height,
        min_size,
        Rect {
            x,
            y,
            width,
            height,
        },
    )
}

/// Root パーツ（`div`）。`role="group"` + `aria-roledescription` で
/// WAI-ARIA の一般的なグルーピングパターンに従う（ImageCropper 専用の
/// APG パターンは存在しないため、chakra-ui 実装に倣い group ロールを使う）。
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("role", "group"), ("aria-roledescription", "image cropper")];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Viewport パーツ（`div`）。画像・selection・grid を包含するクリップ
/// コンテナ（styled 層が `overflow: hidden` を付与する想定）。
#[must_use]
pub fn viewport<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("viewport", "div", attrs, children)
}

/// Image パーツ（`img`）。`src`/`alt` を必須引数とする（[`crate::avatar::image`]
/// と同じくアクセシビリティ担保のため）。
#[must_use]
pub fn image<'a>(src: &'a str, alt: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("src", src), ("alt", alt)];
    merged.extend(attrs);
    ANATOMY.part("image", "img", merged, Vec::new())
}

/// Selection パーツ（`div`）。crop 矩形の視覚的な枠。位置・寸法は
/// 呼び出し側/styled 層が [`ImageCropper::x_percent`] 等から `style` を
/// 組み立てて `attrs` 経由で渡す（headless 中立、[`crate::slider::range`]
/// と同型の判断。本関数自体は `style` を持たない）。
#[must_use]
pub fn selection<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("selection", "div", attrs, children)
}

/// Handle パーツ（`div`）。8 方位のリサイズハンドル。`data-handle-position`・
/// focusable（`tabindex="0"`）・方位別 `aria-label` を出力する。
#[must_use]
pub fn handle<'a>(position: HandlePosition, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-handle-position", position.as_str()),
        ("tabindex", "0"),
        ("aria-label", position.aria_label()),
    ];
    merged.extend(attrs);
    ANATOMY.part("handle", "div", merged, Vec::new())
}

/// Grid パーツ（`div`）。3 分割ガイド線の装飾用コンテナ（styled 層が
/// `linear-gradient` 等で描画する。headless 層は anatomy のみ）。
#[must_use]
pub fn grid<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    ANATOMY.part("grid", "div", attrs, Vec::new())
}

/// ImageCropper のアクション（WASM 境界の文字列 dispatch と
/// [`ImageCropper::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCropperAction {
    /// 矩形の寸法を維持したまま平行移動する（境界クランプ込み）。
    Move {
        /// x 方向の移動量。
        dx: i32,
        /// y 方向の移動量。
        dy: i32,
    },
    /// 指定した handle をアンカー規則に従いリサイズする。
    Resize {
        /// リサイズするハンドルの方位。
        handle: HandlePosition,
        /// x 方向の移動量。
        dx: i32,
        /// y 方向の移動量。
        dy: i32,
    },
    /// 矩形を直接設定する（[`ImageCropper::new`] 相当の正規化を経由）。
    Set {
        /// 左端 x 座標。
        x: u32,
        /// 上端 y 座標。
        y: u32,
        /// 幅。
        width: u32,
        /// 高さ。
        height: u32,
    },
    /// 初期矩形（生成時に渡した値）へ復元する。
    Reset,
}

/// ImageCropper の crop 矩形状態機械。
///
/// canvas・ピクセル・ポインタ座標を一切保持しない（モジュール doc
/// 「非採用理由を設計で無効化する」参照）。`Default` は
/// `image_width=100, image_height=100, x=0, y=0, width=100, height=100,
/// aspect=None, min_size=1`（SSR の初期描画に対応する既定値。画像全体を
/// 選択した状態）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageCropper {
    image_width: u32,
    image_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    aspect: Option<(u32, u32)>,
    min_size: u32,
    // "reset" dispatch / hydration ラウンドトリップの復元先（生成時に渡した
    // 初期矩形。正規化済みの `Rect` として保持する）。
    initial: Rect,
}

impl Default for ImageCropper {
    fn default() -> Self {
        Self::new(100, 100, 0, 0, 100, 100, None, 1)
    }
}

impl ImageCropper {
    /// `data-hydrate-image-width` 属性名のフィールド部分。
    pub const FIELD_IMAGE_WIDTH: &'static str = "image-width";
    /// `data-hydrate-image-height` 属性名のフィールド部分。
    pub const FIELD_IMAGE_HEIGHT: &'static str = "image-height";
    /// `data-hydrate-x` 属性名のフィールド部分。
    pub const FIELD_X: &'static str = "x";
    /// `data-hydrate-y` 属性名のフィールド部分。
    pub const FIELD_Y: &'static str = "y";
    /// `data-hydrate-width` 属性名のフィールド部分。
    pub const FIELD_WIDTH: &'static str = "width";
    /// `data-hydrate-height` 属性名のフィールド部分。
    pub const FIELD_HEIGHT: &'static str = "height";
    /// `data-hydrate-aspect-w` 属性名のフィールド部分（`0` は `None`）。
    pub const FIELD_ASPECT_W: &'static str = "aspect-w";
    /// `data-hydrate-aspect-h` 属性名のフィールド部分（`0` は `None`）。
    pub const FIELD_ASPECT_H: &'static str = "aspect-h";
    /// `data-hydrate-min-size` 属性名のフィールド部分。
    pub const FIELD_MIN_SIZE: &'static str = "min-size";
    /// `data-hydrate-initial-x` 属性名のフィールド部分。
    pub const FIELD_INITIAL_X: &'static str = "initial-x";
    /// `data-hydrate-initial-y` 属性名のフィールド部分。
    pub const FIELD_INITIAL_Y: &'static str = "initial-y";
    /// `data-hydrate-initial-width` 属性名のフィールド部分。
    pub const FIELD_INITIAL_WIDTH: &'static str = "initial-width";
    /// `data-hydrate-initial-height` 属性名のフィールド部分。
    pub const FIELD_INITIAL_HEIGHT: &'static str = "initial-height";

    /// 指定した値で [`ImageCropper`] を生成する（[`normalize`] で
    /// fail-closed 正規化する。呼び出し側の不正な入力で panic しない）。
    /// 正規化後の矩形が `"reset"` dispatch・hydration ラウンドトリップの
    /// 復元先（`initial`）としても保存される。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        image_width: u32,
        image_height: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        aspect: Option<(u32, u32)>,
        min_size: u32,
    ) -> Self {
        let (image_width, image_height, min_size, rect) = normalize(
            image_width,
            image_height,
            x,
            y,
            width,
            height,
            aspect,
            min_size,
        );
        Self {
            image_width,
            image_height,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            aspect,
            min_size,
            initial: rect,
        }
    }

    /// 画像の自然幅。
    #[must_use]
    pub fn image_width(&self) -> u32 {
        self.image_width
    }

    /// 画像の自然高さ。
    #[must_use]
    pub fn image_height(&self) -> u32 {
        self.image_height
    }

    /// crop 矩形の左端 x 座標。
    #[must_use]
    pub fn x(&self) -> u32 {
        self.x
    }

    /// crop 矩形の上端 y 座標。
    #[must_use]
    pub fn y(&self) -> u32 {
        self.y
    }

    /// crop 矩形の幅。
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// crop 矩形の高さ。
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 固定アスペクト比（`(width, height)` の整数比）。`None` は自由な
    /// アスペクト比。
    #[must_use]
    pub fn aspect(&self) -> Option<(u32, u32)> {
        self.aspect
    }

    /// crop 矩形の最小辺長。
    #[must_use]
    pub fn min_size(&self) -> u32 {
        self.min_size
    }

    /// `image_width` に対する `x` の百分率（`0.0..=100.0`）。CSS custom
    /// property へ渡す唯一の想定経路（styled 層が組み立てる）。
    #[must_use]
    pub fn x_percent(&self) -> f64 {
        f64::from(self.x) / f64::from(self.image_width) * 100.0
    }

    /// `image_height` に対する `y` の百分率（`0.0..=100.0`）。
    #[must_use]
    pub fn y_percent(&self) -> f64 {
        f64::from(self.y) / f64::from(self.image_height) * 100.0
    }

    /// `image_width` に対する `width` の百分率（`0.0..=100.0`）。
    #[must_use]
    pub fn width_percent(&self) -> f64 {
        f64::from(self.width) / f64::from(self.image_width) * 100.0
    }

    /// `image_height` に対する `height` の百分率（`0.0..=100.0`）。
    #[must_use]
    pub fn height_percent(&self) -> f64 {
        f64::from(self.height) / f64::from(self.image_height) * 100.0
    }

    /// [`root`] へ委譲する利便メソッド（状態を持たない装飾用パーツ）。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(attrs, children)
    }

    /// [`viewport`] へ委譲する利便メソッド。
    #[must_use]
    pub fn viewport<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        viewport(attrs, children)
    }

    /// [`image`] へ委譲する利便メソッド。
    #[must_use]
    pub fn image<'a>(&self, src: &'a str, alt: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
        image(src, alt, attrs)
    }

    /// [`selection`] へ委譲する利便メソッド。
    #[must_use]
    pub fn selection<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        selection(attrs, children)
    }

    /// [`handle`] へ委譲する利便メソッド。
    #[must_use]
    pub fn handle<'a>(&self, position: HandlePosition, attrs: Vec<(&'a str, &'a str)>) -> Node {
        handle(position, attrs)
    }

    /// [`grid`] へ委譲する利便メソッド。
    #[must_use]
    pub fn grid<'a>(&self, attrs: Vec<(&'a str, &'a str)>) -> Node {
        grid(attrs)
    }

    /// 現在のフィールド一式を [`normalize`] へ通して再正規化した新しい
    /// 矩形を返す（`update()` の各アクションが共有する内部ヘルパ）。
    /// `initial`/`aspect`/`min_size`/`image_width`/`image_height` は
    /// 変更しない。
    fn renormalize(&self, x: u32, y: u32, width: u32, height: u32) -> Rect {
        let (_, _, _, rect) = normalize(
            self.image_width,
            self.image_height,
            x,
            y,
            width,
            height,
            self.aspect,
            self.min_size,
        );
        rect
    }

    /// アスペクト比固定時にこの矩形が取りうる `width` の上限
    /// （`image_height` の制約から逆算、モジュール doc「決定的な整数演算」
    /// 参照）。アスペクト比が `None` の場合は `image_width` を返す。
    #[must_use]
    fn max_width_for_aspect(&self) -> u32 {
        match self.aspect {
            Some((aw, ah)) => {
                width_from_height_for_aspect(self.image_height, aw, ah).min(self.image_width)
            }
            None => self.image_width,
        }
    }
}

impl Component for ImageCropper {
    type Action = ImageCropperAction;

    fn update(&mut self, action: ImageCropperAction) {
        match action {
            ImageCropperAction::Move { dx, dy } => {
                let new_x = i64::from(self.x) + i64::from(dx);
                let new_y = i64::from(self.y) + i64::from(dy);
                let new_x = new_x.clamp(0, i64::from(u32::MAX)) as u32;
                let new_y = new_y.clamp(0, i64::from(u32::MAX)) as u32;
                let rect = self.renormalize(new_x, new_y, self.width, self.height);
                self.x = rect.x;
                self.y = rect.y;
                self.width = rect.width;
                self.height = rect.height;
            }
            ImageCropperAction::Resize { handle, dx, dy } => {
                self.apply_resize(handle, dx, dy);
            }
            ImageCropperAction::Set {
                x,
                y,
                width,
                height,
            } => {
                let rect = self.renormalize(x, y, width, height);
                self.x = rect.x;
                self.y = rect.y;
                self.width = rect.width;
                self.height = rect.height;
            }
            ImageCropperAction::Reset => {
                self.x = self.initial.x;
                self.y = self.initial.y;
                self.width = self.initial.width;
                self.height = self.initial.height;
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// viewport > (image, selection > handle)）。公開 UI としての利用は
    /// 想定しない。
    fn view(&self) -> Node {
        self.root(
            Vec::new(),
            vec![self.viewport(
                Vec::new(),
                vec![
                    self.image("", "", Vec::new()),
                    self.selection(
                        Vec::new(),
                        vec![self.handle(HandlePosition::Se, Vec::new())],
                    ),
                ],
            )],
        )
    }

    /// `"move"`: payload `"dx,dy"`（`i32` 2 値）をパース。
    /// `"resize"`: payload `"<handle>,dx,dy"`（`HandlePosition` 文字列 +
    /// `i32` 2 値）をパース。`"set"`: payload `"x,y,w,h"`（`u32` 4 値）を
    /// パース。`"reset"`: payload 不使用。いずれもパース不能・要素数不一致は
    /// `None`（fail-closed、dispatch は no-op）。
    fn decode_action(name: &str, payload: &str) -> Option<ImageCropperAction> {
        match name {
            "move" => {
                let mut parts = payload.split(',');
                let dx = parts.next()?.parse::<i32>().ok()?;
                let dy = parts.next()?.parse::<i32>().ok()?;
                if parts.next().is_some() {
                    return None;
                }
                Some(ImageCropperAction::Move { dx, dy })
            }
            "resize" => {
                let mut parts = payload.split(',');
                let handle = HandlePosition::from_str_value(parts.next()?)?;
                let dx = parts.next()?.parse::<i32>().ok()?;
                let dy = parts.next()?.parse::<i32>().ok()?;
                if parts.next().is_some() {
                    return None;
                }
                Some(ImageCropperAction::Resize { handle, dx, dy })
            }
            "set" => {
                let mut parts = payload.split(',');
                let x = parts.next()?.parse::<u32>().ok()?;
                let y = parts.next()?.parse::<u32>().ok()?;
                let width = parts.next()?.parse::<u32>().ok()?;
                let height = parts.next()?.parse::<u32>().ok()?;
                if parts.next().is_some() {
                    return None;
                }
                Some(ImageCropperAction::Set {
                    x,
                    y,
                    width,
                    height,
                })
            }
            "reset" => Some(ImageCropperAction::Reset),
            _ => None,
        }
    }
}

impl ImageCropper {
    /// [`ImageCropperAction::Resize`] の実体（モジュール doc「resize の
    /// アンカー規則」参照）。反対側の辺・角を不動点として固定し、対応する
    /// 辺のみを移動してから [`renormalize`](Self::renormalize) で境界
    /// クランプする。
    fn apply_resize(&mut self, handle: HandlePosition, dx: i32, dy: i32) {
        let x = i64::from(self.x);
        let y = i64::from(self.y);
        let w = i64::from(self.width);
        let h = i64::from(self.height);
        let dx = i64::from(dx);
        let dy = i64::from(dy);

        // アスペクト比固定時、E/W/角は width が主軸、N/S は height が主軸
        // （モジュール doc「resize のアンカー規則」参照）。まず主軸側の
        // 生の移動量を適用し、renormalize が従属軸を導出・クランプする。
        let (new_x, new_y, new_w, new_h) = match handle {
            HandlePosition::E => (x, y, w + dx, h),
            HandlePosition::W => (x + dx, y, w - dx, h),
            HandlePosition::S => (x, y, w, h + dy),
            HandlePosition::N => (x, y + dy, w, h - dy),
            HandlePosition::Se => (x, y, w + dx, h + dy),
            HandlePosition::Sw => (x + dx, y, w - dx, h + dy),
            HandlePosition::Ne => (x, y + dy, w + dx, h - dy),
            HandlePosition::Nw => (x + dx, y + dy, w - dx, h - dy),
        };

        let clamp_u32 = |v: i64| v.clamp(0, i64::from(u32::MAX)) as u32;
        let new_x = clamp_u32(new_x);
        let new_y = clamp_u32(new_y);
        let new_w = clamp_u32(new_w);
        let new_h = clamp_u32(new_h);

        // N/S（純縦方向）はアスペクト比固定時に height を主軸として扱う
        // （モジュール doc「resize のアンカー規則」参照）。normalize は
        // width 主軸の導出しかサポートしないため、
        // `width_from_height_for_aspect` で height から width を逆算して
        // から渡すことで、実質的に height を主軸として振る舞わせる。
        let is_ns_with_aspect = matches!(handle, HandlePosition::N | HandlePosition::S);
        let rect = match (is_ns_with_aspect, self.aspect) {
            (true, Some((aw, ah))) => {
                let derived_w =
                    width_from_height_for_aspect(new_h, aw, ah).min(self.max_width_for_aspect());
                self.renormalize(new_x, new_y, derived_w, new_h)
            }
            _ => self.renormalize(new_x, new_y, new_w, new_h),
        };

        self.x = rect.x;
        self.y = rect.y;
        self.width = rect.width;
        self.height = rect.height;
    }
}

impl Hydrate for ImageCropper {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let (aspect_w, aspect_h) = self.aspect.unwrap_or((0, 0));
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IMAGE_WIDTH),
                fmt_u32(self.image_width),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IMAGE_HEIGHT),
                fmt_u32(self.image_height),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_X),
                fmt_u32(self.x),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_Y),
                fmt_u32(self.y),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_WIDTH),
                fmt_u32(self.width),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_HEIGHT),
                fmt_u32(self.height),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ASPECT_W),
                fmt_u32(aspect_w),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ASPECT_H),
                fmt_u32(aspect_h),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN_SIZE),
                fmt_u32(self.min_size),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INITIAL_X),
                fmt_u32(self.initial.x),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INITIAL_Y),
                fmt_u32(self.initial.y),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INITIAL_WIDTH),
                fmt_u32(self.initial.width),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INITIAL_HEIGHT),
                fmt_u32(self.initial.height),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・寸法 0・矩形はみ出し・
    /// アスペクト比不整合は [`HydrateError::InvalidValue`]（panic しない）。
    /// 受理した値はさらに [`normalize`] へ通してから復元する（多層防御。
    /// [`crate::slider::Slider`] と同型の fail-closed 契約）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let parse_u32 = |field: &str, raw: &str| -> Result<u32, HydrateError> {
            raw.parse::<u32>().map_err(|_| HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{field}"),
                reason: "expected a non-negative integer".to_string(),
            })
        };

        let image_width = parse_u32(Self::FIELD_IMAGE_WIDTH, find(Self::FIELD_IMAGE_WIDTH)?)?;
        let image_height = parse_u32(Self::FIELD_IMAGE_HEIGHT, find(Self::FIELD_IMAGE_HEIGHT)?)?;
        if image_width == 0 || image_height == 0 {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IMAGE_WIDTH),
                reason: "expected image dimensions >= 1".to_string(),
            });
        }

        let x = parse_u32(Self::FIELD_X, find(Self::FIELD_X)?)?;
        let y = parse_u32(Self::FIELD_Y, find(Self::FIELD_Y)?)?;
        let width = parse_u32(Self::FIELD_WIDTH, find(Self::FIELD_WIDTH)?)?;
        let height = parse_u32(Self::FIELD_HEIGHT, find(Self::FIELD_HEIGHT)?)?;

        let aspect_w = parse_u32(Self::FIELD_ASPECT_W, find(Self::FIELD_ASPECT_W)?)?;
        let aspect_h = parse_u32(Self::FIELD_ASPECT_H, find(Self::FIELD_ASPECT_H)?)?;
        let aspect = if aspect_w == 0 && aspect_h == 0 {
            None
        } else if aspect_w > 0 && aspect_h > 0 {
            Some((aspect_w, aspect_h))
        } else {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ASPECT_W),
                reason: "expected both aspect-w and aspect-h to be zero or both positive"
                    .to_string(),
            });
        };

        let min_size = parse_u32(Self::FIELD_MIN_SIZE, find(Self::FIELD_MIN_SIZE)?)?;
        if min_size == 0 {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN_SIZE),
                reason: "expected min-size >= 1".to_string(),
            });
        }

        if width < min_size || height < min_size {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_WIDTH),
                reason: "expected width/height >= min-size".to_string(),
            });
        }
        if x.checked_add(width).is_none_or(|sum| sum > image_width) {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_X),
                reason: "expected x + width <= image-width".to_string(),
            });
        }
        if y.checked_add(height).is_none_or(|sum| sum > image_height) {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_Y),
                reason: "expected y + height <= image-height".to_string(),
            });
        }
        if let Some((aw, ah)) = aspect {
            let expected_height = height_from_width_for_aspect(width, aw, ah);
            if expected_height != height {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_HEIGHT),
                    reason: "expected height to match width * aspect-h / aspect-w".to_string(),
                });
            }
        }

        let initial_x = parse_u32(Self::FIELD_INITIAL_X, find(Self::FIELD_INITIAL_X)?)?;
        let initial_y = parse_u32(Self::FIELD_INITIAL_Y, find(Self::FIELD_INITIAL_Y)?)?;
        let initial_width = parse_u32(Self::FIELD_INITIAL_WIDTH, find(Self::FIELD_INITIAL_WIDTH)?)?;
        let initial_height = parse_u32(
            Self::FIELD_INITIAL_HEIGHT,
            find(Self::FIELD_INITIAL_HEIGHT)?,
        )?;
        if initial_width < min_size || initial_height < min_size {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INITIAL_WIDTH),
                reason: "expected initial width/height >= min-size".to_string(),
            });
        }
        if initial_x
            .checked_add(initial_width)
            .is_none_or(|sum| sum > image_width)
        {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INITIAL_X),
                reason: "expected initial x + width <= image-width".to_string(),
            });
        }
        if initial_y
            .checked_add(initial_height)
            .is_none_or(|sum| sum > image_height)
        {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INITIAL_Y),
                reason: "expected initial y + height <= image-height".to_string(),
            });
        }

        // 多層防御: 基本検証を通過した値にも normalize を再適用する
        // （モジュール doc「hydration」参照。現在値・初期値の双方に適用）。
        let (image_width, image_height, min_size, rect) = normalize(
            image_width,
            image_height,
            x,
            y,
            width,
            height,
            aspect,
            min_size,
        );
        let (_, _, _, initial_rect) = normalize(
            image_width,
            image_height,
            initial_x,
            initial_y,
            initial_width,
            initial_height,
            aspect,
            min_size,
        );

        Ok(Self {
            image_width,
            image_height,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            aspect,
            min_size,
            initial: initial_rect,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part 出力 ---

    #[test]
    fn root_outputs_scope_part_role_and_roledescription() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="image-cropper""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-roledescription="image cropper""#));
    }

    #[test]
    fn viewport_outputs_scope_and_part() {
        let html = render(&viewport(vec![], vec![]));
        assert!(html.contains(r#"data-part="viewport""#));
    }

    #[test]
    fn image_outputs_src_and_alt() {
        let html = render(&image("/a.png", "photo", vec![]));
        assert!(html.contains(r#"data-part="image""#));
        assert!(html.contains(r#"src="/a.png""#));
        assert!(html.contains(r#"alt="photo""#));
    }

    #[test]
    fn selection_outputs_scope_and_part() {
        let html = render(&selection(vec![], vec![text("x")]));
        assert!(html.contains(r#"data-part="selection""#));
        assert!(html.contains('x'));
    }

    #[test]
    fn handle_outputs_position_tabindex_and_aria_label() {
        let html = render(&handle(HandlePosition::Se, vec![]));
        assert!(html.contains(r#"data-part="handle""#));
        assert!(html.contains(r#"data-handle-position="se""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"aria-label="Resize from bottom right""#));
    }

    #[test]
    fn handle_position_as_str_and_from_str_value_round_trip() {
        for pos in HandlePosition::all() {
            assert_eq!(HandlePosition::from_str_value(pos.as_str()), Some(pos));
        }
        assert_eq!(HandlePosition::from_str_value("nesw"), None);
    }

    #[test]
    fn grid_outputs_scope_and_part() {
        let html = render(&grid(vec![]));
        assert!(html.contains(r#"data-part="grid""#));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="image-cropper""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_clamps_out_of_range_rect() {
        let c = ImageCropper::new(100, 100, 200, 200, 200, 200, None, 1);
        assert_eq!((c.x(), c.y(), c.width(), c.height()), (0, 0, 100, 100));
    }

    #[test]
    fn new_clamps_zero_dimensions_to_at_least_one() {
        let c = ImageCropper::new(0, 0, 0, 0, 0, 0, None, 1);
        assert_eq!(c.image_width(), 1);
        assert_eq!(c.image_height(), 1);
        assert_eq!(c.width(), 1);
        assert_eq!(c.height(), 1);
    }

    #[test]
    fn new_keeps_x_y_within_bounds_after_width_height_clamp() {
        let c = ImageCropper::new(100, 100, 90, 90, 50, 50, None, 1);
        // x=90, width=50 は image_width=100 を超えるため x は 50 へクランプ。
        assert_eq!(c.x(), 50);
        assert_eq!(c.y(), 50);
        assert_eq!(c.width(), 50);
        assert_eq!(c.height(), 50);
    }

    #[test]
    fn default_is_full_image_selection() {
        let c = ImageCropper::default();
        assert_eq!(c.image_width(), 100);
        assert_eq!(c.image_height(), 100);
        assert_eq!((c.x(), c.y(), c.width(), c.height()), (0, 0, 100, 100));
        assert_eq!(c.aspect(), None);
        assert_eq!(c.min_size(), 1);
    }

    // --- アスペクト比の決定的丸め（網羅表） ---

    #[test]
    fn aspect_ratio_derives_height_deterministically() {
        // (width, aw, ah) -> expected height。同一入力は常に同一出力。
        let cases = [
            (100u32, 1u32, 1u32, 100u32),
            (100, 16, 9, 56), // 100*9/16 = 56.25 -> 四捨五入で 56
            (100, 4, 3, 75),
            (99, 16, 9, 56), // 99*9/16 = 55.6875 -> 四捨五入で 56
            (16, 16, 9, 9),
        ];
        for (width, aw, ah, expected) in cases {
            assert_eq!(
                height_from_width_for_aspect(width, aw, ah),
                expected,
                "width={width} aw={aw} ah={ah}"
            );
        }
    }

    #[test]
    fn aspect_ratio_rounding_is_idempotent_and_deterministic() {
        for _ in 0..3 {
            assert_eq!(height_from_width_for_aspect(100, 16, 9), 56);
        }
    }

    #[test]
    fn new_applies_fixed_aspect_ratio_to_height() {
        let c = ImageCropper::new(1000, 1000, 0, 0, 100, 999, Some((16, 9)), 1);
        assert_eq!(c.width(), 100);
        assert_eq!(c.height(), 56);
    }

    #[test]
    fn new_shrinks_width_when_aspect_height_exceeds_image_height() {
        // image_height=50 に対し width=100, aspect=1:1 だと height=100 は
        // 収まらないため width 側が縮小される（fail-closed 再クランプ）。
        let c = ImageCropper::new(1000, 50, 0, 0, 100, 100, Some((1, 1)), 1);
        assert!(c.height() <= 50);
        assert_eq!(c.width(), c.height()); // 1:1 のため width == height
    }

    #[test]
    fn new_shrinks_width_when_asymmetric_aspect_height_exceeds_image_height() {
        // 非対称アスペクト比（2:3）+ width が image_height 側で再クランプ
        // される境界条件（イシュー #844 レビュー指摘の実バグの再現ケース）。
        // 丸め誤差により width_from_height_for_aspect の一発逆算だけでは
        // height_from_width_for_aspect(width, aw, ah) と最終 height が
        // 一致しない自己矛盾状態が生成されていた（width=667 へ再クランプ
        // 後 height_from_width_for_aspect(667,2,3)=1001 が image_height=1000
        // へ最終 clamp され height=1000 との不整合が生じる）。
        let c = ImageCropper::new(2000, 1000, 0, 0, 800, 800, Some((2, 3)), 1);
        assert!(c.height() <= 1000);
        assert_eq!(
            height_from_width_for_aspect(c.width(), 2, 3),
            c.height(),
            "width={} と height={} が自己無矛盾でない",
            c.width(),
            c.height()
        );

        // SSR → hydration ラウンドトリップが自己矛盾で fail-closed 拒否
        // されないことを確認する（サーバーが自ら生成した正当な出力を
        // 同じコンポーネントの hydration が拒否してはならない）。
        let attrs = c.hydration_attrs();
        let restored = ImageCropper::from_hydration_attrs(&attrs)
            .expect("正規化済みの hydration 属性は必ず受理されるべき");
        assert_eq!(restored.width(), c.width());
        assert_eq!(restored.height(), c.height());
    }

    #[test]
    fn new_ignores_incomplete_aspect_ratio() {
        // aw/ah のどちらかが 0 のアスペクト比は無効として扱われ、
        // 自由なアスペクト比として処理される（fail-closed）。
        let c = ImageCropper::new(100, 100, 0, 0, 50, 30, Some((0, 5)), 1);
        assert_eq!((c.width(), c.height()), (50, 30));
    }

    // --- dispatch: move ---

    #[test]
    fn dispatch_move_translates_rect_and_clamps_at_bounds() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        assert!(dispatch(&mut c, "move", "5,5"));
        assert_eq!((c.x(), c.y()), (15, 15));

        assert!(dispatch(&mut c, "move", "-100,-100"));
        assert_eq!((c.x(), c.y()), (0, 0));

        assert!(dispatch(&mut c, "move", "1000,1000"));
        assert_eq!((c.x(), c.y()), (80, 80)); // image_width/height - width/height
    }

    #[test]
    fn dispatch_move_preserves_dimensions() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 30, None, 1);
        assert!(dispatch(&mut c, "move", "5,5"));
        assert_eq!((c.width(), c.height()), (20, 30));
    }

    #[test]
    fn dispatch_move_rejects_invalid_payload() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        for bogus in ["", "abc,1", "1", "1,2,3", "1.5,2"] {
            assert!(!dispatch(&mut c, "move", bogus));
        }
        assert_eq!((c.x(), c.y()), (10, 10));
    }

    // --- dispatch: resize（8 方位の網羅表、アンカー固定の検証） ---

    #[test]
    fn dispatch_resize_east_grows_width_keeping_left_edge_anchored() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        assert!(dispatch(&mut c, "resize", "e,5,0"));
        assert_eq!(c.x(), 10); // 左辺（アンカー）不動
        assert_eq!(c.width(), 25);
    }

    #[test]
    fn dispatch_resize_west_grows_width_keeping_right_edge_anchored() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        let right_edge = c.x() + c.width();
        assert!(dispatch(&mut c, "resize", "w,-5,0"));
        assert_eq!(c.x() + c.width(), right_edge); // 右辺（アンカー）不動
        assert_eq!(c.width(), 25);
        assert_eq!(c.x(), 5);
    }

    #[test]
    fn dispatch_resize_south_grows_height_keeping_top_edge_anchored() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        assert!(dispatch(&mut c, "resize", "s,0,5"));
        assert_eq!(c.y(), 10); // 上辺（アンカー）不動
        assert_eq!(c.height(), 25);
    }

    #[test]
    fn dispatch_resize_north_grows_height_keeping_bottom_edge_anchored() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        let bottom_edge = c.y() + c.height();
        assert!(dispatch(&mut c, "resize", "n,0,-5"));
        assert_eq!(c.y() + c.height(), bottom_edge); // 下辺（アンカー）不動
        assert_eq!(c.height(), 25);
        assert_eq!(c.y(), 5);
    }

    #[test]
    fn dispatch_resize_corners_compose_both_axes() {
        // 各角を「外側（中心から離れる方向）へ引っ張って矩形を拡大する」
        // dx/dy の符号を選ぶ（west 側は dx が負、north 側は dy が負のとき
        // 矩形が拡大する。モジュール doc「resize のアンカー規則」参照）。
        /// (handle 文字列, dx, dy, 期待 (x, y), 期待 (width, height))。
        type CornerCase = (&'static str, i32, i32, (u32, u32), (u32, u32));
        let cases: [CornerCase; 4] = [
            ("se", 5, 5, (10, 10), (25, 25)),
            ("sw", -5, 5, (5, 10), (25, 25)),
            ("ne", 5, -5, (10, 5), (25, 25)),
            ("nw", -5, -5, (5, 5), (25, 25)),
        ];
        for (handle_str, dx, dy, expected_xy, expected_wh) in cases {
            let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
            assert!(dispatch(
                &mut c,
                "resize",
                &format!("{handle_str},{dx},{dy}")
            ));
            assert_eq!(
                (c.x(), c.y()),
                expected_xy,
                "handle={handle_str} xy mismatch"
            );
            assert_eq!(
                (c.width(), c.height()),
                expected_wh,
                "handle={handle_str} wh mismatch"
            );
        }
    }

    #[test]
    fn dispatch_resize_clamps_at_image_bounds() {
        let mut c = ImageCropper::new(100, 100, 0, 0, 20, 20, None, 1);
        assert!(dispatch(&mut c, "resize", "e,1000,0"));
        assert_eq!(c.width(), 100);
        assert_eq!(c.x(), 0);
    }

    #[test]
    fn dispatch_resize_respects_min_size() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 5);
        assert!(dispatch(&mut c, "resize", "e,-1000,0"));
        assert_eq!(c.width(), 5);
    }

    #[test]
    fn dispatch_resize_with_fixed_aspect_ew_derives_height_from_width() {
        let mut c = ImageCropper::new(1000, 1000, 0, 0, 100, 100, Some((1, 1)), 1);
        assert!(dispatch(&mut c, "resize", "e,50,0"));
        assert_eq!(c.width(), 150);
        assert_eq!(c.height(), 150); // 1:1 のため height も追随
    }

    #[test]
    fn dispatch_resize_with_fixed_aspect_ns_derives_width_from_height() {
        let mut c = ImageCropper::new(1000, 1000, 0, 0, 100, 100, Some((1, 1)), 1);
        assert!(dispatch(&mut c, "resize", "s,0,50"));
        assert_eq!(c.height(), 150);
        assert_eq!(c.width(), 150); // 1:1 のため width も追随
    }

    #[test]
    fn dispatch_resize_rejects_invalid_payload() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        for bogus in ["", "x,1,2", "e,1", "e,1,2,3", "e,1.5,2"] {
            assert!(!dispatch(&mut c, "resize", bogus));
        }
        assert_eq!((c.width(), c.height()), (20, 20));
    }

    // --- dispatch: set / reset ---

    #[test]
    fn dispatch_set_updates_rect_with_normalization() {
        let mut c = ImageCropper::new(100, 100, 0, 0, 20, 20, None, 1);
        assert!(dispatch(&mut c, "set", "10,10,30,40"));
        assert_eq!((c.x(), c.y(), c.width(), c.height()), (10, 10, 30, 40));

        assert!(dispatch(&mut c, "set", "200,200,200,200"));
        assert_eq!((c.x(), c.y(), c.width(), c.height()), (0, 0, 100, 100));
    }

    #[test]
    fn dispatch_set_rejects_invalid_payload() {
        let mut c = ImageCropper::new(100, 100, 0, 0, 20, 20, None, 1);
        for bogus in ["", "1,2,3", "1,2,3,4,5", "-1,2,3,4", "a,b,c,d"] {
            assert!(!dispatch(&mut c, "set", bogus));
        }
        assert_eq!((c.x(), c.y(), c.width(), c.height()), (0, 0, 20, 20));
    }

    #[test]
    fn dispatch_reset_restores_initial_rect() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        assert!(dispatch(&mut c, "move", "50,50"));
        assert_ne!((c.x(), c.y()), (10, 10));

        assert!(dispatch(&mut c, "reset", ""));
        assert_eq!((c.x(), c.y(), c.width(), c.height()), (10, 10, 20, 20));
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut c = ImageCropper::new(100, 100, 10, 10, 20, 20, None, 1);
        assert!(!dispatch(&mut c, "no_such_action", "x"));
        assert_eq!((c.x(), c.y()), (10, 10));
    }

    // --- percent アクセサ ---

    #[test]
    fn percent_accessors_reflect_rect_within_image() {
        let c = ImageCropper::new(200, 100, 50, 25, 100, 50, None, 1);
        assert_eq!(c.x_percent(), 25.0);
        assert_eq!(c.y_percent(), 25.0);
        assert_eq!(c.width_percent(), 50.0);
        assert_eq!(c.height_percent(), 50.0);
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&ImageCropper::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let c = ImageCropper::new(200, 100, 20, 10, 50, 30, None, 1);
        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains(r#"data-hydrate-image-width="200""#));
        assert!(rendered.contains(r#"data-hydrate-image-height="100""#));
        assert!(rendered.contains(r#"data-hydrate-x="20""#));
        assert!(rendered.contains(r#"data-hydrate-y="10""#));
        assert!(rendered.contains(r#"data-hydrate-width="50""#));
        assert!(rendered.contains(r#"data-hydrate-height="30""#));
        assert!(rendered.contains(r#"data-hydrate-aspect-w="0""#));
        assert!(rendered.contains(r#"data-hydrate-aspect-h="0""#));
        assert!(rendered.contains(r#"data-hydrate-min-size="1""#));
        assert!(rendered.contains(r#"data-hydrate-initial-x="20""#));

        let restored = ImageCropper::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    #[test]
    fn hydration_round_trip_with_aspect_and_after_move() {
        let mut c = ImageCropper::new(1000, 1000, 0, 0, 100, 100, Some((16, 9)), 1);
        assert!(dispatch(&mut c, "move", "10,10"));
        let restored = ImageCropper::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
        assert_eq!(restored.aspect(), Some((16, 9)));
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = ImageCropper::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-image-width".to_string())
        );
    }

    fn valid_attrs() -> Vec<(String, String)> {
        vec![
            ("data-hydrate-image-width".to_string(), "100".to_string()),
            ("data-hydrate-image-height".to_string(), "100".to_string()),
            ("data-hydrate-x".to_string(), "10".to_string()),
            ("data-hydrate-y".to_string(), "10".to_string()),
            ("data-hydrate-width".to_string(), "20".to_string()),
            ("data-hydrate-height".to_string(), "20".to_string()),
            ("data-hydrate-aspect-w".to_string(), "0".to_string()),
            ("data-hydrate-aspect-h".to_string(), "0".to_string()),
            ("data-hydrate-min-size".to_string(), "1".to_string()),
            ("data-hydrate-initial-x".to_string(), "10".to_string()),
            ("data-hydrate-initial-y".to_string(), "10".to_string()),
            ("data-hydrate-initial-width".to_string(), "20".to_string()),
            ("data-hydrate-initial-height".to_string(), "20".to_string()),
        ]
    }

    fn with_override(field: &str, value: &str) -> Vec<(String, String)> {
        let mut attrs = valid_attrs();
        let name = format!("data-hydrate-{field}");
        if let Some(entry) = attrs.iter_mut().find(|(k, _)| *k == name) {
            entry.1 = value.to_string();
        }
        attrs
    }

    #[test]
    fn from_hydration_attrs_valid_round_trips() {
        let restored = ImageCropper::from_hydration_attrs(&valid_attrs()).unwrap();
        assert_eq!(
            (
                restored.x(),
                restored.y(),
                restored.width(),
                restored.height()
            ),
            (10, 10, 20, 20)
        );
    }

    #[test]
    fn from_hydration_attrs_rejects_zero_image_dimensions() {
        let err =
            ImageCropper::from_hydration_attrs(&with_override("image-width", "0")).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_out_of_bounds_rect() {
        let err = ImageCropper::from_hydration_attrs(&with_override("x", "95")).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_zero_min_size() {
        let err = ImageCropper::from_hydration_attrs(&with_override("min-size", "0")).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_incomplete_aspect_ratio() {
        let err = ImageCropper::from_hydration_attrs(&with_override("aspect-w", "16")).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_aspect_ratio_height_mismatch() {
        let mut attrs = with_override("aspect-w", "16");
        if let Some(entry) = attrs.iter_mut().find(|(k, _)| k == "data-hydrate-aspect-h") {
            entry.1 = "9".to_string();
        }
        // width=20 に対し 16:9 の期待 height は 20*9/16 の丸めであり 20 とは
        // 一致しないため拒否される。
        let err = ImageCropper::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_non_numeric_value() {
        let err = ImageCropper::from_hydration_attrs(&with_override("x", "abc")).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_xss_payload_in_x_is_rejected_not_rendered() {
        let err =
            ImageCropper::from_hydration_attrs(&with_override("x", "<script>alert(1)</script>"))
                .unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: src/alt/attrs/children/aria-label にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn image_src_and_alt_payload_is_escaped_on_render() {
        let html = render(&image(ATTR_BREAK_PAYLOAD, ATTR_BREAK_PAYLOAD, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&selection(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn handle_caller_attrs_payload_is_escaped_on_render() {
        let html = render(&handle(
            HandlePosition::N,
            vec![("data-x", ATTR_BREAK_PAYLOAD)],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }
}
