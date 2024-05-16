use hyper_ast::store::{defaults::NodeIdentifier, SimpleStores};

fn f(query: &str, text: &[u8]) {
    let (matcher, stores, code) = f_aux(query, text);

    use crate::iter::IterAll as JavaIter;
    use crate::types::TStore;
    type It<'a, HAST> = JavaIter<'a, hyper_ast::position::StructuralPosition, HAST>;
    let matchs =
        matcher.apply_matcher::<SimpleStores<TStore>, It<_>, crate::types::TIdN<_>>(&stores, code);
    dbg!();
    for m in matchs {
        for c in &m.1 .0 {
            dbg!(&matcher.captures[c.id as usize]);
        }
        dbg!(m);
    }
}

fn f_aux<'store>(
    query: &str,
    text: &[u8],
) -> (
    hyper_ast_gen_ts_tsquery::search::PreparedMatcher<crate::types::Type>,
    SimpleStores<crate::types::TStore>,
    NodeIdentifier,
) {
    use crate::legion_with_refs;
    let matcher = hyper_ast_gen_ts_tsquery::prepare_matcher::<crate::types::Type>(query);

    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: crate::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = legion_with_refs::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    (matcher, stores, full_node.local.compressed_node)
}
fn g(query: &str, text: &[u8]) {
    let mut cursor = tree_sitter::QueryCursor::default();
    let (query, tree) = g_aux(query, text);
    dbg!(&tree);
    dbg!(tree.root_node().to_sexp());
    let matches = cursor.matches(&query, tree.root_node(), text);
    for m in matches {
        dbg!(&m);
        dbg!(m.pattern_index);
        for capt in m.captures {
            let index = capt.index;
            let name = query.capture_names()[index as usize];
            let i = query.capture_index_for_name(name).unwrap();
            let n = capt.node;
            let k = n.kind();
            let r = n.byte_range();
            dbg!(name);
            dbg!(k);
        }
    }
}

fn g_aux<'query, 'tree>(
    query: &'query str,
    text: &'tree [u8],
) -> (tree_sitter::Query, tree_sitter::Tree) {
    let language = tree_sitter_java::language();

    let query = tree_sitter::Query::new(&language, query).unwrap();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(text, None).unwrap();

    (query, tree)
}

const CODE: &str = r#"
package a.b;

public class AAA {}

"#;

const CODE1: &str = r#"
package a.b;

public class AAA {}

class BBB {}

"#;

const CODE2: &str = r#"
package a.b;

public class AAA {
  public AAA() {}
}

"#;

const CODE3: &str = r#"
package a.b;

public class AAA {
  int b = 0;
}

"#;

const CODES: &[&str] = &[CODE, CODE1, CODE2, CODE3];

const QUERIES: &[&str] = &[
    A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20,
    A21, A22, A23, A24, A25, A26, A27, A28, A29, A30, A31, A32, A33, A34, A35, A36, A37, A38, A39,
    A40, A41, A42, A43, A44, A45, A46, A47, A48, A49, A50, A51, A52, A53, A54, A55, A56, A57, A58,
    A59, A60, A61, A62, A63, A64, A65, A66, A67, A68, A69, A70, A71, A72, A73, A74, A75, A76, A77,
    A78, A79, A80, A81, A82, A83, A84, A85, A86, A87, A88, A89, A90, A91, A92, A93, A94, A95, A96,
    A97, A98, A99, A100, A101, A102, A103, A104, A105, A106, A107, A108, A109, A110, A111, A112,
    A113, A114, A115, A116, A117, A118, A119, A120, A121, A122, A123, A124, A125, A126, A127, A128,
    A129, A130, A131, A132, A133, A134, A135, A136, A137, A138, A139, A140, A141, A142, A143, A144,
    A145, A146, A147, A148, A149, A150, A151, A152, A153, A154,
];

#[test]
fn it_f() {
    for (i, text) in CODES.iter().enumerate() {
        for (j, query) in QUERIES.iter().enumerate() {
            dbg!(i, j);
            f(query, text.as_bytes());
        }
    }
}

