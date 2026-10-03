use fandhe_frontend_app::page_shell;
use fandhe_frontend_core::{el, p, text};

/// リクエストごとに HTML 文書全体を組み立てて返す（SSR）。
/// `name` に何が来ても `text()` が既定でエスケープする。
pub fn hello_page(name: &str) -> String {
    let greeting = p(vec![], vec![text(format!("Hello, {name}!"))]);
    page_shell("Hello", el("main", vec![], vec![greeting]))
}
