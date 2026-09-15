//! `prefers-reduced-motion: reduce` の判定ヘルパ（イシュー #2535）。
//!
//! `scroll_driver::Env::detect` が同種の判定を内包しているが、本モジュールは
//! CSS `@keyframes` を使わない機能（`drag` の release 時スナップ判定等）が
//! 単独で問い合わせるための汎用エントリポイントとして独立させる。carousel
//! （#2541）・parallax（#2534）等、後続の C 群機能からの再利用を見込む。
//!
//! # フェイルセーフ方針
//!
//! `matchMedia` の呼び出し自体が失敗する（API 不在・例外）場合は
//! `scroll_driver::Env::detect` と同じ安全側（reduce = `true`、アニメー
//! ションを抑制する）へ倒す（security.md A05 対応）。

/// 実行環境の `prefers-reduced-motion: reduce` を判定する。
///
/// wasm32 以外のターゲット（native/SSR）では JS ホストが存在せず
/// `web_sys::window()` の呼び出し自体が panic するため、[`crate::raf_driver::RafDriver::new`]
/// と同じ方針で JS 呼び出しを `cfg(target_arch = "wasm32")` でガードし、
/// native では常に安全側デフォルト（`true`）を返す。
#[must_use]
pub fn prefers_reduced_motion() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| {
                window
                    .match_media("(prefers-reduced-motion: reduce)")
                    .ok()
                    .flatten()
            })
            .map(|list| list.matches())
            .unwrap_or(true)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

/// native（非 wasm32）実行時に panic せず安全側デフォルトを返すことを
/// 固定する回帰テスト（`raf_driver.rs::native_no_panic_tests` と同型）。
#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_no_panic_tests {
    use super::prefers_reduced_motion;

    #[test]
    fn returns_true_on_native_without_panicking() {
        assert!(prefers_reduced_motion());
    }
}
