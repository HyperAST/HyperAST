//hyper_ast::types::AnyType
pub(super) fn parse_tsquery<
    'a, // ,K: TryFrom<&'a str>
>(
    query: &'a str,
) -> Result<Pattern, VerboseError<&str>>
// where
//     <K as TryFrom<&'a str>>::Error: std::fmt::Debug,
{
    all_consuming(parse_query)(query).finish().map(|(i, o)| o)
}

use std::fmt::{Debug, Display};

use nom::bytes::streaming::is_not;
use nom::character::complete::{anychar, newline};
use nom::character::streaming::alphanumeric1;
use nom::combinator::{all_consuming, eof, recognize};
use nom::multi::{many0, many0_count, many1, many_m_n};
use nom::sequence::{pair, tuple};
use nom::Finish;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::multispace0,
    character::complete::multispace1,
    character::complete::{alpha1, alphanumeric1 as alphanumeric, char, digit1, one_of},
    combinator::{cut, map, map_res, opt, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated},
    Err, IResult, Parser,
};
use syn::Pat;
/// parser combinators are constructed from the bottom up:
/// first we write parsers for the smallest elements (here a space character),
/// then we'll combine them in larger parsers
fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| chars.contains(c))(i)
}
/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace1, inner, multispace1)
}
pub fn identifier(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, alphanumeric1},
        combinator::recognize,
        multi::many0_count,
        sequence::pair,
        IResult,
    };
    // stream->next == '_' ||
    // stream->next == '-' ||
    // stream->next == '.' ||
    // stream->next == '?' ||
    // stream->next == '!'
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((
            alphanumeric1,
            tag("_"),
            tag("-"),
            tag("."),
            tag("?"),
            tag("!"),
        ))),
    ))(input)
}

/// We start by defining the types that define the shape of data that we want.
/// In this case, we want something tree-like

/// Starting from the most basic, we define some built-in functions that our lisp has
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BuiltIn {
    Plus,
    Minus,
    Times,
    Divide,
    Equal,
    Not,
}

/// We now wrap this type and a few other primitives into our Atom type.
/// Remember from before that Atoms form one half of our language.

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Atom {
    Num(i32),
    Keyword(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
}

/// The remaining half is Lists. We implement these as recursive Expressions.
/// For a list of numbers, we have `'(1 2 3)`, which we'll parse to:
/// ```
/// Expr::Quote(vec![Expr::Constant(Atom::Num(1)),
///                  Expr::Constant(Atom::Num(2)),
///                  Expr::Constant(Atom::Num(3))])
/// Quote takes an S-expression and prevents evaluation of it, making it a data
/// structure that we can deal with programmatically. Thus any valid expression
/// is also a valid data structure in Lisp itself.

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Pattern {
    Alternation(Vec<Pattern>),
    NamedNode { k: Option<String>, cs: Vec<Pattern> },
    Negated(String),
    WithField { field: String, pat: Box<Pattern> },
    Dot,
    Anon(String),
    Comment(String),
    Capture { name: String, pat: Box<Pattern> },
}

/// Continuing the trend of starting from the simplest piece and building up,
/// we start by creating a parser for the built-in operator functions.
fn parse_builtin_op(i: &str) -> IResult<&str, BuiltIn, VerboseError<&str>> {
    // one_of matches one of the characters we give it
    let (i, t) = one_of("+-*/=")(i)?;

    // because we are matching single character tokens, we can do the matching logic
    // on the returned value
    Ok((
        i,
        match t {
            '+' => BuiltIn::Plus,
            '-' => BuiltIn::Minus,
            '*' => BuiltIn::Times,
            '/' => BuiltIn::Divide,
            '=' => BuiltIn::Equal,
            _ => unreachable!(),
        },
    ))
}

fn parse_builtin(i: &str) -> IResult<&str, BuiltIn, VerboseError<&str>> {
    // alt gives us the result of first parser that succeeds, of the series of
    // parsers we give it
    alt((
        parse_builtin_op,
        // map lets us process the parsed output, in this case we know what we parsed,
        // so we ignore the input and return the BuiltIn directly
        map(tag("not"), |_| BuiltIn::Not),
    ))
    .parse(i)
}

/// Our boolean values are also constant, so we can do it the same way
fn parse_bool(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    alt((
        map(tag("#t"), |_| Atom::Boolean(true)),
        map(tag("#f"), |_| Atom::Boolean(false)),
    ))
    .parse(i)
}

/// The next easiest thing to parse are keywords.
/// We introduce some error handling combinators: `context` for human readable errors
/// and `cut` to prevent back-tracking.
///
/// Put plainly: `preceded(tag(":"), cut(alpha1))` means that once we see the `:`
/// character, we have to see one or more alphabetic characters or the input is invalid.
fn parse_keyword(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        context("keyword", preceded(tag(":"), cut(alpha1))),
        |sym_str: &str| Atom::Keyword(sym_str.to_string()),
    )
    .parse(i)
}

