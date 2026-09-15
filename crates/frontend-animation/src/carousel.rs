//! carousel の spring スナップ + ポインタドラッグ演算（イシュー #2541、親
//! `docs/design/motion-reference-adoption-policy.md` §4 Carousel 行）。
//!
//! # 責務境界
//!
//! 「ドラッグ進行度 → 着地 index」の純粋関数（[`snap_target`]）と、その
//! 着地までの spring アニメーションを rAF で実行し
//! [`crate::dom_target::DomTarget`] 経由で `--fandhe-carousel-index`
//! （単位なし小数）へ書き込む [`CarouselTrack`] のみを本モジュールが担う。
//! pointer イベントの購読・`item` 幅/高さの計測（`getBoundingClientRect`）・
//! opt-in 属性判定は `fandhe-frontend-wasm-full::carousel_motion` の責務
//! （[`crate::drag`]/[`crate::magnetic`] と同じ 3 層構成）。
//!
//! # 既存 CSS との契約
//!
//! `crates/pre-styled-ui/src/carousel.rs` の `item-group`/`item` recipe は
//! `--fandhe-carousel-index` を `calc()` で読む既存の transform 宣言を
//! 持つため、本モジュールは同名プロパティへ**連続値**（0.5 等の途中値も
//! 含む）を書くだけで、CSS 側の変更なしにドラッグ追従が成立する。
//!
//! # `DragController`（[`crate::drag`]）を再利用しない理由
//!
//! [`crate::drag::DragController`] は「2 軸自由位置 + 範囲制約 + px 書き込み」
//! の意味論で、本モジュールが必要とする「1 軸 + スナップ点への着地」とは
//! 合わないため専用の小さな型を新設する。速度推定（[`crate::drag::
//! estimate_velocity`]）と陳腐化判定（[`crate::drag::is_velocity_stale`]）
//! のみを再利用する。

/// [`CarouselTrack`] が `--fandhe-carousel-index` へ書き込む CSS カスタム
/// プロパティ名（`crates/pre-styled-ui/src/carousel.rs` の recipe と一致
/// する必要がある契約）。
pub const CAROUSEL_INDEX_PROPERTY: &str = "--fandhe-carousel-index";

/// release 時の速度（スライド/秒）を projection time 分だけ進行度へ
/// 加算してから最寄りの整数へ丸める、その projection time（秒）。大きい
/// ほど弱いフリックでも隣のスライドへ届きやすくなる。
const VELOCITY_PROJECTION_S: f64 = 0.15;

