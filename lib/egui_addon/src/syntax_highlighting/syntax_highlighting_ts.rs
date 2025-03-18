use std::sync::{Arc, Mutex};

use egui::text::LayoutJob;

use crate::Lang;

use crate::code_editor::default_parser;
use crate::code_editor::editor_content::EditAwareString;

#[cfg(feature = "syntect")]
pub(crate) use super::syntect::CodeTheme;

// #[cfg(not(feature = "syntect"))]
// pub(crate) use super::syntect::CodeTheme;

// #[cfg(not(feature = "syntect"))]
// pub(crate) use super::syntect::CodeTheme;

use super::TokenType;

// /// View some code with syntax highlighting and selection.
// pub fn code_view_ui(ui: &mut egui::Ui, mut code: &str) {
//     let language = todo!(); //"rs";
//     let theme = CodeTheme::from_memory(ui.ctx());

//     let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
//         let layout_job = highlight(ui.ctx(), &theme, &mut string.to_string().into(), language);
//         // layout_job.wrap.max_width = wrap_width; // no wrapping
//         ui.fonts(|f| f.layout_job(layout_job))
//     };

//     ui.add(
//         egui::TextEdit::multiline(&mut code)
//             .font(egui::TextStyle::Monospace) // for cursor height
//             .code_editor()
//             .desired_rows(1)
//             .lock_focus(true)
//             .layouter(&mut layouter),
//     );
// }

// /// Memoized Code highlighting
// pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &Lang) -> LayoutJob {
//     impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &Lang), LayoutJob> for Highlighter {
//         fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &Lang)) -> LayoutJob {
//             {
//                 let mut parser = self.parser.lock().unwrap();
//                 if parser.language().as_ref() != Some(&lang.lang) {
//                     parser.set_language(&lang.lang).unwrap();
//                     self.parsed = None;
//                 }
//                 self.parsed = None;
//                 self.parsed = parser.parse(code, self.parsed.take().as_ref()).unwrap();
//             }
//             self.highlight2(theme, code)
//         }
//     }

//     // pub(super) parsed: Option<tree_sitter::Tree>,
//     type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

//     ctx.memory_mut(|mem| {
//         mem.caches
//             .cache::<HighlightCache>()
//             .get((theme, code, language))
//     })
// }

