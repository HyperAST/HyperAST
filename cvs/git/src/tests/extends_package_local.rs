use hyper_ast::{
    filter::{Bloom, BloomResult, BF},
    nodes::RefContainer,
    position::{
         Scout, StructuralPosition, StructuralPositionStore, TreePath,
    },
    store::{labels::LabelStore, nodes::DefaultNodeStore as NodeStore, SimpleStores, TypeStore},
    types::WithChildren,
    types::{LabelStore as _, Type, Typed}, impact::serialize::CachedHasher,
};

use hyper_ast_gen_ts_java::legion_with_refs::{
    print_tree_syntax, BulkHasher,
};

use crate::java::handle_java_file;

use hyper_ast_gen_ts_java::impact::{
    element::{IdentifierFormat, LabelPtr},
    partial_analysis::PartialAnalysis,
};
use hyper_ast_gen_ts_java::{
    impact::{element::RefsEnum, usage},
    legion_with_refs as java_tree_gen,
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
    handle_java_file(&mut java_tree_gen, "A.java".as_bytes(), text).unwrap();
}

fn run1(text: &[u8]) {
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
    print!("{}", AA);
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

    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let i = scoped!(package_ref, "SpoonAPI");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 2);
    println!("-------------2----------------");
    let package_ref2 = scoped!(root, "org");
    let i = scoped!(package_ref2, "Klass");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 17);
    println!("-------------3----------------");
    let i = scoped!(package_ref, "SpoonModelBuilder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 5);
    println!("-------------4----------------");
    let i = scoped!(package_ref, "SpoonException");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 4);
    println!("-------------5----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------6----------------");
    let i = scoped!(package_ref, "SpoonException2");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------7----------------");
    let i = scoped!(package_ref, "MavenLauncher");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 3);
    println!("-------------8----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType2");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 3);
    println!("-------------9----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType3");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------10----------------");
    let i = scoped!(package_ref, "SpoonFile");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 2);
    println!("-------------11----------------");
    let i = scoped!(package_ref, "Z");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
}

#[test]
fn test_case() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run1(CASE_1.as_bytes())
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

        new Z.B();

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

    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let i = scoped!(package_ref, "SpoonModelBuilder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 3);
    println!("-------------2----------------");
    let i = scoped!(package_ref, "Launcher");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 2);
    println!("-------------3----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder"), "InputType");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 6);
    println!("-------------4----------------");
    let package_ref2 = scoped!(scoped!(scoped!(package_ref, "support"), "compiler"), "jdt");
    let i = scoped!(package_ref2, "SpoonFolder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------5----------------");
    let package_ref2 = scoped!(package_ref, "support");
    let i = scoped!(scoped!(package_ref2, "Envir"), "MultipleAlt");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 3);
    println!("-------------6----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder2"), "InputType2");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------7----------------");
    let package_ref2 = scoped!(package_ref, "pattern");
    let i = scoped!(package_ref2, "PatternBuilder");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------8----------------");
    let package_ref2 = scoped!(package_ref, "pattern");
    let i = scoped!(scoped!(package_ref2, "PatternBuilder"), "TARGET_TYPE");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------9----------------");
    let i = scoped!(scoped!(package_ref, "SpoonModelBuilder3"), "InputType");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 2);
    println!("-------------10---------------");
    let package_ref2 = scoped!(package_ref, "processor");
    let i = scoped!(package_ref2, "AbstractProcessor");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 3);
    println!("-------------11----------------");
    let package_ref2 = scoped!(scoped!(scoped!(package_ref, "support"), "compiler"), "jdt");
    let i = scoped!(scoped!(package_ref2, "JDTBasedSpoonCompiler"), "AAA");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 2);
}