#[test]
fn it_g() {
    for (i, text) in CODES.iter().enumerate() {
        for (j, query) in QUERIES.iter().enumerate() {
            dbg!(i, j);
            g(query, text.as_bytes());
        }
    }
}

#[test]
fn it() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let mut good = vec![];
    let mut bad = vec![];
    for (i, text) in CODES.iter().enumerate() {
        for (j, query) in QUERIES.iter().enumerate() {
            dbg!(i, j);
            let text = text.as_bytes();
            let mut cursor = tree_sitter::QueryCursor::default();
            let g_res = g_aux(query, text);
            let g_matches = { cursor.matches(&g_res.0, g_res.1.root_node(), text) };
            let f_res = f_aux(query, text);
            let f_matches = {
                type It<'a, HAST> =
                    crate::iter::IterAll<'a, hyper_ast::position::StructuralPosition, HAST>;
                f_res.0
                .apply_matcher::<SimpleStores<crate::types::TStore>, It<_>, crate::types::TIdN<_>>(
                    &f_res.1, f_res.2,
                )
            };
            let g_c = g_matches.into_iter().count();
            let f_c = f_matches.into_iter().count();
            if g_c != 0 || f_c != 0 {
                if g_c != f_c {
                    bad.push(((i, j), (g_c, f_c)));
                    dbg!(g_res.1.root_node().to_sexp());
                    dbg!(g_c, f_c);
                } else {
                    good.push(((i, j), g_c));
                }
            }
            // {
            //     for m in g_matches {
            //         dbg!(&m);
            //         dbg!(m.pattern_index);
            //         for capt in m.captures {
            //             let query = &g_res.0;
            //             let index = capt.index;
            //             let name = query.capture_names()[index as usize];
            //             let i = query.capture_index_for_name(name).unwrap();
            //             let n = capt.node;
            //             let k = n.kind();
            //             let r = n.byte_range();
            //             dbg!(name);
            //             dbg!(k);
            //         }
            //     }
            // };
            // {
            //     dbg!();
            //     for m in f_matchs {
            //         for c in &m.1 .0 {
            //             dbg!(&f_res.0.captures[c.id as usize]);
            //         }
            //         dbg!(m);
            //     }
            // };
        }
    }
    println!("good:");
    for good in &good {
        println!("{:?}", good);
    }
    println!("bads:");
    for bad in &bad {
        println!("{:?}", bad);
    }
    dbg!(bad.len()); // should be zero
    dbg!(good.len());
    dbg!(QUERIES.len()*CODES.len()-good.len()-bad.len()); // should reach 0 for matching coverage
}

const A0: &str = r#"(program)@prog @__tsg__full_match"#;

#[test]
fn f0() {
    let query = A0;
    let text = CODE.as_bytes();
    f(query, text);
}

#[test]
fn g0() {
    let query = A0;
    let text = CODE.as_bytes();
    g(query, text);
}

const A1: &str = r#"(program (_)@declaration)@prog @__tsg__full_match"#;
#[test]
fn f1() {
    let query = A1;
    let text = CODE.as_bytes();
    f(query, text);
    // TODO should match 2 times
    // Not sure how to handle that
    // Would be safer to add another code example
    // CODE1 matches 3 times with tsqueries.
}
const A2: &str = r#"[
  (module_declaration)
  (package_declaration)
  (import_declaration)
] @decl
@__tsg__full_match"#;
#[test]
fn f2() {
    let query = A2;
    let text = CODE.as_bytes();
    f(query, text);
}
const A3: &str = r#"(program
  (package_declaration
    (identifier)@pkg_name)? @package) @prog @__tsg__full_match"#;

#[test]
fn f3() {
    let query = A3;
    let text = CODE.as_bytes();
    f(query, text);
}

#[test]
fn g3() {
    let query = A3;
    let text = CODE.as_bytes();
    g(query, text);
}

