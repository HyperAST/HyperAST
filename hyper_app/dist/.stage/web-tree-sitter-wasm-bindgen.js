// @ts-check
"use-strict";
// import * as TreeSitter from 'web-tree-sitter';

async function initialize_tree_sitter() {
  // console.log(TreeSitter)
  // await TreeSitter.init();
  // globalThis.Parser = TreeSitter;
  // globalThis.Language = TreeSitter.Language;
  const TreeSitter = window["TreeSitter"];
  await TreeSitter.init();
  globalThis.Parser = TreeSitter;
  globalThis.Language = TreeSitter.Language;
  globalThis.QueryCursor = TreeSitter.QueryCursor;
}

// module.exports = {
//   initialize_tree_sitter,
// };

export {
  initialize_tree_sitter
}