//! TASK-5.3 コンパイルエラー品質レビュー用フィクスチャ（意図的にコンパイル不能）。
//!
//! `el()` のタグ名引数は `&'static str` に固定されており、動的に組み立てた
//! `String` への参照は受け付けない（タグ名注入を型で防ぐ設計、不変条件 5）。
//! 期待するエラーコード: E0597（borrowed value does not live long enough）。
//! 本ファイルは cargo のテストターゲットには含まれない（README 参照）。

use rws_core::{el, Node};

pub fn case03() -> Node {
    let t = String::from("div");
    el(&t, vec![], vec![])
}