const A4: &str = r#"(import_declaration (_) @ref) @import @__tsg__full_match"#;
const A5: &str = r#"(identifier) @name @__tsg__full_match"#;
const A6: &str =
    r#"(scoped_identifier scope: (_) @scope name: (_) @name) @scoped_name @__tsg__full_match"#;
const A7: &str = r#"(scoped_absolute_identifier scope: (_) @scope name: (_) @name) @scoped_name @__tsg__full_match"#;
#[test]
fn f7() {
    let query = A7;
    let text = CODE.as_bytes();
    f(query, text);
}
const A8: &str = r#"[
  (import_declaration                                             (identifier) @_scope @name)
  (import_declaration (scoped_absolute_identifier scope: (_) @_scope name: (identifier) @name))
] @import @__tsg__full_match"#;
const A9: &str = r#"(class_declaration
  name: (identifier) @name
  body: (class_body) @class_body) @class @__tsg__full_match"#;
const A10: &str = r#"(class_declaration
  superclass: (superclass
    (_) @superclass_name)
    body: (class_body) @class_body) @class @__tsg__full_match"#;
const A11: &str = r#"(class_declaration (type_parameters)) @class @__tsg__full_match"#;
const A12: &str =
    r#"(class_declaration (type_parameters (type_parameter) @param)) @class @__tsg__full_match"#;
const A13: &str = r#"(type_parameter (type_identifier) @name) @this @__tsg__full_match"#;
const A14: &str = r#"(spread_parameter) @spread_param @__tsg__full_match"#;
const A15: &str = r#"(class_declaration interfaces: (super_interfaces (type_list (_) @type))) @this @__tsg__full_match"#;
const A16: &str = r#"(class_body) @class_body @__tsg__full_match"#;
const A17: &str = r#"(class_body (_)@declaration)@class_body @__tsg__full_match"#;
const A18: &str = r#"(class_body (block) @block) @__tsg__full_match"#;
const A19: &str = r#"[
  (class_declaration)
  (enum_declaration)
  (field_declaration)
  (interface_declaration)
  (method_declaration)
  (constructor_declaration)
  (annotation_type_declaration)
  (constant_declaration)
  (record_declaration)
] @decl
@__tsg__full_match"#;
const A20: &str = r#"(annotation_type_declaration
  name: (identifier) @name) @annotation @__tsg__full_match"#;
const A21: &str =
    r#"(constructor_declaration body: (constructor_body) @body) @this @__tsg__full_match"#;
const A22: &str = r#"(constructor_body) @this @__tsg__full_match"#;
const A23: &str = r#"(constructor_body . (_) @first) @this @__tsg__full_match"#;
const A24: &str = r#"(constructor_body (_) @a . (_) @b) @__tsg__full_match"#;
const A25: &str = r#"(explicit_constructor_invocation) @this @__tsg__full_match"#;
const A26: &str =
    r#"(explicit_constructor_invocation constructor: (_) @constructor) @this @__tsg__full_match"#;
const A27: &str =
    r#"(explicit_constructor_invocation object: (_) @object) @this @__tsg__full_match"#;
const A28: &str = r#"(explicit_constructor_invocation arguments: (argument_list (_) @arg)) @this @__tsg__full_match"#;
const A29: &str = r#"(enum_declaration name: (_) @name) @this @__tsg__full_match"#;
const A30: &str =
    r#"(enum_declaration (enum_body (enum_constant name: (_) @name))) @this @__tsg__full_match"#;
const A31: &str = r#"(field_declaration
  type: (_) @type
  declarator: (variable_declarator
    name: (_) @name
  )
) @field_decl
@__tsg__full_match"#;
const A32: &str = r#"(modifiers) @this @__tsg__full_match"#;
// const A33: &str = r#"(modifiers (_) @annotation) @this @__tsg__full_match"#;
const A33: &str = r#"(modifiers (annotation) @annotation) @this @__tsg__full_match"#;
const A34: &str = r#"(marker_annotation name: (_) @name) @this @__tsg__full_match"#;
const A35: &str = r#"(annotation name: (_) @name) @this @__tsg__full_match"#;
const A36: &str = r#"(modifiers (annotation arguments: (annotation_argument_list (_) @value))) @this @__tsg__full_match"#;
const A37: &str = r#"(element_value_array_initializer) @this @__tsg__full_match"#;
const A38: &str = r#"(element_value_pair value: (_) @value) @this @__tsg__full_match"#;
const A39: &str = r#"(field_declaration (modifiers) @modifiers) @decl @__tsg__full_match"#;
#[test]
fn f39() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A39;
    let text = CODE3.as_bytes();
    f(query, text);
}
const A40: &str =
    r#"(interface_declaration name: (_) @name body: (_) @body) @this @__tsg__full_match"#;
