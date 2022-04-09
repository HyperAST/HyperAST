// use crate::position::TreePath;

// pub trait HyperAstIterator<'a,'b:'a,'c:'b, IdN: 'b, N, NS: 'c, T: 'b>
// where
//     IdN: Eq + Clone,
//     N: crate::types::Tree<TreeId = IdN>,
//     NS: crate::types::NodeStore<'a, IdN, N>,
//     T: TreePath<IdN> + Clone,
// {
//     fn is_dead_end(&'c self, b: &N) -> bool;
//     fn is_matching(&self, b: &N) -> bool;
//     fn node_store(&self) -> &NS;
//     // fn path(&mut self) -> &mut T;
//     fn all(&mut self) -> (&NS,&mut T,&mut Vec<(IdN, usize, Option<Vec<IdN>>)>);
//     // fn stack(&mut self) -> &mut Vec<(IdN, usize, Option<Vec<IdN>>)>;
//     // fn pop(&mut self) -> Option<(IdN, usize, Option<Vec<IdN>>)>;
//     fn next(&'a mut self) -> Option<T> {
//         let (ns,path,stack) = self.all();
//         loop {
//             // let (node, offset, children) = self.stack().pop()?;
//             let (node, offset, children) = stack.pop()?;
//             if let Some(children) = children {
//                 // if offset < children.len() {
//                 //     let child = children[offset];
//                 //     if offset == 0 {
//                 //         self.path().goto(child, offset);
//                 //     } else {
//                 //         self.path().inc(child);
//                 //         assert_eq!(*self.path().offset().unwrap(), offset + 1);
//                 //     }
//                 //     self.stack().push((node, offset + 1, Some(children)));
//                 //     self.stack().push((child, 0, None));
//                 //     continue;
//                 // } else {
//                 //     self.path().pop().expect("should not go higher than root");
//                 //     continue;
//                 // }
//                 return None;
//             } else { 
//                 let b = ns.resolve(&node);
//                 if self.is_dead_end(&b) {
//                     continue;
//                 }

//                 // if b.has_children() {
//                 //     let children = b.get_children();
//                 //     self.stack().push((node, 0, Some(children.to_vec())));
//                 // }

//                 // if self.is_matching(&b) {
//                 //     return Some(self.path().clone());
//                 // }
//             }
//         }
//     }
// }
