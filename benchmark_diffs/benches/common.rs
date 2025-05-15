use criterion::black_box;
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::path::Path;

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
        "change_distiller" => algorithms::change_distiller::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        "change_distiller_lazy" => algorithms::change_distiller_lazy::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        _ => panic!("Unknown diff algorithm"),
    };

    black_box(diff_result);
}

// Define the test cases with their paths relative to root/../datasets/defects4j
const TEST_CASES: &[&str] = &[
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
    "Jsoup/23/src_main_java_org_jsoup_parser_CharacterReader.java",
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
    "Chart/.DS_Store",
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
    "JacksonDatabind/109/src_main_java_com_fasterxml_jackson_databind_ser_std_NumberSerializer.java",
    "JacksonDatabind/110/src_main_java_com_fasterxml_jackson_databind_deser_impl_JavaUtilCollectionsDeserializers.java",
];

pub fn get_test_data() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let test_inputs: Vec<_> = TEST_CASES
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