#[test]
fn test_case2() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
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
    assert_eq!(r.len(), 2);
    println!("-------------2----------------");
    let i = scoped!(mm, "SpoonFile");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(), Type::Program);
    let xx = bb.child(&18).unwrap();
    x.goto(xx, 18);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassDeclaration);
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 4);
    println!("-------------3----------------");
    let i = scoped!(package_ref2, "AnnotationProcessingOptions");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------4----------------");
    let i = scoped!(package_ref, "NameFilter");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 2);
    println!("-------------5----------------");
    let i = scoped!(mm, "StringAttr");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(), Type::Program);
    let xx = bb.child(&18).unwrap();
    x.goto(xx, 18);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassDeclaration);
    let xx = bb.child(&9).unwrap();
    x.goto(xx, 9);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassBody);
    let xx = bb.child(&2).unwrap();
    x.goto(xx, 2);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ConstructorDeclaration);
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 2);
    println!("-------------6----------------");
    let package_ref2 = scoped!(package_ref, "reflect");
    let i = scoped!(scoped!(package_ref2, "CtModelImpl"), "CtRootPackage");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 1);
    println!("-------------7----------------");
    let package_ref2 = scoped!(package_ref, "compiler");
    let i = scoped!(package_ref2, "SpoonFile");
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 5);
}

#[test]
fn test_case3() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
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

fn run3_1(text: &[u8]) {
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
    let _ = AA;
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
    macro_rules! scoped_type {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern_ref(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    let mut sp_store =
        StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");
    let package_lang = scoped!(scoped!(root, "java"), "lang");

    let package_ref2 = scoped!(package_ref, "compiler");
    println!("-------------3----------------");
    let i = scoped_type!(package_ref2, "AnnotationProcessingOptions");
    {
        let uncertain = ana.solver.intern(RefsEnum::Or(
            vec![package_ref2, root, package_lang].into(),
        ));
        let i = scoped_type!(uncertain, "AnnotationProcessingOptions");
        let d = ana.solver.nodes.with(i);
        eprintln!("i: {:?}",d);
        type T = Bloom<&'static [u8], u16>;
        let r = CachedHasher::<usize, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::once(d);
        eprintln!("CachedHasher or: {:?}", r)
    }
    {
        let it = ana.solver.iter_refs();
        type T = Bloom<&'static [u8], u16>;
        let it = BulkHasher::<_, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::from(it);
        // let it:Vec<_> = it.collect();
        eprintln!("search list: {:?}", it.collect::<Vec<_>>())
    } 
    {
        let it = ana.solver.iter_refs();
        type T = Bloom<&'static [u8], u16>;
        let it = BulkHasher::<_, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::from(it);
        // let it:Vec<_> = it.collect();
        let bloom = T::from(it);
        eprintln!("search bloom: {:?}", bloom)
    } 
    {
        let d = ana.solver.nodes.with(i);
        eprintln!("i: {:?}",d);
        type T = Bloom<&'static [u8], u16>;
        let r = CachedHasher::<usize, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::once(d);
        eprintln!("CachedHasher result: {:?}", r)
    }
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 1);
}

#[test]
fn test_case3_1() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run3_1(CASE_3_1.as_bytes())
}

/// search spoon.compile.SpoonResource
/// search spoon.compile.SpoonFile in class
/// TODO SpoonFile.this
static CASE_3_1: &'static str = r#"package spoon.compiler;

public class A {
	public void h(AnnotationProcessingOptions<?> options) {}
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

#[test]
fn test_case4() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run(CASE_4.as_bytes())
}

/// search spoon.legacy.NameFilter
static CASE_5: &'static str = r#"package spoon.test.filters;

import spoon.legacy.NameFIlter

public class FilterTest {
    public void f() {
        NameFilter<CtNamedElement> nameFilter = new NameFilter<>(name);
    }
}"#;

#[test]
fn test_case5() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run(CASE_5.as_bytes())
}

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
    assert_eq!(bb.get_type(), Type::Program);
    let xx = bb.child(&4).unwrap();
    x.goto(xx, 4);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassDeclaration);
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
    assert_eq!(r.len(), 1);
}

#[test]
fn test_case6() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
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

