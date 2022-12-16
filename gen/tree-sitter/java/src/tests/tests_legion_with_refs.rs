use core::fmt;
use std::{
    io::{stdout, Write},
};

use hyper_ast::{
    store::{TypeStore, SimpleStores, labels::LabelStore, nodes::DefaultNodeStore as NodeStore}, 
    tree_gen::ZippedTreeGen,
    position::{ExploreStructuralPositions, StructuralPositionStore, StructuralPosition, Scout, TreePath}, 
    types::WithChildren, utils::memusage_linux, nodes::RefContainer, filter::BloomResult};
use pretty_assertions::assert_eq;

use crate::{
    legion_with_refs::{
         print_tree_syntax, serialize, JavaTreeGen,
    }, impact::element::{RefsEnum, LabelPtr, IdentifierFormat},
};

fn run(text: &[u8]) {
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    
    let tree = match JavaTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!();
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
    );
    println!();
    stdout().flush().unwrap();

    let mut out = IoOut { stream: stdout() };
    serialize(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
}
#[test]
fn test_cases() {
    let cases = [
        CASE_1, CASE_1_1, CASE_1_2, CASE_1_3, CASE_1_4, CASE_1_5, CASE_1_5, CASE_1_6, CASE_1_7, CASE_1_8, CASE_1_9, CASE_1_10,
        CASE_2, CASE_3, CASE_4, CASE_5, CASE_6, CASE_7, CASE_8, CASE_8_1, CASE_9, 
        CASE_10, CASE_11, CASE_11_BIS, CASE_12, CASE_13, CASE_14, 
        CASE_15, CASE_15_1, CASE_15_2, CASE_16, CASE_17, CASE_18, CASE_19,
        CASE_20, CASE_21, CASE_22, CASE_23, CASE_24,
        CASE_25, CASE_26, CASE_27, CASE_28, CASE_29, 
        CASE_30, CASE_31, CASE_32, CASE_33, A
    ];
    for case in cases {
        run(case.as_bytes())
    }
}

#[test]
fn test_equals() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).is_test(true).init();
    let text = CASE_33.as_bytes();
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let tree = match JavaTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!();
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
    );
    println!();
    stdout().flush().unwrap();

    let mut out = IoOut { stream: stdout() };
    serialize(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
    println!();

    {
        let stores = java_tree_gen.stores;
        // playing with refs
        let a = &full_node.local.compressed_node;
        let Some(mut ana) = full_node.local.ana else {
            panic!("None");
        };
        println!("refs:",);
        ana.print_refs(&stores.label_store);

        let b = stores.node_store.resolve(*a);
        use hyper_ast::types::LabelStore;
        macro_rules! scoped_ref {
            ( $o:expr, $i:expr ) => {{
                let o = $o;
                let i = $i;
                let f = IdentifierFormat::from(i);
                let i = stores.label_store.get_or_insert(i);
                let i = LabelPtr::new(i, f);
                ana.solver.intern_ref(RefsEnum::ScopedIdentifier(o, i))
            }};
        }
        let root = ana.solver.intern(RefsEnum::Root);
        let i = scoped_ref!(root, "B");
        let d = ana.solver.nodes.with(i);
        let c = b.check(d);
        match c {
            BloomResult::MaybeContain => println!("Maybe contains B"),
            BloomResult::DoNotContain => println!("Do not contains B"),
        }
    }
//     use hyper_ast::position::extract_position;
//     let mut position = extract_position(&java_tree_gen.stores, d_it.parents(), d_it.offsets());
//     position.set_len(b.get_bytes_len() as usize);
//     println!("position: {:?}", position);
}

#[test]
fn test_special() {

    // let mut parser: Parser, old_tree: Option<&Tree>
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };

    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };

    let text = {
        let source_code1 = "package p.y;
public class A {
    class A0 {
        int a = 0xffff;
    }
    class B { int a = 0xfffa;
              int b = 0xffff;

              void test() {
                a;
              }
    } class C { int a = 0xffff;
           int b = 0xfffa;

        void test() {
            a.b;
            a.f();
        } void test2() {
            b;
        }
    }
    class D { 
        int a = 0xffff;
        int b = 0xffff;

     void test() {
         a;
     } void test2() {
         b;
     }
 }
    }";
        // let source_code1 = "class A {
        //     class A0 {
        //         int a = 0xffff;
        // }
        //     }
        // ";
        source_code1.as_bytes()
    };
    let tree = match JavaTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{}", tree.root_node().to_sexp());

    let _full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    let text = {
        let source_code1 = "class A {
    class A0 {
        int a = 0xffff;
    }
    class B { int a = 0xfffa;
              int b = 0xfffa;
              
              Object test0() {
                    get();
                    aaa.getM();
                    aaa.getM(AAA::ff);
                    return factory.getModel().getAllModules().stream()
                    .map(module -> getPackageFromModule(qualifiedName, module))
                    .filter(Objects::nonNull)
                    .findFirst()
                    .orElse(null);
              }
              void test() {
                while (Arrays.asList(SmPLLexer.TokenType.MetavarIdentifier, SmPLLexer.TokenType.WhenMatches).contains(tokens.get(pos).getType())) {
                    switch (tokens.get(pos).getType()) {
                        case MetavarIdentifier:
                            if (genericMetavarTypes.contains(metavarType)) {
                                output.append(metavarType).append(\"(\").append(tokens.get(pos).getText().strip()).append(\");\\n\");
                            } else {
                                output.append(metavarType).append(\" \").append(tokens.get(pos).getText().strip()).append(\";\\n\");
                            }
                            break;
    
                        case WhenMatches:
                            output.append(\"constraint(\\\"regex-match\\\", \" + tokens.get(pos + 1).getText() + \");\\n\");
                            ++pos;
                            break;
    
                        default:
                            throw new IllegalStateException(\"impossible\");
                    }
    
                    ++pos;
                }
              }
    } class C { int a = 0xffff;
           int b = 0xffff;

        void test() {
            a;
        } void test2() {
            b;
        }
    }
    class E { 
        int a = 0xffff;
        int b = 0xffff;

     void test() {
         a;
     } void test2() {
         a;
     }
     public void testBasicSingle() {

        // contract: NaiveExceptionControlFlowStrategy should result in every statement parented by a try
        //           block having a path to the corresponding catch block.

        CtMethod<?> method = Launcher.parseClass(\"class A {\\n\" +
                                                 \"  void m() {\\n\" +
                                                 \"    try {\\n\" +
                                                 \"      a();\\n\" +
                                                 \"      b();\\n\" +
                                                 \"      c();\\n\" +
                                                 \"    } catch (Exception e) {\\n\" +
                                                 \"      bang();\\n\" +
                                                 \"    }\\n\" +
                                                 \"    x();\\n\" +
                                                 \"  }\\n\" +
                                                 \"}\\n\").getMethods().iterator().next();

        ControlFlowBuilder builder = new ControlFlowBuilder();
        builder.setExceptionControlFlowStrategy(new NaiveExceptionControlFlowStrategy());
        builder.build(method);
        ControlFlowGraph cfg = builder.getResult();
     }
 }
    }";
        // let source_code1 = "class A {
        //     class A0 {
        //         int a = 0xffff;
        // }
        //     }
        // ";
        source_code1.as_bytes()
    };
    let tree = match JavaTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!("debug full node: {:?}", &full_node);
    // let mut out = String::new();
    let mut out = IoOut { stream: stdout() };
    serialize(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
    println!();
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
    );
    println!();
    stdout().flush().unwrap();

    let mut out = BuffOut {
        buff: "".to_owned(),
    };
    serialize(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
    assert_eq!(std::str::from_utf8(text).unwrap(), out.buff);

    println!("{:?}", java_tree_gen.stores().node_store);
    println!("{}", java_tree_gen.stores.label_store);

    let mu = memusage_linux();
    drop(java_tree_gen);
    let mu = mu - memusage_linux();
    println!("mu {}", mu);
}

struct IoOut<W: std::io::Write> {
    stream: W,
}

impl<W: std::io::Write> std::fmt::Write for IoOut<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.stream
            .write_all(s.as_bytes())
            .map_err(|_| std::fmt::Error)
    }
}

struct BuffOut {
    buff: String,
}

impl std::fmt::Write for BuffOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(self.buff.extend(s.chars()))
    }
}

#[test]
fn test_offset_computation() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).is_test(true).init();
    let text = CASE_29.as_bytes();
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let tree = match JavaTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    let mut s = StructuralPositionStore::from(StructuralPosition::new(full_node.local.compressed_node));
    let mut scout = Scout::from((StructuralPosition::from((vec![],vec![])),0));
    {
        let mut f = |x,i:u16| {
            let b = stores.node_store.resolve(x);
            let x = b.child(&i).unwrap();
            scout.goto(x,i as usize);
            scout.up(&s);
            scout.goto(x,i as usize);
            x
        };
        let x = full_node.local.compressed_node;
        let x = f(x,30);
        let x = f(x,6);
        let x = f(x,2);
        let x = f(x,7);
        let x = f(x,24);
        let _ = x;
    }
    s.check(&stores).unwrap();
    let x = s.push(&mut scout);
    let z = ExploreStructuralPositions::from((&s,x));
    let p = z.make_position(&stores);
    println!("{}",p);
    println!("|{}|",std::str::from_utf8(&text[p.range()]).unwrap());
    assert_eq!(std::str::from_utf8(&text[p.range()]).unwrap(),r#"ModelUtils.canBeBuilt(new File("./target/spooned/spoon/test/template/ReturnReplaceResult.java"), 8);"#);
}

#[test]
fn test_offset_computation2() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).is_test(true).init();
    let text = CASE_30.as_bytes();
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let tree = match JavaTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    let mut s = StructuralPositionStore::from(StructuralPosition::new(full_node.local.compressed_node));
    let mut scout = Scout::from((StructuralPosition::from((vec![],vec![])),0));
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
    );
    println!();
    {
        let mut f = |x,i:u16| {
            let b = stores.node_store.resolve(x);
            let x = b.child(&i).unwrap();
            scout.goto(x,i as usize);
            scout.up(&s);
            scout.goto(x,i as usize);
            x
        };
        let x = full_node.local.compressed_node;
        let x = f(x,5);
        let _ = x;
    }
    s.check(&stores).unwrap();
    let x = s.push(&mut scout);
    let z = ExploreStructuralPositions::from((&s,x));
    let p = z.make_position(&stores);
    println!("{}",p);
    println!("|{}|",std::str::from_utf8(&text[p.range()]).unwrap());
    assert_eq!(std::str::from_utf8(&text[p.range()]).unwrap(),r#"public class InnerTypeOk {
  private void test() {
    Entry<String, String> test;
  }
}"#);
}

/// historic regression test for static analysis
static CASE_1: &'static str = "
class A {
    char[] c = new char[] { (char) x };
}
";
static CASE_1_1: &'static str = "package q.w.e;
class A {
    char c = null;
}
";
static CASE_1_2: &'static str = "package q.w.e;
class A {
    A c = null;
}
";
static CASE_1_3: &'static str = "package q.w.e;
class A {
    E c = null;
}
";

static CASE_1_4: &'static str = "package q.w.e;
class A  extends E {
    X c = null;
}
";
static CASE_1_5: &'static str = "package q.w.e;
class A  extends E {
    E c = null;
}
";
static CASE_1_6: &'static str = "package q.w.e;
class A  extends E {
    E E = null;
}
";

static CASE_1_7: &'static str = "package q.w.e;
class A  extends E {
    E E = null;
    E e = E;
}
";
static CASE_1_8: &'static str = "package q.w.e;
import a.z.E;

class A  extends E {
    E E = null;
    E e = E;
}
";
static CASE_1_9: &'static str = "package q.w.e;
import a.z.*;

class A  extends E {
    E E = null;
    E e = E;
}
";
static CASE_1_10: &'static str = "package q.w.e;
import a.z.E;
import a.z.*;

class A  extends E {
    E E = null;
    E e = E;
}
";

