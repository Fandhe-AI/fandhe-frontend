//! `wireframe_css()` の連結順契約テスト。
//!
//! `docs/design/wireframe-ui-architecture.md` §10.4 が定める出力順
//! （tokens → size → `PARTS`（登録順） → frame padding、各セグメント間は
//! `"\n"` 区切り）を、`crates/wireframe-ui/src/css.rs` の実装から独立に
//! 組み立て直して照合する。集約出力全体を 1 本の巨大なリテラル golden に
//! 固定する方式は、部品が 1 つ増えるたびに書き換えが必要になり並行 PR と
//! 必ず競合するため採らない（`docs/internal/wireframe-ui-golden-test-update-guide.md`
//! 参照）。

use fandhe_frontend_wireframe_ui as w;

#[test]
fn wireframe_css_matches_documented_assembly_order() {
    let mut expected = String::new();
    expected.push_str(&w::tokens::css());
    expected.push('\n');
    expected.push_str(&w::size::css());
    for part in w::css::PARTS {
        expected.push('\n');
        expected.push_str(part);
    }
    expected.push('\n');
    expected.push_str(&w::frame::frame_padding_css());

    assert_eq!(w::wireframe_css(), expected);
}