/// Memoized Code highlighting
#[cfg(not(feature = "syntect"))]
pub fn highlight(
    ctx: &egui::Context,
    theme: &super::simple::CodeTheme,
    code: &EditAwareString,
    language: &Lang,
) -> LayoutJob {
    use crate::code_editor::generic_text_buffer::TextBuffer;

    impl
        egui::util::cache::ComputerMut<
            (&super::simple::CodeTheme, &EditAwareString, &Lang),
            LayoutJob,
        > for Highlighter
    {
        fn compute(
            &mut self,
            (theme, code, lang): (&super::simple::CodeTheme, &EditAwareString, &Lang),
        ) -> LayoutJob {
            {
                let mut parser = self.parser.lock().unwrap();
                if parser.language().as_ref() != Some(&lang.lang) {
                    parser.set_language(&lang.lang).unwrap();
                    self.parsed = None;
                    code.edit.take();
                }
                if code.reset.swap(false, std::sync::atomic::Ordering::Relaxed) {
                    self.parsed = None;
                    code.edit.take();
                } else if let (Some(edit), Some(parsed)) = (code.edit.take(), &mut self.parsed) {
                    parsed.edit(&edit.into())
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.parsed = parser.parse(code.as_str(), self.parsed.take().as_ref());
                    if self.parsed.as_ref().unwrap().root_node().has_error() {
                        self.parsed = None;
                        code.edit.take();
                        self.parsed = parser.parse(code.as_str(), self.parsed.take().as_ref());
                        if !self.parsed.as_ref().unwrap().root_node().has_error() {
                            panic!()
                        }
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    self.parsed = parser
                        .parse(code.as_str(), self.parsed.take().as_ref())
                        .unwrap();
                    if self.parsed.as_ref().unwrap().root_node().has_error() {
                        self.parsed = None;
                        code.edit.take();
                        self.parsed = parser
                            .parse(code.as_str(), self.parsed.take().as_ref())
                            .unwrap();
                        if !self.parsed.as_ref().unwrap().root_node().has_error() {
                            panic!()
                        }
                    }
                }
            }
            self.highlight2(theme, &code.string)
        }
    }

    // pub(super) parsed: Option<tree_sitter::Tree>,
    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    ctx.memory_mut(|mem| {
        mem.caches
            .cache::<HighlightCache>()
            .get((theme, code, language))
    })
}

// ----------------------------------------------------------------------------

#[cfg(feature = "syntect")]
struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

#[cfg(feature = "syntect")]
impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

#[cfg(feature = "syntect")]
impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, code: &str, lang: &str) -> LayoutJob {
        self.highlight_impl(theme, code, lang).unwrap_or_else(|| {
            // Fallback:
            LayoutJob::simple(
                code.into(),
                egui::FontId::monospace(12.0),
                if theme.dark_mode {
                    egui::Color32::LIGHT_GRAY
                } else {
                    egui::Color32::DARK_GRAY
                },
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(&self, theme: &CodeTheme, text: &str, language: &str) -> Option<LayoutJob> {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;

        let theme = theme.syntect_theme.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        use egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight_line(line, &self.ps).ok()? {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::monospace(12.0),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

#[cfg(feature = "syntect")]
fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

// ----------------------------------------------------------------------------

#[cfg(not(feature = "syntect"))]
struct Highlighter {
    parser: Arc<Mutex<tree_sitter::Parser>>,
    pub(super) parsed: Option<tree_sitter::Tree>,
}

#[cfg(not(feature = "syntect"))]
impl Default for Highlighter {
    fn default() -> Self {
        Self {
            parser: Mutex::new(default_parser()).into(),
            parsed: None,
        }
    }
}
#[cfg(not(feature = "syntect"))]
impl Highlighter {
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight2(&self, theme: &super::simple::CodeTheme, text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();

        const HIGHLIGHT_NAMES: &[&str; 19] = &[
            "attribute",
            "constant",
            "function.builtin",
            "function",
            "keyword",
            "operator",
            "property",
            "punctuation",
            "punctuation.bracket",
            "punctuation.delimiter",
            "string",
            "string.special",
            "tag",
            "type",
            "type.builtin",
            "variable",
            "variable.builtin",
            "variable.parameter",
            "comment",
        ];

        const HIGHLIGHT_ENUMS: &[TokenType; 19] = &[
            TokenType::Punctuation,   // "attribute",
            TokenType::Literal,       // "constant",
            TokenType::Keyword,       // "function.builtin",
            TokenType::Keyword,       // "function",
            TokenType::Keyword,       // "keyword",
            TokenType::Punctuation,   // "operator",
            TokenType::Punctuation,   // "property",
            TokenType::Punctuation,   // "punctuation",
            TokenType::Punctuation,   // "punctuation.bracket",
            TokenType::Punctuation,   // "punctuation.delimiter",
            TokenType::StringLiteral, // "string",
            TokenType::StringLiteral, // "string.special",
            TokenType::Punctuation,   // "tag",
            TokenType::Punctuation,   // "type",
            TokenType::Keyword,       // "type.builtin",
            TokenType::Punctuation,   // "variable",
            TokenType::Keyword,       // "variable.builtin",
            TokenType::Punctuation,   // "variable.parameter",
            TokenType::Comment,       // "comment",
        ];
        use tree_sitter_highlight::Highlighter;

        let mut highlighter = Highlighter::new();

        use tree_sitter_highlight::HighlightConfiguration;

        let mut javascript_config = HighlightConfiguration::new(
            tree_sitter_javascript::language(),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTION_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();

        javascript_config.configure(HIGHLIGHT_NAMES);

        use tree_sitter_highlight::HighlightEvent;

        let highlights = highlighter
            .highlight(&javascript_config, text.as_bytes(), None, |_| None)
            .unwrap();

        let mut curr_style = TokenType::Punctuation;
        let mut prev = 0;

        for event in highlights {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    if prev < start {
                        job.append(
                            &text[prev..end],
                            0.0,
                            theme.formats[TokenType::Whitespace].clone(),
                        );
                    }
                    job.append(&text[start..end], 0.0, theme.formats[curr_style].clone());
                    prev = end;
                }
                HighlightEvent::HighlightStart(s) => {
                    curr_style = HIGHLIGHT_ENUMS[s.0];
                }
                HighlightEvent::HighlightEnd => {
                    curr_style = TokenType::Punctuation;
                }
            }
        }

        // while !text.is_empty() {
        //     let end = text.len(); //text.find('\n').unwrap_or(text.len());
        //     job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
        //     text = &text[end..];
        // }
        // while !text.is_empty() {
        //     if text.starts_with("//") {
        //         let end = text.find('\n').unwrap_or(text.len());
        //         job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
        //         text = &text[end..];
        //     } else {
        //         let mut it = text.char_indices();
        //         it.next();
        //         let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
        //         job.append(
        //             &text[..end],
        //             0.0,
        //             theme.formats[TokenType::Punctuation].clone(),
        //         );
        //         text = &text[end..];
        //     }
        // }

        job
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight2(&self, theme: &super::simple::CodeTheme, mut text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();
        // eframe::web_sys::console::log_1(&text.into());

        const HIGHLIGHT_NAMES: &[&str; 19] = &[
            "attribute",
            "constant",
            "function.builtin",
            "function",
            "keyword",
            "operator",
            "property",
            "punctuation",
            "punctuation.bracket",
            "punctuation.delimiter",
            "string",
            "string.special",
            "tag",
            "type",
            "type.builtin",
            "variable",
            "variable.builtin",
            "variable.parameter",
            "comment",
        ];

        const HIGHLIGHT_ENUMS: &[TokenType; 19] = &[
            TokenType::Punctuation,   // "attribute",
            TokenType::Literal,       // "constant",
            TokenType::Keyword,       // "function.builtin",
            TokenType::Keyword,       // "function",
            TokenType::Keyword,       // "keyword",
            TokenType::Punctuation,   // "operator",
            TokenType::Punctuation,   // "property",
            TokenType::Punctuation,   // "punctuation",
            TokenType::Punctuation,   // "punctuation.bracket",
            TokenType::Punctuation,   // "punctuation.delimiter",
            TokenType::StringLiteral, // "string",
            TokenType::StringLiteral, // "string.special",
            TokenType::Punctuation,   // "tag",
            TokenType::Punctuation,   // "type",
            TokenType::Keyword,       // "type.builtin",
            TokenType::Punctuation,   // "variable",
            TokenType::Keyword,       // "variable.builtin",
            TokenType::Punctuation,   // "variable.parameter",
            TokenType::Comment,       // "comment",
        ];

        // use crate::app::ts_highlight::web3::{HighlightConfiguration, Highlighter};

        // let mut highlighter = Highlighter::new();

        let lang = self.parser.lock().unwrap().language().unwrap();
        let lang: web_tree_sitter_sg::Language = unsafe { std::mem::transmute(lang) };
        // let mut config = HighlightConfiguration::new(
        //     lang,
        //     HIGHLIGHT_QUERY, //tree_sitter_javascript::HIGHLIGHT_QUERY,
        //     "",              //tree_sitter_javascript::INJECTION_QUERY,
        //     "",              //tree_sitter_javascript::LOCALS_QUERY,
        // )
        // .unwrap();

        let query = lang.query(&HIGHLIGHT_QUERY.into()).unwrap();
        let tree: &web_tree_sitter_sg::Tree =
            unsafe { std::mem::transmute(self.parsed.as_ref().unwrap()) };
        // eframe::web_sys::console::log_1(&tree.root_node().to_string());
        let node: &wasm_bindgen::JsValue =
            unsafe { std::mem::transmute(self.parsed.as_ref().unwrap().root_node()) };

        let start_position = unsafe {
            std::mem::transmute(&self.parsed.as_ref().unwrap().root_node().start_position())
        };
        let end_position = unsafe {
            std::mem::transmute(&self.parsed.as_ref().unwrap().root_node().end_position())
        };

        let matches = query.matches(
            unsafe { std::mem::transmute(&self.parsed.as_ref().unwrap().root_node()) },
            Some(start_position),
            Some(end_position),
        );
        // eframe::web_sys::console::log_1(&matches.len().into());

        let mut curr_index = 0;
        for matche in matches.iter() {
            // eframe::web_sys::console::log_1(matche);
            let m: &web_tree_sitter_sg::QueryMatch = unsafe { std::mem::transmute(matche) };
            // eframe::web_sys::console::log_1(&m.captures().len().into());
            let cap = m.captures();
            if cap.len() == 1 {
                let cap = &cap[0];
                let cap: &web_tree_sitter_sg::QueryCapture = unsafe { std::mem::transmute(cap) };
                let name = cap.name();
                // eframe::web_sys::console::log_1(&name);
                let name = name.as_string().unwrap();
                let tt = match name.as_str() {
                    "attribute" => TokenType::Punctuation,
                    "constant" => TokenType::Punctuation,
                    // "function.builtin" => TokenType::Keyword,
                    // "function" => TokenType::Keyword,
                    "keyword" => TokenType::Keyword,
                    // "operator" => TokenType::Punctuation,
                    // "property" => TokenType::Punctuation,
                    "punctuation" => TokenType::Punctuation,
                    // "punctuation.bracket"=> TokenType::Punctuation,
                    // "punctuation.delimiter"=> TokenType::Punctuation,
                    "string" => TokenType::StringLiteral,
                    // "string.special" => TokenType::Punctuation,
                    // "tag" => TokenType::Punctuation,
                    // "type" => TokenType::Punctuation,
                    // "type.builtin" => TokenType::Punctuation,
                    // "variable" => TokenType::Punctuation,
                    // "variable.builtin" => TokenType::Keyword,
                    // "variable.parameter" => TokenType::Punctuation,
                    "comment" => TokenType::Comment,
                    _ => continue,
                };
                let node = cap.node();
                // eframe::web_sys::console::log_1(&node);
                let node: &web_tree_sitter_sg::SyntaxNode = unsafe { std::mem::transmute(&node) };
                if node.child_count() != 0 {
                    continue;
                }
                let start = node.start_index();
                let end = node.end_index();
                // eframe::web_sys::console::log_0();
                // eframe::web_sys::console::log_1(&name.clone().into());
                // eframe::web_sys::console::log_1(&curr_index.into());
                // eframe::web_sys::console::log_1(&start.into());
                if text.len() <= curr_index as usize {
                    break;
                }
                if curr_index < start {
                    // eframe::web_sys::console::log_1(
                    //     &text[curr_index as usize..(start as usize).min(text.len())].into(),
                    // );
                    job.append(
                        &text[curr_index as usize..(start as usize).min(text.len())],
                        0.0,
                        theme.formats[TokenType::Punctuation].clone(),
                    );
                } else if curr_index > start {
                    continue;
                }
                // eframe::web_sys::console::log_1(&end.into());
                if text.len() <= start as usize {
                    break;
                }
                // eframe::web_sys::console::log_1(&text[start as usize..(end as usize).min(text.len())].into());
                job.append(
                    &text[start as usize..(end as usize).min(text.len())],
                    0.0,
                    theme.formats[tt].clone(),
                );
                curr_index = end;
            }
        }
        if curr_index < text.len() as u32 - 1 {
            job.append(
                &text[curr_index as usize..],
                0.0,
                theme.formats[TokenType::Punctuation].clone(),
            );
        }

        // config.configure(HIGHLIGHT_NAMES);

        // let highlights = highlighter
        //     .highlight(&config, text.as_bytes(), None, |_| None)
        //     .unwrap();

        // panic!("{}", highlights.into_iter().count());

        // while !text.is_empty() {
        //     let end = text.len(); //text.find('\n').unwrap_or(text.len());
        //     job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
        //     text = &text[end..];
        // }

        // while !text.is_empty() {
        //     if text.starts_with("//") {
        //         let end = text.find('\n').unwrap_or(text.len());
        //         job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
        //         text = &text[end..];
        //     } else {
        //         let mut it = text.char_indices();
        //         it.next();
        //         let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
        //         job.append(
        //             &text[..end],
        //             0.0,
        //             theme.formats[TokenType::Punctuation].clone(),
        //         );
        //         text = &text[end..];
        //     }
        // }

        job
    }
}

#[cfg(not(feature = "syntect"))]
impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(
        &self,
        theme: &super::simple::CodeTheme,
        mut text: &str,
        _language: &str,
    ) -> LayoutJob {
        // Extremely simple syntax highlighter for when we compile without syntect

        let mut job = LayoutJob::default();

        while !text.is_empty() {
            if text.starts_with("//") {
                let end = text.find('\n').unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..]
                    .find('"')
                    .map(|i| i + 2)
                    .or_else(|| text.find('\n'))
                    .unwrap_or(text.len());
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::StringLiteral].clone(),
                );
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_keyword(word) {
                    TokenType::Keyword
                } else {
                    TokenType::Literal
                };
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Whitespace].clone(),
                );
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Punctuation].clone(),
                );
                text = &text[end..];
            }
        }

        job
    }
}

