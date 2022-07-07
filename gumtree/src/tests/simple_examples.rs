use crate::tree::simple_tree::{tree, SimpleTree};

type ST<K> = SimpleTree<K>;

/// example of simple delete
/// 
/// 0:f is removed
pub(crate) fn example_delete_action() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
    ]);
    let dst = tree!(
        0,"a"; [
            tree!(0, "e"),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
    ]);
    (src, dst)
}
/// example of simple rename
/// 
/// 0:f is renamed to g
pub(crate) fn example_rename_action() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
    ]);
    let dst = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "g")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
    ]);
    (src, dst)
}
/// example of simple move
/// 
/// 0:f is move to b.1
pub(crate) fn example_move_action() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
    ]);
    let dst = tree!(
        0,"a"; [
            tree!(0, "e"),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "f"),
                tree!(0, "d")]),
    ]);
    (src, dst)
}