/// ドラッグ進行度（`progress`、スライド単位の連続値）と release 時の速度
/// （`velocity_slides_per_s`、正方向 = index 増加）から、着地する
/// スライド index を決定する。
///
/// - `slide_count == 0` は `0` を返す（呼び出し元が事前に弾く前提だが
///   fail-safe として）。
/// - `progress`/`velocity_slides_per_s` が非有限（NaN/inf）の場合、速度は
///   `0.0` として扱い進行度のみで丸める。進行度自体が非有限なら `0` を
///   返す。
/// - `loop_ == false`（非 loop）: 丸めた値を `0..=slide_count-1` へ clamp
///   する（既存 headless `Carousel::goto` の範囲外 no-op と整合する安全側
///   の挙動）。
/// - `loop_ == true`: `slide_count` を法とする剰余で折り返す（既存
///   `Carousel::loop_` の端折り返し意味論、負値も `rem_euclid` 相当で
///   `0..slide_count` へ正規化する）。
#[must_use]
pub fn snap_target(
    progress: f64,
    velocity_slides_per_s: f64,
    slide_count: usize,
    loop_: bool,
) -> usize {
    if slide_count == 0 || !progress.is_finite() {
        return 0;
    }
    let velocity = if velocity_slides_per_s.is_finite() {
        velocity_slides_per_s
    } else {
        0.0
    };
    let projected = (progress + velocity * VELOCITY_PROJECTION_S).round();
    if loop_ {
        let n = slide_count as f64;
        let wrapped = projected.rem_euclid(n);
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            reason = "rem_euclid(n) は [0, n) の整数値のみを返す"
        )]
        let index = wrapped as usize;
        index.min(slide_count - 1)
    } else {
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            reason = "clamp 後の値は 0..slide_count-1 の範囲に収まる"
        )]
        let index = projected.clamp(0.0, (slide_count - 1) as f64) as usize;
        index
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use std::cell::Cell;
    use std::rc::Rc;

    use fandhe_animation::driver::Driver;
    use fandhe_animation::interpolate::Vec2;
    use fandhe_animation::spring::{Spring, SpringConfig};
    use fandhe_animation::target::Target;
    use web_sys::HtmlElement;

    use super::snap_target;
    use crate::dom_target::DomTarget;
    use crate::drag::{estimate_velocity, is_velocity_stale};
    use crate::raf_driver::{AnimationLoop, RafDriver};
    use crate::reduced_motion::prefers_reduced_motion;

    /// スナップ着地に使う spring の物理パラメータ（`SpringConfig::default()`
    /// と同値だが、意図を明示するため本モジュール専用の定数として複製する）。
    const SNAP_SPRING_CONFIG: SpringConfig = SpringConfig {
        stiffness: 100.0,
        damping: 10.0,
        mass: 1.0,
    };

    /// 1 つの `item-group` に紐づくドラッグ + spring スナップのセッション。
    ///
    /// `Drop` で進行中の rAF ループ（[`AnimationLoop`]）を確実に止める
    /// （`AnimationLoop` 自身の `Drop` 実装に委ねる、`crate::drag::
    /// DragController`/`crate::magnetic` と同じ契約）。
    pub struct CarouselTrack {
        element: HtmlElement,
        slide_count: usize,
        loop_: bool,
        // 収束前の spring 途中経過値を毎 tick 反映する共有セル（issue #2541
        // レビュー指摘 是正）。settle 完了前に外部から `on_pointer_down` が
        // 再度呼ばれても、`Cell::get()` が返すのは常にその瞬間の実際の
        // 表示位置であり、`to`（着地先）への先取りジャンプを起こさない。
        progress: Rc<Cell<f64>>,
        drag_origin: Option<(f64, f64)>,
        previous_sample: Option<(Vec2, f64)>,
        latest_sample: Option<(Vec2, f64)>,
        settle_anim: Option<AnimationLoop>,
    }

    impl CarouselTrack {
        /// `item_group` の現在の `--fandhe-carousel-index` インライン style
        /// を初期進行度として読む（パース不能・未設定は `0.0`）。
        #[must_use]
        pub fn attach(item_group: HtmlElement, slide_count: usize, loop_: bool) -> Self {
            let progress = item_group
                .style()
                .get_property_value(super::CAROUSEL_INDEX_PROPERTY)
                .ok()
                .and_then(|value| value.trim().parse::<f64>().ok())
                .filter(|value| value.is_finite())
                .unwrap_or(0.0);
            Self {
                element: item_group,
                slide_count,
                loop_,
                progress: Rc::new(Cell::new(progress)),
                drag_origin: None,
                previous_sample: None,
                latest_sample: None,
                settle_anim: None,
            }
        }

        /// `pointerdown` 相当の入力。進行中の spring を打ち切り、ドラッグ
        /// 起点を記録する。
        pub fn on_pointer_down(&mut self, client_x: f64, time_ms: f64) {
            self.settle_anim = None;
            self.drag_origin = Some((self.progress.get(), client_x));
            self.previous_sample = None;
            self.latest_sample = Some((
                Vec2 {
                    x: client_x,
                    y: 0.0,
                },
                time_ms,
            ));
        }

        /// `pointermove` 相当の入力。`slide_px`（1 スライド分の幅/高さ、
        /// px）で正規化した相対移動量を進行度へ加算し毎回 DOM へ書く。
        /// `on_pointer_down` を経ていない・`slide_px` が非正/非有限の場合は
        /// no-op（fail-safe）。
        pub fn on_pointer_move(&mut self, client_x: f64, slide_px: f64, time_ms: f64) {
            let Some((origin_progress, origin_x)) = self.drag_origin else {
                return;
            };
            if !slide_px.is_finite() || slide_px <= 0.0 {
                return;
            }
            // ポインタを負方向（前のスライド側）へ引くと index が減る向き
            // （既存 headless `Carousel::prev`/`next` の index 増加方向と
            // 一致させる）。
            let next = origin_progress - (client_x - origin_x) / slide_px;
            self.progress.set(next);
            self.write_progress(next);
            self.previous_sample = self.latest_sample;
            self.latest_sample = Some((
                Vec2 {
                    x: client_x,
                    y: 0.0,
                },
                time_ms,
            ));
        }

        /// `pointerup`/`pointercancel` 相当の入力。直近の速度から着地
        /// index を決め、spring アニメーションを開始する（`prefers-
        /// reduced-motion: reduce` は即時に整数値を書いて終える）。
        /// `on_settle` は着地が完了した時点で一度だけ呼ばれる。
        pub fn on_release(
            &mut self,
            time_ms: f64,
            slide_px: f64,
            on_settle: impl FnOnce(usize) + 'static,
        ) -> usize {
            self.drag_origin = None;
            let velocity_px_s = if is_velocity_stale(self.latest_sample, time_ms) {
                Vec2::default()
            } else {
                estimate_velocity(self.previous_sample, self.latest_sample)
            };
            let velocity_slides_s = if slide_px.is_finite() && slide_px > 0.0 {
                -velocity_px_s.x / slide_px
            } else {
                0.0
            };
            let target = snap_target(
                self.progress.get(),
                velocity_slides_s,
                self.slide_count,
                self.loop_,
            );
            self.settle_to(target, velocity_slides_s, on_settle);
            target
        }

        /// `self.progress` を `target`（整数 index）へ spring で収束させる。
        ///
        /// # loop 境界の短い経路（codex-review 指摘 是正）
        ///
        /// `loop_ == true` のとき、着地先は `target`（`0..slide_count` に
        /// 正規化済み）だが、spring の `to` にはそのまま `target` を使わず
        /// `from` に最も近い合同値（`target + k * slide_count`）を使う。
        /// 例えば 5 枚中 `from = -0.8`（末尾方向へドラッグ中）で
        /// `target = 4`（末尾スライド）のとき、そのまま `to = 4.0` にすると
        /// spring が正方向へ 4 枚分横断してしまう。`to = -1.0`
        /// （`4 - 5`、`from` との差 `0.2`）を選べば末尾への短い折り返しで
        /// 済む。終端の見た目は `on_settle` が dispatch する `"goto"`
        /// アクション（headless 側の正規レンダリング）が正規化済み
        /// `target` で上書きするため、`to` が `0..slide_count-1` の外に
        /// 出ても最終表示に影響しない。
        ///
        /// # `self.progress` を先取り更新しない理由（codex-review 指摘
        /// 是正）
        ///
        /// `self.progress` は [`Rc<Cell<f64>>`] で共有しており、着地先
        /// （`to`）への先取りジャンプはせず、tick 毎に spring の
        /// **実際の途中経過値**（`state.value`）を書き込む。settle
        /// 完了前に（再 `attach` を経ずに）同一インスタンスへ
        /// `on_pointer_down` が再度呼ばれても、`self.progress.get()` は
        /// その瞬間の実表示位置を返すため、次の `on_pointer_move` が
        /// 未到達の `to` を起点にして飛ぶことがない。
        fn settle_to(
            &mut self,
            target: usize,
            initial_velocity: f64,
            on_settle: impl FnOnce(usize) + 'static,
        ) {
            let from = self.progress.get();
            let to = if self.loop_ && self.slide_count > 0 {
                let n = self.slide_count as f64;
                let shift = ((from - target as f64) / n).round() * n;
                target as f64 + shift
            } else {
                target as f64
            };
            if prefers_reduced_motion() {
                self.progress.set(to);
                self.write_progress(to);
                on_settle(target);
                return;
            }
            let Some(spring) = Spring::new(SNAP_SPRING_CONFIG, from, to, initial_velocity) else {
                self.progress.set(to);
                self.write_progress(to);
                on_settle(target);
                return;
            };
            let Some(mut driver) = RafDriver::new() else {
                self.progress.set(to);
                self.write_progress(to);
                on_settle(target);
                return;
            };
            let element = self.element.clone();
            let progress_cell = self.progress.clone();
            let mut elapsed_s = 0.0;
            let mut on_settle = Some(on_settle);
            self.settle_anim = Some(AnimationLoop::start(move || {
                let dt = driver.tick().unwrap_or(0.0);
                elapsed_s += dt;
                let state = spring.at(elapsed_s);
                progress_cell.set(state.value);
                DomTarget::custom_property(element.clone(), super::CAROUSEL_INDEX_PROPERTY)
                    .write(state.value);
                if state.done {
                    if let Some(cb) = on_settle.take() {
                        cb(target);
                    }
                    false
                } else {
                    true
                }
            }));
        }

        fn write_progress(&self, value: f64) {
            DomTarget::custom_property(self.element.clone(), super::CAROUSEL_INDEX_PROPERTY)
                .write(value);
        }
    }

    // `CarouselTrack` を drop すると `settle_anim`（`Option<AnimationLoop>`）
    // フィールドの既定 drop 挙動により進行中の rAF ループも止まる
    // （`AnimationLoop::drop` が `stop()` を呼ぶ、`raf_driver.rs` doc 参照）。
    // ただし呼び出し側（wasm-full）が settle 完了前に `CarouselTrack` 自体を
    // drop してはならない契約は変わらない——settle 中の tick 自身が
    // `CarouselTrack` を保持していないため、その保持責任は呼び出し側にある
    // （`crates/wasm-full/src/carousel_motion.rs` の doc「`CarouselTrack` の
    // 保持責任」節参照、`hold_to_confirm.rs::finish_confirmation` doc の
    // 「never drop here」と同じ設計）。
}

