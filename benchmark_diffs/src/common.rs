use criterion::black_box;
use hyper_diff::algorithms;
use hyperast::{store::SimpleStores, types::HyperAST};
use std::{cmp::max, fs, path::Path};

use crate::preprocess::parse_string_pair;

pub fn run_diff(src: &str, dst: &str, algorithm: &str) {
    // Initialize stores for each iteration
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    // Parse the two Java files
    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));

    // Perform the diff using specified algorithm
    let diff_result = match algorithm {
        "gumtree_lazy" => algorithms::gumtree_lazy::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        // "change_distiller" => algorithms::change_distiller_optimized::diff_baseline(
        //     &stores,
        //     &src_tr.local.compressed_node,
        //     &dst_tr.local.compressed_node,
        // ),
        // "change_distiller_lazy" => {
        //     algorithms::change_distiller_optimized::diff_with_all_optimizations(
        //         &stores,
        //         &src_tr.local.compressed_node,
        //         &dst_tr.local.compressed_node,
        //     )
        // }
        // "change_distiller_lazy_2" => algorithms::change_distiller_lazy_2::diff(
        //     &stores,
        //     &src_tr.local.compressed_node,
        //     &dst_tr.local.compressed_node,
        // ),
        _ => panic!("Unknown diff algorithm"),
    };

    black_box(diff_result);
}