const A41: &str = r#"(interface_declaration (extends_interfaces (type_list (_) @type))) @this @__tsg__full_match"#;
const A42: &str = r#"(interface_declaration
  type_parameters: (type_parameters
    (type_parameter
      (type_identifier) @type_identifier))
  body: (_) @body) @_this @__tsg__full_match"#;
const A43: &str = r#"(interface_body) @this @__tsg__full_match"#;
const A44: &str = r#"(interface_body (_) @child) @this @__tsg__full_match"#;
const A45: &str = r#"(method_declaration
  (modifiers "static"?@is_static)?
  type: (_) @type
  name: (identifier) @name
  body: (block) @_block) @method
@__tsg__full_match"#;
const A46: &str = r#"(method_declaration (formal_parameters (_) @param)) @method
@__tsg__full_match"#;
const A47: &str = r#"(formal_parameter type: (_) @type (_) @name) @param
@__tsg__full_match"#;
const A48: &str = r#"(formal_parameter (modifiers) @modifiers) @this @__tsg__full_match"#;
const A49: &str = r#"(method_declaration
  (modifiers) @modifiers) @this @__tsg__full_match"#;
const A50: &str = r#"(method_declaration
  body: (_) @stmt) @method
@__tsg__full_match"#;
const A51: &str =
    r#"(record_declaration name: (_) @name body: (_) @body) @this @__tsg__full_match"#;
const A52: &str =
    r#"(record_declaration parameters: (formal_parameters (_) @param)) @this @__tsg__full_match"#;
const A53: &str = r#"[
  (assert_statement)
  (block)
  (break_statement)
  (continue_statement)
  (declaration)
  (do_statement)
  (expression_statement)
  (enhanced_for_statement)
  (for_statement)
  (if_statement)
  (labeled_statement)
  (local_variable_declaration)
  (return_statement)
  (switch_expression)
  (synchronized_statement)
  (throw_statement)
  (try_statement)
  (try_with_resources_statement)
  (while_statement)
  (yield_statement)
] @stmt
@__tsg__full_match"#;
#[test]
fn f53() {
    let query = A53;
    let text = CODE.as_bytes();
    f(query, text);
    // TODO missing matches using supertypes
    // NOTE adding (class_declaration) and (package_declaration) to the list makes it match the nodes like tsqueries
}
#[test]
fn f53_declaration() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = "(declaration)";
    let text = CODE.as_bytes();
    f(query, text);
    // TODO missing matches using supertypes
    // NOTE adding (class_declaration) and (package_declaration) to the list makes it match the nodes like tsqueries
}
#[test]
fn g53() {
    let query = A53;
    let text = CODE.as_bytes();
    g(query, text);
}
const A54: &str = r#"(assert_statement) @stmt @__tsg__full_match"#;
const A55: &str = r#"(assert_statement (expression) @expr) @stmt @__tsg__full_match"#;
const A56: &str = r#"(block
  (_) @left
  .
  (_) @right
)
@__tsg__full_match"#;
const A57: &str = r#"(block
  .
  (_) @first) @block @__tsg__full_match"#;
const A58: &str = r#"(block
  (_) @last
  . ) @block @__tsg__full_match"#;