#[cfg(target_arch = "wasm32")]
pub use wiring::CarouselTrack;

#[cfg(test)]
mod tests {
    use super::snap_target;

    #[test]
    fn snap_target_rounds_progress_without_velocity() {
        assert_eq!(snap_target(0.0, 0.0, 5, false), 0);
        assert_eq!(snap_target(1.4, 0.0, 5, false), 1);
        assert_eq!(snap_target(1.6, 0.0, 5, false), 2);
    }

    #[test]
    fn snap_target_clamps_to_bounds_when_not_looping() {
        assert_eq!(snap_target(-3.0, 0.0, 5, false), 0);
        assert_eq!(snap_target(99.0, 0.0, 5, false), 4);
    }

    #[test]
    fn snap_target_wraps_when_looping() {
        assert_eq!(snap_target(-1.0, 0.0, 5, true), 4);
        assert_eq!(snap_target(5.0, 0.0, 5, true), 0);
        assert_eq!(snap_target(7.0, 0.0, 5, true), 2);
    }

    #[test]
    fn snap_target_projects_by_velocity() {
        // progress=1.0・速度 +10 slides/s・projection 0.15s → projected 2.5 → round 2 or 3
        // (round-half-to-even/away は標準ライブラリの round() 挙動に委ねる)。
        let target = snap_target(1.0, 10.0, 5, false);
        assert!(target >= 2);
    }

    #[test]
    fn snap_target_treats_non_finite_velocity_as_zero() {
        assert_eq!(snap_target(2.0, f64::NAN, 5, false), 2);
        assert_eq!(snap_target(2.0, f64::INFINITY, 5, false), 2);
    }

    #[test]
    fn snap_target_returns_zero_for_non_finite_progress_or_empty() {
        assert_eq!(snap_target(f64::NAN, 0.0, 5, false), 0);
        assert_eq!(snap_target(0.0, 0.0, 0, false), 0);
    }
}