/// mostly simple resolutions
static CASE_2: &'static str = "package q.w.e;
import a.z.e.r.t.y.ControlFlowGraph;
import a.z.e.r.Exc;
import a.z.e.r.t.y.ControlFlowBuilder;
class A {
    A() {
        this();
    }
    int a = 0;
    void test(int x) {
        x;
        a;
        test(1);
        A b = new A();
        String s = \"\";
        b;
        b.a;
        B.c;
        b.a;
        b;
        b.test(a);
        A method = null;
        ControlFlowBuilder builder = new ControlFlowBuilder();
        builder.set(new Exc());
        builder.build(method);
        ControlFlowGraph cfg = builder.getResult();
    }
}
class B {long c = 0}";

/// a part from java.lang.Character.java
static CASE_3: &'static str = "
package java.lang;

import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import java.util.Locale;

import jdk.internal.HotSpotIntrinsicCandidate;

public final
class Character implements java.io.Serializable, Comparable<Character> {

    public static boolean isLowerCase(char ch) {
        return isLowerCase((int)ch);
    }

    public static boolean isLowerCase(int codePoint) {
        return getType(codePoint) == Character.LOWERCASE_LETTER ||
               CharacterData.of(codePoint).isOtherLowercase(codePoint);    }

    public static boolean isUpperCase(char ch) {
        return isUpperCase((int)ch);
    }

    public static boolean isUpperCase(int codePoint) {
        return getType(codePoint) == Character.UPPERCASE_LETTER ||
               CharacterData.of(codePoint).isOtherUppercase(codePoint);    }

    public static boolean isTitleCase(char ch) {
        return isTitleCase((int)ch);
    }

    public static boolean isTitleCase(int codePoint) {
        return getType(codePoint) == Character.TITLECASE_LETTER;
    }

    public static boolean isDigit(char ch) {
        return isDigit((int)ch);    }

    public static boolean isDigit(int codePoint) {
        return getType(codePoint) == Character.DECIMAL_DIGIT_NUMBER;    }

    public static boolean isDefined(char ch) {
        return isDefined((int)ch);
    }

    public static boolean isDefined(int codePoint) {
        return getType(codePoint) != Character.UNASSIGNED;
    }

    public static boolean isLetter(char ch) {
        return isLetter((int)ch);    }

    public static boolean isLetter(int codePoint) {
        return ((((1 << Character.UPPERCASE_LETTER) |
            (1 << Character.LOWERCASE_LETTER) |
            (1 << Character.TITLECASE_LETTER) |
            (1 << Character.MODIFIER_LETTER) |
            (1 << Character.OTHER_LETTER)) >> getType(codePoint)) & 1)
            != 0;    }

    public static boolean isLetterOrDigit(char ch) {
        return isLetterOrDigit((int)ch);
    }
}";

/// about super
static CASE_4: &'static str = "
package java.lang;

public
class AbstractMethodError extends IncompatibleClassChangeError {

    public AbstractMethodError() {
        super();
    }

    public AbstractMethodError(String s) {
        super(s);
    }
}
";
/// about constructor in java.lang with String as parameters
static CASE_5: &'static str = "
package java.lang;//azer.ty;

public final
class Character {

    public static final class UnicodeBlock {

        private UnicodeBlock(String idName, String alias) {
        }

        public static final UnicodeBlock  BASIC_LATIN =
        new UnicodeBlock(\"BASIC_LATIN\",
                         \"BASIC LATIN\");

    }
}
";

/// about self import
static CASE_6: &'static str = "
package java.lang;

import static java.lang.StackStreamFactory.WalkerState.*;
final class StackStreamFactory {
    enum WalkerState {
        NEW; 
    }
}
";

/// about hierarchical resolutions
static CASE_7: &'static str = "
package p;

class A {
	long a = 0;
	class D {

	}
}

class D {
	
}
class E {
	
}
class F{}

class B extends A implements C {
	// a can be ambiguous between A and C
	long b = a;
	// if decl Both in A and C, D and E are ambiguous
	D d=null; // /.p.A.D
	E e=null; // /.p.C.E
	F f=null; // /.p.F
}

interface C {
	int c = 0;
	class E {
		
	}
}
";

static CASE_8: &'static str = "package q.w.e;
class A {
    Integer a = 0;
    <T> void test(T x) {
        test(1);
        A b = new A();
        b.test(a);
        b.test(x);
        test(a);
        String s = \"\";
        b.test(s);
    }
}";

static CASE_8_1: &'static str = "package q.w.e;
class A {
    <T> void test(T x) {
        test(x);
    }
}";

static CASE_9: &'static str = "
class D {
	int a = 1;
    int f() {
		a = 3;
		int a = 5;
		return a + this.a;
	}
}
";

static CASE_10: &'static str = "package a;
public class A {
    public void f() {
        int second = 0;
        second = second;
        second = second;
    }
}
";

static CASE_11: &'static str = "package a;
public class A {
    public static long f() {
        int start = 0, len = 0;
        A x = new A(start);
    }
}
";

static CASE_11_BIS: &'static str = "package a;
public class A {
    int start, len;
    public static long f() {
        A x = new A(start);
    }
}
";

// TODO handle fall through variable declaration
static CASE_12: &'static str = "package a;
import z.VM;
public class A {
    public static long f() {
        switch (VM.initLevel()) {
            case 0:
            case 1:
            case 2:
                // the system class loader is the built-in app class loader during startup
                return getBuiltinAppClassLoader();
            case 3:
                String msg = null;
                throw new IllegalStateException(msg);
            default:
                // system fully initialized
                assert VM.isBooted() && scl != null;
                A d = null;
                SecurityManager sm = System.getSecurityManager();
                if (sm != null) {
                    checkClassLoaderPermission(scl, Reflection.getCallerClass());
                }
                return scl;
        }
    }
}
";

static CASE_13: &'static str = "package a;
public class A {
    public A(byte ascii[], int hibyte) {
        this(ascii, hibyte, 0, ascii.length);
    }
    public A(byte[] bytes) {
        this(bytes, 0, bytes.length);
    }

    private final byte[] value;

    byte length() {
        return value.length;
    }

    byte length2() {
        return this.value.length;
    }