fn run7(text: &[u8]) {
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
    let mut sp_store =
        StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let package_ref2 = scoped!(scoped!(package_ref, "reflect"), "declaration");
    let i = scoped_ref!(package_ref2, "CtAnonymousExecutable");
    {
        let it = ana.solver.iter_refs();
        type T = Bloom<&'static [u8], u16>;
        let it = BulkHasher::<_, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::from(it);
        // let it:Vec<_> = it.collect();
        let bloom = T::from(it);
        eprintln!("search bloom: {:?}", bloom)
    } 
    {
        let d = ana.solver.nodes.with(i);
        type T = Bloom<&'static [u8], u16>;
        let r = CachedHasher::<usize, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::once(d);
        eprintln!("CachedHasher result: {:?}", r)
    }
    let d = ana.solver.nodes.with(i);
    if let BloomResult::MaybeContain = stores.node_store.resolve(a.local.compressed_node).check(d) {
    } else {
        assert!(false);
    }
    let x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref2, i, x);
    assert_eq!(r.len(), 1);
}
//[1001011111000000]
#[test]
fn test_case7() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run7(CASE_7.as_bytes())
}

/// search spoon.reflect.declaration.CtAnonymousExecutable
static CASE_7: &'static str = r#"package spoon;

import spoon.reflect.declaration.CtAnonymousExecutable;

public enum CtRole {
    ANNONYMOUS_EXECUTABLE(TYPE_MEMBER, obj -> obj instanceof CtAnonymousExecutable),

}"#;

#[test]
fn test_hashing() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;
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
    macro_rules! scoped_type {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern_ref(RefsEnum::TypeIdentifier(o, i))
        }};
    }
    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");
    let package_ref2 = scoped!(scoped!(package_ref, "reflect"), "declaration");
    let i = scoped_ref!(package_ref2, "CtAnonymousExecutable");
    let _ = i;
    let package_lang = scoped!(scoped!(root, "java"), "lang");
    let lang_obj = scoped_type!(package_lang, "Object");
    let uncertain = ana.solver.intern(RefsEnum::Or(
        vec![lang_obj, package_lang, package_ref, root].into(),
    ));
    // let i = scoped_ref!(uncertain, "TYPE_MEMBER");
    let i = scoped_ref!(uncertain, "obj");
    let d = ana.solver.nodes.with(i);
    let it = ana.solver.iter_refs();
    type T = Bloom<&'static [u8], u16>;
    let it = BulkHasher::<_, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::from(it);
    // let it:Vec<_> = it.collect();
    let bloom = T::from(it);
    eprintln!("{:?}", bloom);
    let r = CachedHasher::<usize, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::once(d)[0];
    bloom.check_raw(r);
}

fn run8(text: &[u8]) {
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
    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern_ref(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    let mut sp_store =
        StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");

    println!("-------------1----------------");
    let i = scoped!(mm, "CtAnnotationImpl");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(), Type::Program);
    let xx = bb.child(&4).unwrap();
    x.goto(xx, 4);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassDeclaration);
    let xx = bb.child(&6).unwrap();
    x.goto(xx, 6);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassBody);
    // let xx = bb.get_child(&2);
    // x.goto(xx, 2);
    // let bb = stores.node_store.resolve(xx);
    // assert_eq!(bb.get_type(),Type::ClassDeclaration);
    // let xx = bb.get_child(&6);
    // x.goto(xx, 6);
    // let bb = stores.node_store.resolve(xx);
    // assert_eq!(bb.get_type(),Type::ClassBody);

    {
        let it = ana.solver.iter_refs();
        type T = Bloom<&'static [u8], u64>;
        let it = BulkHasher::<_, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::from(it);
        // let it:Vec<_> = it.collect();
        let bloom = T::from(it);
        eprintln!("search bloom: {:?}", bloom)
    } 
    {
        let d = ana.solver.nodes.with(i);
        type T = Bloom<&'static [u8], u64>;
        let r = CachedHasher::<usize, <T as BF<[u8]>>::S, <T as BF<[u8]>>::H>::once(d);
        eprintln!("CachedHasher result: {:?}", r)
    }
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
}

#[test]
fn test_case8() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run8(CASE_8.as_bytes())
}

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

#[test]
fn test_case9() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run(CASE_9.as_bytes())
}

