use std::fmt;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Pair<T1, T2>(pub(crate) T1, pub(crate) T2);

// impl<T1,T2> Pair<T1,T2> {}

impl<T1, T2> From<(T1, T2)> for Pair<T1, T2> {
    fn from(pair: (T1, T2)) -> Self {
        Pair {
            0: pair.0,
            1: pair.1,
        }
    }
}

impl<T1, T2> fmt::Display for Pair<T1, T2>
where
    T1: fmt::Display,
    T2: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Pair(first, second) = self;
        write!(f, "({}, {})", first, second)
    }
}

// public JSitterJavaTreeGenerator() {
//     super();
//     NodeType nodetype = new NodeType("source_file");
//     Language<NodeType> lang = Language.load(
//             nodetype,
//             "java",
//             "tree_sitter_java",
//             "libtsjava",
//             Language.class.getClassLoader());
//     lang.register(nodetype);

//     this.parser = lang.parser();
// }