/// Define the test cases with their paths relative to root/../datasets/defects4j/<before|after>/
/// ~100 loc
const TEST_CASES_S: &[&str] = &[
    "Mockito/31/src_org_mockito_internal_stubbing_defaultanswers_ReturnsSmartNulls.java",
    "Mockito/32/src_org_mockito_internal_configuration_SpyAnnotationEngine.java",
    "Mockito/34/src_org_mockito_internal_invocation_InvocationMatcher.java",
    "Mockito/37/src_org_mockito_internal_stubbing_answers_AnswersValidator.java",
    "Mockito/38/src_org_mockito_internal_verification_argumentmatching_ArgumentMatchingTool.java",
    "Mockito/9/src_org_mockito_internal_stubbing_answers_CallsRealMethods.java",
    "Time/26/src_main_java_org_joda_time_field_LenientDateTimeField.java",
    "Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
    "Cli/12/src_java_org_apache_commons_cli_GnuParser.java",
    "Cli/13/src_java_org_apache_commons_cli2_WriteableCommandLine.java",
    "Cli/21/src_java_org_apache_commons_cli2_WriteableCommandLine.java",
    "Cli/29/src_java_org_apache_commons_cli_Util.java",
    "JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThan.java",
    "JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThanOrEqual.java",
    "Jsoup/18/src_main_java_org_jsoup_parser_CharacterReader.java",
    "Jsoup/20/src_main_java_org_jsoup_helper_DataUtil.java",
    "JxPath/11/src_java_org_apache_commons_jxpath_ri_model_dom_DOMAttributeIterator.java",
    "JxPath/11/src_java_org_apache_commons_jxpath_ri_model_jdom_JDOMAttributeIterator.java",
    "JxPath/13/src_java_org_apache_commons_jxpath_ri_NamespaceResolver.java",
    "JxPath/17/src_java_org_apache_commons_jxpath_ri_model_dom_DOMAttributeIterator.java",
    "Lang/17/src_main_java_org_apache_commons_lang3_text_translate_CharSequenceTranslator.java",
    "Lang/6/src_main_java_org_apache_commons_lang3_text_translate_CharSequenceTranslator.java",
    "Lang/64/src_java_org_apache_commons_lang_enums_ValuedEnum.java",
    "Math/103/src_java_org_apache_commons_math_distribution_NormalDistributionImpl.java",
    "Math/106/src_java_org_apache_commons_math_fraction_ProperFractionFormat.java",
    "Math/12/src_main_java_org_apache_commons_math3_random_BitsStreamGenerator.java",
    "Math/21/src_main_java_org_apache_commons_math3_linear_RectangularCholeskyDecomposition.java",
    "Time/1/src_main_java_org_joda_time_field_UnsupportedDurationField.java",
    "Time/2/src_main_java_org_joda_time_field_UnsupportedDurationField.java",
    "Chart/20/source_org_jfree_chart_plot_ValueMarker.java",
    "Chart/24/source_org_jfree_chart_renderer_GrayPaintScale.java",
    "Chart/6/source_org_jfree_chart_util_ShapeList.java",
    "Cli/13/src_java_org_apache_commons_cli2_commandline_WriteableCommandLineImpl.java",
    "Cli/16/src_java_org_apache_commons_cli2_Option.java",
    "Cli/16/src_java_org_apache_commons_cli2_option_OptionImpl.java",
    "Gson/17/gson_src_main_java_com_google_gson_DefaultDateTypeAdapter.java",
    "Gson/3/gson_src_main_java_com_google_gson_internal_ConstructorConstructor.java",
    "Gson/9/gson_src_main_java_com_google_gson_internal_bind_JsonTreeWriter.java",
    "JacksonCore/13/src_main_java_com_fasterxml_jackson_core_json_JsonGeneratorImpl.java",
    "JacksonCore/16/src_main_java_com_fasterxml_jackson_core_util_JsonParserSequence.java",
    "JacksonCore/7/src_main_java_com_fasterxml_jackson_core_json_JsonWriteContext.java",
    "JacksonDatabind/109/src_main_java_com_fasterxml_jackson_databind_ser_std_NumberSerializer.java",
    "JacksonDatabind/110/src_main_java_com_fasterxml_jackson_databind_deser_impl_JavaUtilCollectionsDeserializers.java",
    "Cli/5/src_java_org_apache_commons_cli_Util.java",
    "Closure/143/src_com_google_javascript_jscomp_RemoveConstantExpressions.java",
    "Closure/165/src_com_google_javascript_rhino_jstype_RecordTypeBuilder.java",
    "Closure/174/src_com_google_javascript_jscomp_JsAst.java",
    "Closure/28/src_com_google_javascript_jscomp_InlineCostEstimator.java",
    "Compress/33/src_main_java_org_apache_commons_compress_compressors_deflate_DeflateCompressorInputStream.java",
    "Compress/40/src_main_java_org_apache_commons_compress_utils_BitInputStream.java",
    "Compress/42/src_main_java_org_apache_commons_compress_archivers_zip_UnixStat.java",
    "Compress/44/src_main_java_org_apache_commons_compress_utils_ChecksumCalculatingInputStream.java",
    "Gson/6/gson_src_main_java_com_google_gson_internal_bind_JsonAdapterAnnotationTypeAdapterFactory.java",
    "Gson/8/gson_src_main_java_com_google_gson_internal_UnsafeAllocator.java",
    "JacksonDatabind/10/src_main_java_com_fasterxml_jackson_databind_ser_AnyGetterWriter.java",
    "JacksonDatabind/105/src_main_java_com_fasterxml_jackson_databind_deser_std_JdkDeserializers.java",
    "JacksonDatabind/111/src_main_java_com_fasterxml_jackson_databind_deser_std_AtomicReferenceDeserializer.java",
    "JacksonDatabind/13/src_main_java_com_fasterxml_jackson_databind_deser_impl_ObjectIdValueProperty.java",
    "JacksonDatabind/16/src_main_java_com_fasterxml_jackson_databind_introspect_AnnotationMap.java",
    "JacksonDatabind/25/src_main_java_com_fasterxml_jackson_databind_module_SimpleAbstractTypeResolver.java",
    "JacksonDatabind/34/src_main_java_com_fasterxml_jackson_databind_ser_std_NumberSerializer.java",
    "JacksonDatabind/39/src_main_java_com_fasterxml_jackson_databind_deser_std_NullifyingDeserializer.java",
    "JacksonDatabind/43/src_main_java_com_fasterxml_jackson_databind_deser_impl_ObjectIdValueProperty.java",
    "JacksonDatabind/49/src_main_java_com_fasterxml_jackson_databind_ser_impl_WritableObjectId.java",
    "JacksonDatabind/79/src_main_java_com_fasterxml_jackson_databind_introspect_ObjectIdInfo.java",
    "JxPath/9/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationNotEqual.java",
    "Lang/19/src_main_java_org_apache_commons_lang3_text_translate_NumericEntityUnescaper.java",
    "Lang/28/src_main_java_org_apache_commons_lang3_text_translate_NumericEntityUnescaper.java",
    "Lang/4/src_main_java_org_apache_commons_lang3_text_translate_LookupTranslator.java",
    "Math/14/src_main_java_org_apache_commons_math3_optim_nonlinear_vector_Weight.java",
    "Math/70/src_main_java_org_apache_commons_math_analysis_solvers_BisectionSolver.java",
    "Mockito/11/src_org_mockito_internal_creation_DelegatingMethod.java",
    "Mockito/12/src_org_mockito_internal_util_reflection_GenericMaster.java",
    "Mockito/15/src_org_mockito_internal_configuration_injection_FinalMockCandidateFilter.java",
    "Mockito/17/src_org_mockito_internal_creation_MockSettingsImpl.java",
    "Mockito/17/src_org_mockito_internal_util_MockUtil.java",
    "Mockito/19/src_org_mockito_internal_configuration_injection_filter_FinalMockCandidateFilter.java",
    "Mockito/19/src_org_mockito_internal_configuration_injection_filter_MockCandidateFilter.java",
    "Mockito/19/src_org_mockito_internal_configuration_injection_filter_NameBasedCandidateFilter.java",
    "Mockito/19/src_org_mockito_internal_configuration_injection_filter_TypeBasedCandidateFilter.java",
    "Mockito/2/src_org_mockito_internal_util_Timer.java",
    "Mockito/21/src_org_mockito_internal_creation_instance_ConstructorInstantiator.java",
    "Mockito/22/src_org_mockito_internal_matchers_Equality.java",
    "Mockito/25/src_org_mockito_internal_stubbing_defaultanswers_ReturnsDeepStubs.java",
    "Mockito/26/src_org_mockito_internal_util_Primitives.java",
    "Mockito/29/src_org_mockito_internal_matchers_Same.java",
    "Mockito/30/src_org_mockito_internal_stubbing_defaultanswers_ReturnsSmartNulls.java",
    "Closure/124/src_com_google_javascript_jscomp_ExploitAssigns.java",
    "Closure/129/src_com_google_javascript_jscomp_PrepareAst.java",
    "Closure/13/src_com_google_javascript_jscomp_PeepholeOptimizationsPass.java",
    "Closure/144/src_com_google_javascript_rhino_jstype_FunctionBuilder.java",
    "Closure/147/src_com_google_javascript_jscomp_CheckGlobalThis.java",
    "Closure/153/src_com_google_javascript_jscomp_SyntacticScopeCreator.java",
    "Closure/158/src_com_google_javascript_jscomp_DiagnosticGroups.java",
    "Closure/163/src_com_google_javascript_jscomp_CrossModuleMethodMotion.java",
    "Closure/21/src_com_google_javascript_jscomp_CheckSideEffects.java",
    "Closure/22/src_com_google_javascript_jscomp_CheckSideEffects.java",
    "Closure/38/src_com_google_javascript_jscomp_CodeConsumer.java",
    "Closure/44/src_com_google_javascript_jscomp_CodeConsumer.java",
    "Closure/47/src_com_google_javascript_jscomp_SourceMap.java",
    "Closure/51/src_com_google_javascript_jscomp_CodeConsumer.java",
];

