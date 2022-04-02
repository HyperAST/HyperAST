use core::fmt;
use std::{
    io::{stdout, Write},
    ops::Deref,
};

use hyper_ast::{
    filter::BloomResult,
    nodes::RefContainer,
    position::{ExploreStructuralPositions, Scout, StructuralPosition, StructuralPositionStore, TreePath},
    store::{labels::LabelStore, nodes::DefaultNodeStore as NodeStore, SimpleStores, TypeStore},
    tree_gen::TreeGen,
    types::{LabelStore as _, Typed, Type},
    types::WithChildren,
};

use tree_sitter::{Language, Parser};

use crate::java::handle_java_file;

use rusted_gumtree_gen_ts_java::impact::{
    element::{IdentifierFormat, LabelPtr},
    partial_analysis::PartialAnalysis,
};
use rusted_gumtree_gen_ts_java::{
    impact::{element::RefsEnum, usage},
    java_tree_gen_full_compress_legion_ref as java_tree_gen,
};

fn run(text: &[u8]) {
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = java_tree_gen::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let a = handle_java_file(&mut java_tree_gen, "A.java".as_bytes(), text).unwrap();

    // let b = java_tree_gen.stores.node_store.resolve(a.local.compressed_node);
    match a.local.ana.as_ref() {
        Some(x) => {
            println!("refs:",);
            x.print_refs(&java_tree_gen.stores.label_store);
        }
        None => println!("None"),
    };
    let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;
    print!("{}",AA);
    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    let mut sp_store =
        StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let i = scoped!(package_ref, "SpoonAPI");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),2);
    println!("-------------2----------------");
    let package_ref2 = scoped!(root, "org");
    let i = scoped!(package_ref2, "Klass");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),7);
    println!("-------------3----------------");
    let i = scoped!(package_ref, "SpoonModelBuilder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),5);
    println!("-------------4----------------");
    let i = scoped!(package_ref, "SpoonException");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),4);
    println!("-------------5----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),1);
    println!("-------------6----------------");
    let i = scoped!(package_ref, "SpoonException2");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),1);
    println!("-------------7----------------");
    let i = scoped!(package_ref, "MavenLauncher");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),2);
    println!("-------------8----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType2");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),3);
    println!("-------------9----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType3");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),1);
    println!("-------------10----------------");
    let i = scoped!(package_ref, "SpoonFile");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),2);
}

#[test]
fn test_case() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).is_test(true).init();
    run(CASE_1.as_bytes())
}

// #[test]
// fn test_cases() {
//     let cases = [
//         CASE_1, CASE_2, CASE_3, CASE_4, CASE_5, CASE_6, CASE_7, CASE_8, CASE_9, CASE_10, CASE_11,
//         CASE_12, CASE_13,
//     ];
//     for case in cases {
//         run(case.as_bytes())
//     }
// }

static CASE_1: &'static str = r#"package spoon;

import com.martiansoftware.jsap.FlaggedOption;
import com.martiansoftware.jsap.JSAP;
import com.martiansoftware.jsap.JSAPException;
import com.martiansoftware.jsap.JSAPResult;
import com.martiansoftware.jsap.Switch;
import com.martiansoftware.jsap.stringparsers.EnumeratedStringParser;
import com.martiansoftware.jsap.stringparsers.FileStringParser;
import org.apache.commons.io.FileUtils;
import org.Klass;

import spoon.SpoonModelBuilder.InputType2;
import spoon.SpoonModelBuilder.InputType3;

import static spoon.support.StandardEnvironment.DEFAULT_CODE_COMPLIANCE_LEVEL;

/**
 */
public class Launcher extends Klass implements SpoonAPI, A {

    public SpoonModelBuilder f(Object e) {
        if (!(e instanceof SpoonException)) {
            
        }
        System.out.println(SpoonModelBuilder.InputType.FILES);
        if (!(e instanceof @TypeAnnotation(integer=1) SpoonException)) {
            
        }
        if (!(e instanceof @TypeAnnotation(integer=1) SpoonException[])) {
            
        }
        getModelBuilder().compile(SpoonModelBuilder.InputType2.FILES);
        try {
            f();
        } catch (SpoonException2 e) {

        }

        assertThrows(SpoonException.class, () -> {
            new MavenLauncher("./pomm.xml", MavenLauncher.SOURCE_TYPE.APP_SOURCE);
        });

        getModelBuilder2().compile(InputType3.FILES);

        getAllFiles().stream().filter(SpoonFile::isJava).collect(Collectors.toList());
        getAllFiles().stream().filter(spoon.SpoonFile::isJava).collect(Collectors.toList());

    }