fn run10(text: &[u8]) {
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
    let i = scoped!(mm, "CtAnnotationImpl");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(), Type::Program);
    let xx = bb.child(&8).unwrap();
    x.goto(xx, 8);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassDeclaration);
    let xx = bb.child(&9).unwrap();
    x.goto(xx, 9);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassBody);
    // let xx = bb.get_child(&2);
    // x.goto(xx, 2);
    // let bb = stores.node_store.resolve(xx);
    // assert_eq!(bb.get_type(),Type::ClassDeclaration);
    // let xx = bb.get_child(&6);
    // x.goto(xx, 6);
    // let bb = stores.node_store.resolve(xx);
    // assert_eq!(bb.get_type(),Type::ClassBody);
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 0);
    println!("-------------2----------------");
    let i = scoped!(mm, "node");
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let bb = stores.node_store.resolve(a.local.compressed_node);
    assert_eq!(bb.get_type(), Type::Program);
    let xx = bb.child(&8).unwrap();
    x.goto(xx, 8);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassDeclaration);
    let xx = bb.child(&9).unwrap();
    x.goto(xx, 9);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(), Type::ClassBody);
    let xx = bb.child(&6).unwrap();
    x.goto(xx, 6);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::MethodDeclaration);
    let xx = bb.child(&7).unwrap();
    x.goto(xx, 7);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::Block);
    let xx = bb.child(&2).unwrap();
    x.goto(xx, 2);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::EnhancedForStatement);
    let xx = bb.child(&10).unwrap();
    x.goto(xx, 10);
    let bb = stores.node_store.resolve(xx);
    assert_eq!(bb.get_type(),Type::Block);
    let r = usage::RefsFinder::new(&stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    assert_eq!(r.len(), 1);
}

#[test]
fn test_case10() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run10(CASE_10.as_bytes())
}

/// search `Tacos.Burritos` in Tacos.Burritos body
static CASE_10: &'static str = r#"package spoon.test.generics.testclasses;

import javax.lang.model.util.SimpleTypeVisitor7;
import java.util.ArrayList;
import java.util.List;

public class Tacos<K, V extends String> implements ITacos<V> {

        public Tacos() {
            <String>this(1);
        }

        public <T> Tacos(int nbTacos) {
        }

        public void m() {
            for (ControlFlowNode node : cfg.vertexSet()) {
                if (node.getKind() == BranchKind.BEGIN) {
                    // Dont add a state for the BEGIN node
                    continue;
                }
            }
            List<String> l = new ArrayList<>();
            List l2;
            IBurritos<?, ?> burritos = new Burritos<>();
            List<?> l3 = new ArrayList<Object>();
            new <Integer>Tacos<Object, String>();
            new Tacos<>();
        }
        
        public List<Label> getLabels(int state) {
            return labels.get(state);
        }

        public void m2() {
            this.<String>makeTacos(null);
            this.makeTacos(null);
        }

        public void m3() {
            new SimpleTypeVisitor7<Tacos, Void>() {
            };
            new javax.lang.model.util.SimpleTypeVisitor7<Tacos, Void>() {
            };
        }

        public <V, C extends List<V>> void m4() {
            Tacos.<V, C>makeTacos();
            Tacos.makeTacos();
        }

        public static <V, C extends List<V>> List<C> makeTacos() {
                return null;
        }

        public <T> void makeTacos(T anObject) {
        }

        class Burritos<K, V> implements IBurritos<K, V> {
            Tacos<K, String>.Burritos<K, V> burritos;
            public Tacos<K, String>.Burritos<K, V> b() {
                    new Burritos<K, V>();
                    return null;
            }

            class Pozole {
                    public Tacos<K, String>.Burritos<K, V>.Pozole p() {
                            new Pozole();
                            return null;
                    }
            }

            @Override
            public IBurritos<K, V> make() {
                    return new Burritos<K, V>() {};
            }
    }

    public class BeerFactory {
            public Beer newBeer() {
                    return new Beer();
            }
    }

    class Beer {
    }
}
"#;

fn run11(text: &[u8]) {
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
    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = java_tree_gen.stores.label_store.get_or_insert(i);
            let i = LabelPtr::new(i, f);
            ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    // let sp_store =
    //     StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    let root = ana.solver.intern(RefsEnum::Root);
    let package_ref = scoped!(root, "spoon");
    let _ = package_ref;

    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &a.local.compressed_node,
    );
    println!();
}

#[test]
fn test_case11() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run11(CASE_11.as_bytes());
    println!("{}", CASE_11.as_bytes().len());
    println!("{}", CASE_11_BIS.as_bytes().len());
}