/// Next up is number parsing. We're keeping it simple here by accepting any number (> 1)
/// of digits but ending the program if it doesn't fit into an i32.
fn parse_num(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    alt((
        map_res(digit1, |digit_str: &str| {
            digit_str.parse::<i32>().map(Atom::Num)
        }),
        map(preceded(tag("-"), digit1), |digit_str: &str| {
            Atom::Num(-digit_str.parse::<i32>().unwrap())
        }),
    ))
    .parse(i)
}

/// Before continuing, we need a helper function to parse lists.
/// A list starts with `(` and ends with a matching `)`.
/// By putting whitespace and newline parsing here, we can avoid having to worry about it
/// in much of the rest of the parser.
///
/// Unlike the previous functions, this function doesn't take or consume input, instead it
/// takes a parsing function and returns a new parsing function.
fn s_exp<'a, O1, F>(inner: F) -> impl Parser<&'a str, O1, VerboseError<&'a str>>
where
    F: Parser<&'a str, O1, VerboseError<&'a str>>,
{
    delimited(
        char('('),
        preceded(multispace0, inner),
        context("closing paren", cut(preceded(multispace0, char(')')))),
    )
}

fn parse_alternation(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    let alternation_inner = map(many0(parse_pattern), Pattern::Alternation);
    delimited(
        char('['),
        preceded(multispace0, alternation_inner),
        context("closing bracket", cut(preceded(multispace0, char(')')))),
    )
    .parse(i)
}

fn parse_grouped_seq(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    // let application_inner = todo!();
    // s_exp(application_inner).parse(i)
    todo!()
}

fn parse_named_node(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    alt((
        tuple((
            identifier,
            preceded(multispace1, many0(parse_named_node_child)),
        ))
        .map(|(k, cs)| Pattern::NamedNode {
            k: k.ne("_").then(|| k.to_string()),
            cs,
        }),
        terminated(identifier, multispace0).map(|k| Pattern::NamedNode {
            k: k.ne("_").then(|| k.to_string()),
            cs: vec![],
        }),
    ))
    .parse(i)
}
fn parse_named_node_child(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    preceded(
        multispace0,
        alt((
            parse_pattern,
            parse_named_node_negated_field,
            parse_with_field,
            parse_dot,
        )),
    )
    .parse(i)
}
fn parse_named_node_negated_field(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    preceded(multispace0, preceded(char('!'), identifier))
        .map(|x| Pattern::Negated(x.to_string()))
        .parse(i)
}
fn parse_with_field(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    preceded(
        multispace0,
        tuple((identifier, char(':'), parse_s_exp_like)),
    )
    .map(|(field, _, pat)| Pattern::WithField {
        field: field.to_string(),
        pat: Box::new(pat),
    })
    .parse(i)
}

fn parse_dot(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    map(preceded(multispace0, char('.')), |x| Pattern::Dot)(i)
}

fn parse_predicate(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    todo!()
}

/// A nom parser has the following signature:
/// `Input -> IResult<Input, Output, Error>`, with `IResult` defined as:
/// `type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), Err<E>>;`
///
/// most of the times you can ignore the error type and use the default (but this
/// examples shows custom error types later on!)
///
/// Here we use `&str` as input type, but nom parsers can be generic over
/// the input type, and work directly with `&[u8]` or any other type that
/// implements the required traits.
///
/// Finally, we can see here that the input and output type are both `&str`
/// with the same lifetime tag. This means that the produced value is a subslice
/// of the input data. and there is no allocation needed. This is the main idea
/// behind nom's performance.
fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    // let p = many0_count(alt((
    //         alphanumeric1,
    //         tag("{"),
    //         tag("}"),
    //         tag("("),
    //         tag(")"),
    //         tag("_"),
    //         tag("-"),
    //         tag("."),
    //         tag("?"),
    //         tag("!"),
    //     )),
    // );
    let p = take_while(move |c: char| !"\\\"".contains(c));
    escaped(recognize(p), '\\', one_of("\"n\\"))(i)
}