    Klass M = new Klass() {
    };

    public static final SpoonModelBuilder.InputType2 INSTANCE = new FactoryCompilerConfig();

    public class FactoryCompilerConfig implements SpoonModelBuilder.InputType2 {
    }
    public interface AALaucher extends SpoonAPI {
        Klass M = new Klass() {
        };
    }
}
interface I {
    Klass M = new Klass() {
    };
}"#;


fn run2(text: &[u8]) {
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = java_tree_gen::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let a = handle_java_file(&mut java_tree_gen, "A.java".as_bytes(), text).unwrap();

    // let b = java_tree_gen.stores.node_store.resolve(a.local.compressed_node);
    match a.local.ana.as_ref() {
        Some(x) => {
            println!("refs:",);
            x.print_refs(&java_tree_gen.stores.label_store);
        }
        None => println!("None"),
    };
    let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;
    let v = AA;
    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    let mut sp_store =
        StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let i = scoped!(package_ref, "SpoonModelBuilder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),3);
    println!("-------------2----------------");
    let i = scoped!(package_ref, "Launcher");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),2);
    println!("-------------3----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),6);
    println!("-------------4----------------");
    let package_ref2 = scoped!(scoped!(scoped!(package_ref, "support"), "compiler"), "jdt");
    let i = scoped!(package_ref2, "SpoonFolder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
    println!("-------------5----------------");
    let package_ref2 = scoped!(package_ref, "support");
    let i = scoped!(scoped!(package_ref2, "Envir"), "MultipleAlt");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),2);
    println!("-------------6----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder2"), "InputType2");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),1);
    println!("-------------7----------------");
    let package_ref2 = scoped!(package_ref, "pattern");
    let i = scoped!(package_ref2, "PatternBuilder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
    println!("-------------8----------------");
    let package_ref2 = scoped!(package_ref, "pattern");
    let i = scoped!(scoped!(package_ref2, "PatternBuilder"), "TARGET_TYPE");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
    println!("-------------9----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder3"), "InputType");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),2);
    println!("-------------10---------------");
    let package_ref2 = scoped!(package_ref, "processor");
    let i = scoped!(package_ref2, "AbstractProcessor");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),2);
    println!("-------------11----------------");
    let package_ref2 = scoped!(scoped!(scoped!(package_ref, "support"), "compiler"), "jdt");
    let i = scoped!(scoped!(package_ref2, "JDTBasedSpoonCompiler"), "AAA");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
}

#[test]
fn test_case2() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).is_test(true).init();
    run2(CASE_2.as_bytes())
}

/// src/main/java/spoon/support/compiler/jdt/JDTBasedSpoonCompiler.java:(2753,21527): 
static CASE_2: &'static str = r#"package spoon.support.compiler.jdt;

import spoon.SpoonModelBuilder;
import spoon.SpoonModelBuilder3;
import spoon.SpoonModelBuilder.InputType;
import spoon.SpoonModelBuilder2.InputType2;
import spoon.Launcher;
import spoon.support.Envir;
import spoon.processor.AbstractProcessor;

public class JDTBasedSpoonCompiler implements spoon.SpoonModelBuilder {
    
    void f(Object e) {
        spoon.Launcher.main();

        getModelBuilder().compile(InputType.FILES);
        getModelBuilder().compile(InputType2.FILES);
        Launcher.LOGGER.error(e.getMessage(), e);
        for (SpoonFolder fol : getSubFolders()) {
			files.addAll(fol.getAllJavaFiles());
        }
        Envir.MultipleAlt alternatives = new Envir.MultipleAlt();
        templateParametersAsMap.put(spoon.pattern.PatternBuilder.TARGET_TYPE, targetType.getReference());
        SpoonModelBuilder3.InputType<T> result = new SpoonModelBuilder3.InputType<>(expectedType);
        launcher.addProcessor(new AbstractProcessor<>() {
			public void process(CtElement element) {
				markElementForSniperPrinting(element);
			}
		});
        types = new InputType[]{InputType.CTTYPES};
        for (InputType inputType : types) {}
		SpoonModelBuilder.InputType.FILES.initializeCompiler(batchCompiler);

        new spoon.support.compiler.jdt.JDTBasedSpoonCompiler.AAA(x)
    }

    static class AAA {}
}

class FactoryCompilerConfig implements SpoonModelBuilder.InputType {

}"#;