    byte length3() {
        byte v1[] = null;
        return v1.length;
    }
}
";

static CASE_14: &'static str = "package q.w.e;
import a.z.e.r.*;
import a.z.e.r.Y;
class A {
    Integer a = 0;
    <T> void test(T x) {
        test(1);
        A b = new A();
        b.test(a);
        test(a);
        String s = \"\";
        b.test(s);
        Y y;
    }
}";

static CASE_15: &'static str = "package q.w.e;
class A<V> {
    public Enumeration<V> elements() {
        return this.<V>getEnumeration(VALUES);
    }
}";
static CASE_15_1: &'static str = "package q.w.e;
class A<V> {
    public Enumeration<V> elements() {
        return this.<V>getEnumeration(VALUES);
    }
    public Enumeration<V> getEnumeration(V v) {
        return v.a;
    }
}";
static CASE_15_2: &'static str = "package q.w.e;
class A<V> {
    public Enumeration<V> elements() {
        return this.<V>getEnumeration();
    }
    public Enumeration<V> getEnumeration() {
        return v.a;
    }
}";

static CASE_16: &'static str = "package q.w.e;
class A {
    public <V> Enumeration<V> elements(V x) {
        return this.<V>getEnumeration(VALUES);
    }
}";

static CASE_17: &'static str = "package q.w.e;
enum SSLCipher {
    // exportable ciphers
    @SuppressWarnings({\"unchecked\", \"rawtypes\"})
    B_NULL(\"NULL\", NULL_CIPHER, 0, 0, 0, 0, true, true,
        (Map.Entry<ReadCipherGenerator,
                ProtocolVersion[]>[])(new Map.Entry[] {
            new SimpleImmutableEntry<ReadCipherGenerator, ProtocolVersion[]>(
                new NullReadCipherGenerator(),
                ProtocolVersion.PROTOCOLS_OF_NONE
            ),
            new SimpleImmutableEntry<ReadCipherGenerator, ProtocolVersion[]>(
                new NullReadCipherGenerator(),
                ProtocolVersion.PROTOCOLS_TO_13
            )
        }),
        (Map.Entry<WriteCipherGenerator,
                ProtocolVersion[]>[])(new Map.Entry[] {
            new SimpleImmutableEntry<WriteCipherGenerator, ProtocolVersion[]>(
                new NullWriteCipherGenerator(),
                ProtocolVersion.PROTOCOLS_OF_NONE
            ),
            new SimpleImmutableEntry<WriteCipherGenerator, ProtocolVersion[]>(
                new NullWriteCipherGenerator(),
                ProtocolVersion.PROTOCOLS_TO_13
            )
        })),
";

static CASE_18: &'static str = "
module java.compiler {
    exports javax.annotation.processing;
    exports javax.lang.model;
    exports javax.lang.model.element;
    exports javax.lang.model.type;
    exports javax.lang.model.util;
    exports javax.tools;

    uses javax.tools.DocumentationTool;
    uses javax.tools.JavaCompiler;
}";

static CASE_19: &'static str = "package q.w.e;
class A {
    static class BnM extends Node {
        int[] buffer;
        int[] lastOcc;
        int[] optoSft;
        static Node optimize(Node node) {

            int[] optoSft = new int[patternLength];
            optoSft[j-1] = i;
            while (j > 0) {
                optoSft[--j] = i;
            }
            optoSft[patternLength-1] = 1;
            if (node instanceof SliceS)
                return new BnMS(src, lastOcc, optoSft, node.next);
        }
        BnM(int[] src, int[] lastOcc, int[] optoSft, Node next) {
            this.buffer = src;
            this.lastOcc = lastOcc;
            this.optoSft = optoSft;
            this.next = next;
        }
        boolean match(Matcher matcher, int i, CharSequence seq) {
            i += Math.max(j + 1 - lastOcc[ch&0x7F], optoSft[j]);
        }
    }
    static final class BnMS extends BnM {
        BnMS(int[] src, int[] lastOcc, int[] optoSft, Node next) {
            super(src, lastOcc, optoSft, next);
            for (int cp : buffer) {
                lengthInChars += Character.charCount(cp);
            }
        }
        boolean match(Matcher matcher, int i, CharSequence seq) {
            i += Math.max(j + 1 - lastOcc[ch&0x7F], optoSft[j]);
        }
    }
}";
static CASE_20: &'static str = "package q.w.e;
class A {
    static class BnM extends Node {
        static Node optimize(Node node) {
            return null;
        }
    }
}";

static CASE_21: &'static str = "package q.w.e;
class A {
    static class BnM extends Node {
        int[] optoSft;
    }
    static final class BnMS extends BnM {
        boolean match() {
            optoSft[j];
        }
    }
}";

static CASE_22: &'static str = "package q.w.e;
class A {
    public interface Cleanable {
        void clean();
    }
}";

/// Same name field and type
// TODO need mandatory type ref and mandatory member ref
// so that we do not mask ref to super and interfaces with a field of the same name.
static CASE_23: &'static str = 
"package spoon.test.template.testclasses;

import spoon.reflect.code.CtStatement;
import spoon.reflect.declaration.CtClass;
import spoon.reflect.declaration.CtConstructor;
import spoon.reflect.declaration.CtType;
import spoon.reflect.reference.CtTypeReference;
import spoon.template.AbstractTemplate;
import spoon.template.Local;
import spoon.template.Parameter;
import spoon.template.Substitution;

public class NtonCodeTemplate extends AbstractTemplate<CtClass<?>> implements _TargetType_ {
	@Parameter
	static int _n_;

	@Parameter
    CtTypeReference<?> _TargetType_;

	static _TargetType_[] instances = new _TargetType_[_n_];

	static int instanceCount = 0;

	@Local
	public NtonCodeTemplate(CtTypeReference<?> targetType, int n) {
		_n_ = n;
		_TargetType_ = targetType;
	}

	@Local
	public void initializer() {
		if (instanceCount >= _n_) {
			throw new RuntimeException(\"max number of instances reached\");
		}
		instances[instanceCount++] = this;
	}

	public int getInstanceCount() {
		return instanceCount;
	}

	public _TargetType_ getInstance(int i) {
		if (i > _n_)
			throw new RuntimeException(\"instance number greater than \" + _n_);
		return instances[i];
	}

	public int getMaxInstanceCount() {
		return _n_;
	}

	@Override
	public CtClass<?> apply(CtType<?> ctType) {
		if (ctType instanceof CtClass) {
			CtClass<?> zeClass = (CtClass) ctType;
			Substitution.insertAll(zeClass, this);

			for (CtConstructor<?> c : zeClass.getConstructors()) {
				c.getBody().insertEnd((CtStatement) Substitution.substituteMethodBody(zeClass, this, \"initializer\"));
			}

			return zeClass;
		} else {
			return null;
		}
	}

	class Test {
		public void _name_() {}
	}
}

interface _TargetType_ {

}";

static CASE_24: &'static str = 
"/**
* Licensed to the Apache Software Foundation (ASF) under one
* or more contributor license agreements.  See the NOTICE file
* distributed with this work for additional information
* regarding copyright ownership.  The ASF licenses this file
* to you under the Apache License, Version 2.0 (the
* \"License\"); you may not use this file except in compliance
* with the License.  You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an \"AS IS\" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*/
package org.apache.hadoop.fs;

import org.apache.hadoop.thirdparty.com.google.common.base.Preconditions;
import org.apache.hadoop.conf.Configuration;
import org.apache.hadoop.fs.FileSystem.Statistics;
import org.apache.hadoop.fs.permission.FsPermission;
import org.apache.hadoop.io.IOUtils;
import org.apache.hadoop.test.GenericTestUtils;
import org.apache.hadoop.test.LambdaTestUtils;
import org.apache.hadoop.test.Whitebox;
import org.apache.hadoop.util.StringUtils;

import static org.apache.hadoop.fs.CommonConfigurationKeysPublic.IO_FILE_BUFFER_SIZE_DEFAULT;
import static org.apache.hadoop.fs.CommonConfigurationKeysPublic.IO_FILE_BUFFER_SIZE_KEY;
import static org.apache.hadoop.fs.FileSystemTestHelper.*;

import java.io.*;
import java.net.URI;
import java.util.Arrays;
import java.util.Collection;
import java.util.EnumSet;
import java.util.HashSet;
import java.util.List;
import java.util.Random;
import java.util.Set;
import java.util.stream.Collectors;

import static org.apache.hadoop.test.PlatformAssumptions.assumeNotWindows;
import static org.apache.hadoop.test.PlatformAssumptions.assumeWindows;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;
import static org.mockito.Mockito.*;

import org.junit.After;
import org.junit.Assert;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.Timeout;

import javax.annotation.Nonnull;

import static org.assertj.core.api.Assertions.assertThat;

/**
* This class tests the local file system via the FileSystem abstraction.
*/
public class TestLocalFileSystem {
 private static final File base =
     GenericTestUtils.getTestDir(\"work-dir/localfs\");

 private static final String TEST_ROOT_DIR = base.getAbsolutePath();
 private final Path TEST_PATH = new Path(TEST_ROOT_DIR, \"test-file\");
 private Configuration conf;
 private LocalFileSystem fileSys;

 /**
  * standard test timeout: {@value}.
  */
 public static final int DEFAULT_TEST_TIMEOUT = 60 * 1000;

 /**
  * Set the timeout for every test.
  */
 @Rule
 public Timeout testTimeout = new Timeout(DEFAULT_TEST_TIMEOUT);

 private void cleanupFile(FileSystem fs, Path name) throws IOException {
   assertTrue(fs.exists(name));
   fs.delete(name, true);
   assertTrue(!fs.exists(name));
 }
 
 @Before
 public void setup() throws IOException {
   conf = new Configuration(false);
   conf.set(\"fs.file.impl\", LocalFileSystem.class.getName());
   fileSys = FileSystem.getLocal(conf);
   fileSys.delete(new Path(TEST_ROOT_DIR), true);
 }
 
 @After
 public void after() throws IOException {
   FileUtil.setWritable(base, true);
   FileUtil.fullyDelete(base);
   assertTrue(!base.exists());
   RawLocalFileSystem.useStatIfAvailable();
 }

 /**
  * Test the capability of setting the working directory.
  */
 @Test
 public void testWorkingDirectory() throws IOException {
   Path origDir = fileSys.getWorkingDirectory();
   Path subdir = new Path(TEST_ROOT_DIR, \"new\");
   try {
     // make sure it doesn't already exist
     assertTrue(!fileSys.exists(subdir));
     // make it and check for it
     assertTrue(fileSys.mkdirs(subdir));
     assertTrue(fileSys.isDirectory(subdir));
     
     fileSys.setWorkingDirectory(subdir);
     
     // create a directory and check for it
     Path dir1 = new Path(\"dir1\");
     assertTrue(fileSys.mkdirs(dir1));
     assertTrue(fileSys.isDirectory(dir1));
     
     // delete the directory and make sure it went away
     fileSys.delete(dir1, true);
     assertTrue(!fileSys.exists(dir1));
     
     // create files and manipulate them.
     Path file1 = new Path(\"file1\");
     Path file2 = new Path(\"sub/file2\");
     String contents = writeFile(fileSys, file1, 1);
     fileSys.copyFromLocalFile(file1, file2);
     assertTrue(fileSys.exists(file1));
     assertTrue(fileSys.isFile(file1));
     cleanupFile(fileSys, file2);
     fileSys.copyToLocalFile(file1, file2);
     cleanupFile(fileSys, file2);
     
     // try a rename
     fileSys.rename(file1, file2);
     assertTrue(!fileSys.exists(file1));
     assertTrue(fileSys.exists(file2));
     fileSys.rename(file2, file1);
     
     // try reading a file
     InputStream stm = fileSys.open(file1);
     byte[] buffer = new byte[3];
     int bytesRead = stm.read(buffer, 0, 3);
     assertEquals(contents, new String(buffer, 0, bytesRead));
     stm.close();
   } finally {
     fileSys.setWorkingDirectory(origDir);
   }
 }

 /**
  * test Syncable interface on raw local file system
  * @throws IOException
  */
 @Test
 public void testSyncable() throws IOException {
   FileSystem fs = fileSys.getRawFileSystem();
   Path file = new Path(TEST_ROOT_DIR, \"syncable\");
   FSDataOutputStream out = fs.create(file);;
   final int bytesWritten = 1;
   byte[] expectedBuf = new byte[] {'0', '1', '2', '3'};
   try {
     out.write(expectedBuf, 0, 1);
     out.hflush();
     verifyFile(fs, file, bytesWritten, expectedBuf);
     out.write(expectedBuf, bytesWritten, expectedBuf.length-bytesWritten);
     out.hsync();
     verifyFile(fs, file, expectedBuf.length, expectedBuf);
   } finally {
     out.close();
   }
 }
 
 private void verifyFile(FileSystem fs, Path file, int bytesToVerify, 
     byte[] expectedBytes) throws IOException {
   FSDataInputStream in = fs.open(file);
   try {
     byte[] readBuf = new byte[bytesToVerify];
     in.readFully(readBuf, 0, bytesToVerify);
     for (int i=0; i<bytesToVerify; i++) {
       assertEquals(expectedBytes[i], readBuf[i]);
     }
   } finally {
     in.close();
   }
 }
 
 @Test
 public void testCopy() throws IOException {
   Path src = new Path(TEST_ROOT_DIR, \"dingo\");
   Path dst = new Path(TEST_ROOT_DIR, \"yak\");
   writeFile(fileSys, src, 1);
   assertTrue(FileUtil.copy(fileSys, src, fileSys, dst, true, false, conf));
   assertTrue(!fileSys.exists(src) && fileSys.exists(dst));
   assertTrue(FileUtil.copy(fileSys, dst, fileSys, src, false, false, conf));
   assertTrue(fileSys.exists(src) && fileSys.exists(dst));
   assertTrue(FileUtil.copy(fileSys, src, fileSys, dst, true, true, conf));
   assertTrue(!fileSys.exists(src) && fileSys.exists(dst));
   fileSys.mkdirs(src);
   assertTrue(FileUtil.copy(fileSys, dst, fileSys, src, false, false, conf));
   Path tmp = new Path(src, dst.getName());
   assertTrue(fileSys.exists(tmp) && fileSys.exists(dst));
   assertTrue(FileUtil.copy(fileSys, dst, fileSys, src, false, true, conf));
   assertTrue(fileSys.delete(tmp, true));
   fileSys.mkdirs(tmp);
   try {
     FileUtil.copy(fileSys, dst, fileSys, src, true, true, conf);
     fail(\"Failed to detect existing dir\");
   } catch (IOException e) {
     // Expected
   }
 }

 @Test
 public void testHomeDirectory() throws IOException {
   Path home = fileSys.makeQualified(
       new Path(System.getProperty(\"user.home\")));
   Path fsHome = fileSys.getHomeDirectory();
   assertEquals(home, fsHome);
 }

 @Test
 public void testPathEscapes() throws IOException {
   Path path = new Path(TEST_ROOT_DIR, \"foo%bar\");
   writeFile(fileSys, path, 1);
   FileStatus status = fileSys.getFileStatus(path);
   assertEquals(fileSys.makeQualified(path), status.getPath());
   cleanupFile(fileSys, path);
 }
 
 @Test
 public void testCreateFileAndMkdirs() throws IOException {
   Path test_dir = new Path(TEST_ROOT_DIR, \"test_dir\");
   Path test_file = new Path(test_dir, \"file1\");
   assertTrue(fileSys.mkdirs(test_dir));
  
   final int fileSize = new Random().nextInt(1 << 20) + 1;
   writeFile(fileSys, test_file, fileSize);

   {
     //check FileStatus and ContentSummary 
     final FileStatus status = fileSys.getFileStatus(test_file);
     Assert.assertEquals(fileSize, status.getLen());
     final ContentSummary summary = fileSys.getContentSummary(test_dir);
     Assert.assertEquals(fileSize, summary.getLength());
   }
   
   // creating dir over a file
   Path bad_dir = new Path(test_file, \"another_dir\");
   
   try {
     fileSys.mkdirs(bad_dir);
     fail(\"Failed to detect existing file in path\");
   } catch (ParentNotDirectoryException e) {
     // Expected
   }
   
   try {
       fileSys.mkdirs(null);
     fail(\"Failed to detect null in mkdir arg\");
   } catch (IllegalArgumentException e) {
     // Expected
   }
 }

 /** Test deleting a file, directory, and non-existent path */
 @Test
 public void testBasicDelete() throws IOException {
   Path dir1 = new Path(TEST_ROOT_DIR, \"dir1\");
   Path file1 = new Path(TEST_ROOT_DIR, \"file1\");
   Path file2 = new Path(TEST_ROOT_DIR+\"/dir1\", \"file2\");
   Path file3 = new Path(TEST_ROOT_DIR, \"does-not-exist\");
   assertTrue(fileSys.mkdirs(dir1));
   writeFile(fileSys, file1, 1);
   writeFile(fileSys, file2, 1);
   assertFalse(\"Returned true deleting non-existant path\", 
           fileSys.delete(file3));
   assertTrue(\"Did not delete file\", fileSys.delete(file1));
   assertTrue(\"Did not delete non-empty dir\", fileSys.delete(dir1));
 }
 
 @Test
 public void testStatistics() throws Exception {
   int fileSchemeCount = 0;
   for (Statistics stats : FileSystem.getAllStatistics()) {
     if (stats.getScheme().equals(\"file\")) {
       fileSchemeCount++;
     }
   }
   assertEquals(1, fileSchemeCount);
 }

 @Test
 public void testHasFileDescriptor() throws IOException {
   Path path = new Path(TEST_ROOT_DIR, \"test-file\");
   writeFile(fileSys, path, 1);
   BufferedFSInputStream bis = null;
   try {
     bis = new BufferedFSInputStream(new RawLocalFileSystem()
       .new LocalFSFileInputStream(path), 1024);
     assertNotNull(bis.getFileDescriptor());
   } finally {
     IOUtils.cleanupWithLogger(null, bis);
   }
 }

 @Test
 public void testListStatusWithColons() throws IOException {
   assumeNotWindows();
   File colonFile = new File(TEST_ROOT_DIR, \"foo:bar\");
   colonFile.mkdirs();
   FileStatus[] stats = fileSys.listStatus(new Path(TEST_ROOT_DIR));
   assertEquals(\"Unexpected number of stats\", 1, stats.length);
   assertEquals(\"Bad path from stat\", colonFile.getAbsolutePath(),
       stats[0].getPath().toUri().getPath());
 }
 
 @Test
 public void testListStatusReturnConsistentPathOnWindows() throws IOException {
   assumeWindows();
   String dirNoDriveSpec = TEST_ROOT_DIR;
   if (dirNoDriveSpec.charAt(1) == ':')
       dirNoDriveSpec = dirNoDriveSpec.substring(2);
   
   File file = new File(dirNoDriveSpec, \"foo\");
   file.mkdirs();
   FileStatus[] stats = fileSys.listStatus(new Path(dirNoDriveSpec));
   assertEquals(\"Unexpected number of stats\", 1, stats.length);
   assertEquals(\"Bad path from stat\", new Path(file.getPath()).toUri().getPath(),
       stats[0].getPath().toUri().getPath());
 }
 
 @Test
 public void testReportChecksumFailure() throws IOException {
   base.mkdirs();
   assertTrue(base.exists() && base.isDirectory());
   
   final File dir1 = new File(base, \"dir1\");
   final File dir2 = new File(dir1, \"dir2\");
   dir2.mkdirs();
   assertTrue(dir2.exists() && FileUtil.canWrite(dir2));
   
   final String dataFileName = \"corruptedData\";
   final Path dataPath = new Path(new File(dir2, dataFileName).toURI());
   final Path checksumPath = fileSys.getChecksumFile(dataPath);
   final FSDataOutputStream fsdos = fileSys.create(dataPath);
   try {
     fsdos.writeUTF(\"foo\");
   } finally {
     fsdos.close();
   }
   assertTrue(fileSys.pathToFile(dataPath).exists());
   final long dataFileLength = fileSys.getFileStatus(dataPath).getLen();
   assertTrue(dataFileLength > 0);
   
   // check the the checksum file is created and not empty:
   assertTrue(fileSys.pathToFile(checksumPath).exists());
   final long checksumFileLength = fileSys.getFileStatus(checksumPath).getLen();
   assertTrue(checksumFileLength > 0);
   
   // this is a hack to force the #reportChecksumFailure() method to stop
   // climbing up at the 'base' directory and use 'dir1/bad_files' as the 
   // corrupted files storage:
   FileUtil.setWritable(base, false);
   
   FSDataInputStream dataFsdis = fileSys.open(dataPath);
   FSDataInputStream checksumFsdis = fileSys.open(checksumPath);
   
   boolean retryIsNecessary = fileSys.reportChecksumFailure(dataPath, dataFsdis, 0, checksumFsdis, 0);
   assertTrue(!retryIsNecessary);
   
   // the data file should be moved:
   assertTrue(!fileSys.pathToFile(dataPath).exists());
   // the checksum file should be moved:
   assertTrue(!fileSys.pathToFile(checksumPath).exists());
   
   // check that the files exist in the new location where they were moved:
   File[] dir1files = dir1.listFiles(new FileFilter() {
     @Override
     public boolean accept(File pathname) {
       return pathname != null && !pathname.getName().equals(\"dir2\");
     }
   });
   assertTrue(dir1files != null);
   assertTrue(dir1files.length == 1);
   File badFilesDir = dir1files[0];
   
   File[] badFiles = badFilesDir.listFiles();
   assertTrue(badFiles != null);
   assertTrue(badFiles.length == 2);
   boolean dataFileFound = false;
   boolean checksumFileFound = false;
   for (File badFile: badFiles) {
     if (badFile.getName().startsWith(dataFileName)) {
       assertTrue(dataFileLength == badFile.length());
       dataFileFound = true;
     } else if (badFile.getName().contains(dataFileName + \".crc\")) {
       assertTrue(checksumFileLength == badFile.length());
       checksumFileFound = true;
     }
   }
   assertTrue(dataFileFound);
   assertTrue(checksumFileFound);
 }

 private void checkTimesStatus(Path path,
   long expectedModTime, long expectedAccTime) throws IOException {
   FileStatus status = fileSys.getFileStatus(path);
   assertEquals(expectedModTime, status.getModificationTime());
   assertEquals(expectedAccTime, status.getAccessTime());
 }

 @Test
 public void testSetTimes() throws Exception {
   Path path = new Path(TEST_ROOT_DIR, \"set-times\");
   writeFile(fileSys, path, 1);

   // test only to the nearest second, as the raw FS may not
   // support millisecond timestamps
   long newModTime = 12345000;
   long newAccTime = 23456000;

   FileStatus status = fileSys.getFileStatus(path);
   assertTrue(\"check we're actually changing something\", newModTime != status.getModificationTime());
   assertTrue(\"check we're actually changing something\", newAccTime != status.getAccessTime());

   fileSys.setTimes(path, newModTime, newAccTime);
   checkTimesStatus(path, newModTime, newAccTime);

   newModTime = 34567000;

   fileSys.setTimes(path, newModTime, -1);
   checkTimesStatus(path, newModTime, newAccTime);

   newAccTime = 45678000;

   fileSys.setTimes(path, -1, newAccTime);
   checkTimesStatus(path, newModTime, newAccTime);
 }

 /**
  * Regression test for HADOOP-9307: BufferedFSInputStream returning
  * wrong results after certain sequences of seeks and reads.
  */
 @Test
 public void testBufferedFSInputStream() throws IOException {
   Configuration conf = new Configuration();
   conf.setClass(\"fs.file.impl\", RawLocalFileSystem.class, FileSystem.class);
   conf.setInt(CommonConfigurationKeysPublic.IO_FILE_BUFFER_SIZE_KEY, 4096);
   FileSystem fs = FileSystem.newInstance(conf);
   
   byte[] buf = new byte[10*1024];
   new Random().nextBytes(buf);
   
   // Write random bytes to file
   FSDataOutputStream stream = fs.create(TEST_PATH);
   try {
     stream.write(buf);
   } finally {
     stream.close();
   }
   
   Random r = new Random();

   FSDataInputStream stm = fs.open(TEST_PATH);
   // Record the sequence of seeks and reads which trigger a failure.
   int seeks[] = new int[10];
   int reads[] = new int[10];
   try {
     for (int i = 0; i < 1000; i++) {
       int seekOff = r.nextInt(buf.length); 
       int toRead = r.nextInt(Math.min(buf.length - seekOff, 32000));
       
       seeks[i % seeks.length] = seekOff;
       reads[i % reads.length] = toRead;
       verifyRead(stm, buf, seekOff, toRead);
       
     }
   } catch (AssertionError afe) {
     StringBuilder sb = new StringBuilder();
     sb.append(\"Sequence of actions:\\n\");
     for (int j = 0; j < seeks.length; j++) {
       sb.append(\"seek @ \").append(seeks[j]).append(\"  \")
         .append(\"read \").append(reads[j]).append(\"\\n\");
     }
     System.err.println(sb.toString());
     throw afe;
   } finally {
     stm.close();
   }
 }

 /**
  * Tests a simple rename of a directory.
  */
 @Test
 public void testRenameDirectory() throws IOException {
   Path src = new Path(TEST_ROOT_DIR, \"dir1\");
   Path dst = new Path(TEST_ROOT_DIR, \"dir2\");
   fileSys.delete(src, true);
   fileSys.delete(dst, true);
   assertTrue(fileSys.mkdirs(src));
   assertTrue(fileSys.rename(src, dst));
   assertTrue(fileSys.exists(dst));
   assertFalse(fileSys.exists(src));
 }

 /**
  * Tests that renaming a directory replaces the destination if the destination
  * is an existing empty directory.
  * 
  * Before:
  *   /dir1
  *     /file1
  *     /file2
  *   /dir2
  * 
  * After rename(\"/dir1\", \"/dir2\"):
  *   /dir2
  *     /file1
  *     /file2
  */
 @Test
 public void testRenameReplaceExistingEmptyDirectory() throws IOException {
   Path src = new Path(TEST_ROOT_DIR, \"dir1\");
   Path dst = new Path(TEST_ROOT_DIR, \"dir2\");
   fileSys.delete(src, true);
   fileSys.delete(dst, true);
   assertTrue(fileSys.mkdirs(src));
   writeFile(fileSys, new Path(src, \"file1\"), 1);
   writeFile(fileSys, new Path(src, \"file2\"), 1);
   assertTrue(fileSys.mkdirs(dst));
   assertTrue(fileSys.rename(src, dst));
   assertTrue(fileSys.exists(dst));
   assertTrue(fileSys.exists(new Path(dst, \"file1\")));
   assertTrue(fileSys.exists(new Path(dst, \"file2\")));
   assertFalse(fileSys.exists(src));
 }

 /**
  * Tests that renaming a directory to an existing directory that is not empty
  * results in a full copy of source to destination.
  * 
  * Before:
  *   /dir1
  *     /dir2
  *       /dir3
  *         /file1
  *         /file2
  * 
  * After rename(\"/dir1/dir2/dir3\", \"/dir1\"):
  *   /dir1
  *     /dir3
  *       /file1
  *       /file2
  */
 @Test
 public void testRenameMoveToExistingNonEmptyDirectory() throws IOException {
   Path src = new Path(TEST_ROOT_DIR, \"dir1/dir2/dir3\");
   Path dst = new Path(TEST_ROOT_DIR, \"dir1\");
   fileSys.delete(src, true);
   fileSys.delete(dst, true);
   assertTrue(fileSys.mkdirs(src));
   writeFile(fileSys, new Path(src, \"file1\"), 1);
   writeFile(fileSys, new Path(src, \"file2\"), 1);
   assertTrue(fileSys.exists(dst));
   assertTrue(fileSys.rename(src, dst));
   assertTrue(fileSys.exists(dst));
   assertTrue(fileSys.exists(new Path(dst, \"dir3\")));
   assertTrue(fileSys.exists(new Path(dst, \"dir3/file1\")));
   assertTrue(fileSys.exists(new Path(dst, \"dir3/file2\")));
   assertFalse(fileSys.exists(src));
 }
 
 private void verifyRead(FSDataInputStream stm, byte[] fileContents,
      int seekOff, int toRead) throws IOException {
   byte[] out = new byte[toRead];
   stm.seek(seekOff);
   stm.readFully(out);
   byte[] expected = Arrays.copyOfRange(fileContents, seekOff, seekOff+toRead);
   if (!Arrays.equals(out, expected)) {
     String s =\"\\nExpected: \" +
         StringUtils.byteToHexString(expected) +
         \"\\ngot:      \" +
         StringUtils.byteToHexString(out) + 
         \"\\noff=\" + seekOff + \" len=\" + toRead;
     fail(s);
   }
 }

 @Test
 public void testStripFragmentFromPath() throws Exception {
   FileSystem fs = FileSystem.getLocal(new Configuration());
   Path pathQualified = TEST_PATH.makeQualified(fs.getUri(),
       fs.getWorkingDirectory());
   Path pathWithFragment = new Path(
       new URI(pathQualified.toString() + \"#glacier\"));
   // Create test file with fragment
   FileSystemTestHelper.createFile(fs, pathWithFragment);
   Path resolved = fs.resolvePath(pathWithFragment);
   assertEquals(\"resolvePath did not strip fragment from Path\", pathQualified,
       resolved);
 }

 @Test
 public void testAppendSetsPosCorrectly() throws Exception {
   FileSystem fs = fileSys.getRawFileSystem();
   Path file = new Path(TEST_ROOT_DIR, \"test-append\");

   fs.delete(file, true);
   FSDataOutputStream out = fs.create(file);

   try {
     out.write(\"text1\".getBytes());
   } finally {
     out.close();
   }

   // Verify the position
   out = fs.append(file);
   try {
     assertEquals(5, out.getPos());
     out.write(\"text2\".getBytes());
   } finally {
     out.close();
   }

   // Verify the content
   FSDataInputStream in = fs.open(file);
   try {
     byte[] buf = new byte[in.available()];
     in.readFully(buf);
     assertEquals(\"text1text2\", new String(buf));
   } finally {
     in.close();
   }
 }

 @Test
 public void testFileStatusPipeFile() throws Exception {
   RawLocalFileSystem origFs = new RawLocalFileSystem();
   RawLocalFileSystem fs = spy(origFs);
   Configuration conf = mock(Configuration.class);
   fs.setConf(conf);
   Whitebox.setInternalState(fs, \"useDeprecatedFileStatus\", false);
   Path path = new Path(\"/foo\");
   File pipe = mock(File.class);
   when(pipe.isFile()).thenReturn(false);
   when(pipe.isDirectory()).thenReturn(false);
   when(pipe.exists()).thenReturn(true);

   FileStatus stat = mock(FileStatus.class);
   doReturn(pipe).when(fs).pathToFile(path);
   doReturn(stat).when(fs).getFileStatus(path);
   FileStatus[] stats = fs.listStatus(path);
   assertTrue(stats != null && stats.length == 1 && stats[0] == stat);
 }

 @Test
 public void testFSOutputStreamBuilder() throws Exception {
   Path path = new Path(TEST_ROOT_DIR, \"testBuilder\");

   try {
     FSDataOutputStreamBuilder builder =
         fileSys.createFile(path).recursive();
     FSDataOutputStream out = builder.build();
     String content = \"Create with a generic type of createFile!\";
     byte[] contentOrigin = content.getBytes(\"UTF8\");
     out.write(contentOrigin);
     out.close();

     FSDataInputStream input = fileSys.open(path);
     byte[] buffer =
         new byte[(int) (fileSys.getFileStatus(path).getLen())];
     input.readFully(0, buffer);
     input.close();
     Assert.assertArrayEquals(\"The data be read should equals with the \"
         + \"data written.\", contentOrigin, buffer);
   } catch (IOException e) {
     throw e;
   }

   // Test value not being set for replication, block size, buffer size
   // and permission
   FSDataOutputStreamBuilder builder =
       fileSys.createFile(path);
   try (FSDataOutputStream stream = builder.build()) {
     assertThat(builder.getBlockSize())
         .withFailMessage(\"Should be default block size\")
         .isEqualTo(fileSys.getDefaultBlockSize());
     assertThat(builder.getReplication())
         .withFailMessage(\"Should be default replication factor\")
         .isEqualTo(fileSys.getDefaultReplication());
     assertThat(builder.getBufferSize())
         .withFailMessage(\"Should be default buffer size\")
         .isEqualTo(fileSys.getConf().getInt(IO_FILE_BUFFER_SIZE_KEY,
             IO_FILE_BUFFER_SIZE_DEFAULT));
     assertThat(builder.getPermission())
         .withFailMessage(\"Should be default permission\")
         .isEqualTo(FsPermission.getFileDefault());
   }

   // Test set 0 to replication, block size and buffer size
   builder = fileSys.createFile(path);
   builder.bufferSize(0).blockSize(0).replication((short) 0);
   assertThat(builder.getBlockSize())
       .withFailMessage(\"Block size should be 0\")
       .isZero();
   assertThat(builder.getReplication())
       .withFailMessage(\"Replication factor should be 0\")
       .isZero();
   assertThat(builder.getBufferSize())
       .withFailMessage(\"Buffer size should be 0\")
       .isZero();
 }

 /**
  * A builder to verify configuration keys are supported.
  */
 private static class BuilderWithSupportedKeys
     extends FSDataOutputStreamBuilder<FSDataOutputStream,
     BuilderWithSupportedKeys> {

   private final Set<String> supportedKeys = new HashSet<>();

   BuilderWithSupportedKeys(@Nonnull final Collection<String> supportedKeys,
       @Nonnull FileSystem fileSystem, @Nonnull Path p) {
     super(fileSystem, p);
     this.supportedKeys.addAll(supportedKeys);
   }

   @Override
   public BuilderWithSupportedKeys getThisBuilder() {
     return this;
   }

   @Override
   public FSDataOutputStream build()
       throws IllegalArgumentException, IOException {
     Set<String> unsupported = new HashSet<>(getMandatoryKeys());
     unsupported.removeAll(supportedKeys);
     Preconditions.checkArgument(unsupported.isEmpty(),
         \"unsupported key found: \" + supportedKeys);
     return getFS().create(
         getPath(), getPermission(), getFlags(), getBufferSize(),
         getReplication(), getBlockSize(), getProgress(), getChecksumOpt());
   }
 }

 @Test
 public void testFSOutputStreamBuilderOptions() throws Exception {
   Path path = new Path(TEST_ROOT_DIR, \"testBuilderOpt\");
   final List<String> supportedKeys = Arrays.asList(\"strM\");

   FSDataOutputStreamBuilder<?, ?> builder =
       new BuilderWithSupportedKeys(supportedKeys, fileSys, path);
   builder.opt(\"strKey\", \"value\");
   builder.opt(\"intKey\", 123);
   builder.opt(\"strM\", \"ignored\");
   // Over-write an optional value with a mandatory value.
   builder.must(\"strM\", \"value\");
   builder.must(\"unsupported\", 12.34);

   assertEquals(\"Optional value should be overwrite by a mandatory value\",
       \"value\", builder.getOptions().get(\"strM\"));

   Set<String> mandatoryKeys = builder.getMandatoryKeys();
   Set<String> expectedKeys = new HashSet<>();
   expectedKeys.add(\"strM\");
   expectedKeys.add(\"unsupported\");
   assertEquals(expectedKeys, mandatoryKeys);
   assertEquals(2, mandatoryKeys.size());

   LambdaTestUtils.intercept(IllegalArgumentException.class,
       \"unsupported key found\", builder::build
   );
 }

 private static final int CRC_SIZE = 12;

 private static final byte[] DATA = \"1234567890\".getBytes();

 /**
  * Get the statistics for the file schema. Contains assertions
  * @return the statistics on all file:// IO.
  */
 protected Statistics getFileStatistics() {
   final List<Statistics> all = FileSystem.getAllStatistics();
   final List<Statistics> fileStats = all
       .stream()
       .filter(s -> s.getScheme().equals(\"file\"))
       .collect(Collectors.toList());
   assertEquals(\"Number of statistics counters for file://\",
       1, fileStats.size());
   // this should be used for local and rawLocal, as they share the
   // same schema (although their class is different)
   return fileStats.get(0);
 }

 /**
  * Write the byte array {@link #DATA} to the given output stream.
  * @param s stream to write to.
  * @throws IOException failure to write/close the file
  */
 private void writeData(FSDataOutputStream s) throws IOException {
   s.write(DATA);
   s.close();
 }

 /**
  * Evaluate the closure while counting bytes written during
  * its execution, and verify that the count included the CRC
  * write as well as the data.
  * After the operation, the file is deleted.
  * @param operation operation for assertion method.
  * @param path path to write
  * @param callable expression evaluated
  * @param delete should the file be deleted after?
  */
 private void assertWritesCRC(String operation, Path path,
     LambdaTestUtils.VoidCallable callable, boolean delete) throws Exception {
   final Statistics stats = getFileStatistics();
   final long bytesOut0 = stats.getBytesWritten();
   try {
     callable.call();
     assertEquals(\"Bytes written in \" + operation + \"; stats=\" + stats,
         CRC_SIZE + DATA.length, stats.getBytesWritten() - bytesOut0);
   } finally {
     if (delete) {
       // clean up
       try {
         fileSys.delete(path, false);
       } catch (IOException ignored) {
         // ignore this cleanup failure
       }
     }
   }
 }

 /**
  * Verify that File IO through the classic non-builder APIs generate
  * statistics which imply that CRCs were read and written.
  */
 @Test
 public void testCRCwithClassicAPIs() throws Throwable {
   final Path file = new Path(TEST_ROOT_DIR, \"testByteCountersClassicAPIs\");
   assertWritesCRC(\"create()\",
       file,
       () -> writeData(fileSys.create(file, true)),
       false);

   final Statistics stats = getFileStatistics();
   final long bytesRead0 = stats.getBytesRead();
   fileSys.open(file).close();
   final long bytesRead1 = stats.getBytesRead();
   assertEquals(\"Bytes read in open() call with stats \" + stats,
       CRC_SIZE, bytesRead1 - bytesRead0);
 }

 /**
  * create/7 to use write the CRC.
  */
 @Test
 public void testCRCwithCreate7() throws Throwable {
   final Path file = new Path(TEST_ROOT_DIR, \"testCRCwithCreate7\");
   assertWritesCRC(\"create/7\",
       file,
       () -> writeData(
           fileSys.create(file,
               FsPermission.getFileDefault(),
               true,
               8192,
               (short)1,
               16384,
               null)),
       true);
 }

 /**
  * Create with ChecksumOpt to create checksums.
  * If the LocalFS ever interpreted the flag, this test may fail.
  */
 @Test
 public void testCRCwithCreateChecksumOpt() throws Throwable {
   final Path file = new Path(TEST_ROOT_DIR, \"testCRCwithCreateChecksumOpt\");
   assertWritesCRC(\"create with checksum opt\",
       file,
       () -> writeData(
           fileSys.create(file,
               FsPermission.getFileDefault(),
               EnumSet.of(CreateFlag.CREATE),
               8192,
               (short)1,
               16384,
               null,
               Options.ChecksumOpt.createDisabled())),
       true);
 }

 /**
  * Create createNonRecursive/6.
  */
 @Test
 public void testCRCwithCreateNonRecursive6() throws Throwable {
   fileSys.mkdirs(TEST_PATH);
   final Path file = new Path(TEST_ROOT_DIR,
       \"testCRCwithCreateNonRecursive6\");
   assertWritesCRC(\"create with checksum opt\",
       file,
       () -> writeData(
           fileSys.createNonRecursive(file,
               FsPermission.getFileDefault(),
               true,
               8192,
               (short)1,
               16384,
               null)),
       true);
 }

 /**
  * Create createNonRecursive with CreateFlags.
  */
 @Test
 public void testCRCwithCreateNonRecursiveCreateFlags() throws Throwable {
   fileSys.mkdirs(TEST_PATH);
   final Path file = new Path(TEST_ROOT_DIR,
       \"testCRCwithCreateNonRecursiveCreateFlags\");
   assertWritesCRC(\"create with checksum opt\",
       file,
       () -> writeData(
           fileSys.createNonRecursive(file,
               FsPermission.getFileDefault(),
               EnumSet.of(CreateFlag.CREATE),
               8192,
               (short)1,
               16384,
               null)),
       true);
 }


 /**
  * This relates to MAPREDUCE-7184, where the openFile() call's
  * CRC count wasn't making into the statistics for the current thread.
  * If the evaluation was in a separate thread you'd expect that,
  * but if the completable future is in fact being synchronously completed
  * it should not happen.
  */
 @Test
 public void testReadIncludesCRCwithBuilders() throws Throwable {

   final Path file = new Path(TEST_ROOT_DIR,
       \"testReadIncludesCRCwithBuilders\");
   Statistics stats = getFileStatistics();
   // write the file using the builder API
   assertWritesCRC(\"createFile()\",
       file,
       () -> writeData(
           fileSys.createFile(file)
               .overwrite(true).recursive()
               .build()),
       false);

   // now read back the data, again with the builder API
   final long bytesRead0 = stats.getBytesRead();
   fileSys.openFile(file).build().get().close();
   assertEquals(\"Bytes read in openFile() call with stats \" + stats,
       CRC_SIZE, stats.getBytesRead() - bytesRead0);
   // now write with overwrite = true
   assertWritesCRC(\"createFileNonRecursive()\",
       file,
       () -> {
         try (FSDataOutputStream s = fileSys.createFile(file)
             .overwrite(true)
             .build()) {
           s.write(DATA);
         }
       },
       true);
 }

 /**
  * Write with the builder, using the normal recursive create
  * with create flags containing the overwrite option.
  */
 @Test
 public void testWriteWithBuildersRecursive() throws Throwable {

   final Path file = new Path(TEST_ROOT_DIR,
       \"testWriteWithBuildersRecursive\");
   Statistics stats = getFileStatistics();
   // write the file using the builder API
   assertWritesCRC(\"createFile()\",
       file,
       () -> writeData(
           fileSys.createFile(file)
               .overwrite(false)
               .recursive()
               .build()),
       true);
 }
}";


// TODO handle same identical name for parameter and its type
static CASE_25: &'static str = 
"

package org.apache.hadoop.yarn.api.protocolrecords.impl.pb;

public class GetApplicationAttemptReportResponsePBImpl extends
    GetApplicationAttemptReportResponse {

  @Override
  public void setApplicationAttemptReport(
      ApplicationAttemptReport ApplicationAttemptReport) {
    maybeInitBuilder();
    if (ApplicationAttemptReport == null) {
      builder.clearApplicationAttemptReport();
    }
    this.applicationAttemptReport = ApplicationAttemptReport;
  }

}
";
static CASE_26: &'static str = 
"

package p;

import x.HttpServer2;

public class A{

  public void f() {
        new HttpServer2.Builder()
        .setName();
  }

}
";

// Nothing in X.java file
// see https://github.com/apache/hadoop/blob/03cfc852791c14fad39db4e5b14104a276c08e59/hadoop-yarn-project/hadoop-yarn/hadoop-yarn-server/hadoop-yarn-server-nodemanager/src/main/java/org/apache/hadoop/yarn/server/nodemanager/webapp/AggregatedLogsBlock.java
static CASE_27: &'static str = 
"/**
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * \"License\"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an \"AS IS\" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */";

 static CASE_28: &'static str = 
 "package p; 
class A {
    org.apa.B x;
}";

static CASE_29: &'static str = 
"/**\r
 * Copyright (C) 2006-2018 INRIA and contributors\r
 * Spoon - http://spoon.gforge.inria.fr/\r
 *\r
 * This software is governed by the CeCILL-C License under French law and\r
 * abiding by the rules of distribution of free software. You can use, modify\r
 * and/or redistribute the software under the terms of the CeCILL-C license as\r
 * circulated by CEA, CNRS and INRIA at http://www.cecill.info.\r
 *\r
 * This program is distributed in the hope that it will be useful, but WITHOUT\r
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or\r
 * FITNESS FOR A PARTICULAR PURPOSE. See the CeCILL-C License for more details.\r
 *\r
 * The fact that you are presently reading this means that you have had\r
 * knowledge of the CeCILL-C license and that you accept its terms.\r
 */\r
package spoon.test.template;\r
\r
import static org.junit.jupiter.api.Assertions.assertEquals;\r
import static spoon.testing.utils.ModelUtils.getOptimizedString;\r
\r
import java.io.File;\r
\r
import org.junit.jupiter.api.Test;\r
\r
import spoon.Launcher;\r
import spoon.OutputType;\r
import spoon.reflect.code.CtBlock;\r
import spoon.reflect.code.CtExpression;\r
import spoon.reflect.declaration.CtClass;\r
import spoon.reflect.factory.Factory;\r
import spoon.support.compiler.FileSystemFile;\r
import spoon.test.template.testclasses.ReturnReplaceTemplate;\r
import spoon.testing.utils.ModelUtils;\r
\r
public class TemplateReplaceReturnTest {\r
\r
	@Test\r
	public void testReturnReplaceTemplate() {\r
		//contract: the template engine supports replace of `return _param_.S()` by `<CtBlock>`\r
		Launcher launcher = new Launcher();\r
		launcher.addTemplateResource(new FileSystemFile(\"./src/test/java/spoon/test/template/testclasses/ReturnReplaceTemplate.java\"));\r
\r
		launcher.buildModel();\r
		Factory factory = launcher.getFactory();\r
\r
		CtBlock<String> model = (CtBlock) factory.Templates().Class().get(ReturnReplaceTemplate.class).getMethod(\"sample\").getBody();\r
		\r
		CtClass<?> resultKlass = factory.Class().create(factory.Package().getOrCreate(\"spoon.test.template\"), \"ReturnReplaceResult\");\r
		new ReturnReplaceTemplate(model).apply(resultKlass);\r
		assertEquals(\"{ if ((java.lang.System.currentTimeMillis() % 2L) == 0) { return \\\"Panna\\\"; } else { return \\\"Orel\\\"; }}\", getOptimizedString(resultKlass.getMethod(\"method\").getBody()));\r
		launcher.setSourceOutputDirectory(new File(\"./target/spooned/\"));\r
		launcher.getModelBuilder().generateProcessedSourceFiles(OutputType.CLASSES);\r
		ModelUtils.canBeBuilt(new File(\"./target/spooned/spoon/test/template/ReturnReplaceResult.java\"), 8);\r
	}\r
\r
	@Test\r
	public void testNoReturnReplaceTemplate() {\r
		//contract: the template engine supports replace of return expression by `<CtExpression>`\r
		Launcher launcher = new Launcher();\r
		launcher.addTemplateResource(new FileSystemFile(\"./src/test/java/spoon/test/template/testclasses/ReturnReplaceTemplate.java\"));\r
\r
		launcher.buildModel();\r
		Factory factory = launcher.getFactory();\r
\r
		CtExpression<String> model = factory.createLiteral(\"AStringLiteral\");\r
		\r
		CtClass<?> resultKlass = factory.Class().create(factory.Package().getOrCreate(\"spoon.test.template\"), \"ReturnReplaceResult\");\r
		new ReturnReplaceTemplate(model).apply(resultKlass);\r
		assertEquals(\"{ return \\\"AStringLiteral\\\";}\", getOptimizedString(resultKlass.getMethod(\"method\").getBody()));\r
		launcher.setSourceOutputDirectory(new File(\"./target/spooned/\"));\r
		launcher.getModelBuilder().generateProcessedSourceFiles(OutputType.CLASSES);\r
		ModelUtils.canBeBuilt(new File(\"./target/spooned/spoon/test/template/ReturnReplaceResult.java\"), 8);\r
	}\r

}";

static CASE_30: &'static str = r#"
package spoon.test.prettyprinter.testclasses.innertype;

import java.util.Map.*;

public class InnerTypeOk {
  private void test() {
    Entry<String, String> test;
  }
}"#;

static CASE_31: &'static str = r#"
package q.w.e;

public class A {
    public static class B {
        public A f() {
            return null;
        }
    }
}"#;

static CASE_32: &'static str = r#"
package q.w.e;

public class A {
    public static void main(final String[] args) {
        launch(args);
    }
}"#;


static CASE_33: &'static str = r#"package io.quarkus.spring.web.deployment;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

import javax.ws.rs.core.Response;
import javax.ws.rs.ext.ExceptionMapper;
import javax.ws.rs.ext.Provider;

import org.jboss.jandex.AnnotationInstance;
import org.jboss.jandex.AnnotationValue;
import org.jboss.jandex.DotName;

import io.quarkus.gizmo.ClassCreator;
import io.quarkus.gizmo.ClassOutput;
import io.quarkus.gizmo.MethodCreator;
import io.quarkus.gizmo.MethodDescriptor;
import io.quarkus.gizmo.ResultHandle;
import io.quarkus.runtime.util.HashUtil;

abstract class AbstractExceptionMapperGenerator {

    protected static final DotName RESPONSE_STATUS = DotName
            .createSimple("org.springframework.web.bind.annotation.ResponseStatus");

    protected final DotName exceptionDotName;
    protected final ClassOutput classOutput;

    private final boolean isResteasyClassic;

    AbstractExceptionMapperGenerator(DotName exceptionDotName, ClassOutput classOutput, boolean isResteasyClassic) {
        this.exceptionDotName = exceptionDotName;
        this.classOutput = classOutput;
        this.isResteasyClassic = isResteasyClassic;
    }

    abstract void generateMethodBody(MethodCreator toResponse);

    String generate() {
        String generatedClassName = "io.quarkus.spring.web.mappers." + exceptionDotName.withoutPackagePrefix() + "_Mapper_"
                + HashUtil.sha1(exceptionDotName.toString());
        String exceptionClassName = exceptionDotName.toString();

        try (ClassCreator cc = ClassCreator.builder()
                .classOutput(classOutput).className(generatedClassName)
                .interfaces(ExceptionMapper.class)
                .signature(String.format("Ljava/lang/Object;Ljavax/ws/rs/ext/ExceptionMapper<L%s;>;",
                        exceptionClassName.replace('.', '/')))
                .build()) {

            preGenerateMethodBody(cc);

            try (MethodCreator toResponse = cc.getMethodCreator("toResponse", Response.class.getName(), exceptionClassName)) {
                generateMethodBody(toResponse);
            }

            // bridge method
            try (MethodCreator bridgeToResponse = cc.getMethodCreator("toResponse", Response.class, Throwable.class)) {
                MethodDescriptor toResponse = MethodDescriptor.ofMethod(generatedClassName, "toResponse",
                        Response.class.getName(), exceptionClassName);
                ResultHandle castedObject = bridgeToResponse.checkCast(bridgeToResponse.getMethodParam(0), exceptionClassName);
                ResultHandle result = bridgeToResponse.invokeVirtualMethod(toResponse, bridgeToResponse.getThis(),
                        castedObject);
                bridgeToResponse.returnValue(result);
            }
        }

        if (isResteasyClassic) {
            String generatedSubtypeClassName = "io.quarkus.spring.web.mappers.Subtype" + exceptionDotName.withoutPackagePrefix()
                    + "Mapper_" + HashUtil.sha1(exceptionDotName.toString());
            // additionally generate a dummy subtype to get past the RESTEasy's ExceptionMapper check for synthetic classes
            try (ClassCreator cc = ClassCreator.builder()
                    .classOutput(classOutput).className(generatedSubtypeClassName)
                    .superClass(generatedClassName)
                    .build()) {
                cc.addAnnotation(Provider.class);
            }

            return generatedSubtypeClassName;
        }
        return generatedClassName;
    }

    protected void preGenerateMethodBody(ClassCreator cc) {

    }

    protected int getHttpStatusFromAnnotation(AnnotationInstance responseStatusInstance) {
        AnnotationValue code = responseStatusInstance.value("code");
        if (code != null) {
            return enumValueToHttpStatus(code.asString());
        }

        AnnotationValue value = responseStatusInstance.value();
        if (value != null) {
            return enumValueToHttpStatus(value.asString());
        }

        return 500; // the default value of @ResponseStatus
    }

    @SuppressWarnings({ "rawtypes", "unchecked" })
    private int enumValueToHttpStatus(String enumValue) {
        try {
            Class<?> httpStatusClass = Class.forName("org.springframework.http.HttpStatus");
            Enum correspondingEnum = Enum.valueOf((Class<Enum>) httpStatusClass, enumValue);
            Method valueMethod = httpStatusClass.getDeclaredMethod("value");
            return (int) valueMethod.invoke(correspondingEnum);
        } catch (ClassNotFoundException e) {
            throw new RuntimeException("No spring web dependency found on the build classpath");
        } catch (NoSuchMethodException | IllegalAccessException | InvocationTargetException e) {
            throw new RuntimeException(e);
        }
    }
}"#;

#[test]
fn test() {
    let _ = PACKAGE_CASE_0;
}

enum D {
    F(&'static str),
    D(&'static [(&'static str, &'static D)]),
}

// TODO make a case where there is a late resolve (in dir/package) to check if decls uses fully qual types
static PACKAGE_CASE_0: D = D::D(&[(
    "q",
    &D::D(&[(
        "w",
        &D::D(&[
            (
                "A.java",
                &D::F(
                    "package q.w;
class A {
    static class BnM extends Node {
        int[] optoSft;
    }
    static final class BnMS extends BnM {
        boolean match() {
            optoSft[j];
        }
    }
}",
                ),
            ),
            (
                "Node.java",
                &D::F(
                    "package q.w;
class Node {}",
                ),
            ),
        ]),
    )]),
)]);

static A:&'static str = "
package java.lang;

import java.lang.annotation.Native;
import java.math.*;
import java.util.Objects;
import jdk.internal.HotSpotIntrinsicCandidate;

import static java.lang.String.COMPACT_STRINGS;
import static java.lang.String.LATIN1;
import static java.lang.String.UTF16;


public final class Long extends Number implements Comparable<Long> {

    @Native public static final long MIN_VALUE = 0x8000000000000000L;


    @Native public static final long MAX_VALUE = 0x7fffffffffffffffL;


    @SuppressWarnings(\"unchecked\")
    public static final Class<Long>     TYPE = (Class<Long>) Class.getPrimitiveClass(\"long\");


    public static String toString(long i, int radix) {
        if (radix < Character.MIN_RADIX || radix > Character.MAX_RADIX)
            radix = 10;
        if (radix == 10)
            return toString(i);

        if (COMPACT_STRINGS) {
            byte[] buf = new byte[65];
            int charPos = 64;
            boolean negative = (i < 0);

            if (!negative) {
                i = -i;
            }

            while (i <= -radix) {
                buf[charPos--] = (byte)Integer.digits[(int)(-(i % radix))];
                i = i / radix;
            }
            buf[charPos] = (byte)Integer.digits[(int)(-i)];

            if (negative) {
                buf[--charPos] = '-';
            }
            return StringLatin1.newString(buf, charPos, (65 - charPos));
        }
        return toStringUTF16(i, radix);
    }

    private static String toStringUTF16(long i, int radix) {
        byte[] buf = new byte[65 * 2];
        int charPos = 64;
        boolean negative = (i < 0);
        if (!negative) {
            i = -i;
        }
        while (i <= -radix) {
            StringUTF16.putChar(buf, charPos--, Integer.digits[(int)(-(i % radix))]);
            i = i / radix;
        }
        StringUTF16.putChar(buf, charPos, Integer.digits[(int)(-i)]);
        if (negative) {
            StringUTF16.putChar(buf, --charPos, '-');
        }
        return StringUTF16.newString(buf, charPos, (65 - charPos));
    }


    public static String toUnsignedString(long i, int radix) {
        if (i >= 0)
            return toString(i, radix);
        else {
            switch (radix) {
            case 2:
                return toBinaryString(i);

            case 4:
                return toUnsignedString0(i, 2);

            case 8:
                return toOctalString(i);

            case 10:

                long quot = (i >>> 1) / 5;
                long rem = i - quot * 10;
                return toString(quot) + rem;

            case 16:
                return toHexString(i);

            case 32:
                return toUnsignedString0(i, 5);

            default:
                return toUnsignedBigInteger(i).toString(radix);
            }
        }
    }


    private static BigInteger toUnsignedBigInteger(long i) {
        if (i >= 0L)
            return BigInteger.valueOf(i);
        else {
            int upper = (int) (i >>> 32);
            int lower = (int) i;

            // return (upper << 32) + lower
            return (BigInteger.valueOf(Integer.toUnsignedLong(upper))).shiftLeft(32).
                add(BigInteger.valueOf(Integer.toUnsignedLong(lower)));
        }
    }


    public static String toHexString(long i) {
        return toUnsignedString0(i, 4);
    }


    public static String toOctalString(long i) {
        return toUnsignedString0(i, 3);
    }


    public static String toBinaryString(long i) {
        return toUnsignedString0(i, 1);
    }


    static String toUnsignedString0(long val, int shift) {
        // assert shift > 0 && shift <=5 : \"Illegal shift value\";
        int mag = Long.SIZE - Long.numberOfLeadingZeros(val);
        int chars = Math.max(((mag + (shift - 1)) / shift), 1);
        if (COMPACT_STRINGS) {
            byte[] buf = new byte[chars];
            formatUnsignedLong0(val, shift, buf, 0, chars);
            return new String(buf, LATIN1);
        } else {
            byte[] buf = new byte[chars * 2];
            formatUnsignedLong0UTF16(val, shift, buf, 0, chars);
            return new String(buf, UTF16);
        }
    }



    /** byte[]/LATIN1 version    */
    static void formatUnsignedLong0(long val, int shift, byte[] buf, int offset, int len) {
        int charPos = offset + len;
        int radix = 1 << shift;
        int mask = radix - 1;
        do {
            buf[--charPos] = (byte)Integer.digits[((int) val) & mask];
            val >>>= shift;
        } while (charPos > offset);
    }

    /** byte[]/UTF16 version    */
    private static void formatUnsignedLong0UTF16(long val, int shift, byte[] buf, int offset, int len) {
        int charPos = offset + len;
        int radix = 1 << shift;
        int mask = radix - 1;
        do {
            StringUTF16.putChar(buf, --charPos, Integer.digits[((int) val) & mask]);
            val >>>= shift;
        } while (charPos > offset);
    }

    static String fastUUID(long lsb, long msb) {
        if (COMPACT_STRINGS) {
            byte[] buf = new byte[36];
            formatUnsignedLong0(lsb,        4, buf, 24, 12);
            formatUnsignedLong0(lsb >>> 48, 4, buf, 19, 4);
            formatUnsignedLong0(msb,        4, buf, 14, 4);
            formatUnsignedLong0(msb >>> 16, 4, buf, 9,  4);
            formatUnsignedLong0(msb >>> 32, 4, buf, 0,  8);

            buf[23] = '-';
            buf[18] = '-';
            buf[13] = '-';
            buf[8]  = '-';

            return new String(buf, LATIN1);
        } else {
            byte[] buf = new byte[72];

            formatUnsignedLong0UTF16(lsb,        4, buf, 24, 12);
            formatUnsignedLong0UTF16(lsb >>> 48, 4, buf, 19, 4);
            formatUnsignedLong0UTF16(msb,        4, buf, 14, 4);
            formatUnsignedLong0UTF16(msb >>> 16, 4, buf, 9,  4);
            formatUnsignedLong0UTF16(msb >>> 32, 4, buf, 0,  8);

            StringUTF16.putChar(buf, 23, '-');
            StringUTF16.putChar(buf, 18, '-');
            StringUTF16.putChar(buf, 13, '-');
            StringUTF16.putChar(buf,  8, '-');

            return new String(buf, UTF16);
        }
    }


    public static String toString(long i) {
        int size = stringSize(i);
        if (COMPACT_STRINGS) {
            byte[] buf = new byte[size];
            getChars(i, size, buf);
            return new String(buf, LATIN1);
        } else {
            byte[] buf = new byte[size * 2];
            StringUTF16.getChars(i, size, buf);
            return new String(buf, UTF16);
        }
    }


    public static String toUnsignedString(long i) {
        return toUnsignedString(i, 10);
    }


    static int getChars(long i, int index, byte[] buf) {
        long q;
        int r;
        int charPos = index;

        boolean negative = (i < 0);
        if (!negative) {
            i = -i;
        }

        // Get 2 digits/iteration using longs until quotient fits into an int
        while (i <= Integer.MIN_VALUE) {
            q = i / 100;
            r = (int)((q * 100) - i);
            i = q;
            buf[--charPos] = Integer.DigitOnes[r];
            buf[--charPos] = Integer.DigitTens[r];
        }

        // Get 2 digits/iteration using ints
        int q2;
        int i2 = (int)i;
        while (i2 <= -100) {
            q2 = i2 / 100;
            r  = (q2 * 100) - i2;
            i2 = q2;
            buf[--charPos] = Integer.DigitOnes[r];
            buf[--charPos] = Integer.DigitTens[r];
        }

        // We know there are at most two digits left at this point.
        q2 = i2 / 10;
        r  = (q2 * 10) - i2;
        buf[--charPos] = (byte)('0' + r);

        // Whatever left is the remaining digit.
        if (q2 < 0) {
            buf[--charPos] = (byte)('0' - q2);
        }

        if (negative) {
            buf[--charPos] = (byte)'-';
        }
        return charPos;
    }


    static int stringSize(long x) {
        int d = 1;
        if (x >= 0) {
            d = 0;
            x = -x;
        }
        long p = -10;
        for (int i = 1; i < 19; i++) {
            if (x > p)
                return i + d;
            p = 10 * p;
        }
        return 19 + d;
    }


    public static long parseLong(String s, int radix)
              throws NumberFormatException
    {
        if (s == null) {
            throw new NumberFormatException(\"null\");
        }

        if (radix < Character.MIN_RADIX) {
            throw new NumberFormatException(\"radix \" + radix +
                                            \" less than Character.MIN_RADIX\");
        }
        if (radix > Character.MAX_RADIX) {
            throw new NumberFormatException(\"radix \" + radix +
                                            \" greater than Character.MAX_RADIX\");
        }

        boolean negative = false;
        int i = 0, len = s.length();
        long limit = -Long.MAX_VALUE;

        if (len > 0) {
            char firstChar = s.charAt(0);
            if (firstChar < '0') { // Possible leading \"+\" or \"-\"
                if (firstChar == '-') {
                    negative = true;
                    limit = Long.MIN_VALUE;
                } else if (firstChar != '+') {
                    throw NumberFormatException.forInputString(s);
                }

                if (len == 1) { // Cannot have lone \"+\" or \"-\"
                    throw NumberFormatException.forInputString(s);
                }
                i++;
            }
            long multmin = limit / radix;
            long result = 0;
            while (i < len) {
                // Accumulating negatively avoids surprises near MAX_VALUE
                int digit = Character.digit(s.charAt(i++),radix);
                if (digit < 0 || result < multmin) {
                    throw NumberFormatException.forInputString(s);
                }
                result *= radix;
                if (result < limit + digit) {
                    throw NumberFormatException.forInputString(s);
                }
                result -= digit;
            }
            return negative ? result : -result;
        } else {
            throw NumberFormatException.forInputString(s);
        }
    }


    public static long parseLong(CharSequence s, int beginIndex, int endIndex, int radix)
                throws NumberFormatException {
        s = Objects.requireNonNull(s);

        if (beginIndex < 0 || beginIndex > endIndex || endIndex > s.length()) {
            throw new IndexOutOfBoundsException();
        }
        if (radix < Character.MIN_RADIX) {
            throw new NumberFormatException(\"radix \" + radix +
                    \" less than Character.MIN_RADIX\");
        }
        if (radix > Character.MAX_RADIX) {
            throw new NumberFormatException(\"radix \" + radix +
                    \" greater than Character.MAX_RADIX\");
        }

        boolean negative = false;
        int i = beginIndex;
        long limit = -Long.MAX_VALUE;

        if (i < endIndex) {
            char firstChar = s.charAt(i);
            if (firstChar < '0') { // Possible leading \"+\" or \"-\"
                if (firstChar == '-') {
                    negative = true;
                    limit = Long.MIN_VALUE;
                } else if (firstChar != '+') {
                    throw NumberFormatException.forCharSequence(s, beginIndex,
                            endIndex, i);
                }
                i++;
            }
            if (i >= endIndex) { // Cannot have lone \"+\", \"-\" or \"\"
                throw NumberFormatException.forCharSequence(s, beginIndex,
                        endIndex, i);
            }
            long multmin = limit / radix;
            long result = 0;
            while (i < endIndex) {
                // Accumulating negatively avoids surprises near MAX_VALUE
                int digit = Character.digit(s.charAt(i), radix);
                if (digit < 0 || result < multmin) {
                    throw NumberFormatException.forCharSequence(s, beginIndex,
                            endIndex, i);
                }
                result *= radix;
                if (result < limit + digit) {
                    throw NumberFormatException.forCharSequence(s, beginIndex,
                            endIndex, i);
                }
                i++;
                result -= digit;
            }
            return negative ? result : -result;
        } else {
            throw new NumberFormatException(\"\");
        }
    }


    public static long parseLong(String s) throws NumberFormatException {
        return parseLong(s, 10);
    }


    public static long parseUnsignedLong(String s, int radix)
                throws NumberFormatException {
        if (s == null)  {
            throw new NumberFormatException(\"null\");
        }

        int len = s.length();
        if (len > 0) {
            char firstChar = s.charAt(0);
            if (firstChar == '-') {
                throw new
                    NumberFormatException(String.format(\"Illegal leading minus sign \" +
                                                       \"on unsigned string %s.\", s));
            } else {
                if (len <= 12 || // Long.MAX_VALUE in Character.MAX_RADIX is 13 digits
                    (radix == 10 && len <= 18) ) { // Long.MAX_VALUE in base 10 is 19 digits
                    return parseLong(s, radix);
                }

                // No need for range checks on len due to testing above.
                long first = parseLong(s, 0, len - 1, radix);
                int second = Character.digit(s.charAt(len - 1), radix);
                if (second < 0) {
                    throw new NumberFormatException(\"Bad digit at end of \" + s);
                }
                long result = first * radix + second;


                int guard = radix * (int) (first >>> 57);
                if (guard >= 128 ||
                    (result >= 0 && guard >= 128 - Character.MAX_RADIX)) {

                    throw new NumberFormatException(String.format(\"String value %s exceeds \" +
                                                                  \"range of unsigned long.\", s));
                }
                return result;
            }
        } else {
            throw NumberFormatException.forInputString(s);
        }
    }


    public static long parseUnsignedLong(CharSequence s, int beginIndex, int endIndex, int radix)
                throws NumberFormatException {
        s = Objects.requireNonNull(s);

        if (beginIndex < 0 || beginIndex > endIndex || endIndex > s.length()) {
            throw new IndexOutOfBoundsException();
        }
        int start = beginIndex, len = endIndex - beginIndex;

        if (len > 0) {
            char firstChar = s.charAt(start);
            if (firstChar == '-') {
                throw new NumberFormatException(String.format(\"Illegal leading minus sign \" +
                        \"on unsigned string %s.\", s.subSequence(start, start + len)));
            } else {
                if (len <= 12 || // Long.MAX_VALUE in Character.MAX_RADIX is 13 digits
                    (radix == 10 && len <= 18) ) { // Long.MAX_VALUE in base 10 is 19 digits
                    return parseLong(s, start, start + len, radix);
                }

                // No need for range checks on end due to testing above.
                long first = parseLong(s, start, start + len - 1, radix);
                int second = Character.digit(s.charAt(start + len - 1), radix);
                if (second < 0) {
                    throw new NumberFormatException(\"Bad digit at end of \" +
                            s.subSequence(start, start + len));
                }
                long result = first * radix + second;


                int guard = radix * (int) (first >>> 57);
                if (guard >= 128 ||
                        (result >= 0 && guard >= 128 - Character.MAX_RADIX)) {

                    throw new NumberFormatException(String.format(\"String value %s exceeds \" +
                            \"range of unsigned long.\", s.subSequence(start, start + len)));
                }
                return result;
            }
        } else {
            throw NumberFormatException.forInputString(\"\");
        }
    }


    public static long parseUnsignedLong(String s) throws NumberFormatException {
        return parseUnsignedLong(s, 10);
    }


    public static Long valueOf(String s, int radix) throws NumberFormatException {
        return Long.valueOf(parseLong(s, radix));
    }


    public static Long valueOf(String s) throws NumberFormatException
    {
        return Long.valueOf(parseLong(s, 10));
    }

    private static class LongCache {
        private LongCache(){}

        static final Long cache[] = new Long[-(-128) + 127 + 1];

        static {
            for(int i = 0; i < cache.length; i++)
                cache[i] = new Long(i - 128);
        }
    }


    @HotSpotIntrinsicCandidate
    public static Long valueOf(long l) {
        final int offset = 128;
        if (l >= -128 && l <= 127) { // will cache
            return LongCache.cache[(int)l + offset];
        }
        return new Long(l);
    }


    public static Long decode(String nm) throws NumberFormatException {
        int radix = 10;
        int index = 0;
        boolean negative = false;
        Long result;

        if (nm.length() == 0)
            throw new NumberFormatException(\"Zero length string\");
        char firstChar = nm.charAt(0);
        // Handle sign, if present
        if (firstChar == '-') {
            negative = true;
            index++;
        } else if (firstChar == '+')
            index++;

        // Handle radix specifier, if present
        if (nm.startsWith(\"0x\", index) || nm.startsWith(\"0X\", index)) {
            index += 2;
            radix = 16;
        }
        else if (nm.startsWith(\"#\", index)) {
            index ++;
            radix = 16;
        }
        else if (nm.startsWith(\"0\", index) && nm.length() > 1 + index) {
            index ++;
            radix = 8;
        }

        if (nm.startsWith(\"-\", index) || nm.startsWith(\"+\", index))
            throw new NumberFormatException(\"Sign character in wrong position\");

        try {
            result = Long.valueOf(nm.substring(index), radix);
            result = negative ? Long.valueOf(-result.longValue()) : result;
        } catch (NumberFormatException e) {
            // If number is Long.MIN_VALUE, we'll end up here. The next line
            // handles this case, and causes any genuine format error to be
            // rethrown.
            String constant = negative ? (\"-\" + nm.substring(index))
                                       : nm.substring(index);
            result = Long.valueOf(constant, radix);
        }
        return result;
    }


    private final long value;


    @Deprecated(since=\"9\")
    public Long(long value) {
        this.value = value;
    }


    @Deprecated(since=\"9\")
    public Long(String s) throws NumberFormatException {
        this.value = parseLong(s, 10);
    }


    public byte byteValue() {
        return (byte)value;
    }


    public short shortValue() {
        return (short)value;
    }


    public int intValue() {
        return (int)value;
    }


    @HotSpotIntrinsicCandidate
    public long longValue() {
        return value;
    }


    public float floatValue() {
        return (float)value;
    }


    public double doubleValue() {
        return (double)value;
    }


    public String toString() {
        return toString(value);
    }


    @Override
    public int hashCode() {
        return Long.hashCode(value);
    }


    public static int hashCode(long value) {
        return (int)(value ^ (value >>> 32));
    }


    public boolean equals(Object obj) {
        if (obj instanceof Long) {
            return value == ((Long)obj).longValue();
        }
        return false;
    }


    public static Long getLong(String nm) {
        return getLong(nm, null);
    }


    public static Long getLong(String nm, long val) {
        Long result = Long.getLong(nm, null);
        return (result == null) ? Long.valueOf(val) : result;
    }


    public static Long getLong(String nm, Long val) {
        String v = null;
        try {
            v = System.getProperty(nm);
        } catch (IllegalArgumentException | NullPointerException e) {
        }
        if (v != null) {
            try {
                return Long.decode(v);
            } catch (NumberFormatException e) {
            }
        }
        return val;
    }


    public int compareTo(Long anotherLong) {
        return compare(this.value, anotherLong.value);
    }


    public static int compare(long x, long y) {
        return (x < y) ? -1 : ((x == y) ? 0 : 1);
    }


    public static int compareUnsigned(long x, long y) {
        return compare(x + MIN_VALUE, y + MIN_VALUE);
    }



    public static long divideUnsigned(long dividend, long divisor) {
        if (divisor < 0L) { // signed comparison
            // Answer must be 0 or 1 depending on relative magnitude
            // of dividend and divisor.
            return (compareUnsigned(dividend, divisor)) < 0 ? 0L :1L;
        }

        if (dividend > 0) //  Both inputs non-negative
            return dividend/divisor;
        else {

            return toUnsignedBigInteger(dividend).
                divide(toUnsignedBigInteger(divisor)).longValue();
        }
    }


    public static long remainderUnsigned(long dividend, long divisor) {
        if (dividend > 0 && divisor > 0) { // signed comparisons
            return dividend % divisor;
        } else {
            if (compareUnsigned(dividend, divisor) < 0) // Avoid explicit check for 0 divisor
                return dividend;
            else
                return toUnsignedBigInteger(dividend).
                    remainder(toUnsignedBigInteger(divisor)).longValue();
        }
    }

    // Bit Twiddling


    @Native public static final int SIZE = 64;


    public static final int BYTES = SIZE / Byte.SIZE;


    public static long highestOneBit(long i) {
        return i & (MIN_VALUE >>> numberOfLeadingZeros(i));
    }


    public static long lowestOneBit(long i) {
        // HD, Section 2-1
        return i & -i;
    }


    @HotSpotIntrinsicCandidate
    public static int numberOfLeadingZeros(long i) {
        // HD, Figure 5-6
         if (i <= 0)
            return i == 0 ? 64 : 0;
        int n = 1;
        int x = (int)(i >>> 32);
        if (x == 0) { n += 32; x = (int)i; }
        if (x >>> 16 == 0) { n += 16; x <<= 16; }
        if (x >>> 24 == 0) { n +=  8; x <<=  8; }
        if (x >>> 28 == 0) { n +=  4; x <<=  4; }
        if (x >>> 30 == 0) { n +=  2; x <<=  2; }
        n -= x >>> 31;
        return n;
    }


    @HotSpotIntrinsicCandidate
    public static int numberOfTrailingZeros(long i) {
        // HD, Figure 5-14
        int x, y;
        if (i == 0) return 64;
        int n = 63;
        y = (int)i; if (y != 0) { n = n -32; x = y; } else x = (int)(i>>>32);
        y = x <<16; if (y != 0) { n = n -16; x = y; }
        y = x << 8; if (y != 0) { n = n - 8; x = y; }
        y = x << 4; if (y != 0) { n = n - 4; x = y; }
        y = x << 2; if (y != 0) { n = n - 2; x = y; }
        return n - ((x << 1) >>> 31);
    }


     @HotSpotIntrinsicCandidate
     public static int bitCount(long i) {
        // HD, Figure 5-2
        i = i - ((i >>> 1) & 0x5555555555555555L);
        i = (i & 0x3333333333333333L) + ((i >>> 2) & 0x3333333333333333L);
        i = (i + (i >>> 4)) & 0x0f0f0f0f0f0f0f0fL;
        i = i + (i >>> 8);
        i = i + (i >>> 16);
        i = i + (i >>> 32);
        return (int)i & 0x7f;
     }


    public static long rotateLeft(long i, int distance) {
        return (i << distance) | (i >>> -distance);
    }


    public static long rotateRight(long i, int distance) {
        return (i >>> distance) | (i << -distance);
    }


    public static long reverse(long i) {
        // HD, Figure 7-1
        i = (i & 0x5555555555555555L) << 1 | (i >>> 1) & 0x5555555555555555L;
        i = (i & 0x3333333333333333L) << 2 | (i >>> 2) & 0x3333333333333333L;
        i = (i & 0x0f0f0f0f0f0f0f0fL) << 4 | (i >>> 4) & 0x0f0f0f0f0f0f0f0fL;

        return reverseBytes(i);
    }


    public static int signum(long i) {
        // HD, Section 2-7
        return (int) ((i >> 63) | (-i >>> 63));
    }


    @HotSpotIntrinsicCandidate
    public static long reverseBytes(long i) {
        i = (i & 0x00ff00ff00ff00ffL) << 8 | (i >>> 8) & 0x00ff00ff00ff00ffL;
        return (i << 48) | ((i & 0xffff0000L) << 16) |
            ((i >>> 16) & 0xffff0000L) | (i >>> 48);
    }


    public static long sum(long a, long b) {
        return a + b;
    }


    public static long max(long a, long b) {
        return Math.max(a, b);
    }


    public static long min(long a, long b) {
        return Math.min(a, b);
    }

    @Native private static final long serialVersionUID = 4290774380558885855L;
}
";