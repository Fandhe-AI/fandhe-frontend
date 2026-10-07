//! document/window 部品が keydown を消費するかの述語登録簿（イシュー #3768、方式 2b）。
//!
//! 汎用 keydown 配線（[`crate::events`] の `wire_keydown`、feature `action-keydown`）は root の
//! bubble にあり、document の overlay・sidebar と window の command より先に動く。
//! `defaultPrevented` では排他にできないため、部品側が「この keydown を自分が消費するか」を
//! 返す述語を本登録簿へ登録し、汎用配線が dispatch の直前に [`claims`] で問い合わせる。
//! 真なら汎用配線は dispatch も `preventDefault` も行わず、部品が単独で処理する。
//!
//! # 責務境界
//!
//! - 登録簿は crate 内部に閉じる（`pub(crate)`）。公開 API には出さない。
//! - 述語は各部品が実処理と同じ純粋関数を再利用して組み立てる（判定の重複を避ける）。
//! - web-sys 非依存のため native の `cargo test` で検証できる。
//! - 登録簿が空・述語が偽・再入で借用できないときは偽を返し、従来動作（並行発火）に倒れる。
//!   設計の正は `docs/design/wasm-full-architecture.md` §41.5。

// 述語の登録元は wasm32 の配線層のみで、native ビルドでは単体テストからしか使われない。
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// 述語へ渡す keydown の純粋な入力（`KeyboardEvent` 非依存）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct KeydownClaimInput<'a> {
    /// `KeyboardEvent.key`。
    pub key: &'a str,
    /// Ctrl 押下。
    pub ctrl: bool,
    /// Alt 押下。
    pub alt: bool,
    /// Shift 押下。
    pub shift: bool,
    /// Meta 押下。
    pub meta: bool,
    /// イベントの伝播先 Document（`target.ownerDocument`）。部品は自分のリスナーが載る
    /// Document と一致するときだけ消費してよい（同一オリジン iframe など別 Document の部品が
    /// 親 Document の汎用配線を抑止しないため）。型を `Any` で隠して web-sys 非依存を保つ。
    /// 取得できなかったときは `None`（どの部品も消費しない側に倒す）。
    pub document: Option<&'a dyn std::any::Any>,
}

impl KeydownClaimInput<'_> {
    /// イベントの伝播先 Document が `document` と等しいかを返す。`None` や型違いは偽。
    pub fn is_document<T: PartialEq + 'static>(&self, document: &T) -> bool {
        self.document
            .and_then(|any| any.downcast_ref::<T>())
            .is_some_and(|event_document| event_document == document)
    }
}

/// 述語の判定結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// 部品がこの keydown を消費する（汎用配線は見送る）。
    Consumes,
    /// 部品は消費しない。
    Passes,
}

/// [`register`] が返す登録の識別子。[`unregister`] へ渡して解除する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClaimHandle(u64);

/// 消費述語。登録元が `Rc` で状態を共有できるよう `Rc<dyn Fn>` とする。
pub(crate) type Predicate = Rc<dyn Fn(&KeydownClaimInput) -> Verdict>;

thread_local! {
    static CLAIMS: RefCell<Vec<(u64, Predicate)>> = const { RefCell::new(Vec::new()) };
    static NEXT_ID: Cell<u64> = const { Cell::new(0) };
}

/// 述語を登録する。
pub(crate) fn register(predicate: Predicate) -> ClaimHandle {
    let id = NEXT_ID.with(|next| {
        let id = next.get();
        next.set(id.wrapping_add(1));
        id
    });
    CLAIMS.with(|claims| {
        if let Ok(mut claims) = claims.try_borrow_mut() {
            claims.push((id, predicate));
        }
    });
    ClaimHandle(id)
}

/// 登録を解除する。未登録なら何もしない。
pub(crate) fn unregister(handle: ClaimHandle) {
    CLAIMS.with(|claims| {
        if let Ok(mut claims) = claims.try_borrow_mut() {
            claims.retain(|(id, _)| *id != handle.0);
        }
    });
}

/// いずれかの部品が `input` を消費するかを返す。
///
/// 述語は借用を解放してから呼ぶ（述語内の登録・解除と衝突しない）。
/// 登録の除去は [`unregister`] だけが行う（一時的な DOM 切断で恒久的に失わないため）。
pub(crate) fn claims(input: &KeydownClaimInput) -> bool {
    let snapshot: Vec<(u64, Predicate)> = CLAIMS.with(|claims| match claims.try_borrow() {
        Ok(claims) => claims.clone(),
        Err(_) => Vec::new(),
    });
    let mut consumed = false;
    for (_, predicate) in &snapshot {
        if predicate(input) == Verdict::Consumes {
            consumed = true;
        }
    }
    consumed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(key: &str) -> KeydownClaimInput<'_> {
        KeydownClaimInput {
            key,
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
            document: None,
        }
    }

    fn reset() {
        CLAIMS.with(|claims| claims.borrow_mut().clear());
    }

    fn registered_count() -> usize {
        CLAIMS.with(|claims| claims.borrow().len())
    }

    #[test]
    fn empty_registry_does_not_claim() {
        reset();
        assert!(!claims(&input("Escape")));
    }

    #[test]
    fn consuming_predicate_claims() {
        reset();
        let handle = register(Rc::new(|i| {
            if i.key == "Escape" {
                Verdict::Consumes
            } else {
                Verdict::Passes
            }
        }));
        assert!(claims(&input("Escape")));
        assert!(!claims(&input("a")));
        unregister(handle);
        assert!(!claims(&input("Escape")));
    }

    #[test]
    fn passes_only_does_not_claim() {
        reset();
        register(Rc::new(|_| Verdict::Passes));
        assert!(!claims(&input("Escape")));
    }

    #[test]
    fn is_document_matches_only_equal_document() {
        let doc = 7_u32;
        let mut i = input("x");
        assert!(!i.is_document(&doc));
        i.document = Some(&doc);
        assert!(i.is_document(&7_u32));
        assert!(!i.is_document(&8_u32));
        assert!(!i.is_document(&"x"));
    }

    #[test]
    fn reentrant_register_and_unregister_do_not_panic() {
        reset();
        let handle: Rc<Cell<Option<ClaimHandle>>> = Rc::new(Cell::new(None));
        let inner = handle.clone();
        let h = register(Rc::new(move |_| {
            register(Rc::new(|_| Verdict::Passes));
            if let Some(own) = inner.get() {
                unregister(own);
            }
            Verdict::Passes
        }));
        handle.set(Some(h));
        assert!(!claims(&input("x")));
        assert_eq!(registered_count(), 1);
    }
}