static AA: &'static str = r#"
/.java.lang.{
    /.spoon.test.template.testclasses,/,
}.{
    /.java.lang.{

        /.spoon.test.template.testclasses,/,
    }%SubstitutionByExpressionTemplate,
    /.spoon.test.template.testclasses%SubstitutionByExpressionTemplate,
    /.java.lang.{
        /.spoon.test.template.testclasses,/,
    }%BlockTemplate,
    /.spoon.test.template.testclasses%BlockTemplate,
}%String


@B = {/.spoon.test.template,/,}%SubstitutionTest
@C = {
    /.java.lang.@B,
    /.spoon.reflect.declaration.@B,
    /.spoon.template.@B,
    /.java.util.@B,
    /.spoon.test.template%SubstitutionTest,
    /.java.lang%Object,
}
@D = {
    /.spoon.test.template,/,
}.@C

/.java.util.@D.{
    /.java.lang.@D%FieldWithTemplatedInitializer,
    /.spoon.reflect.declaration.@D%FieldWithTemplatedInitializer,
    /.spoon.template.@D%FieldWithTemplatedInitializer,
    /.java.util.@D%FieldWithTemplatedInitializer,
    /.spoon.test.template.@C%FieldWithTemplatedInitializer,
    /.@C%FieldWithTemplatedInitializer,
    /.java.lang.@D%ExtensionTemplate,
    /.spoon.reflect.declaration.@D%ExtensionTemplate,
    /.spoon.template.@D%ExtensionTemplate,
    /.java.util.@D%ExtensionTemplate,
    /.spoon.test.template.@C%ExtensionTemplate,
    /.@C%ExtensionTemplate,
}%TemplateParameter.S()


@A = /.java.lang.{
    /.spoon,/,
}

@B = /.spoon.compiler.{
    /.spoon,/,
}

@C = {
    @A%PatternBuiler,
    @B%PatternBuiler,
    /.spoon%PatternBuiler,
    /.java.lang%Object,
}

@A.@C.{
    @A.@C%PatternQuery,
    @B.@C%PatternQuery,
    /.spoon.@C%PatternQuery,
    /.@C%PatternQuery,
    /.java.lang%Object,
}%Environment
"#;


fn run3(text: &[u8]) {
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = java_tree_gen::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let a = handle_java_file(&mut java_tree_gen, "A.java".as_bytes(), text).unwrap();

    // let b = java_tree_gen.stores.node_store.resolve(a.local.compressed_node);
    match a.local.ana.as_ref() {
        Some(x) => {
            println!("refs:",);
            x.print_refs(&java_tree_gen.stores.label_store);
        }
        None => println!("None"),
    };
    let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;
    let v = AA;
    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    let mut sp_store =
        StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let package_ref2 = scoped!(package_ref, "compiler");
    let i = scoped!(package_ref2, "SpoonResource");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),2);
    println!("-------------2----------------");
    let i = scoped!(mm, "SpoonFile");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(),Type::Program);
    let xx = bb.get_child(&18);
    x.goto(xx, 18);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::ClassDeclaration);
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 4);
    println!("-------------3----------------");
    let i = scoped!(package_ref2, "AnnotationProcessingOptions");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
    println!("-------------4----------------");
    let i = scoped!(package_ref, "NameFilter");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(),1);
    println!("-------------5----------------");
    let i = scoped!(mm, "StringAttr");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(),Type::Program);
    let xx = bb.get_child(&18);
    x.goto(xx, 18);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::ClassDeclaration);
    let xx = bb.get_child(&9);
    x.goto(xx, 9);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::ClassBody);
    let xx = bb.get_child(&2);
    x.goto(xx, 2);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::ConstructorDeclaration);
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------6----------------");
    let package_ref2 = scoped!(package_ref, "reflect");
    let i = scoped!(scoped!(package_ref2, "CtModelImpl"), "CtRootPackage");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
    println!("-------------7----------------");
    let package_ref2 = scoped!(package_ref, "compiler");
    let i = scoped!(package_ref2, "SpoonFile");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
}

#[test]
fn test_case3() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).is_test(true).init();
    run3(CASE_3.as_bytes())
}

/// search spoon.compile.SpoonResource
/// search spoon.compile.SpoonFile in class
/// TODO SpoonFile.this
static CASE_3: &'static str = r#"package spoon.compiler;

import org.apache.commons.io.IOUtils;
import spoon.SpoonException;
import spoon.NameFilter;
import spoon.reflect.CtModelImpl;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.Charset;

public class SpoonFile<T extends SpoonFile<T>> implements SpoonResource {

    public SpoonFile() {
        super(SpoonFile.class);
        new StringAttr() {
            f();
        }.scan();
        if (element instanceof CtModelImpl.CtRootPackage) {

        }
    }

