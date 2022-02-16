use core::fmt;
use std::{
    io::{stdout, Write},
    ops::Deref,
};

use pretty_assertions::assert_eq;

use rusted_gumtree_core::tree::tree::NodeStore as _;
use tree_sitter::{Language, Parser};

use crate::{
    filter::BloomResult,
    java_tree_gen::spaces_after_lb,
    java_tree_gen_full_compress_legion_ref::{
        print_tree_labels, print_tree_syntax, serialize, JavaTreeGen, LabelStore, NodeStore,
        SimpleStores,
    },
    nodes::RefContainer,
    store::TypeStore,
    tree_gen::TreeGen,
    utils::memusage_linux,
};

// use crate::java_tree_gen::{JavaTreeGen, TreeContext, TreeGenerator};

extern "C" {
    fn tree_sitter_java() -> Language;
}

fn run(text: &[u8]) {
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();
    }
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        },
    };
    let tree = parser.parse(text, None).unwrap();
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
        CASE_1, CASE_2, CASE_3, CASE_4, CASE_5, CASE_6, CASE_7, CASE_8, CASE_9, CASE_10, CASE_11,
        CASE_12, CASE_13,
    ];
    for case in cases {
        run(case.as_bytes())
    }
}

#[test]
fn test_equals() {
    let text = CASE_23.as_bytes();
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();
    }
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        },
    };
    let tree = parser.parse(text, None).unwrap();
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

    {
        // playing with refs
        let a = &full_node.local.compressed_node;

        let b = java_tree_gen.stores.node_store.resolve(*a);
        match full_node.local.ana.as_ref() {
            Some(x) => {
                println!("refs:",);
                x.print_refs(&java_tree_gen.stores.label_store);
            }
            None => println!("None"),
        };
        let bb = "B".as_bytes().to_owned().into_boxed_slice();
        let d = bb.as_ref(); //_full_node.local.refs.unwrap().iter().next().unwrap();

        let c = b.check(d);

        let s = std::str::from_utf8(d).unwrap();
        println!("{}", java_tree_gen.stores.label_store);
        match c {
            BloomResult::MaybeContain => println!("Maybe contains {}", s),
            BloomResult::DoNotContain => println!("Do not contains {}", s),
        }
    }
}

#[test]
fn test_special() {
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();
    }

    // let mut parser: Parser, old_tree: Option<&Tree>
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        },
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
    let tree = parser.parse(text, None).unwrap();
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
    let tree = parser.parse(text, None).unwrap();
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
fn test_2_spaces_after_lb() {
    let r = spaces_after_lb("\n".as_bytes(), "\n  ".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some("  ")
    )
}

#[test]
fn test_1_space_after_lb() {
    let r = spaces_after_lb("\n".as_bytes(), "\n ".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some(" ")
    )
}

#[test]
fn test_no_spaces_after_lb() {
    let r = spaces_after_lb("\n".as_bytes(), "\n".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some("")
    )
}

#[test]
fn test_spaces_after_lb_special() {
    let r = spaces_after_lb("\n\r".as_bytes(), "\n\r\t ".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some("\t ")
    )
}

/// historic regression test for static analysis
static CASE_1: &'static str = "
class A {
    char[] c = new char[] { (char) x };
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
        test(a);
        String s = \"\";
        b.test(s);
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

static CASE_11_bis: &'static str = "package a;
public class A {
    int start, len;
    public static long f() {
        A x = new A(start);
    }
}
";

// TODO handle fall through variable declaration
static CASE_12: &'static str = "package a;
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
