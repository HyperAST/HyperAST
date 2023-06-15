#[derive(Clone)]
pub(super) struct Scripts {
    pub(crate) init: &'static str,
    pub(crate) filter: &'static str,
    pub(crate) accumulate: &'static str,
}
#[derive(Clone)]
pub(super) struct Example {
    pub(crate) name: &'static str,
    pub(crate) commit: Commit,
    pub(crate) config: Config,
    pub(crate) commits: usize,
    pub(crate) scripts: Scripts,
}

#[derive(Clone)]
pub(crate) enum Forge {
    GitHub,
    GitLab,
}

#[derive(Clone)]
pub(crate) enum Config {
    MavenJava,
    MakeCpp,
}

#[derive(Clone)]
pub(crate) struct Repo {
    pub(crate) forge: Forge,
    pub(crate) user: &'static str,
    pub(crate) name: &'static str,
}
#[derive(Clone)]
pub(crate) struct Commit {
    pub(crate) repo: Repo,
    pub(crate) id: &'static str,
}

impl From<&Repo> for super::super::types::Repo {
    fn from(value: &Repo) -> Self {
        Self {
            user: value.user.into(),
            name: value.name.into(),
        }
    }
}
impl From<&Commit> for super::super::types::Commit {
    fn from(value: &Commit) -> Self {
        Self {
            repo: (&value.repo).into(),
            id: value.id.into(),
        }
    }
}


pub(super) const EXAMPLES: &[Example] = &[
    Example {
        name: "default example (Java)",
        commit: Commit {
            repo: Repo {
                forge: Forge::GitHub,
                user: "INRIA",
                name: "spoon",
            },
            id: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
        },
        config: Config::MavenJava,
        commits: 1,
        scripts: Scripts {
            init: r##"#{ depth:0, files: 0, type_decl: 0 }"##,
            filter: r##"if is_directory() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        files: s.files,
        type_decl: s.type_decl,
    }])
} else if is_file() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        type_decl: s.type_decl,
    }])
} else {
    []
}"##,
            accumulate: r##"if is_directory() {
    p.files += s.files;
    p.type_decl += s.type_decl;
} else if is_file() {
    p.files += 1;
    p.type_decl += s.type_decl;
} else if is_type_decl() {
    p.type_decl += 1; 
}"##,
        },
    },
    Example {
        name: "default example (Cpp)",
        commit: Commit {
            repo: Repo {
                forge: Forge::GitHub,
                user: "official-stockfish",
                name: "Stockfish",
            },
            id: "7f2eb10e93879bc569c7ddf6fb51d6f812cc477c",
        },
        config: Config::MakeCpp,
        commits: 1,
        scripts: Scripts {
            init: r##"#{ depth:0, files: 0, type_decl: 0 }"##,
            filter: r##"if is_directory() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        files: s.files,
        type_decl: s.type_decl,
    }])
} else if is_file() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        type_decl: s.type_decl,
    }])
} else if type() == "preproc_ifdef"
        || type() == "namespace_definition"
        || type() == "declaration_list" {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        type_decl: s.type_decl,
    }])
} else {
    []
}"##,
            accumulate: r##"if is_directory() {
    p.files += s.files;
    p.type_decl += s.type_decl;
} else if is_file() {
    p.files += 1;
    p.type_decl += s.type_decl;
} else if type() == "preproc_ifdef"
        || type() == "namespace_definition"
        || type() == "declaration_list" {
    p.type_decl += s.type_decl;
} else if type() == "preproc_include" {
    p.type_decl += 1; 
} else if type() == "declaration" {
    p.type_decl += 1; 
}"##,
        },
    },
    Example {
        name: "naive size on Stockfish",
        commit: Commit {
            repo: Repo {
                forge: Forge::GitHub,
                user: "official-stockfish",
                name: "Stockfish",
            },
            id: "7f2eb10e93879bc569c7ddf6fb51d6f812cc477c",
        },
        config: Config::MakeCpp,
        commits: 10,
        scripts: Scripts {
            init: r##"#{ depth:0, files: 0, size: 0 }"##,
            filter: r##"if is_directory() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        files: s.files,
        size: s.size,
    }])
} else if is_file() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        size: s.size,
    }])
} else {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        size: s.size,
    }])
}"##,
            accumulate: r##"if is_directory() {
    p.files += s.files;
    p.size += s.size + 1;
} else if is_file() {
    p.files += 1;
    p.size += s.size + 1;
} else {
    p.size += s.size + 1; 
}"##,
        },
    },
    Example {
        name: "metadata size on Stockfish",
        commit: Commit {
            repo: Repo {
                forge: Forge::GitHub,
                user: "official-stockfish",
                name: "Stockfish",
            },
            id: "7f2eb10e93879bc569c7ddf6fb51d6f812cc477c",
        },
        config: Config::MakeCpp,
        commits: 10,
        scripts: Scripts {
            init: r##"#{ depth:0, files: 0, size: 0 }"##,
            filter: r##"if is_directory() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        files: s.files,
        size: s.size,
    }])
} else if is_file() {
    []
} else { // will not reach
    []
}"##,
            accumulate: r##"if is_directory() {
    p.files += s.files;
    p.size += s.size + 1;
} else if is_file() {
    p.files += 1;
    p.size += size();
} else { // will not reach
    p.size += size(); 
}"##,
        },
    },
    Example {
        name: "naive size on Spoon",
        commit: Commit {
            repo: Repo {
                forge: Forge::GitHub,
                user: "INRIA",
                name: "spoon",
            },
            id: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
        },
        config: Config::MavenJava,
        commits: 1,
        scripts: Scripts {
            init: r##"#{ depth:0, files: 0, size: 0 }"##,
            filter: r##"if is_directory() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        files: s.files,
        size: s.size,
    }])
} else if is_file() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        size: s.size,
    }])
} else {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        size: s.size,
    }])
}"##,
            accumulate: r##"if is_directory() {
    p.files += s.files;
    p.size += s.size + 1;
} else if is_file() {
    p.files += 1;
    p.size += s.size + 1;
} else {
    p.size += s.size + 1; 
}"##,
        },
    },
    Example {
        name: "metadata size on Spoon",
        commit: Commit {
            repo: Repo {
                forge: Forge::GitHub,
                user: "INRIA",
                name: "spoon",
            },
            id: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
        },
        config: Config::MavenJava,
        commits: 10,
        scripts: Scripts {
            init: r##"#{ depth:0, files: 0, size: 0 }"##,
            filter: r##"if is_directory() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        files: s.files,
        size: s.size,
    }])
} else if is_file() {
    []
} else { // will not reach
    []
}"##,
            accumulate: r##"if is_directory() {
    p.files += s.files;
    p.size += s.size + 1;
} else if is_file() {
    p.files += 1;
    p.size += size();
} else { // will not reach
    p.size += size(); 
}"##,
        },
    },
];