static CASE_11: &'static str = r#"/**
* The MIT License
* <p>
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
* <p>
* The above copyright notice and this permission notice shall be included in
* all copies or substantial portions of the Software.
* <p>
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
* THE SOFTWARE.
*/

package fr.inria.controlflow;

import org.junit.Test;
import spoon.processing.AbstractProcessor;
import spoon.processing.ProcessingManager;
import spoon.reflect.code.CtIf;
import spoon.reflect.declaration.CtMethod;
import spoon.reflect.factory.Factory;
import spoon.support.QueueProcessingManager;

import static junit.framework.TestCase.assertFalse;
import static org.junit.Assert.assertTrue;

/**
* Created by marodrig on 04/01/2016.
*/
public class AllBranchesReturnTest {
/*
   private ModifierKind getInvocationMethodVisibility(CtInvocation inv) {
       if (inv.getExecutable().getDeclaration() != null &&
               inv.getExecutable().getDeclaration() instanceof CtMethodImpl)
           return (inv.getExecutable().getDeclaration()).getVisibility();
       return null;
   }

   @Test
   public void testSegment2() throws Exception {
       final Factory factory = new SpoonMetaFactory().buildNewFactory(
               "C:\\MarcelStuff\\DATA\\DIVERSE\\input_programs\\MATH_3_2\\src\\main\\java", 7);
       //        "C:\\MarcelStuff\\DATA\\DIVERSE\\input_programs\\easymock-light-3.2\\src\\main\\javaz", 7);
       ProcessingManager pm = new QueueProcessingManager(factory);


       AbstractProcessor<CtMethod> p = new AbstractProcessor<CtMethod>() {
           @Override
           public void process(CtMethod ctMethod) {
               List<CtFor> fors = ctMethod.getElements(new TypeFilter<CtFor>(CtFor.class));
               if (ctMethod.getBody() == null || ctMethod.getBody().getStatements() == null) return;

               int size = ctMethod.getBody().getStatements().size();

               if (size > 6 || fors.size() < 1 || !hasInterfaceVariables(ctMethod) ) return;

               printMethod(ctMethod);

           }
       };

       pm.addProcessor(p);
       pm.process();
   }

   private boolean hasInterfaceVariables(CtMethod ctMethod) {
       List<CtVariableAccess> vars =
               ctMethod.getElements(new TypeFilter<CtVariableAccess>(CtVariableAccess.class));
       for ( CtVariableAccess a : vars ) {
           try {
               if (!a.getVariable().getDeclaration().getModifiers().contains(ModifierKind.FINAL) &&
                       a.getVariable().getType().isInterface()) return true;
           } catch (Exception e) {
               System.out.print(".");
           }
       }
       return false;
   }

   private void printMethod(CtMethod ctMethod) {
       System.out.println(ctMethod.getPosition().toString());
       System.out.println(ctMethod);
       //System.out.println(invName);
       System.out.println("+++++++++++++++++++++++++++++++++++++");

   }

   private void printStaticInvocations(CtMethodImpl ctMethod) {
       List<CtInvocation> invs = ctMethod.getElements(new TypeFilter<CtInvocation>(CtInvocation.class));
       boolean staticInv = true;
       boolean abstractVarAccess = false;
       String invName = "";
       for (CtInvocation inv : invs) {
           ModifierKind mk = getInvocationMethodVisibility(inv);
           if (inv.getExecutable().isStatic() &&
                   (mk == ModifierKind.PRIVATE || mk == ModifierKind.PROTECTED)) {
               invName = inv.toString();
               staticInv = true;
               break;
           }
       }
       if( staticInv) {
           System.out.println(ctMethod.getPosition().toString());
           System.out.println(ctMethod);
           System.out.println(invName);
           System.out.println("+++++++++++++++++++++++++++++++++++++");
       }
   }*/

   public void testSegment(AbstractProcessor processor) throws Exception {
       //ControlFlowGraph graph = buildGraph(this.getClass().getResource("/control-flow").toURI().getPath(),
       //        "nestedIfSomeNotReturning", false);

       Factory factory = new SpoonMetaFactory().buildNewFactory(
               this.getClass().getResource("/control-flow").toURI().getPath(), 7);
       ProcessingManager pm = new QueueProcessingManager(factory);
       pm.addProcessor(processor);
       pm.process(factory.getModel().getRootPackage());
   }