fn parse_anonymous(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    context(
        "anonymous",
        preceded(
            char('\"'),
            cut(terminated(
                parse_str.map(ToString::to_string).map(Pattern::Anon),
                char('\"'),
            )),
        ),
    )(i)
}

/// this parser combines the previous `parse_str` parser, that recognizes the
/// interior of a string, with a parse to recognize the double quote character,
/// before the string (using `preceded`) and after the string (using `terminated`).
///
/// `context` and `cut` are related to error management:
/// - `cut` transforms an `Err::Error(e)` in `Err::Failure(e)`, signaling to
/// combinators like  `alt` that they should not try other parsers. We were in the
/// right branch (since we found the `"` character) but encountered an error when
/// parsing the string
/// - `context` lets you add a static string to provide more information in the
/// error chain (to indicate which parser had an error)
fn string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
    )(i)
}

fn parse_s_exp_like(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    let inner = parse_named_node;
    // finally, we wrap it in an s-expression
    let s = s_exp(inner);

    context(
        "s_exp_like",
        map(
            pair(delimited(multispace0, s, multispace0), opt(parse_capture)),
            |(p, c)| {
                if let Some(c) = c {
                    Pattern::Capture {
                        name: c.to_string(),
                        pat: Box::new(p),
                    }
                } else {
                    p
                }
            },
        ),
    )
    .parse(i)
}

fn parse_capture(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(
        preceded(multispace0, char('@')),
        recognize(take_while(|c: char| {
            c.is_alphanumeric() || c == '.' || c == '_'
        })),
        multispace0,
    )
    .parse(i)
}

fn parse_pattern(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    context(
        "pattern",
    preceded(
        multispace0,
        alt((parse_s_exp_like, parse_anonymous, parse_comment)),
        // alt((parse_s_exp_like,)),
    ))
    .parse(i)
}

fn parse_comment(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    context(
        "comment",
        delimited(
            delimited(multispace0, char(';'), multispace0),
            map(
                recognize(take_while(move |c: char| c != '\n')),
                ToString::to_string,
            )
            .map(Pattern::Comment),
            preceded(multispace0, opt(char('\n')).map(|_| ())),
        ),
    )
    .parse(i)
}

// /// Because `Expr::If` and `Expr::IfElse` are so similar (we easily could have
// /// defined `Expr::If` to have an `Option` for the else block), we parse both
// /// in a single function.
// ///
// /// In fact, we define our parser as if `Expr::If` was defined with an Option in it,
// /// we have the `opt` combinator which fits very nicely here.
// fn parse_if(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
//     let if_inner = context(
//         "if expression",
//         map(
//             preceded(
//                 // here to avoid ambiguity with other names starting with `if`, if we added
//                 // variables to our language, we say that if must be terminated by at least
//                 // one whitespace character
//                 terminated(tag("if"), multispace1),
//                 cut((parse_expr, parse_expr, opt(parse_expr))),
//             ),
//             |(pred, true_branch, maybe_false_branch)| {
//                 if let Some(false_branch) = maybe_false_branch {
//                     Expr::IfElse(
//                         Box::new(pred),
//                         Box::new(true_branch),
//                         Box::new(false_branch),
//                     )
//                 } else {
//                     Expr::If(Box::new(pred), Box::new(true_branch))
//                 }
//             },
//         ),
//     );
//     s_exp(if_inner).parse(i)
// }

// /// A quoted S-expression is list data structure.
// ///
// /// This example doesn't have the symbol atom, but by adding variables and changing
// /// the definition of quote to not always be around an S-expression, we'd get them
// /// naturally.
// fn parse_quote(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
//     // this should look very straight-forward after all we've done:
//     // we find the `'` (quote) character, use cut to say that we're unambiguously
//     // looking for an s-expression of 0 or more expressions, and then parse them
//     map(
//         context("quote", preceded(tag("'"), cut(s_exp(many0(parse_expr))))),
//         Pattern::Quote,
//     )
//     .parse(i)
// }

/// We tie them all together again, making a top-level expression parser!

fn parse_query(i: &str) -> IResult<&str, Pattern, VerboseError<&str>> {
    delimited(multispace0, parse_pattern,multispace0).parse(i)
}