const TEST_CASES_M: &[&str] = &[
    "Math/55/src_main_java_org_apache_commons_math_geometry_Vector3D.java",
    "Math/7/src_main_java_org_apache_commons_math3_ode_AbstractIntegrator.java",
    "Math/75/src_main_java_org_apache_commons_math_stat_Frequency.java",
    "Math/87/src_java_org_apache_commons_math_optimization_linear_SimplexTableau.java",
    "Math/88/src_java_org_apache_commons_math_optimization_linear_SimplexTableau.java",
    "Math/91/src_java_org_apache_commons_math_fraction_Fraction.java",
    "Time/24/src_main_java_org_joda_time_format_DateTimeParserBucket.java",
    "Chart/12/source_org_jfree_chart_plot_MultiplePiePlot.java",
    "Chart/13/source_org_jfree_chart_block_BorderArrangement.java",
    "Chart/25/source_org_jfree_chart_renderer_category_StatisticalBarRenderer.java",
    "Chart/8/source_org_jfree_data_time_Week.java",
    "Cli/30/src_main_java_org_apache_commons_cli_DefaultParser.java",
    "Cli/37/src_main_java_org_apache_commons_cli_DefaultParser.java",
    "Cli/38/src_main_java_org_apache_commons_cli_DefaultParser.java",
    "Closure/106/src_com_google_javascript_rhino_JSDocInfoBuilder.java",
    "Closure/108/src_com_google_javascript_jscomp_ScopedAliases.java",
    "Closure/110/src_com_google_javascript_jscomp_ScopedAliases.java",
    "Closure/67/src_com_google_javascript_jscomp_AnalyzePrototypeProperties.java",
    "Closure/7/src_com_google_javascript_jscomp_type_ChainableReverseAbstractInterpreter.java",
    "Closure/71/src_com_google_javascript_jscomp_CheckAccessControls.java",
    "Closure/83/src_com_google_javascript_jscomp_CommandLineRunner.java",
    "Codec/14/src_main_java_org_apache_commons_codec_language_bm_PhoneticEngine.java",
    "Collections/27/src_main_java_org_apache_commons_collections4_map_MultiValueMap.java",
    "Compress/13/src_main_java_org_apache_commons_compress_archivers_zip_ZipArchiveEntry.java",
    "Compress/15/src_main_java_org_apache_commons_compress_archivers_zip_ZipArchiveEntry.java",
    "Compress/17/src_main_java_org_apache_commons_compress_archivers_tar_TarUtils.java",
    "Compress/35/src_main_java_org_apache_commons_compress_archivers_tar_TarUtils.java",
    "Compress/45/src_main_java_org_apache_commons_compress_archivers_tar_TarUtils.java",
    "Compress/46/src_main_java_org_apache_commons_compress_archivers_zip_X5455_ExtendedTimestamp.java",
    "Csv/16/src_main_java_org_apache_commons_csv_CSVParser.java",
    "Gson/14/gson_src_main_java_com_google_gson_internal_$Gson$Types.java",
    "Gson/16/gson_src_main_java_com_google_gson_internal_$Gson$Types.java",
    "Gson/18/gson_src_main_java_com_google_gson_internal_$Gson$Types.java",
    "JacksonCore/1/src_main_java_com_fasterxml_jackson_core_util_TextBuffer.java",
    "JacksonCore/4/src_main_java_com_fasterxml_jackson_core_util_TextBuffer.java",
    "JacksonCore/8/src_main_java_com_fasterxml_jackson_core_util_TextBuffer.java",
    "JacksonDatabind/1/src_main_java_com_fasterxml_jackson_databind_ser_BeanPropertyWriter.java",
    "JacksonDatabind/103/src_main_java_com_fasterxml_jackson_databind_ser_DefaultSerializerProvider.java",
    "JacksonDatabind/20/src_main_java_com_fasterxml_jackson_databind_node_ObjectNode.java",
    "JacksonDatabind/65/src_main_java_com_fasterxml_jackson_databind_introspect_BasicBeanDescription.java",
    "JacksonDatabind/87/src_main_java_com_fasterxml_jackson_databind_util_StdDateFormat.java",
    "Jsoup/33/src_main_java_org_jsoup_parser_HtmlTreeBuilder.java",
    "Jsoup/49/src_main_java_org_jsoup_nodes_Node.java",
    "Jsoup/71/src_main_java_org_jsoup_select_Evaluator.java",
    "JxPath/12/src_java_org_apache_commons_jxpath_ri_model_dom_DOMNodePointer.java",
    "JxPath/5/src_java_org_apache_commons_jxpath_ri_model_NodePointer.java",
    "Lang/44/src_java_org_apache_commons_lang_NumberUtils.java",
    "Lang/63/src_java_org_apache_commons_lang_time_DurationFormatUtils.java",
    "Math/1/src_main_java_org_apache_commons_math3_fraction_Fraction.java",
];