    class StringAttr extends Scanner {
        void g() {
            SpoonFile.this.forEachParameterInfo(consumer);
        }
    }

    void f() {
        foo.getElements(new NameFilter<>("i"));
        g(SpoonFile::h);
    }

    @Override
	public JDTBuilder h(AnnotationProcessingOptions<?> options) {}

    @Override
	public <T extends SpoonResource> T setA(boolean f) {
    }

    public static <T> T build(spoon.compiler.SpoonFile builder, Object o) {return null;}

}"#;

/// search spoon.compiler.builder.ClasspathOptions
/// search spoon.compiler.builder.SourceOptions
/// TODO search spoon.compiler.builder.AAA
static CASE_4: &'static str = r#"package spoon.compiler.builder;
/**
 * Helper to build arguments for the JDT compiler.
 */
public interface JDTBuilder extends Builder {
    /**
    * Classpath options for the compiler.
    */
    JDTBuilder classpathOptions(ClasspathOptions<?> options);

    public JDTBuilder sources(SourceOptions<?> options) {
    }
    @DerivedProperty // aaa
    AAA getAAA();
}"#;

/// search spoon.legacy.NameFilter
static CASE_5: &'static str = r#"package spoon.test.filters;

import spoon.legacy.NameFIlter

public class FilterTest {
    public void f() {
        NameFilter<CtNamedElement> nameFilter = new NameFilter<>(name);
    }
}"#;



fn run6(text: &[u8]) {
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = java_tree_gen::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let a = handle_java_file(&mut java_tree_gen, "A.java".as_bytes(), text).unwrap();

    // let b = java_tree_gen.stores.node_store.resolve(a.local.compressed_node);
    match a.local.ana.as_ref() {
        Some(x) => {
            println!("refs:",);
            x.print_refs(&java_tree_gen.stores.label_store);
        }
        None => println!("None"),
    };
    let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;
    let v = AA;
    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    let mut sp_store =
        StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let i = scoped!(scoped!(mm, "PatternBuiler"), "PatternQuery");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(),Type::Program);
    let xx = bb.get_child(&4);
    x.goto(xx, 4);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::ClassDeclaration);
    // let xx = bb.get_child(&6);
    // x.goto(xx, 6);
    // let bb = stores.node_store.resolve(xx);
    // assert_eq!(bb.get_type(),Type::ClassBody);
    // let xx = bb.get_child(&2);
    // x.goto(xx, 2);
    // let bb = stores.node_store.resolve(xx);
    // assert_eq!(bb.get_type(),Type::ClassDeclaration);
    // let xx = bb.get_child(&6);
    // x.goto(xx, 6);
    // let bb = stores.node_store.resolve(xx);
    // assert_eq!(bb.get_type(),Type::ClassBody);
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 2);
    println!("-------------2----------------");
    let package_ref2 = scoped!(package_ref, "compiler");
    let i = scoped!(package_ref2, "Environment");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(),1);
}

#[test]
fn test_case6() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).is_test(true).init();
    run6(CASE_6.as_bytes())
}

/// search PatternQuery in body
/// search spoon.compiler.Environment
static CASE_6: &'static str = r#"package spoon;

import spoon.compiler.*;

public class PatternBuiler {
    static class PatternQuery {
        public void f() {
            PatternBuiler.PatternQuery patternQuery = new PatternBuiler.PatternQuery(getFactory().Query(), patternModel);
        }
    }
    Environment a;

}"#;

/// search spoon.reflect.declaration.CtAnonymousExecutable
static CASE_7: &'static str = r#"package spoon;

import spoon.reflect.declaration.CtAnonymousExecutable;

public enum CtRole {
    ANNONYMOUS_EXECUTABLE(TYPE_MEMBER, obj -> obj instanceof CtAnonymousExecutable),

}"#;

/// CtAnnotationImpl in body
static CASE_8: &'static str = r#"package spoon;

import spoon.reflect.declaration.CtAnonymousExecutable;

public class CtAnnotationImpl {
    class A {
        String f() {
            return CtAnnotationImpl.this.toString();
        }
    }

}"#;

/// find SwitchNode in body one time
static CASE_9: &'static str = r#"package spoon.pattern.internal.node;

public class SwitchNode extends AbstractNode implements InlineNode {

    private class CaseNode extends AbstractNode implements InlineNode {
        @Override
        public void forEachParameterInfo(BiConsumer<ParameterInfo, RootNode> consumer) {
                SwitchNode.this.forEachParameterInfo(consumer);
        }
    }

}"#;