fn parse_queries(i: &str) -> IResult<&str, Vec<Pattern>, VerboseError<&str>> {
    terminated(many0(preceded(multispace0, (parse_pattern))),multispace0).parse(i)
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::file;

    use super::*;

    #[test]
    fn simple_java() {
        let java_lang = tree_sitter_java::language();
        let expression_1 = "(_)";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(_ (_))";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(_ (_) (_))";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(class_declaration (_) (_))";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(class_declaration)";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(class_declaration name:(_) (_))";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(class_declaration !type_parameters)";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(class_declaration name:(identifier) !type_parameters)";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = "(class_declaration . name:(identifier) . (_) .)";
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = r#"(class_declaration "class" name:(identifier) (_))"#;
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = r#"(class_body "{")"#;
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 =
            r#"(class_declaration (modifiers "public") name:(identifier) (class_body "{") )"#;
        compare_to_original(java_lang, expression_1).assert();
        let expression_1 = r#"(class_declaration 
                (modifiers "public") name:(identifier)
                !superclass
                (class_body 
                    (method_declaration)
                    .
                    (method_declaration)
                )
            )"#;
    }

    #[test]
    fn simple_cpp() {
        let cpp_lang = tree_sitter_cpp::language();
        let expression_1 = r#"(identifier) @function"#;
        dbg!(compare_to_original(cpp_lang, expression_1).assert());
        let expression_1 = r#"(qualified_identifier
        name: (identifier) @function)"#;
        dbg!(compare_to_original(cpp_lang, expression_1).assert());
        let expression_1 = r#"(call_expression
    function: (qualified_identifier
        name: (identifier) @function))"#;
        dbg!(compare_to_original(cpp_lang, expression_1).assert());
        let expression_1 = r#"; Functions
"#;
        dbg!(compare_to_original(cpp_lang, expression_1).assert());
        let expression_1 = r#"; Functions
(_) @a"#;
        dbg!(compare_to_original(cpp_lang, expression_1).assert());
        let expression_1 = r#"; Functions
(call_expression
    function: (qualified_identifier
        name: (identifier) @function))"#;
        dbg!(compare_to_original(cpp_lang, expression_1).assert());
    }

    #[test]
    fn test_using_cpp_lang() {
        let lang = tree_sitter_cpp::language();
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../gen/tree-sitter/cpp/tree-sitter-cpp/queries/highlights.scm");
        dbg!(&d);
        // let f = File::open(d).unwrap();
        // dbg!(&f);
        let queries = std::fs::read_to_string(d).unwrap();

        let ts = tree_sitter::Query::new(lang, &queries);
        dbg!(ts);
        let ours = all_consuming(parse_queries)(&queries).finish();
        dbg!(ours);
    }

    enum Comparison<T, TE, O, OE> {
        AcceptTooLittle(OE),
        AcceptTooMuch(TE),
        BadTestCase(TE),
        Ok(T, O),
    }

    struct ComparisonWithInput<I, T, TE, O, OE> {
        input: I,
        result: Comparison<T, TE, O, OE>,
    }

    impl<I, T, TE, O, OE> ComparisonWithInput<I, T, TE, O, OE>
    where
        I: Display,
        T: Debug,
        TE: Debug,
        O: Debug,
        OE: Debug,
    {
        fn assert(self) -> (T, O) {
            match self.result {
                Comparison::AcceptTooLittle(ours) => {
                    panic!("we accept too little: \"{}\" => {:?}", self.input, ours)
                }
                Comparison::AcceptTooMuch(theirs) => {
                    panic!("we accept too much: \"{}\" => {:?}", self.input, theirs)
                }
                Comparison::BadTestCase(theirs) => {
                    panic!("bad test case: \"{}\" => {:?}", self.input, theirs)
                }
                Comparison::Ok(t, o) => (t, o),
            }
        }
    }

    fn compare_to_original(
        lang: tree_sitter::Language,
        query: &str,
    ) -> ComparisonWithInput<
        &str,
        tree_sitter::Query,
        tree_sitter::QueryError,
        Pattern,
        VerboseError<&str>,
    > {
        let ts = tree_sitter::Query::new(lang, query);
        let ours = parse_query(query).finish();
        ComparisonWithInput {
            input: query,
            result: match (ts, ours) {
                (Result::Err(ts), Result::Err(_)) => Comparison::BadTestCase(ts),
                (Result::Err(ts), Ok(_)) => Comparison::AcceptTooMuch(ts),
                (Ok(ts), Ok(ours)) => Comparison::Ok(ts, ours.1),
                (Ok(_), Result::Err(ours)) => Comparison::AcceptTooLittle(ours),
            },
        }
    }
}