const A59: &str = r#"(break_statement (identifier) @_name) @this @__tsg__full_match"#;
const A60: &str = r#"(break_statement (identifier) @name) @stmt @__tsg__full_match"#;
const A61: &str = r#"(continue_statement) @this @__tsg__full_match"#;
const A62: &str = r#"(continue_statement (identifier) @name) @this @__tsg__full_match"#;
const A63: &str = r#"(declaration) @_decl @__tsg__full_match"#;
const A64: &str = r#"(do_statement body: (_) @body condition: (_) @cond) @stmt @__tsg__full_match"#;
const A65: &str = r#"(expression_statement (_) @expr) @expr_stmt
@__tsg__full_match"#;
const A66: &str = r#"(enhanced_for_statement type: (_) @type (_) @name value: (_) @value body: (_) @body) @stmt @__tsg__full_match"#;
const A67: &str = r#"(for_statement) @this @__tsg__full_match"#;
const A68: &str =
    r#"(for_statement !init !condition !update body: (_) @body) @this @__tsg__full_match"#;
const A69: &str = r#"(for_statement init: (expression) @init condition: (_) @condition update: (_) @update body: (_) @body) @stmt @__tsg__full_match"#;
const A70: &str = r#"(for_statement init: (local_variable_declaration) @init condition: (_) @condition update: (_) @update body: (_) @body) @stmt @__tsg__full_match"#;
const A71: &str = r#"(if_statement condition: (_) @condition consequence: (_) @consequence) @stmt @__tsg__full_match"#;
const A72: &str = r#"(if_statement alternative: (_) @alternative) @stmt @__tsg__full_match"#;
const A73: &str =
    r#"(labeled_statement (identifier) @name (statement) @child) @stmt @__tsg__full_match"#;
const A74: &str = r#"(local_variable_declaration
  type: (_) @type
  declarator: (variable_declarator) @var_decl
) @_local_var
@__tsg__full_match"#;
const A75: &str = r#"(local_variable_declaration
  type: (_) @type) @local_var @__tsg__full_match"#;
const A76: &str = r#"(variable_declarator value: (_) @value) @this @__tsg__full_match"#;
const A77: &str = r#"(local_variable_declaration
  declarator: (_) @last
  . ) @local_var @__tsg__full_match"#;
const A78: &str = r#"(local_variable_declaration
  type: (_)
  .
  declarator: (_) @first) @local_var @__tsg__full_match"#;
const A79: &str = r#"(local_variable_declaration
  declarator: (_) @left
  .
  declarator: (_) @right
  ) @_local_var @__tsg__full_match"#;
const A80: &str = r#"(variable_declarator
  name: (_) @name) @var_decl @__tsg__full_match"#;
const A81: &str = r#"(return_statement (_) @expr) @stmt
@__tsg__full_match"#;
const A82: &str =
    r#"(switch_expression condition: (_) @condition body: (_) @body) @stmt @__tsg__full_match"#;
const A83: &str = r#"(method_declaration
 parameters:
 (formal_parameters
  (formal_parameter
   type: (generic_type
     (type_arguments
      (type_identifier) @type))))
 body:
 (block
  (switch_expression
   condition: (_)
   body: (switch_block
     (switch_block_statement_group
      (switch_label
       (identifier))
      @label))
  )) @stmt) @__tsg__full_match"#;
const A84: &str = r#"(switch_block) @this @__tsg__full_match"#;
const A85: &str = r#"(switch_block (switch_block_statement_group (switch_label) @label)) @this @__tsg__full_match"#;
const A86: &str = r#"(switch_block (switch_block_statement_group (switch_label)+ . (statement) @first)) @this @__tsg__full_match"#;
const A87: &str = r#"(switch_block (switch_block_statement_group (switch_label)+ (statement) @a . (statement) @b)) @_this @__tsg__full_match"#;
const A88: &str =
    r#"(switch_block (switch_rule (switch_label) @label (_) @body)) @this @__tsg__full_match"#;
