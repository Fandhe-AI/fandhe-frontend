# Splitter

`fandhe-frontend-pre-styled-ui` の `splitter` mod が提供するスタイル済み Splitter 部品です。

複数パネルの境界をドラッグでリサイズできる部品です。panel は --fandhe-splitter-size custom property を通じてのみ動的な flex-basis を伝え、resize_trigger は role="separator" + aria-controls を固定付与します。状態機械（Splitter）は headless-ui 側にあり、pre-styled-ui からは再エクスポートしません。
