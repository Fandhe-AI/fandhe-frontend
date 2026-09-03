# Image

`fandhe-frontend-pre-styled-ui` の `image` mod が提供するスタイル済み Image 部品です。

img 要素をラップした画像表示部品です。ImageFit（Cover/Contain/Fill/ScaleDown/NoFit）・AspectRatio（Auto/Square/Landscape/Portrait/Video）・ImageShape（Square/Rounded/Circle、角丸）の 3 軸で表示形式を切り替えます。base は `height: auto` を持ち、`max-width: 100%` による縮小時も縦横比を保ちます。src は is_safe_url 検証を経由し、javascript: 等の危険なスキームは出力自体が落とされます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
