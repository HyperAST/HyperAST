use std::time::{Duration, Instant};

use crate::{
    cpp_processor::SimpleStores, preprocessed::IsSkippedAna, processing::ObjectName, Accumulator,
    BasicDirAcc, PROPAGATE_ERROR_ON_BAD_CST_NODE,
};

use hyperast::{
    hashed::SyntaxNodeHashs,
    store::defaults::{LabelIdentifier, NodeIdentifier},
    tree_gen::{self, SubTreeMetrics},
};

use hyperast_gen_ts_cpp::{
    legion as cpp_tree_gen,
    types::{TStore, Type},
};

// waiting for residual stabilization https://github.com/rust-lang/rust/issues/84277
// see after the temporary solution
// It is also limiting the usability with more variants
// enum FileProcessingResult<N, D = Duration> {
//     FailedParsing {
//         parsing_time: D,
//         tree: tree_sitter::Tree,
//         error: &'static str,
//     },
//     // ParsingTimedout(D),
//     // FailedProcessing {
//     //     parsing_time: D,
//     //     processing_time: D,
//     //     node: N,
//     // },
//     Success {
//         parsing_time: D,
//         processing_time: D,
//         node: N,
//     },
// }

pub struct FailedParsing<D = Duration> {
    pub parsing_time: D,
    pub tree: tree_sitter::Tree,
    pub error: &'static str,
}

pub struct SuccessProcessing<N, D = Duration> {
    pub parsing_time: D,
    pub processing_time: D,
    pub node: N,
}

pub type FileProcessingResult<N, D = Duration> = Result<SuccessProcessing<N, D>, FailedParsing<D>>;

pub(crate) fn handle_cpp_file<'stores, 'cache, 'b: 'stores, More>(
    tree_gen: &mut cpp_tree_gen::CppTreeGen<'stores, 'cache, TStore, More>,
    name: &ObjectName,
    text: &'b [u8],
) -> FileProcessingResult<cpp_tree_gen::FNode>
where
    More: tree_gen::Prepro<SimpleStores>
        + for<'a> tree_gen::PreproTSG<SimpleStores, Acc = cpp_tree_gen::Acc>,
{
    // handling the parsing explicitly in this function is a good idea
    // to control complex stuff like timeout, instead of the call on next line
    // let tree_sitter_parse = cpp_tree_gen::CppTreeGen::<TStore>::tree_sitter_parse(text);

    let mut parser = tree_sitter::Parser::new();
    // TODO see if a timeout of a cancellation flag could be useful
    // const MINUTE: u64 = 60 * 1000 * 1000;
    // parser.set_timeout_micros(MINUTE);
    // parser.set_cancellation_flag(flag);
    parser
        .set_language(&hyperast_gen_ts_cpp::language())
        .unwrap();
    let time = Instant::now();
    let tree = parser.parse(text, None);
    let parsing_time = time.elapsed();
    let Some(tree) = tree else {
        unimplemented!("You set a timeout or an cancel flag, so it now requires special handling.")
        // return FileProcessingResult::ParsingTimedout(parsing_time)
    };
    if tree.root_node().has_error() {
        log::warn!("bad CST: {:?}", name.try_str());
        if PROPAGATE_ERROR_ON_BAD_CST_NODE {
            return Err(FailedParsing {
                parsing_time,
                tree,
                error: "CST contains parsing errors",
            });
        }
    };
    let time = Instant::now();
    let subtree = tree_gen.generate_file(name.as_bytes(), text, tree.walk());
    let processing_time = time.elapsed();
    Ok(SuccessProcessing {
        parsing_time,
        processing_time,
        node: subtree,
    })
}

pub struct CppAcc {
    pub(crate) primary:
        BasicDirAcc<NodeIdentifier, LabelIdentifier, SubTreeMetrics<SyntaxNodeHashs<u32>>>,
}

impl CppAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            primary: BasicDirAcc::new(name),
        }
    }
}

impl From<String> for CppAcc {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl CppAcc {
    // pub(crate) fn push_file(
    //     &mut self,
    //     name: LabelIdentifier,
    //     full_node: cpp_tree_gen::FNode,
    // ) {
    //     self.children.push(full_node.local.compressed_node.clone());
    //     self.children_names.push(name);
    //     self.metrics.acc(full_node.local.metrics);
    //     full_node
    //         .local
    //         .ana
    //         .unwrap()
    //         .acc(&Type::Directory, &mut self.ana);
    // }
    // pub(crate) fn push(&mut self, name: LabelIdentifier, full_node: cpp_tree_gen::Local) {
    //     self.children.push(full_node.compressed_node);
    //     self.children_names.push(name);
    //     self.metrics.acc(full_node.metrics);

    //     if let Some(ana) = full_node.ana {
    //         if ana.estimated_refs_count() < MAX_REFS && self.skiped_ana == false {
    //             ana.acc(&Type::Directory, &mut self.ana);
    //         } else {
    //             self.skiped_ana = true;
    //         }
    //     }
    // }
    pub(crate) fn push(
        &mut self,
        name: LabelIdentifier,
        full_node: cpp_tree_gen::Local,
        skiped_ana: bool,
    ) {
        self.primary
            .push(name, full_node.compressed_node, full_node.metrics);
    }
}

impl hyperast::tree_gen::Accumulator for CppAcc {
    type Node = (LabelIdentifier, (cpp_tree_gen::Local, IsSkippedAna));
    fn push(&mut self, (name, (full_node, skiped_ana)): Self::Node) {
        self.primary
            .push(name, full_node.compressed_node, full_node.metrics);
    }
}

impl Accumulator for CppAcc {
    type Unlabeled = (cpp_tree_gen::Local, IsSkippedAna);
}