const A89: &str = r#"(switch_label) @label @__tsg__full_match"#;
const A90: &str = r#"(switch_label (expression) @expr) @label @__tsg__full_match"#;
const A91: &str = r#"(synchronized_statement (_) @expr body: (_) @body) @stmt @__tsg__full_match"#;
const A92: &str = r#"(try_statement body: (_) @body) @stmt @__tsg__full_match"#;
const A93: &str = r#"(try_statement (catch_clause (catch_formal_parameter (catch_type) @type (_) @name) body: (_) @body)) @stmt @__tsg__full_match"#;
const A94: &str = r#"(catch_type) @catch_type @__tsg__full_match"#;
const A95: &str = r#"(catch_type (_) @type) @catch_type @__tsg__full_match"#;
const A96: &str = r#"(try_statement (finally_clause (_) @finally)) @stmt @__tsg__full_match"#;
const A97: &str = r#"(try_with_resources_statement) @stmt @__tsg__full_match"#;
const A98: &str = r#"(try_with_resources_statement resources: (resource_specification . (resource) @first)) @stmt @__tsg__full_match"#;
const A99: &str = r#"(try_with_resources_statement resources: (resource_specification (resource) @a . (resource) @b)) @_stmt @__tsg__full_match"#;
const A100: &str = r#"(try_with_resources_statement resources: (resource_specification (resource) @last .) body: (_) @body) @_stmt @__tsg__full_match"#;
const A101: &str = r#"(resource) @this @__tsg__full_match"#;
const A102: &str =
    r#"(resource type: (_) @type (_) @name value: (_) @value) @this @__tsg__full_match"#;
const A103: &str = r#"(resource . (identifier) @name .) @this @__tsg__full_match"#;
const A104: &str = r#"(resource (field_access) @field_access) @this @__tsg__full_match"#;
const A105: &str = r#"(try_with_resources_statement (catch_clause (catch_formal_parameter (catch_type) @type (_) @name) body: (_) @body)) @stmt @__tsg__full_match"#;
const A106: &str =
    r#"(try_with_resources_statement (finally_clause (_) @finally)) @stmt @__tsg__full_match"#;
const A107: &str =
    r#"(while_statement condition: (_) @condition body: (_) @body) @stmt @__tsg__full_match"#;
const A108: &str = r#"(yield_statement (_) @expr) @stmt @__tsg__full_match"#;
const A109: &str =
    r#"(array_access (primary_expression) @array (expression) @index) @this @__tsg__full_match"#;
const A110: &str = r#"(array_creation_expression type: (_) @type) @this @__tsg__full_match"#;
const A111: &str =
    r#"(array_creation_expression (dimensions_expr (_) @expr)) @this @__tsg__full_match"#;
const A112: &str = r#"(array_initializer (_) @expr) @this @__tsg__full_match"#;
const A113: &str = r#"(class_literal (_) @type) @this @__tsg__full_match"#;
const A114: &str = r#"(primary_expression/identifier) @name
@__tsg__full_match"#;
const A115: &str = r#"(field_access
  object: (_) @object
  field: (identifier) @name) @field_access @__tsg__full_match"#;
const A116: &str = r#"(method_invocation) @method_invocation
@__tsg__full_match"#;
const A117: &str = r#"(method_invocation arguments: (argument_list (expression) @expr)) @method_invocation @__tsg__full_match"#;
const A118: &str = r#"(method_invocation
  !object
  name: (identifier) @method_name) @method_invocation @__tsg__full_match"#;
const A119: &str = r#"(method_reference . (_) @lhs) @this @__tsg__full_match"#;
const A120: &str = r#"(method_reference . (_) @lhs (identifier) @name) @this @__tsg__full_match"#;
const A121: &str =
    r#"(method_reference . (identifier) @lhs (identifier) @_name) @this @__tsg__full_match"#;
