use super::parser;
use super::parser::Visibility;
use super::Accumulator;
use super::Parents;
use super::TotalBytesGlobalData;
use super::TreeGen;
use super::P;

use super::parser::TreeCursor as _;
use super::GlobalData as _;
use super::WithByteRange as _;

/// Define a zipped visitor, where you mostly have to implement,
/// [`ZippedTreeGen::pre`] going down,
/// and [`ZippedTreeGen::post`] going up in the traversal.
pub trait ZippedTreeGen: TreeGen
where
    Self::Global: TotalBytesGlobalData,
{
    type Stores;
    // # source
    type Text: ?Sized;
    type Node<'a>: parser::Node;
    type TreeCursor<'a>: parser::TreeCursor<N = Self::Node<'a>> + std::fmt::Debug + Clone;

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
    ) -> PreResult<<Self as TreeGen>::Acc> {
        PreResult::Ok(self.pre(text, &cursor.node(), stack, global))
    }

    /// Called when going up
    fn pre(
        &mut self,
        text: &Self::Text,
        node: &Self::Node<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc;

    fn acc(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        full_node: <<Self as TreeGen>::Acc as Accumulator>::Node,
    ) {
        parent.push(full_node);
    }

    /// Called when going up
    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        global: &mut Self::Global,
        text: &Self::Text,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node;

    fn acc_s(acc: &<Self as TreeGen>::Acc) -> String {
        "".to_string()
    }

    fn stores(&mut self) -> &mut Self::Stores;

    fn gen(
        &mut self,
        text: &Self::Text,
        stack: &mut Parents<Self::Acc>,
        cursor: &mut Self::TreeCursor<'_>,
        global: &mut Self::Global,
    ) {
        let mut pre_post = super::utils_ts::PrePost::new(cursor);
        while let Some(visibility) = pre_post.next() {
            let (cursor, has) = pre_post.current().unwrap();
            if *has == Has::Up || *has == Has::Right {
                // #post
                if stack.len() == 0 {
                    return;
                }
                // self._post(stack, global, text);
                let is_parent_hidden;
                let full_node: Option<_> = match (stack.pop().unwrap(), stack.parent_mut_with_vis())
                {
                    (P::Visible(acc), None) => {
                        global.up();
                        is_parent_hidden = false;
                        //global.set_sum_byte_length(acc.end_byte());
                        stack.push(P::Visible(acc));
                        None
                    }
                    (_, None) => {
                        panic!();
                    }
                    (P::ManualyHidden, Some((v, _))) => {
                        is_parent_hidden = v == Visibility::Hidden;
                        None
                    }
                    (P::BothHidden, Some((v, _))) => {
                        is_parent_hidden = v == Visibility::Hidden;
                        None
                    }
                    (P::Visible(acc), Some((v, parent))) => {
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

                let parent = stack.parent_mut().unwrap();
                if let Some(full_node) = full_node {
                    self.acc(parent, full_node);
                }
            }
            if *has == Has::Down || *has == Has::Right {
                // #pre
                // self._pre(global, text, cursor, stack, has, vis);
                global.down();
                let n = self.pre_skippable(text, cursor, &stack, global);
                match n {
                    PreResult::Skip => {
                        stack.push(P::BothHidden);
                        *has = Has::Up;
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
                        *has = Has::Up;
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
            }
        }
        return;
    }
}

#[derive(PartialEq, Eq)]
pub enum Has {
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
