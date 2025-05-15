use super::AccIndentation;
use super::Accumulator;
use super::GlobalData;
use super::P;
use super::Parents;
use super::TotalBytesGlobalData;
use super::WithByteRange;
use super::parser;
use super::parser::Visibility;

use super::Accumulator as _;
use super::GlobalData as _;
use super::parser::Node as _;
use super::parser::TreeCursor as _;

/// Define a zipped visitor, where you mostly have to implement,
/// [`ZippedTraversal::pre`] going down,
/// and [`ZippedTraversal::post`] going up in the traversal.
pub trait ZippedTraversal
where
    Self::Global: TotalBytesGlobalData,
{
    type Global: TotalBytesGlobalData + GlobalData;
    type Acc: AccIndentation + WithByteRange;
    // # results
    // type Node1;
    type Stores;
    // # source
    type Text: ?Sized;
    type Node<'a>: parser::Node<'a>;
    type TreeCursor<'a>: parser::TreeCursor<'a, Self::Node<'a>> + std::fmt::Debug;

    fn init_val(&mut self, text: &Self::Text, node: &Self::Node<'_>) -> Self::Acc;

    /// Can be implemented if you want to skip certain nodes,
    /// note that skipping only act on the "overlay" tree structure,
    /// meaning that the content of a skipped node is fed to its parents
    ///
    /// The default implementation skips nothing.
    ///
    ///  see also also the following example use:
    /// [`hyperast_gen_ts_cpp::legion::CppTreeGen::pre_skippable`](../../hyperast_gen_ts_cpp/legion/struct.CppTreeGen.html#method.pre_skippable)
    fn pre_skippable(
        &mut self,
        text: &Self::Text,
        cursor: &Self::TreeCursor<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> PreResult<Self::Acc> {
        PreResult::Ok(self.pre(text, &cursor.node(), stack, global))
    }

    /// Called when going up
    fn pre(
        &mut self,
        text: &Self::Text,
        node: &Self::Node<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> Self::Acc;

    /// Called when going up
    fn post(
        &mut self,
        parent: &mut Self::Acc,
        global: &mut Self::Global,
        text: &Self::Text,
        acc: Self::Acc,
    ) -> <Self::Acc as Accumulator>::Node;

    fn stores(&mut self) -> &mut Self::Stores;

    fn r#gen(
        &mut self,
        text: &Self::Text,
        stack: &mut Parents<Self::Acc>,
        cursor: &mut Self::TreeCursor<'_>,
        global: &mut Self::Global,
    ) {
        let mut has = Has::Down;
        loop {
            if has != Has::Up
                && let Some(visibility) = cursor.goto_first_child_extended()
            {
                has = Has::Down;
                global.down();
                let n = self.pre_skippable(text, cursor, &stack, global);
                match n {
                    PreResult::Skip => {
                        has = Has::Up;
                        global.up();
                    }
                    PreResult::Ignore => {
                        if let Visibility::Visible = visibility {
                            stack.push(P::ManualyHidden);
                        } else {
                            stack.push(P::BothHidden);
                        }
                    }
                    PreResult::SkipChildren(acc) => {
                        has = Has::Up;
                        if let Visibility::Visible = visibility {
                            stack.push(P::Visible(acc));
                        } else {
                            unimplemented!("Only concrete nodes should be leafs")
                        }
                    }
                    PreResult::Ok(acc) => {
                        global.set_sum_byte_length(acc.begin_byte());
                        if let Visibility::Visible = visibility {
                            stack.push(P::Visible(acc));
                        } else {
                            stack.push(P::Hidden(acc));
                        }
                    }
                }
            } else {
                let is_visible;
                let is_parent_hidden;
                let full_node: Option<_> = match (stack.pop().unwrap(), stack.parent_mut_with_vis())
                {
                    (P::Visible(acc), None) => {
                        global.up();
                        is_visible = true;
                        is_parent_hidden = false;
                        //global.set_sum_byte_length(acc.end_byte());
                        stack.push(P::Visible(acc));
                        None
                    }
                    (_, None) => {
                        panic!();
                    }
                    (P::ManualyHidden, Some((v, _))) => {
                        is_visible = false;
                        is_parent_hidden = v == Visibility::Hidden;
                        None
                    }
                    (P::BothHidden, Some((v, _))) => {
                        is_visible = false;
                        is_parent_hidden = v == Visibility::Hidden;
                        None
                    }
                    (P::Visible(acc), Some((v, parent))) => {
                        is_visible = true;
                        is_parent_hidden = v == Visibility::Hidden;
                        if !acc.has_children() {
                            global.set_sum_byte_length(acc.end_byte());
                        }
                        if is_parent_hidden && parent.end_byte() <= acc.begin_byte() {
                            panic!()
                        }
                        global.up();
                        let full_node = self.post(parent, global, text, acc);
                        Some(full_node)
                    }
                    (P::Hidden(acc), Some((v, parent))) => {
                        is_visible = false;
                        is_parent_hidden = v == Visibility::Hidden;
                        if !acc.has_children() {
                            global.set_sum_byte_length(acc.end_byte());
                        }
                        if is_parent_hidden && parent.end_byte() < acc.begin_byte() {
                            panic!("{} {}", parent.end_byte(), acc.begin_byte());
                        } else if is_parent_hidden && parent.end_byte() == acc.begin_byte() {
                            log::error!("{} {}", parent.end_byte(), acc.begin_byte());
                            assert!(!acc.has_children());
                            global.up();
                            None
                        } else {
                            global.up();
                            let full_node = self.post(parent, global, text, acc);
                            Some(full_node)
                        }
                    }
                };

                // TODO opt out of using end_byte other than on leafs,
                // it should help with trailing spaces,
                // something like `cursor.node().child_count().ne(0).then(||cursor.node().end_byte())` then just call set_sum_byte_length if some
                if let Some(visibility) = cursor.goto_next_sibling_extended() {
                    has = Has::Right;
                    let parent = stack.parent_mut().unwrap();
                    if let Some(full_node) = full_node {
                        parent.push(full_node);
                    }
                    loop {
                        let parent = stack.parent_mut().unwrap();
                        if parent.end_byte() <= cursor.node().start_byte() {
                            loop {
                                let p = stack.pop().unwrap();
                                match p {
                                    P::ManualyHidden => (),
                                    P::BothHidden => (),
                                    P::Hidden(acc) => {
                                        let parent = stack.parent_mut().unwrap();
                                        let full_node = self.post(parent, global, text, acc);
                                        parent.push(full_node);
                                        break;
                                    }
                                    P::Visible(acc) => {
                                        let parent = stack.parent_mut().unwrap();
                                        let full_node = self.post(parent, global, text, acc);
                                        parent.push(full_node);
                                        break;
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    global.down();
                    let n = self.pre_skippable(text, cursor, &stack, global);
                    match n {
                        PreResult::Skip => {
                            has = Has::Up;
                            global.up();
                        }
                        PreResult::Ignore => {
                            if let Visibility::Visible = visibility {
                                stack.push(P::ManualyHidden);
                            } else {
                                stack.push(P::BothHidden);
                            }
                        }
                        PreResult::SkipChildren(acc) => {
                            has = Has::Up;
                            if let Visibility::Visible = visibility {
                                stack.push(P::Visible(acc));
                            } else {
                                unimplemented!("Only concrete nodes should be leafs")
                            }
                        }
                        PreResult::Ok(acc) => {
                            global.set_sum_byte_length(acc.begin_byte());
                            if let Visibility::Visible = visibility {
                                stack.push(P::Visible(acc));
                            } else {
                                stack.push(P::Hidden(acc));
                            }
                        }
                    }
                } else {
                    has = Has::Up;
                    if is_parent_hidden || stack.0.last().map_or(false, P::is_both_hidden) {
                        if let Some(full_node) = full_node {
                            let parent = stack.parent_mut().unwrap();
                            parent.push(full_node);
                        }
                    } else if cursor.goto_parent() {
                        if let Some(full_node) = full_node {
                            let parent = stack.parent_mut().unwrap();
                            parent.push(full_node);
                        } else if is_visible {
                            if has == Has::Down {}
                            return;
                        }
                    } else {
                        assert!(full_node.is_none());
                        if has == Has::Down {}
                        return;
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum Has {
    Down,
    Up,
    Right,
}

pub enum PreResult<Acc> {
    /// Do not process node and its children
    Skip,
    /// Do not process node (but process children)
    Ignore,
    /// Do not process children
    SkipChildren(Acc),
    Ok(Acc),
}
