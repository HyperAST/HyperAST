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

#[test]
fn test_equals() {
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();
    }

    // let text = {
    //     let source_code1 = "
    //     class A {void test() {}}
    //     ";
    //     source_code1.as_bytes()
    // };
    // // let mut parser: Parser, old_tree: Option<&Tree>
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        },
    };
    // let tree = parser.parse(text, None).unwrap();
    // // let mut acc_stack = vec![Accumulator::new(java_tree_gen.stores.type_store.get("file"))];

    // let full_node = java_tree_gen.generate_default(text, tree.walk());
    // println!("{}", tree.root_node().to_sexp());
    // // print_tree_structure(&java_tree_gen.node_store, &full_node.compressed_node);
    // print_tree_labels(
    //     &java_tree_gen.stores.node_store,
    //     &java_tree_gen.stores.label_store,
    //     &full_node.local.compressed_node,
    // );
    // println!();
    // println!();
    // println!();

    // let text = {
    //     let source_code1 = "
    //     class A {

    //     }";
    //     source_code1.as_bytes()
    // };
    // let tree = parser.parse(text, None).unwrap();
    // let _full_node = java_tree_gen.generate_default(text, tree.walk());

    // let text = {
    //     let source_code1 = "
    //     class A {
    //         int a = 0xffff;
    //     }";
    //     source_code1.as_bytes()
    // };
    // let tree = parser.parse(text, None).unwrap();
    // let _full_node = java_tree_gen.generate_default(text, tree.walk());

    let text = {
        let source_code1 = "package q.w.e;
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
        // let source_code1 = B;
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_default(text, tree.walk());

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
        // playing with refs
        let a = &full_node.local.compressed_node;

        let b = java_tree_gen.stores.node_store.resolve(a);
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
        println!("{}",java_tree_gen.stores.label_store);
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

    let _full_node = java_tree_gen.generate_default(text, tree.walk());

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
    let full_node = java_tree_gen.generate_default(text, tree.walk());

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
    println!("{}",java_tree_gen.stores.label_store);

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



static B:&'static str ="
class A {
    char[] c = new char[] { (char) x };
}
";

static A:&'static str = "
/*
 * Copyright (c) 2002, 2018, Oracle and/or its affiliates. All rights reserved.
 * DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
 *
 * This code is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 2 only, as
 * published by the Free Software Foundation.  Oracle designates this
 * particular file as subject to the \"Classpath\" exception as provided
 * by Oracle in the LICENSE file that accompanied this code.
 *
 * This code is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
 * version 2 for more details (a copy is included in the LICENSE file that
 * accompanied this code).
 *
 * You should have received a copy of the GNU General Public License version
 * 2 along with this work; if not, write to the Free Software Foundation,
 * Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * Please contact Oracle, 500 Oracle Parkway, Redwood Shores, CA 94065 USA
 * or visit www.oracle.com if you need additional information or have any
 * questions.
 */

package java.lang;

import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import java.util.Locale;

import jdk.internal.HotSpotIntrinsicCandidate;

/**
 * The {@code Character} class wraps a value of the primitive
 * type {@code char} in an object. An object of type
 * {@code Character} contains a single field whose type is
 * {@code char}.
 * <p>
 * In addition, this class provides several methods for determining
 * a character's category (lowercase letter, digit, etc.) and for converting
 * characters from uppercase to lowercase and vice versa.
 * <p>
 * Character information is based on the Unicode Standard, version 10.0.0.
 * <p>
 * The methods and data of class {@code Character} are defined by
 * the information in the <i>UnicodeData</i> file that is part of the
 * Unicode Character Database maintained by the Unicode
 * Consortium. This file specifies various properties including name
 * and general category for every defined Unicode code point or
 * character range.
 * <p>
 * The file and its description are available from the Unicode Consortium at:
 * <ul>
 * <li><a href=\"http://www.unicode.org\">http://www.unicode.org</a>
 * </ul>
 *
 * <h3><a id=\"unicode\">Unicode Character Representations</a></h3>
 *
 * <p>The {@code char} data type (and therefore the value that a
 * {@code Character} object encapsulates) are based on the
 * original Unicode specification, which defined characters as
 * fixed-width 16-bit entities. The Unicode Standard has since been
 * changed to allow for characters whose representation requires more
 * than 16 bits.  The range of legal <em>code point</em>s is now
 * U+0000 to U+10FFFF, known as <em>Unicode scalar value</em>.
 * (Refer to the <a
 * href=\"http://www.unicode.org/reports/tr27/#notation\"><i>
 * definition</i></a> of the U+<i>n</i> notation in the Unicode
 * Standard.)
 *
 * <p><a id=\"BMP\">The set of characters from U+0000 to U+FFFF</a> is
 * sometimes referred to as the <em>Basic Multilingual Plane (BMP)</em>.
 * <a id=\"supplementary\">Characters</a> whose code points are greater
 * than U+FFFF are called <em>supplementary character</em>s.  The Java
 * platform uses the UTF-16 representation in {@code char} arrays and
 * in the {@code String} and {@code StringBuffer} classes. In
 * this representation, supplementary characters are represented as a pair
 * of {@code char} values, the first from the <em>high-surrogates</em>
 * range, (&#92;uD800-&#92;uDBFF), the second from the
 * <em>low-surrogates</em> range (&#92;uDC00-&#92;uDFFF).
 *
 * <p>A {@code char} value, therefore, represents Basic
 * Multilingual Plane (BMP) code points, including the surrogate
 * code points, or code units of the UTF-16 encoding. An
 * {@code int} value represents all Unicode code points,
 * including supplementary code points. The lower (least significant)
 * 21 bits of {@code int} are used to represent Unicode code
 * points and the upper (most significant) 11 bits must be zero.
 * Unless otherwise specified, the behavior with respect to
 * supplementary characters and surrogate {@code char} values is
 * as follows:
 *
 * <ul>
 * <li>The methods that only accept a {@code char} value cannot support
 * supplementary characters. They treat {@code char} values from the
 * surrogate ranges as undefined characters. For example,
 * {@code Character.isLetter('\\u005CuD840')} returns {@code false}, even though
 * this specific value if followed by any low-surrogate value in a string
 * would represent a letter.
 *
 * <li>The methods that accept an {@code int} value support all
 * Unicode characters, including supplementary characters. For
 * example, {@code Character.isLetter(0x2F81A)} returns
 * {@code true} because the code point value represents a letter
 * (a CJK ideograph).
 * </ul>
 *
 * <p>In the Java SE API documentation, <em>Unicode code point</em> is
 * used for character values in the range between U+0000 and U+10FFFF,
 * and <em>Unicode code unit</em> is used for 16-bit
 * {@code char} values that are code units of the <em>UTF-16</em>
 * encoding. For more information on Unicode terminology, refer to the
 * <a href=\"http://www.unicode.org/glossary/\">Unicode Glossary</a>.
 *
 * @author  Lee Boynton
 * @author  Guy Steele
 * @author  Akira Tanaka
 * @author  Martin Buchholz
 * @author  Ulf Zibis
 * @since   1.0
 */
public final
class Character implements java.io.Serializable, Comparable<Character> {


    /**
     *
     *
     */
    public static boolean isLowerCase(char ch) {
        return isLowerCase((int)ch);
    }

    /**
     *
     */
    public static boolean isLowerCase(int codePoint) {
        return getType(codePoint) == Character.LOWERCASE_LETTER ||
               CharacterData.of(codePoint).isOtherLowercase(codePoint);
    }

    /**
     *
     *
     */
    public static boolean isUpperCase(char ch) {
        return isUpperCase((int)ch);
    }

    /**
     *
     */
    public static boolean isUpperCase(int codePoint) {
        return getType(codePoint) == Character.UPPERCASE_LETTER ||
               CharacterData.of(codePoint).isOtherUppercase(codePoint);
    }

    /**
     *
     *
     */
    public static boolean isTitleCase(char ch) {
        return isTitleCase((int)ch);
    }

    /**
     *
     */
    public static boolean isTitleCase(int codePoint) {
        return getType(codePoint) == Character.TITLECASE_LETTER;
    }

    /**
     *
     *
     *
     */
    public static boolean isDigit(char ch) {
        return isDigit((int)ch);
    }

    /**
     *
     *
     */
    public static boolean isDigit(int codePoint) {
        return getType(codePoint) == Character.DECIMAL_DIGIT_NUMBER;
    }

    /**
     *
     *
     */
    public static boolean isDefined(char ch) {
        return isDefined((int)ch);
    }

    /**
     *
     */
    public static boolean isDefined(int codePoint) {
        return getType(codePoint) != Character.UNASSIGNED;
    }

    /**
     *
     *
     *
     */
    public static boolean isLetter(char ch) {
        return isLetter((int)ch);
    }

    /**
     *
     *
     */
    public static boolean isLetter(int codePoint) {
        return ((((1 << Character.UPPERCASE_LETTER) |
            (1 << Character.LOWERCASE_LETTER) |
            (1 << Character.TITLECASE_LETTER) |
            (1 << Character.MODIFIER_LETTER) |
            (1 << Character.OTHER_LETTER)) >> getType(codePoint)) & 1)
            != 0;
    }

    /**
     *
     *
     */
    public static boolean isLetterOrDigit(char ch) {
        return isLetterOrDigit((int)ch);
    }

    /**
     *
     */
    public static boolean isLetterOrDigit(int codePoint) {
        return ((((1 << Character.UPPERCASE_LETTER) |
            (1 << Character.LOWERCASE_LETTER) |
            (1 << Character.TITLECASE_LETTER) |
            (1 << Character.MODIFIER_LETTER) |
            (1 << Character.OTHER_LETTER) |
            (1 << Character.DECIMAL_DIGIT_NUMBER)) >> getType(codePoint)) & 1)
            != 0;
    }

    /**
     *
     */
    @Deprecated(since=\"1.1\")
    public static boolean isJavaLetter(char ch) {
        return isJavaIdentifierStart(ch);
    }

    /**
     *
     */
    @Deprecated(since=\"1.1\")
    public static boolean isJavaLetterOrDigit(char ch) {
        return isJavaIdentifierPart(ch);
    }

    /**
     *
     */
    public static boolean isAlphabetic(int codePoint) {
        return (((((1 << Character.UPPERCASE_LETTER) |
            (1 << Character.LOWERCASE_LETTER) |
            (1 << Character.TITLECASE_LETTER) |
            (1 << Character.MODIFIER_LETTER) |
            (1 << Character.OTHER_LETTER) |
            (1 << Character.LETTER_NUMBER)) >> getType(codePoint)) & 1) != 0) ||
            CharacterData.of(codePoint).isOtherAlphabetic(codePoint);
    }

    /**
     *
     */
    public static boolean isIdeographic(int codePoint) {
        return CharacterData.of(codePoint).isIdeographic(codePoint);
    }

    /**
     *
     *
     */
    public static boolean isJavaIdentifierStart(char ch) {
        return isJavaIdentifierStart((int)ch);
    }

    /**
     *
     */
    public static boolean isJavaIdentifierStart(int codePoint) {
        return CharacterData.of(codePoint).isJavaIdentifierStart(codePoint);
    }

    /**
     *
     *
     */
    public static boolean isJavaIdentifierPart(char ch) {
        return isJavaIdentifierPart((int)ch);
    }

    /**
     *
     */
    public static boolean isJavaIdentifierPart(int codePoint) {
        return CharacterData.of(codePoint).isJavaIdentifierPart(codePoint);
    }

    /**
     *
     *
     */
    public static boolean isUnicodeIdentifierStart(char ch) {
        return isUnicodeIdentifierStart((int)ch);
    }

    /**
     */
    public static boolean isUnicodeIdentifierStart(int codePoint) {
        return CharacterData.of(codePoint).isUnicodeIdentifierStart(codePoint);
    }

    /**
     *
     *
     */
    public static boolean isUnicodeIdentifierPart(char ch) {
        return isUnicodeIdentifierPart((int)ch);
    }

    /**
     */
    public static boolean isUnicodeIdentifierPart(int codePoint) {
        return CharacterData.of(codePoint).isUnicodeIdentifierPart(codePoint);
    }

    /**
     *
     *
     *
     */
    public static boolean isIdentifierIgnorable(char ch) {
        return isIdentifierIgnorable((int)ch);
    }

    /**
     *
     *
     */
    public static boolean isIdentifierIgnorable(int codePoint) {
        return CharacterData.of(codePoint).isIdentifierIgnorable(codePoint);
    }

    /**
     *
     *
     *
     */
    public static char toLowerCase(char ch) {
        return (char)toLowerCase((int)ch);
    }

    /**
     *
     *
     *
     *
     */
    public static int toLowerCase(int codePoint) {
        return CharacterData.of(codePoint).toLowerCase(codePoint);
    }

    /**
     *
     *
     *
     */
    public static char toUpperCase(char ch) {
        return (char)toUpperCase((int)ch);
    }

    /**
     *
     *
     *
     *
     */
    public static int toUpperCase(int codePoint) {
        return CharacterData.of(codePoint).toUpperCase(codePoint);
    }

    /**
     *
     *
     */
    public static char toTitleCase(char ch) {
        return (char)toTitleCase((int)ch);
    }

    /**
     *
     *
     */
    public static int toTitleCase(int codePoint) {
        return CharacterData.of(codePoint).toTitleCase(codePoint);
    }

    /**
     *
     *
     */
    public static int digit(char ch, int radix) {
        return digit((int)ch, radix);
    }

    /**
     *
     *
     */
    public static int digit(int codePoint, int radix) {
        return CharacterData.of(codePoint).digit(codePoint, radix);
    }

    /**
     *
     *
     */
    public static int getNumericValue(char ch) {
        return getNumericValue((int)ch);
    }

    /**
     *
     */
    public static int getNumericValue(int codePoint) {
        return CharacterData.of(codePoint).getNumericValue(codePoint);
    }

    /**
     *
     */
    @Deprecated(since=\"1.1\")
    public static boolean isSpace(char ch) {
        return (ch <= 0x0020) &&
            (((((1L << 0x0009) |
            (1L << 0x000A) |
            (1L << 0x000C) |
            (1L << 0x000D) |
            (1L << 0x0020)) >> ch) & 1L) != 0);
    }


    /**
     *
     *
     */
    public static boolean isSpaceChar(char ch) {
        return isSpaceChar((int)ch);
    }

    /**
     *
     *
     */
    public static boolean isSpaceChar(int codePoint) {
        return ((((1 << Character.SPACE_SEPARATOR) |
                  (1 << Character.LINE_SEPARATOR) |
                  (1 << Character.PARAGRAPH_SEPARATOR)) >> getType(codePoint)) & 1)
            != 0;
    }

    /**
     *
     *
     */
    public static boolean isWhitespace(char ch) {
        return isWhitespace((int)ch);
    }

    /**
     *
     */
    public static boolean isWhitespace(int codePoint) {
        return CharacterData.of(codePoint).isWhitespace(codePoint);
    }

    /**
     *
     *
     *
     */
    public static boolean isISOControl(char ch) {
        return isISOControl((int)ch);
    }

    /**
     *
     */
    public static boolean isISOControl(int codePoint) {
        // Optimized form of:
        //     (codePoint >= 0x00 && codePoint <= 0x1F) ||
        //     (codePoint >= 0x7F && codePoint <= 0x9F);
        return codePoint <= 0x9F &&
            (codePoint >= 0x7F || (codePoint >>> 5 == 0));
    }

    /**
     *
     *
     */
    public static int getType(char ch) {
        return getType((int)ch);
    }

    /**
     *
     */
    public static int getType(int codePoint) {
        return CharacterData.of(codePoint).getType(codePoint);
    }

    /**
     *
     */
    public static char forDigit(int digit, int radix) {
        if ((digit >= radix) || (digit < 0)) {
            return '\\0';
        }
        if ((radix < Character.MIN_RADIX) || (radix > Character.MAX_RADIX)) {
            return '\\0';
        }
        if (digit < 10) {
            return (char)('0' + digit);
        }
        return (char)('a' - 10 + digit);
    }

    /**
     *
     *
     *
     */
    public static byte getDirectionality(char ch) {
        return getDirectionality((int)ch);
    }

    /**
     *
     *
     */
    public static byte getDirectionality(int codePoint) {
        return CharacterData.of(codePoint).getDirectionality(codePoint);
    }

    /**
     *
     *
     */
    public static boolean isMirrored(char ch) {
        return isMirrored((int)ch);
    }

    /**
     *
     */
    public static boolean isMirrored(int codePoint) {
        return CharacterData.of(codePoint).isMirrored(codePoint);
    }

    /**
     *

     */
    public int compareTo(Character anotherCharacter) {
        return compare(this.value, anotherCharacter.value);
    }

    /**
     *
     */
    public static int compare(char x, char y) {
        return x - y;
    }

    /**
     *
     */
    static int toUpperCaseEx(int codePoint) {
        assert isValidCodePoint(codePoint);
        return CharacterData.of(codePoint).toUpperCaseEx(codePoint);
    }

    /**
     *
     */
    static char[] toUpperCaseCharArray(int codePoint) {
        // As of Unicode 6.0, 1:M uppercasings only happen in the BMP.
        assert isBmpCodePoint(codePoint);
        return CharacterData.of(codePoint).toUpperCaseCharArray(codePoint);
    }

    /**
     *
     */
    public static final int SIZE = 16;

    /**
     *
     */
    public static final int BYTES = SIZE / Byte.SIZE;

    /**
     *
     */
    @HotSpotIntrinsicCandidate
    public static char reverseBytes(char ch) {
        return (char) (((ch & 0xFF00) >> 8) | (ch << 8));
    }

    /**
     *
     *
     *
     *
     *
     *
     */
    public static String getName(int codePoint) {
        if (!isValidCodePoint(codePoint)) {
            throw new IllegalArgumentException(
                String.format(\"Not a valid Unicode code point: 0x%X\", codePoint));
        }
        String name = CharacterName.getInstance().getName(codePoint);
        if (name != null)
            return name;
        if (getType(codePoint) == UNASSIGNED)
            return null;
        UnicodeBlock block = UnicodeBlock.of(codePoint);
        if (block != null)
            return block.toString().replace('_', ' ') + \" \"
                   + Integer.toHexString(codePoint).toUpperCase(Locale.ROOT);
        // should never come here
        return Integer.toHexString(codePoint).toUpperCase(Locale.ROOT);
    }

    /**
     *
     *
     *
     *
     *
     *
     */
    public static int codePointOf(String name) {
        name = name.trim().toUpperCase(Locale.ROOT);
        int cp = CharacterName.getInstance().getCodePoint(name);
        if (cp != -1)
            return cp;
        try {
            int off = name.lastIndexOf(' ');
            if (off != -1) {
                cp = Integer.parseInt(name, off + 1, name.length(), 16);
                if (isValidCodePoint(cp) && name.equals(getName(cp)))
                    return cp;
            }
        } catch (Exception x) {}
        throw new IllegalArgumentException(\"Unrecognized character name :\" + name);
    }
}
";