#[cfg(not(feature = "syntect"))]
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

// const HIGHLIGHT_QUERY: &str = r#"

// "#;

const HIGHLIGHT_QUERY: &str = r#"
        
; Special identifiers
;--------------------

([
(identifier)
(shorthand_property_identifier)
(shorthand_property_identifier_pattern)
] @constant
(#match? @constant "^[A-Z_][A-Z\\d_]+$"))


((identifier) @constructor
(#match? @constructor "^[A-Z]"))

((identifier) @variable.builtin
(#match? @variable.builtin "^(arguments|module|console|window|document)$")
(#is-not? local))

((identifier) @function.builtin
(#eq? @function.builtin "require")
(#is-not? local))

; Function and method definitions
;--------------------------------

(function
name: (identifier) @function)
(function_declaration
name: (identifier) @function)
(method_definition
name: (property_identifier) @function.method)

(pair
key: (property_identifier) @function.method
value: [(function) (arrow_function)])

(assignment_expression
left: (member_expression
property: (property_identifier) @function.method)
right: [(function) (arrow_function)])

(variable_declarator
name: (identifier) @function
value: [(function) (arrow_function)])

(assignment_expression
left: (identifier) @function
right: [(function) (arrow_function)])

; Function and method calls
;--------------------------

(call_expression
function: (identifier) @function)

(call_expression
function: (member_expression
property: (property_identifier) @function.method))

; Variables
;----------

(identifier) @variable

; Properties
;-----------

(property_identifier) @property

; Literals
;---------

(this) @variable.builtin
(super) @variable.builtin

[
(true)
(false)
(null)
(undefined)
] @constant.builtin

(comment) @comment

[
(string)
(template_string)
] @string

(regex) @string.special
(number) @number

; Tokens
;-------

(template_substitution
"${" @punctuation.special
"}" @punctuation.special) @embedded

[
";"
;(optional_chain)
"."
","
] @punctuation.delimiter

[
"-"
"--"
"-="
"+"
"++"
"+="
"*"
"*="
"**"
"**="
"/"
"/="
"%"
"%="
"<"
"<="
"<<"
"<<="
"="
"=="
"==="
"!"
"!="
"!=="
"=>"
">"
">="
">>"
">>="
">>>"
">>>="
"~"
"^"
"&"
"|"
"^="
"&="
"|="
"&&"
"||"
"??"
"&&="
"||="
"??="
] @operator

[
"("
")"
"["
"]"
"{"
"}"
]  @punctuation.bracket

[
"as"
"async"
"await"
"break"
"case"
"catch"
"class"
"const"
"continue"
"debugger"
"default"
"delete"
"do"
"else"
"export"
"extends"
"finally"
"for"
"from"
"function"
"get"
"if"
"import"
"in"
"instanceof"
"let"
"new"
"of"
"return"
"set"
"static"
"switch"
"target"
"throw"
"try"
"typeof"
"var"
"void"
"while"
"with"
"yield"
] @keyword

"#;