   @Test
   public void nestedIfSomeNotReturning() throws Exception {
       testSegment(new AbstractProcessor<CtIf>() {
           @Override
           public void process(CtIf element) {
               CtMethod m = element.getParent().getParent(CtMethod.class);
               if (m != null && m.getSimpleName().equals("nestedIfSomeNotReturning"))
                   if (element.getCondition().toString().contains("b < 1")) {
                       AllBranchesReturn alg = new AllBranchesReturn();
                       assertFalse(alg.execute(element));
                   }
           }
       });
   }

   @Test
   public void testNestedIfAllReturning() throws Exception {
       testSegment(new AbstractProcessor<CtIf>() {
           @Override
           public void process(CtIf element) {
               CtMethod m = element.getParent().getParent(CtMethod.class);
               if (m != null && m.getSimpleName().equals("nestedIfAllReturning"))
                   if (element.getCondition().toString().contains("a > 0")) {
                       AllBranchesReturn alg = new AllBranchesReturn();
                       assertTrue(alg.execute(element));
                   }
           }
       });
   }

}
"#;
static CASE_11_BIS: &'static str = r#"/**
 * The MIT License
 * <p>
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 * <p>
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 * <p>
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

package fr.inria.controlflow;

import org.junit.Test;
import spoon.processing.AbstractProcessor;
import spoon.processing.ProcessingManager;
import spoon.reflect.code.CtIf;
import spoon.reflect.declaration.CtMethod;
import spoon.reflect.factory.Factory;
import spoon.support.QueueProcessingManager;

import static junit.framework.TestCase.assertFalse;
import static org.junit.Assert.assertTrue;

/**
 * Created by marodrig on 04/01/2016.
 */
public class AllBranchesReturnTest {
    /*
        private ModifierKind getInvocationMethodVisibility(CtInvocation inv) {
            if (inv.getExecutable().getDeclaration() != null &&
                    inv.getExecutable().getDeclaration() instanceof CtMethodImpl)
                return (inv.getExecutable().getDeclaration()).getVisibility();
            return null;
    }

    @Test
    public void testSegment2() throws Exception {
            final Factory factory = new SpoonMetaFactory().buildNewFactory(
                    "C:\\MarcelStuff\\DATA\\DIVERSE\\input_programs\\MATH_3_2\\src\\main\\java", 7);
        //        "C:\\MarcelStuff\\DATA\\DIVERSE\\input_programs\\easymock-light-3.2\\src\\main\\javaz", 7);
        ProcessingManager pm = new QueueProcessingManager(factory);


        AbstractProcessor<CtMethod> p = new AbstractProcessor<CtMethod>() {
                @Override
                public void process(CtMethod ctMethod) {
                    List<CtFor> fors = ctMethod.getElements(new TypeFilter<CtFor>(CtFor.class));
                    if (ctMethod.getBody() == null || ctMethod.getBody().getStatements() == null) return;
    
                    int size = ctMethod.getBody().getStatements().size();
    
                    if (size > 6 || fors.size() < 1 || !hasInterfaceVariables(ctMethod) ) return;
    
                    printMethod(ctMethod);
    
            }
        };

        pm.addProcessor(p);
        pm.process();
    }

    private boolean hasInterfaceVariables(CtMethod ctMethod) {
            List<CtVariableAccess> vars =
                    ctMethod.getElements(new TypeFilter<CtVariableAccess>(CtVariableAccess.class));
            for ( CtVariableAccess a : vars ) {
                try {
                    if (!a.getVariable().getDeclaration().getModifiers().contains(ModifierKind.FINAL) &&
                            a.getVariable().getType().isInterface()) return true;
            } catch (Exception e) {
                    System.out.print(".");
            }
        }
        return false;
    }

    private void printMethod(CtMethod ctMethod) {
            System.out.println(ctMethod.getPosition().toString());
            System.out.println(ctMethod);
            //System.out.println(invName);
            System.out.println("+++++++++++++++++++++++++++++++++++++");

    }

    private void printStaticInvocations(CtMethodImpl ctMethod) {
            List<CtInvocation> invs = ctMethod.getElements(new TypeFilter<CtInvocation>(CtInvocation.class));
            boolean staticInv = true;
            boolean abstractVarAccess = false;
            String invName = "";
        for (CtInvocation inv : invs) {
                ModifierKind mk = getInvocationMethodVisibility(inv);
                if (inv.getExecutable().isStatic() &&
                        (mk == ModifierKind.PRIVATE || mk == ModifierKind.PROTECTED)) {
                    invName = inv.toString();
                    staticInv = true;
                    break;
            }
        }
        if( staticInv) {
                System.out.println(ctMethod.getPosition().toString());
                System.out.println(ctMethod);
                System.out.println(invName);
                System.out.println("+++++++++++++++++++++++++++++++++++++");
        }
    }*/

