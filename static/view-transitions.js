/**
 * 標準 View Transitions API (`document.startViewTransition()`) の最薄ラッパー。
 * DOM 構築・HTML 文字列組み立て・状態管理は一切行わない（REQ-8）。
 *
 * クロスドキュメント遷移は TASK-8.1 の `@view-transition { navigation: auto; }`
 * （CSS Level 2 at-rule、JS 0 行）が担い、本関数は同一文書内（SPA 的）更新専用。
 *
 * セキュリティ不変条件: `update` は関数としてのみ受け取り、文字列・HTML の
 * 動的評価（`eval` / `innerHTML` 代入等）は行わない。DOM 更新内容の XSS
 * 安全性は呼び出し側（rws-core の既定エスケープ経路、REQ-1）が担保する契約。
 *
 * LOC ルーブリック: 実効行数 10 行以内を維持する（機械ゲート化は #62）。
 */
export function withViewTransition(update) {
  if (typeof document.startViewTransition !== "function") {
    update();
    return null;
  }
  return document.startViewTransition(update);
}
