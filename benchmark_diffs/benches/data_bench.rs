// Taken from LEO!!
use std::path::Path;

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

pub fn get_test_data_small() -> Vec<(String, String, String)> {
    get_test_data(&TEST_CASES_S[0..14])
}

pub fn get_test_data_medium() -> Vec<(String, String, String)> {
    get_test_data(&TEST_CASES_M[0..4])
}

pub fn get_test_data_large() -> Vec<(String, String, String)> {
    get_test_data(&TEST_CASES_L[0..4])
}

pub fn get_test_data_mixed() -> Vec<(String, String, String)> {
    let mixed = TEST_CASES_S[0..10]
        .iter()
        .chain(TEST_CASES_M[0..2].iter())
        .chain(TEST_CASES_L[0..2].iter())
        .cloned()
        .collect::<Vec<_>>();
    println!("Mixed test data size: {}", mixed.len());
    get_test_data(&mixed)
}

fn get_test_data<'a>(data: &[&str]) -> Vec<(String, String, String)> {
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

            // Get name of fix
            let name = path_rel
                .rsplit("/")
                .nth(1)
                .expect(&format!(
                    "Expected at least 2 path separators, got: {:?}",
                    path_rel
                ))
                .to_string();

            // Read file contents
            let buggy_content = std::fs::read_to_string(&buggy_path)
                .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
            let fixed_content = std::fs::read_to_string(&fixed_path)
                .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

            (name, buggy_content, fixed_content)
        })
        .collect();
    test_inputs
}