const A122: &str = r#"(parenthesized_expression (_) @child) @expr @__tsg__full_match"#;
const A123: &str = r#"[
  (array_initializer)
  (assignment_expression)
  (binary_expression)
  (instanceof_expression)
  (lambda_expression)
  (ternary_expression)
  (update_expression)
  (decimal_integer_literal)
  (hex_integer_literal)
  (octal_integer_literal)
  (binary_integer_literal)
  (decimal_floating_point_literal)
  (hex_floating_point_literal)
  (true)
  (false)
  (character_literal)
  (string_literal)
  (null_literal)
  (class_literal)
  (this)
  ; (identifier)
  (parenthesized_expression)
  (object_creation_expression)
  (field_access)
  (array_access)
  (method_invocation)
  (method_reference)
  (array_creation_expression)
  (unary_expression)
  (cast_expression)
  (switch_expression)
  (super)
] @expr
@__tsg__full_match"#;
const A124: &str = r#"(assignment_expression left: (identifier) @name) @this @__tsg__full_match"#;
const A125: &str =
    r#"(assignment_expression left: (field_access) @access) @this @__tsg__full_match"#;
const A126: &str = r#"(assignment_expression right: (_) @right) @this @__tsg__full_match"#;
const A127: &str = r#"(binary_expression left: (_) @lhs right: (_) @rhs) @this @__tsg__full_match"#;
const A128: &str = r#"(cast_expression type: (_) @type) @this @__tsg__full_match"#;
const A129: &str = r#"(cast_expression value: (_) @expr) @this @__tsg__full_match"#;
const A130: &str =
    r#"(instanceof_expression left: (_) @expr right: (_) @type) @this @__tsg__full_match"#;
const A131: &str = r#"(lambda_expression) @this @__tsg__full_match"#;
const A132: &str = r#"(lambda_expression body: (expression) @body) @this @__tsg__full_match"#;
const A133: &str = r#"(lambda_expression body: (block) @body) @this @__tsg__full_match"#;
const A134: &str = r#"(lambda_expression parameters: (_) @param) @this @__tsg__full_match"#;
const A135: &str = r#"[
  (super)
  (this)
] @expr
@__tsg__full_match"#;
const A136: &str = r#"(method_invocation
  object: (_) @object
  name: (identifier) @method_name) @method_invocation @__tsg__full_match"#;
const A137: &str =
    r#"(object_creation_expression (primary_expression) @child) @this @__tsg__full_match"#;
const A138: &str = r#"(object_creation_expression type_arguments: (type_arguments (_) @type)) @this @__tsg__full_match"#;
const A139: &str = r#"(object_creation_expression type: (_) @type) @this @__tsg__full_match"#;
const A140: &str = r#"(object_creation_expression arguments: (argument_list (expression) @expr)) @this @__tsg__full_match"#;
const A141: &str = r#"(ternary_expression (expression) @expr) @this @__tsg__full_match"#;
const A142: &str = r#"(unary_expression (expression) @expr) @this @__tsg__full_match"#;
const A143: &str = r#"(update_expression (expression) @expr) @this @__tsg__full_match"#;
const A144: &str = r#"[
  (annotated_type)
  (array_type)
  (boolean_type)
  (floating_point_type)
  (generic_type)
  (integral_type)
  (scoped_type_identifier)
  (type_identifier)
  (void_type)
] @type
@__tsg__full_match"#;
const A145: &str = r#"[
  (boolean_type)
  (floating_point_type)
  (integral_type)
  (type_identifier)
  (void_type)
] @type
@__tsg__full_match"#;
const A146: &str = r#"(array_type element: (_) @child) @this @__tsg__full_match"#;
const A147: &str = r#"(generic_type . (_) @name) @this @__tsg__full_match"#;
const A148: &str = r#"(generic_type (type_arguments (_) @type)) @this @__tsg__full_match"#;
const A149: &str = r#"(scoped_type_identifier . (_) @name) @this @__tsg__full_match"#;
const A150: &str = r#"(wildcard) @this
@__tsg__full_match"#;
const A151: &str = r#"(type_identifier) @this
@__tsg__full_match"#;
const A152: &str = r#"(scoped_type_identifier
  (type_identifier) @imported_class_name (type_identifier) @method_name) @__tsg__full_match"#;
const A153: &str = r#"(line_comment)@line_comment @__tsg__full_match"#;
const A154: &str = r#"(block_comment)@block_comment @__tsg__full_match"#;