	public void testSegment(AbstractProcessor processor) throws Exception {
    		//ControlFlowGraph graph = buildGraph(this.getClass().getResource("/control-flow").toURI().getPath(),
		//        "nestedIfSomeNotReturning", false);

		Factory factory = new SpoonMetaFactory().buildNewFactory(
    				this.getClass().getResource("/control-flow").toURI().getPath(), 7);
		ProcessingManager pm = new QueueProcessingManager(factory);
		pm.addProcessor(processor);
		pm.process(factory.getModel().getRootPackage());
	}

	@Test
	public void nestedIfSomeNotReturning() throws Exception {
    		testSegment(new AbstractProcessor<CtIf>() {
    			@Override
    			public void process(CtIf element) {
    				CtMethod m = element.getParent().getParent(CtMethod.class);
    				if (m != null && m.getSimpleName().equals("nestedIfSomeNotReturning"))
					if (element.getCondition().toString().contains("b < 1")) {
    						AllBranchesReturn alg = new AllBranchesReturn();
    						assertFalse(alg.execute(element));
    					}
			}
		});
	}

	@Test
	public void testNestedIfAllReturning() throws Exception {
    		testSegment(new AbstractProcessor<CtIf>() {
    			@Override
    			public void process(CtIf element) {
    				CtMethod m = element.getParent().getParent(CtMethod.class);
    				if (m != null && m.getSimpleName().equals("nestedIfAllReturning"))
					if (element.getCondition().toString().contains("a > 0")) {
    						AllBranchesReturn alg = new AllBranchesReturn();
    						assertTrue(alg.execute(element));
    					}
			}
		});
	}

}
"#;

#[test]
fn test_case_11_bis() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run(CASE_11_BIS.as_bytes())
}



fn run12(text: &[u8]) {
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
    // let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;
    // macro_rules! scoped {
    //     ( $o:expr, $i:expr ) => {{
    //         let o = $o;
    //         let i = $i;
    //         let f = IdentifierFormat::from(i);
    //         let i = stores.label_store.get_or_insert(i);
    //         let i = LabelPtr::new(i, f);
    //         ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
    //     }};
    // }
    // let mut sp_store =
    //     StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    // let root = ana.solver.intern(RefsEnum::Root);
    // let package_ref = scoped!(root, "spoon");

    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &a.local.compressed_node,
    );
    println!();
}

#[test]
fn test_case12() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run12(CASE_12.as_bytes());
}

static CASE_12: &'static str = r#"

package a.b.c.d.e.f;

import org.B;

public class A {
    private B B;
    public B getB() {}
}"#;


fn run13(text: &[u8]) {
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
    // let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;
    // macro_rules! scoped {
    //     ( $o:expr, $i:expr ) => {{
    //         let o = $o;
    //         let i = $i;
    //         let f = IdentifierFormat::from(i);
    //         let i = stores.label_store.get_or_insert(i);
    //         let i = LabelPtr::new(i, f);
    //         ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
    //     }};
    // }
    // let mut sp_store =
    //     StructuralPositionStore::from(StructuralPosition::new(a.local.compressed_node));

    // let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    // let root = ana.solver.intern(RefsEnum::Root);
    // let package_ref = scoped!(root, "spoon");

    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &a.local.compressed_node,
    );
    println!();
}

#[test]
fn test_case13() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    run13(CASE_13.as_bytes());
}

static CASE_13: &'static str = r#"

package org.apache.spark.shuffle.sort;

import scala.*;
import scala.collection.Iterator;

public class A {



}"#;