const TEST_CASES_L: &[&str] = &[
    "Math/16/src_main_java_org_apache_commons_math3_util_FastMath.java",
    "JacksonDatabind/17/src_main_java_com_fasterxml_jackson_databind_ObjectMapper.java",
    "Math/15/src_main_java_org_apache_commons_math3_util_FastMath.java",
    "JacksonCore/12/src_main_java_com_fasterxml_jackson_core_json_UTF8StreamJsonParser.java",
    "JacksonCore/9/src_main_java_com_fasterxml_jackson_core_json_UTF8StreamJsonParser.java",
    "JacksonCore/19/src_main_java_com_fasterxml_jackson_core_json_UTF8StreamJsonParser.java",
    "Chart/19/source_org_jfree_chart_plot_CategoryPlot.java",
    "Chart/14/source_org_jfree_chart_plot_CategoryPlot.java",
    "JacksonDatabind/30/src_main_java_com_fasterxml_jackson_databind_ObjectMapper.java",
    "JacksonDatabind/61/src_main_java_com_fasterxml_jackson_databind_ObjectMapper.java",
    "Lang/37/src_java_org_apache_commons_lang3_ArrayUtils.java",
    "Chart/14/source_org_jfree_chart_plot_XYPlot.java",
    "Lang/35/src_main_java_org_apache_commons_lang3_ArrayUtils.java",
    "Chart/4/source_org_jfree_chart_plot_XYPlot.java",
    "Lang/40/src_java_org_apache_commons_lang_StringUtils.java",
    "Lang/39/src_java_org_apache_commons_lang3_StringUtils.java",
    "Lang/31/src_main_java_org_apache_commons_lang3_StringUtils.java",
    "Lang/30/src_main_java_org_apache_commons_lang3_StringUtils.java",
    "Lang/20/src_main_java_org_apache_commons_lang3_StringUtils.java",
    "Lang/14/src_main_java_org_apache_commons_lang3_StringUtils.java",
    "Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
    "Cli/12/src_java_org_apache_commons_cli_GnuParser.java",
    "Cli/13/src_java_org_apache_commons_cli2_WriteableCommandLine.java",
];

