#![allow(unused)]
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::Deref,
};

use crate::store::nodes::{CompressedCompo, ErasedHolder};
use crate::tree_gen::metric_definition::{self, MetricAcc, MetricComputing, Subtree, Ty};
use num::ToPrimitive;
use rhai::*;

#[repr(transparent)]
struct DynMetric<S>(Dynamic, std::marker::PhantomData<S>);

impl<S> Clone for DynMetric<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

unsafe impl<S> Send for DynMetric<S> {}
unsafe impl<S> Sync for DynMetric<S> {}

struct DynMetricComputer<S> {
    name: ImmutableString,
    engine: Engine,
    init: AST,
    acc: AST,
    finish: AST,
    lossy: Option<AST>,
    _phantom: std::marker::PhantomData<S>,
}

struct AAA(STree);

unsafe impl Send for AAA {}
unsafe impl Sync for AAA {}

impl Clone for AAA {
    fn clone(&self) -> Self {
        todo!()
        // Self(self.0.clone())
    }
}
impl AAA {
    fn val(&mut self) -> u32 {
        todo!()
    }
}

#[derive(Debug)]
struct STree(Ty, Vec<Box<dyn Any>>);

impl STree {
    fn new(ty: Ty) -> Self {
        Self(ty, vec![])
    }
}
impl Subtree for STree {
    fn try_get<M: Clone + 'static>(&self) -> Option<M> {
        for x in &self.1 {
            let Some(m) = x.downcast_ref::<M>() else {
                continue;
            };
            return Some(m.clone());
        }
        None
    }
    fn ty(&self) -> Ty {
        self.0
    }
    fn push_metric<M: 'static>(&mut self, m: M) {
        self.1.push(Box::new(m));
    }
}