pub fn get_test_data_small() -> Vec<(String, String)> {
    get_test_data(&TEST_CASES_S[0..101])
}

pub fn get_test_data_medium() -> Vec<(String, String)> {
    get_test_data(&TEST_CASES_M[0..4])
}

pub fn get_test_data_large() -> Vec<(String, String)> {
    get_test_data(&TEST_CASES_L[0..4])
}

pub fn get_test_data_mixed() -> Vec<(String, String)> {
    let mixed = TEST_CASES_S[0..0]
        .iter()
        .chain(TEST_CASES_M[11..49].iter())
        // .chain(TEST_CASES_L[0..22].iter())
        .cloned()
        .collect::<Vec<_>>();
    println!("Mixed test data size: {}", mixed.len());
    get_test_data(&mixed)
}

fn get_test_data<'a>(data: &[&str]) -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let test_inputs: Vec<_> = data
        .iter()
        .map(|path_rel| {
            let buggy_path = root.join("before").join(path_rel);
            let fixed_path = root.join("after").join(path_rel);

            // Read file contents
            let buggy_content = std::fs::read_to_string(&buggy_path)
                .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
            let fixed_content = std::fs::read_to_string(&fixed_path)
                .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

            (buggy_content, fixed_content)
        })
        .collect();
    test_inputs
}

/// This function examines the defect4j dataset and extracts all file changes sorted by file size
/// It returns a vector of tuples containing the relative paths of the before and after files.
pub fn get_all_case_paths() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    // Collect all file paths in "before" recursively
    let mut file_paths = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("before"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && e.file_name().to_string_lossy().ends_with(".java"))
    {
        file_paths.push(
            entry
                .path()
                .strip_prefix(root.join("before"))
                .unwrap()
                .to_owned(),
        );
    }

    // Sort by file size (smallest first)
    file_paths.sort_by_key(|rel_path| {
        let abs_path = root.join("before").join(rel_path);
        std::fs::metadata(&abs_path).map(|m| m.len()).unwrap_or(0)
    });

    file_paths
        .iter()
        .map(|path| {
            let before_path = root.join("before").join(&path);
            let after_path = root.join("after").join(&path);

            (
                before_path.to_string_lossy().into_owned(),
                after_path.to_string_lossy().into_owned(),
            )
        })
        .collect()
}