#[derive(Debug)]
struct DynMetricComputerCompileError(&'static str);

impl std::fmt::Display for DynMetricComputerCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl std::error::Error for DynMetricComputerCompileError {}

// T: 'static + Any + Send + Sync + Clone,
impl<S: Subtree + 'static> DynMetricComputer<S> {
    fn new(
        name: impl AsRef<str>,
        script: impl AsRef<str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let name = name.as_ref().into();
        let mut engine = Engine::new();
        engine.set_allow_shadowing(false);
        engine.register_fn("is_branch", |ty: Ty| matches!(ty, Ty::IfStatement));
        let ast = engine.compile(script)?;
        dbg!(&ast);
        let mut scope = Scope::new();
        let ast = engine.optimize_ast(&mut scope, ast, OptimizationLevel::Full);
        // if !ast.statements().is_empty() {
        //     return Err(DynMetricComputerCompileError("no non const statements in script").into());
        // }
        let Some(init) = ast.shared_lib().get_script_fn("init", 0) else {
            return Err(DynMetricComputerCompileError(
                "fn init() is missing or has a wrong signature".into(),
            )
            .into());
        };
        // TODO find accessed globals, specialize for a single type of node
        let Some(acc) = ast.shared_lib().get_script_fn("acc", 2) else {
            return Err(DynMetricComputerCompileError(
                "fn acc(a, child) is missing or has a wrong signature".into(),
            )
            .into());
        };
        if acc.params[0] != "a" {
            return Err(DynMetricComputerCompileError(
                "fn acc(a, child) is missing or has a wrong signature".into(),
            )
            .into());
        }
        if acc.params[1] != "child" {
            return Err(DynMetricComputerCompileError(
                "fn acc(a, child) is missing or has a wrong signature".into(),
            )
            .into());
        }
        // TODO find accessed fields on child
        let Some(finish) = ast.shared_lib().get_script_fn("finish", 1) else {
            return Err(DynMetricComputerCompileError(
                "fn finish(a) is missing or has a wrong signature".into(),
            )
            .into());
        };
        if finish.params[0] != "a" {
            return Err(DynMetricComputerCompileError(
                "fn finish(a) is missing or has a wrong signature".into(),
            )
            .into());
        }
        let lossy = if let Some(lossy) = ast.shared_lib().get_script_fn("lossy", 1) {
            if lossy.params[0] != "m" {
                return Err(DynMetricComputerCompileError(
                    "fn lossy(m) is missing or has a wrong signature".into(),
                )
                .into());
            }
            Some(AST::new(lossy.body.iter().cloned(), Module::new()))
        } else {
            None
        };

        let init = AST::new(init.body.iter().cloned(), Module::new());
        let acc = AST::new(acc.body.iter().cloned(), Module::new());
        {
            // let mut engine = Engine::new();
            // #[derive(Clone)]
            // struct Child {
            //     role: ImmutableString,
            // }
            // let child = Child {
            //     role: "then".into(),
            // };
            // impl Child {
            //     fn role(&mut self) -> ImmutableString {
            //         dbg!("child.role");
            //         self.role.clone()
            //     }
            // }
            // engine.register_fn("rol", |c: Child| {
            //     dbg!("rol(child)");
            //     c.role
            // });
            // engine.register_get("role", Child::role);
            // let mut scope = Scope::new();
            // scope.push_constant("child", child.clone());
            // let variables = vec![];
            // let locals = vec![];
            let mut child_props = vec![];

            acc.walk(&mut |x| {
                // dbg!(x);
                // TODO search for acc first and second param (also first of finish) and find `Expr(Dot{lhs: Var(param), rhs: Prop(prop)})`
                // and either error because not available or at eval time provide to script.
                match x.last() {
                    Some(ASTNode::Stmt(_)) => {}
                    Some(ASTNode::Expr(Expr::Variable(var, _, _))) => {
                        if var.1 == "child" {
                            dbg!(x.get(x.len() - 2));
                            match x.get(x.len() - 2) {
                                Some(ASTNode::Expr(Expr::Dot(bin, _, _))) => match &bin.rhs {
                                    Expr::Property(prop, _) => {
                                        child_props.push(prop.2.clone());
                                        dbg!(&prop.2);
                                    }
                                    e => todo!("{:?}", e),
                                },
                                e => todo!("{:?}", e),
                            }
                        } else {
                            // variables
                        }
                    }
                    Some(ASTNode::Expr(e)) => {
                        dbg!(e);
                    }
                    None => {}
                    _ => dbg!(),
                }
                true
            });
        }
        let finish = AST::new(finish.body.iter().cloned(), Module::new());
        Ok(Self {
            name,
            engine,
            init,
            acc,
            finish,
            lossy,
            _phantom: std::marker::PhantomData,
        })
    }

    fn new_closured(
        name: impl AsRef<str>,
        script: impl AsRef<str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // let name = name.as_ref().into();
        let mut engine = Engine::new();
        engine.set_allow_shadowing(false);
        engine.register_fn("is_branch", |ty: Ty| matches!(ty, Ty::IfStatement));
        let ast = engine.compile(script)?;
        let mut scope = Scope::new();
        dbg!(&ast);
        let ast = engine.optimize_ast(&mut scope, ast, OptimizationLevel::Full);

        dbg!(&ast);
        dbg!(ast.shared_lib());
        dbg!(ast.statements());
        // engine.run

        let mut map = StaticVec::<(Ident, Expr)>::new();
        let mut template = Map::new();
        for stmt in ast.statements() {
            let Stmt::Var(var, flags, _) = stmt else {
                continue;
            };
            if !flags.contains(ASTFlags::EXPORTED) {
                continue;
            }
            if var.0.name == "acc" {
                const PARA_COUNT: usize = 1;
                let mut params = vec![];
                match &var.1 {
                    Expr::Stmt(block) => {
                        let mut stmts = block.iter();
                        let share = stmts.next().unwrap();
                        dbg!(share);
                        match share {
                            Stmt::Share(scope) => {
                                for s in scope.iter() {
                                    dbg!(s.0.name.clone());
                                    params.push(s.0.name.clone());
                                }
                            }
                            _ => {}
                        }
                        let fct = stmts.next().unwrap();
                        dbg!(fct);
                        match fct {
                            Stmt::FnCall(fct, _) => {
                                assert_eq!(fct.name, "curry");
                                if let Some(Expr::DynamicConstant(fn_ptr, _)) = fct.args.first() {
                                    if let Some(fn_ptr) = fn_ptr.read_lock::<FnPtr>() {
                                        dbg!(fn_ptr.curry());
                                        dbg!(ast.shared_lib());
                                        dbg!(fn_ptr.fn_name());
                                        dbg!(params.len() - fn_ptr.curry().len());
                                        let num_params =
                                            params.len() + PARA_COUNT - fn_ptr.curry().len();
                                        let f = ast
                                            .shared_lib()
                                            .get_script_fn(fn_ptr.fn_name(), num_params)
                                            .unwrap();
                                        dbg!(&f.params);
                                        dbg!(&f.body);
                                        assert_eq!(&f.params[..num_params - PARA_COUNT], &*params);
                                    }
                                    dbg!(fn_ptr);
                                } else {
                                    unreachable!()
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    _ => {}
                }
            } else if var.0.name == "finish" {
                const PARA_COUNT: usize = 0;
                let mut params = vec![];
                match &var.1 {
                    Expr::Stmt(block) => {
                        let mut stmts = block.iter();
                        let share = stmts.next().unwrap();
                        dbg!(share);
                        match share {
                            Stmt::Share(scope) => {
                                for s in scope.iter() {
                                    dbg!(s.0.name.clone());
                                    params.push(s.0.name.clone());
                                }
                            }
                            _ => {}
                        }
                        let fct = stmts.next().unwrap();
                        dbg!(fct);
                        match fct {
                            Stmt::FnCall(fct, _) => {
                                assert_eq!(fct.name, "curry");
                                if let Some(Expr::DynamicConstant(fn_ptr, _)) = fct.args.first() {
                                    if let Some(fn_ptr) = fn_ptr.read_lock::<FnPtr>() {
                                        dbg!(fn_ptr.curry());
                                        let num_params =
                                            params.len() + PARA_COUNT - fn_ptr.curry().len();
                                        let f = ast
                                            .shared_lib()
                                            .get_script_fn(fn_ptr.fn_name(), num_params)
                                            .unwrap();
                                        dbg!(&f.params);
                                        dbg!(&f.body);
                                        // assert_eq!(fct.args);
                                        assert_eq!(&f.params[..num_params - PARA_COUNT], &*params);
                                    }
                                    dbg!(fn_ptr);
                                } else {
                                    unreachable!()
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                dbg!(&var.0.name, &var.1);
                // metric_names.push(stmt);
                let ident = var.0.clone();
                let expr = var.1.clone();
                let name = ident.name.clone();
                template.insert(name.clone().into(), Dynamic::UNIT);

                let name = engine.get_interned_string(name);
                map.push((ident, expr));
            }
        }
        let init = Expr::Map((map, template).into(), Position::NONE);
        dbg!(init);
        todo!()
    }
}

impl<S: Subtree + 'static> MetricComputing for DynMetricComputer<S> {
    type S = S;

    type Acc = DynMetric<S>;

    type M = DynMetric<S>;

    fn init(&self, ty: metric_definition::Ty, l: Option<&str>) -> Self::Acc {
        let l: Option<ImmutableString> = l.map(|s| s.into());
        let mut scope = Scope::new();
        let acc = self
            .engine
            .eval_ast_with_scope(&mut scope, &self.init)
            .unwrap();
        DynMetric(acc, PhantomData)
    }

    fn acc(&self, acc: Self::Acc, c: &Self::S) -> Self::Acc {
        let mut child = Map::new();
        child.insert(self.name.clone().into(), c.get::<Dynamic>().clone());
        let mut scope = Scope::new();
        scope.push("a", acc.0);
        scope.push("child", Dynamic::from(child));
        let acc = self
            .engine
            .eval_ast_with_scope(&mut scope, &self.acc)
            .unwrap();
        DynMetric(acc, PhantomData)
    }

    fn finish(&self, acc: Self::Acc, mut current: Self::S) -> Self::S {
        let mut scope = Scope::new();
        scope.push("TY", current.ty());
        scope.push("a", acc.0);
        dbg!(&self.finish);
        let m: Dynamic = self
            .engine
            .eval_ast_with_scope(&mut scope, &self.finish)
            .unwrap();
        current.push_metric(m);
        current
    }
}

#[test]
fn test_dyn_metric_definition_api() {
    let script = r#"
fn init() {
    0
}
fn acc(a, child) {
    a + child.mcc
}
fn finish(a) {
    if TY.is_branch() {
        a + 1
    } else {
        a
    }
}
fn lossy(m) {
    if TY.is_declaration() {
        m
    } else {
        ()
    }
}
"#;
    let comp = DynMetricComputer::<STree>::new("mcc", script).unwrap();
    let acc = comp.init(Ty::Method, None);
    let current = STree::new(Ty::Method);
    let child = {
        let acc = comp.init(Ty::IfStatement, None);
        let current = STree::new(Ty::IfStatement);
        comp.finish(acc, current)
    };
    let acc = comp.acc(acc, &child);
    let root = comp.finish(acc, current);
    dbg!(&root.get::<Dynamic>());
}

#[test]
fn test_dyn_metric_definition_api_closured() {
    let script = r#"
export let mcc = 0;
export let acc = |child| mcc += child.mcc;
export let finish = || if TY.is_branch() {
    mcc + 1
} else {
    mcc
};
fn lossy(m) {
    if TY.is_declaration() {
        m
    } else {
        ()
    }
}
"#;
    let comp = DynMetricComputer::<STree>::new_closured("mcc", script).unwrap();
    let acc = comp.init(Ty::Method, None);
    let current = STree::new(Ty::Method);
    let child = {
        let acc = comp.init(Ty::IfStatement, None);
        let current = STree::new(Ty::IfStatement);
        comp.finish(acc, current)
    };
    let acc = comp.acc(acc, &child);
    let root = comp.finish(acc, current);
    dbg!(&root.get::<Dynamic>());
}

#[test]
fn test_dyn_metric_definition_api_this_bind() {
    let script = r#"
fn init() {
    0
}
fn acc(a, child) {
    a + child.mcc
}
fn finish(a) {
    #{
        mcc: if TY.is_branch() {
            a + 1
        } else {
            a
        }
    }
}
fn contiguous(m) {
    if TY.is_declaration() {
        this.mcc = m;
    }
}
"#;
}
#[test]
fn test_dyn_metric_definition_api_on_phase() {
    let script = r#"
node.register_metric("mcc", 0,
    |child| this += chld.mcc,
    |a| if TY.is_branch() {
        a + 1
    } else {
        a
    },
    |m| if TY.is_declaration() {
        this.mcc = m;
    }
);
"#;
}
#[test]
fn test_private_fn_opt() {
    let script = r#"
let a = 0;
export let mcc;
export let acc = |child| a += child.mcc;
export let finish = || mcc = if TY.is_branch() {
    a + 1
} else {
    a
};
private fn zero() { 0 }
"#;
    let mut engine = Engine::new();
    engine.set_allow_shadowing(false);
    engine.register_fn("is_branch", |ty: Ty| matches!(ty, Ty::IfStatement));
    let ast = engine.compile(script).unwrap();
    let mut scope = Scope::new();
    dbg!(&ast);
    let ast = engine.optimize_ast(&mut scope, ast, OptimizationLevel::Full);
    assert!(
        ast.shared_lib().get_script_fn("zero", 0).is_none(),
        "should inline private zero()"
    );
}
#[test]
fn test_private_fn_opt2() {
    let script = r#"
let a = 0;
export let mcc;
export let acc = |child| a += child.mcc;
export let finish = || mcc = if is try_statement {
    a + 1
} else {
    a
};
"#;
    let mut engine = Engine::new();
    engine.set_allow_shadowing(false);
    engine
        .register_custom_syntax(["is", "$ident$"], false, |ctx, exprs| {
            let ty: Ty = ctx.scope().get_value("ty").ok_or("no `ty` of type `Ty`")?;
            // match ty {
            let rhs = exprs[0].get_string_value().unwrap();
            dbg!(rhs);
            // }
            Ok(true.into())
        })
        .unwrap();
    let ast = engine.compile(script).unwrap();
    let mut scope = Scope::new();
    dbg!(&ast);
    let ast = engine.optimize_ast(&mut scope, ast, OptimizationLevel::Full);
    dbg!(&ast);
}

#[test]
fn test_dyn_metric_definition_api_anonfct() {
    let script_fn = r#"
fn mcc() {
    let a = 0
    #{
        acc: |child| {
            a += child.mcc;
        },
        finish: || {
            if ty.is_branch() {
                a + 1
            } else {
                a
            }
        },
    }
}
fn lossy() {

}
"#;
    // rhai::Dynamic;
    // legion;
    // metric_definition;
    // std::any::TypeId;
    // engine
    //     .register_type_with_name::<Mean>("Mean")
    //     .register_fn("Mean", Mean::default)
    //     .register_fn("+=", |x: &mut Mean, y: Mean| {
    //         x.merge(&y);
    //     })
    //     .register_fn("+=", |m: &mut Mean, x: i64| m.add_i64(x));

    // // testing a single dot in a named node -> the query is ill-formed...
    // let mut parser = tree_sitter::Parser::new();
    // parser.set_language(&hyperast_gen_ts_java::language()).unwrap();
    // let text = "class A {}";
    // let ast = parser.parse(text, None).unwrap();
    // let query = tree_sitter::Query::new(&hyperast_gen_ts_java::language(), "(class_declaration body: (block .))").unwrap();
    // let mut cursor = tree_sitter::QueryCursor::new();
    // let mut matches = cursor.matches(&query, ast.root_node(), text.as_bytes());
    // dbg!(matches.next().unwrap());
}

#[test]
fn test_this_assign() {
    let mut engine = Engine::new();
    let ast = engine.compile("fn compute() {this = 0;}").unwrap();
    let mut scope = Scope::new();
    let options = CallFnOptions::new();
    let mut value = Dynamic::ZERO;
    let options = options.bind_this_ptr(&mut value);
    let a: () = engine
        .call_fn_with_options(options, &mut scope, &ast, "compute", ())
        .unwrap();
    dbg!(value);
    panic!();
}

#[test]
fn test_ast_opt_id() {
    let engine = Engine::new();
    let script = r#"
if ty == "try_statement" {
    if role == "then" {
        42
    } else {
        1
    }
} else if ty == "if_statement" {
    if role == "then" {
        42
    } else {
        1
    }
} else {
    0
}
"#;
    let ast_default = engine.compile(script).unwrap();
    let mut scope = Scope::new();
    scope.push_constant("ty", "try_statement");
    let ast_match = engine.compile_with_scope(&scope, script).unwrap();
    let mut scope = Scope::new();
    scope.push_constant("ty", "if_statement");
    let ast_match2 = engine.compile_with_scope(&scope, script).unwrap();
    scope.push_constant("ty", "call");
    let ast_nomatch = engine.compile_with_scope(&scope, script).unwrap();
    dbg!(&ast_default, &ast_match, &ast_match2, &ast_nomatch);

    dbg!(compare_asts(&ast_match, &ast_match2));
    dbg!(compare_asts(&ast_match, &ast_nomatch));
    dbg!(compare_asts(&ast_match2, &ast_nomatch));

    // for stmt in a2.iter_fn_def().flat_map(|f| f.body.iter()) {}
}

#[test]
fn test_ast_opt_id_fct() {
    let mut engine = Engine::new();
    engine.register_custom_operator("is", 110).unwrap();
    engine.register_fn("is", |ty: ImmutableString, s: ImmutableString| ty == s);
    engine.register_fn("is", |ty: ImmutableString, s: Array| {
        for x in &s {
            if let Some(s) = x.read_lock::<ImmutableString>().as_deref() {
                if ty == s {
                    return true;
                }
            } else if let Some(s) = x.read_lock::<String>().as_deref() {
                if ty == s {
                    return true;
                }
            }
        }
        false
    });
    engine.register_fn("is", |ty: ImmutableString, s: &[ImmutableString]| {
        s.contains(&ty)
    });
    engine.set_optimization_level(OptimizationLevel::Full);
    let script = r#"
fn compute(child) {
    if ty in ["try_statement", "if_statement"] {
        if child.role == "then" {
            42
        } else {
            1
        }
    } else if ty is "if_statement" {
        if child.role == "then" {
            42
        } else {
            1
        }
    } else {
        0
    }
}
"#;
    let ast_default = engine.compile(script).unwrap();
    let mut scope = Scope::new();
    scope.push_constant("ty", "try_statement");
    let ast_match = engine.compile_with_scope(&scope, script).unwrap();
    let mut scope = Scope::new();
    scope.push_constant("ty", "if_statement");
    let ast_match2 = engine.compile_with_scope(&scope, script).unwrap();
    scope.push_constant("ty", "call");
    let ast_nomatch = engine.compile_with_scope(&scope, script).unwrap();
    dbg!(&ast_default, &ast_match, &ast_match2, &ast_nomatch);

    dbg!(compare_asts(&ast_match, &ast_match2));
    dbg!(compare_asts(&ast_match, &ast_nomatch));
    dbg!(compare_asts(&ast_match2, &ast_nomatch));

    #[derive(Clone)]
    struct Child {
        role: ImmutableString,
    }
    let child = Child {
        role: "then".into(),
    };
    impl Child {
        fn role(&mut self) -> ImmutableString {
            dbg!("child.role");
            self.role.clone()
        }
    }
    engine.register_fn("rol", |c: Child| {
        dbg!("rol(child)");
        c.role
    });
    engine.register_get("role", Child::role);
    let mut scope = Scope::new();
    scope.push_constant("child", child.clone());

    ast_match.walk(&mut |x| {
        // dbg!(x);
        // TODO search for acc first and second param (also first of finish) and find `Expr(Dot{lhs: Var(param), rhs: Prop(prop)})`
        // and either error because not available or at eval time provide to script.
        match x.last() {
            Some(ASTNode::Stmt(x)) => {}
            Some(ASTNode::Expr(x)) => {
                dbg!(x);
            }
            None => {}
            _ => dbg!(),
        }
        true
    });

    // ast_match.clone_functions_only_filtered(|_,_,_,name,_| name == "compute");
    let body = &ast_match
        .shared_lib()
        .get_script_fn("compute", 1)
        .unwrap()
        .body;
    let body = AST::new(body.iter().cloned(), Module::new());
    dbg!(&body);
    engine.set_allow_shadowing(false);
    let opt_ast = engine.optimize_ast(&mut scope, body, OptimizationLevel::Full);
    dbg!(&opt_ast);
    dbg!();
    let r: i64 = engine
        .call_fn(&mut Scope::new(), &ast_match, "compute", (child,))
        .unwrap();
    dbg!(r);

    // for stmt in a2.iter_fn_def().flat_map(|f| f.body.iter()) {}
}

fn compare_asts(a1: &AST, a2: &AST) -> bool {
    if !eq_stmt_block(a1.statements(), a2.statements()) {
        return false;
    }
    let mut f1 = a1.iter_fn_def();
    let mut f2 = a2.iter_fn_def();
    loop {
        let (f1, f2) = match (f1.next(), f2.next()) {
            (Some(f1), Some(f2)) => (f1, f2),
            (None, None) => return true,
            _ => return false,
        };
        if f1.access != f2.access {
            return false;
        }
        if f1.name != f2.name {
            return false;
        }
        if f1.params != f2.params {
            return false;
        }
        if f1.this_type != f2.this_type {
            return false;
        }
        if !eq_stmt_block(f1.body.statements(), f2.body.statements()) {
            return false;
        }
    }
}

macro_rules! soft_todo {
    ($v:expr) => {{
        eprintln!("[{}:{}:{}] TODO", file!(), line!(), column!());
        $v
    }};
}

fn eq_stmt(s1: &Stmt, s2: &Stmt) -> bool {
    match (s1, s2) {
        (Stmt::Noop(_), Stmt::Noop(_)) => true,
        (Stmt::Expr(e1), Stmt::Expr(e2)) => eq_expr(e1.deref(), e2.deref()),
        (Stmt::If(f1, ..), Stmt::If(f2, ..)) => {
            eq_expr(&f1.expr, &f2.expr)
                && eq_stmt_block(f1.body.statements(), f2.body.statements())
                && eq_stmt_block(f1.branch.statements(), f2.branch.statements())
        }
        (Stmt::Switch(..), Stmt::Switch(..)) => soft_todo!(false),
        (Stmt::While(..), Stmt::While(..)) => soft_todo!(false),
        (Stmt::Do(..), Stmt::Do(..)) => soft_todo!(false),
        (Stmt::For(..), Stmt::For(..)) => soft_todo!(false),
        (Stmt::Var(..), Stmt::Var(..)) => soft_todo!(false),
        (Stmt::Assignment(..), Stmt::Assignment(..)) => soft_todo!(false),
        (Stmt::FnCall(..), Stmt::FnCall(..)) => soft_todo!(false),
        (Stmt::Block(..), Stmt::Block(..)) => soft_todo!(false),
        (Stmt::TryCatch(..), Stmt::TryCatch(..)) => soft_todo!(false),
        (Stmt::BreakLoop(..), Stmt::BreakLoop(..)) => soft_todo!(false),
        (Stmt::Return(..), Stmt::Return(..)) => soft_todo!(false),
        (Stmt::Import(..), Stmt::Import(..)) => soft_todo!(false),
        (Stmt::Export(..), Stmt::Export(..)) => soft_todo!(false),
        (Stmt::Share(..), Stmt::Share(..)) => soft_todo!(false),
        _ => false,
    }
}

fn eq_expr(e1: &Expr, e2: &Expr) -> bool {
    match (e1, e2) {
        (Expr::IntegerConstant(v1, ..), Expr::IntegerConstant(v2, ..)) => v1 == v2,
        (Expr::BoolConstant(v1, ..), Expr::BoolConstant(v2, ..)) => v1 == v2,
        (Expr::CharConstant(v1, ..), Expr::CharConstant(v2, ..)) => v1 == v2,
        (Expr::FloatConstant(v1, ..), Expr::FloatConstant(v2, ..)) => v1 == v2,
        (Expr::StringConstant(v1, ..), Expr::StringConstant(v2, ..)) => v1 == v2,
        (Expr::DynamicConstant(v1, ..), Expr::DynamicConstant(v2, ..)) => {
            eq_dynamic(v1.deref(), v2.deref())
        }
        (Expr::And(v1, ..), Expr::And(v2, ..)) => {
            eq_expr(&v1.lhs, &v2.lhs) && eq_expr(&v1.rhs, &v2.rhs)
            // TODO reversed when pure
        }
        (Expr::Or(v1, ..), Expr::Or(v2, ..)) => {
            eq_expr(&v1.lhs, &v2.lhs) && eq_expr(&v1.rhs, &v2.rhs)
            // TODO reversed when pure
        }
        (Expr::Coalesce(v1, ..), Expr::Coalesce(v2, ..)) => {
            eq_expr(&v1.lhs, &v2.lhs) && eq_expr(&v1.rhs, &v2.rhs)
        }
        (Expr::Dot(v1, f1, ..), Expr::Dot(v2, f2, ..)) => {
            eq_expr(&v1.lhs, &v2.lhs) && eq_expr(&v1.rhs, &v2.rhs) && f1.bits() == f2.bits()
        }
        (Expr::Custom(v1, ..), Expr::Custom(v2, ..)) => v1.tokens == v2.tokens,
        (Expr::Unit(..), Expr::Unit(..)) => true,
        (Expr::ThisPtr(..), Expr::ThisPtr(..)) => true,
        (Expr::Stmt(b1, ..), Expr::Stmt(b2, ..)) => eq_stmt_block(b1.statements(), b2.statements()),
        (Expr::InterpolatedString(v1, ..), Expr::InterpolatedString(v2, ..)) => soft_todo!(false),
        (Expr::Array(v1, ..), Expr::Array(v2, ..)) => soft_todo!(false),
        (Expr::Map(v1, ..), Expr::Map(v2, ..)) => soft_todo!(false),
        (Expr::Variable(v1, ..), Expr::Variable(v2, ..)) => {
            v1.0 == v2.0 && v1.1 == v2.1 && v1.2 == v2.2 && v1.3 == v2.3
        }
        (Expr::Property(v1, ..), Expr::Property(v2, ..)) => v1 == v2,
        (Expr::MethodCall(v1, ..), Expr::MethodCall(v2, ..)) => soft_todo!(false),
        (Expr::FnCall(v1, ..), Expr::FnCall(v2, ..)) => {
            v1.name == v1.name
                && v1.args.len() == v1.args.len()
                && v1.capture_parent_scope == v1.capture_parent_scope
                && v1.hashes == v1.hashes
                && v1.namespace == v1.namespace
                && v1.op_token == v1.op_token
                && v1
                    .args
                    .iter()
                    .zip(v2.args.iter())
                    .all(|(e1, e2)| eq_expr(e1, e2))
        }
        (Expr::Index(v1, f1, ..), Expr::Index(v2, f2, ..)) => {
            f1.bits() == f2.bits() && soft_todo!(false)
        }
        _ => false,
    }
}

fn eq_stmt_block(b1: &[Stmt], b2: &[Stmt]) -> bool {
    if b1.len() != b2.len() {
        return false;
    }
    for (s1, s2) in b1.into_iter().zip(b2.into_iter()) {
        if !eq_stmt(s1, s2) {
            return false;
        }
    }
    true
}

fn eq_dynamic(v1: &Dynamic, v2: &Dynamic) -> bool {
    todo!()
}
#[test]
fn test_split_dynamic_enum() {
    let v: Dynamic = Dynamic::from_int(42);
    fn push<T>(v: T) {}
    if v.tag() != 0 {
        push(v)
    } else if v.is_unit() {
        push(DynHolder(()))
    } else if v.is_int() {
        let v = v.as_int().unwrap();
        if v < 0 {
            let v = (-v) as u64;
            if v < u16::MAX as u64 {
                push(DynHolder(Neg(v as u16)))
            } else if v < u32::MAX as u64 {
                push(DynHolder(Neg(v as u32)))
            } else {
                push(DynHolder(Neg(v as u64)))
            }
        } else {
            let v = v as u64;
            if v < u16::MAX as u64 {
                push(DynHolder(v as u16))
            } else if v < u32::MAX as u64 {
                push(DynHolder(v as u32))
            } else {
                push(DynHolder(v as u64))
            }
        }
        // v.as_int();
    } else if v.is_float() {
        v.as_float();
        todo!()
        // } else if v.is_decimal() {
        //     v.as_decimal();
    } else if v.is_bool() {
        if v.as_bool().unwrap() {
            push(DynHolder(True))
        } else {
            push(DynHolder(Fals))
        }
    } else if v.is_char() {
        v.as_char();
        todo!()
    } else if v.is_string() {
        v.into_string();
        todo!()
    } else if v.is_array() {
        v.into_array();
        todo!()
    } else if v.is_blob() {
        v.into_blob();
        todo!()
    } else if v.is_map() {
        todo!("its a kind of js obect and needs to be traversed, cast to appropr btreemap");
        // v.map();
    } else if v.is_fnptr() {
        unimplemented!()
        // v.into_fnptr();
    } else if v.is_timestamp() {
        unimplemented!()
        // v.timestamp();
    } else {
        unreachable!("{}", v.type_name())
    };
}

struct DynHolder<T>(T);
impl<T: ToPrimitive> DynHolder<T> {
    fn to_dyn(&self) -> Dynamic {
        self.0.to_i64().unwrap().into()
    }
}
impl<T: ToPrimitive> DynHolder<Neg<T>> {
    fn to_dyn(&self) -> Dynamic {
        (-self.0.0.to_i64().unwrap()).into()
    }
}
struct Neg<T>(T);
struct True;
struct Fals;
struct D(Dynamic);

impl CompressedCompo for D {
    fn decomp(ptr: impl ErasedHolder, tid: TypeId) -> Self
    where
        Self: Sized,
    {
        let d = if let Some(d) = ptr.unerase_ref::<Dynamic>(tid) {
            d.clone()
        } else if let Some(x) = ptr.unerase_ref::<DynHolder<()>>(tid) {
            x.0.into()
        } else if let Some(x) = ptr.unerase_ref::<DynHolder<u16>>(tid) {
            x.to_dyn()
        } else if let Some(x) = ptr.unerase_ref::<DynHolder<u32>>(tid) {
            x.to_dyn()
        } else if let Some(x) = ptr.unerase_ref::<DynHolder<u64>>(tid) {
            x.to_dyn()
        } else if let Some(x) = ptr.unerase_ref::<DynHolder<Neg<u16>>>(tid) {
            x.to_dyn()
        } else if let Some(x) = ptr.unerase_ref::<DynHolder<Neg<u32>>>(tid) {
            x.to_dyn()
        } else if let Some(x) = ptr.unerase_ref::<DynHolder<Neg<u64>>>(tid) {
            x.to_dyn()
        } else if let Some(_) = ptr.unerase_ref::<DynHolder<True>>(tid) {
            true.into()
        } else if let Some(_) = ptr.unerase_ref::<DynHolder<Fals>>(tid) {
            false.into()
        } else {
            unreachable!()
        };
        D(d)
    }
}

#[test]
fn test_parfect_hash_precomp() {
    let script = "computestuff";
    let types = [1, 2, 3, 4, 5, 6];
    let labels = [Some(()), None];
    let roles = [Some(1), Some(2), Some(3), Some(4), None];
    // compile script for each combination
    let scripts = ["a", "b", "()"];
    let combs = [
        (1, Some(()), Some(3)),
        (2, Some(()), Some(3)),
        (3, Some(()), Some(3)),
    ];
    let mappings = [0, 0, 1];
    // should end up with a fct such that comb -> script
}

#[test]
fn test_hyperast_construction_interface_level_push() {
    // just push nodes in post order
    /// a code element in the HyperAST
    trait Entry {}
    /// the Entry Identifier
    struct Entity;
    // the HyperAST core trait
    trait HyperAST {
        type Id;
        fn get(&self) -> impl Entry;
    }
    /// metadata that can be persisted along each node.
    /// metadata are not identifiying
    trait MD {}
    /// The main construction trait
    trait Push: HyperAST {
        /// Insert a node (a subtree of code) in the HyperAST,
        /// returns Err(Id) if the node was already inside, using ty, label and cs to compare nodes
        /// Note: metadata are not identifying.
        /// Note: In debug mode, if a node is already present and md differs then panics.
        fn try_insert<'a>(
            &'a mut self,
            ty: impl Ty,
            label: &str,
            cs: &[Self::Id],
            md: impl MD,
        ) -> Result<Self::Id, Self::Id>;
    }
    /// Distinguishes case where inserting new node from trying something with an arleady present one
    enum Prepared<'hast, HAST: HyperAST + ?Sized> {
        Absent(Absent<'hast, HAST>),
        Present(Present<HAST::Id>),
    }
    /// In case the node is already present, the id is provided to do the rest of the contruction
    struct Present<Id>(Id);
    /// This is a new node,
    struct Absent<'hast, HAST: ?Sized> {
        hast: &'hast mut HAST,
    }
    struct AnyTy;
    trait Ty {}
    trait PushPrepared: Push {
        unsafe fn prepare_insert(
            &mut self,
            hash: u64,
            eq: impl Fn(&AnyTy, &str, &[Self::Id]) -> bool,
        ) -> Prepared<Self>;
        /// primary/prepare_insertion
        /// distinguishes identifying fields such that metadata only have to be computed if node is absent
        fn primaries<'a>(
            &'a mut self,
            ty: impl Ty,
            label: &str,
            cs: &[Self::Id],
        ) -> Prepared<'a, Self>;
    }
    impl<'hast, HAST: HyperAST> Absent<'hast, HAST> {
        /// metadatas/with_metadata
        fn secondaries(self, md: impl MD) -> HAST::Id {
            todo!()
        }
    }
}

#[test]
fn test_hyperast_construction_interface_level_zipper() {
    // traverse another tree and push the nodes to hyperast
    // just utils over the push level interface
    // usual process: pre-order() post-order()
    // our process: init() acc() finish()
    // where pre-order() { init() }
    // and post-order() { finish(); acc() }
}

fn test_hyperast_construction_interface_level_metric() {}