/// Given a list of relative paths, reads the before/after file contents and returns them as pairs
pub fn get_all_cases_from_paths(paths: &[(String, String)]) -> Vec<(String, String)> {
    let test_inputs: Vec<_> = paths
        .iter()
        .map(|(buggy_path, fixed_path)| {
            // Read file contents
            let buggy_content = std::fs::read_to_string(&buggy_path)
                .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
            let fixed_content = std::fs::read_to_string(&fixed_path)
                .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

            (buggy_content, fixed_content)
        })
        .collect();
    test_inputs
}

pub fn get_all_cases() -> Vec<(String, String)> {
    let paths = get_all_case_paths();
    get_all_cases_from_paths(&paths)
}

use tabled::{Table, Tabled};

#[derive(Tabled)]
struct CaseStats {
    #[tabled(rename = "Index")]
    index: usize,
    #[tabled(rename = "Src LOC")]
    src_loc: usize,
    #[tabled(rename = "Dst LOC")]
    dst_loc: usize,
    #[tabled(rename = "Nodes")]
    nodes: usize,
}

pub fn print_test_case_table(test_inputs: &Vec<(String, String)>) {
    use crate::preprocess::parse_string_pair;
    use hyperast::store::SimpleStores;

    let mut rows = Vec::new();
    let mut src_locs = Vec::new();
    let mut dst_locs = Vec::new();
    let mut nodes = Vec::new();

    for (i, (src, dst)) in test_inputs.iter().enumerate() {
        let src_loc = src.lines().count();
        let dst_loc = dst.lines().count();

        let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();
        let _ = parse_string_pair(&mut stores, &mut md_cache, src, dst);
        let nodes_count = stores.node_store.len();

        rows.push(CaseStats {
            index: i,
            src_loc,
            dst_loc,
            nodes: nodes_count,
        });

        src_locs.push(src_loc);
        dst_locs.push(dst_loc);
        nodes.push(nodes_count);
    }

    // Print table
    let table = Table::new(&rows).to_string();
    println!("{}", table);

    // Print summary statistics
    let avg = |v: &[usize]| v.iter().copied().sum::<usize>() as f64 / v.len() as f64;
    let min = |v: &[usize]| *v.iter().min().unwrap_or(&0);
    let max = |v: &[usize]| *v.iter().max().unwrap_or(&0);

    println!("\nSummary statistics:");
    println!(
        "  Src LOC:    avg {:.1}, min {}, max {}",
        avg(&src_locs),
        min(&src_locs),
        max(&src_locs)
    );
    println!(
        "  Dst LOC:    avg {:.1}, min {}, max {}",
        avg(&dst_locs),
        min(&dst_locs),
        max(&dst_locs)
    );

    println!(
        "  Nodes:      avg {:.1}, min {}, max {}",
        avg(&nodes),
        min(&nodes),
        max(&nodes)
    );

    println!("  Total Nodes: {}", nodes.iter().sum::<usize>());
}

pub struct Input {
    pub stores: SimpleStores<hyperast_gen_ts_java::types::TStore>,
    pub src: hyperast_gen_ts_java::legion_with_refs::NodeIdentifier,
    pub dst: hyperast_gen_ts_java::legion_with_refs::NodeIdentifier,
    pub loc: usize,
    pub node_count: usize,
}

pub fn preprocess(input: &(String, String)) -> Input {
    let (src, dst) = input;
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, src, dst);
    let loc = max(src.lines().count(), dst.lines().count());
    let node_count = stores.node_store().len();

    Input {
        stores,
        src: src_tr.local.compressed_node,
        dst: dst_tr.local.compressed_node,
        loc,
        node_count,
    }
}
