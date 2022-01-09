enum TSTypeLiteral {
    TSFieldBinaryIntegerLiteral(TSTypeBinaryIntegerLiteral),
    TSFieldCharacterLiteral(TSTypeCharacterLiteral),
    TSFieldDecimalFloatingPointLiteral(TSTypeDecimalFloatingPointLiteral),
    TSFieldDecimalIntegerLiteral(TSTypeDecimalIntegerLiteral),
    TSFieldFalse(TSTypeFalse),
    TSFieldHexFloatingPointLiteral(TSTypeHexFloatingPointLiteral),
    TSFieldHexIntegerLiteral(TSTypeHexIntegerLiteral),
    TSFieldNullLiteral(TSTypeNullLiteral),
    TSFieldOctalIntegerLiteral(TSTypeOctalIntegerLiteral),
    TSFieldStringLiteral(TSTypeStringLiteral),
    TSFieldTrue(TSTypeTrue),
}
impl Display for TSTypeLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldBinaryIntegerLiteral(TSTypeBinaryIntegerLiteral) => x.fmt(f),
            TSFieldCharacterLiteral(TSTypeCharacterLiteral) => x.fmt(f),
            TSFieldDecimalFloatingPointLiteral(TSTypeDecimalFloatingPointLiteral) => x.fmt(f),
            TSFieldDecimalIntegerLiteral(TSTypeDecimalIntegerLiteral) => x.fmt(f),
            TSFieldFalse(TSTypeFalse) => x.fmt(f),
            TSFieldHexFloatingPointLiteral(TSTypeHexFloatingPointLiteral) => x.fmt(f),
            TSFieldHexIntegerLiteral(TSTypeHexIntegerLiteral) => x.fmt(f),
            TSFieldNullLiteral(TSTypeNullLiteral) => x.fmt(f),
            TSFieldOctalIntegerLiteral(TSTypeOctalIntegerLiteral) => x.fmt(f),
            TSFieldStringLiteral(TSTypeStringLiteral) => x.fmt(f),
            TSFieldTrue(TSTypeTrue) => x.fmt(f),
        }
    }
}
enum TSTypeSimpleType {
    TSFieldBooleanType(TSTypeBooleanType),
    TSFieldFloatingPointType(TSTypeFloatingPointType),
    TSFieldGenericType(TSTypeGenericType),
    TSFieldIntegralType(TSTypeIntegralType),
    TSFieldScopedTypeIdentifier(TSTypeScopedTypeIdentifier),
    TSFieldTypeIdentifier(TSTypeTypeIdentifier),
    TSFieldVoidType(TSTypeVoidType),
}
impl Display for TSTypeSimpleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldBooleanType(TSTypeBooleanType) => x.fmt(f),
            TSFieldFloatingPointType(TSTypeFloatingPointType) => x.fmt(f),
            TSFieldGenericType(TSTypeGenericType) => x.fmt(f),
            TSFieldIntegralType(TSTypeIntegralType) => x.fmt(f),
            TSFieldScopedTypeIdentifier(TSTypeScopedTypeIdentifier) => x.fmt(f),
            TSFieldTypeIdentifier(TSTypeTypeIdentifier) => x.fmt(f),
            TSFieldVoidType(TSTypeVoidType) => x.fmt(f),
        }
    }
}
enum TSTypeType {
    TSFieldUnannotatedType(TSTypeUnannotatedType),
    TSFieldAnnotatedType(TSTypeAnnotatedType),
}
impl Display for TSTypeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldUnannotatedType(TSTypeUnannotatedType) => x.fmt(f),
            TSFieldAnnotatedType(TSTypeAnnotatedType) => x.fmt(f),
        }
    }
}
enum TSTypeUnannotatedType {
    TSFieldSimpleType(TSTypeSimpleType),
    TSFieldArrayType(TSTypeArrayType),
}
impl Display for TSTypeUnannotatedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldSimpleType(TSTypeSimpleType) => x.fmt(f),
            TSFieldArrayType(TSTypeArrayType) => x.fmt(f),
        }
    }
}
enum TSTypeUnaryExp {
    TSFieldCastExpression(TSTypeCastExpression),
    TSFieldPrimaryExpression(TSTypePrimaryExpression),
    TSFieldSwitchExpression(TSTypeSwitchExpression),
    TSFieldUnaryExpression(TSTypeUnaryExpression),
    TSFieldUpdateExpression(TSTypeUpdateExpression),
}
impl Display for TSTypeUnaryExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldCastExpression(TSTypeCastExpression) => x.fmt(f),
            TSFieldPrimaryExpression(TSTypePrimaryExpression) => x.fmt(f),
            TSFieldSwitchExpression(TSTypeSwitchExpression) => x.fmt(f),
            TSFieldUnaryExpression(TSTypeUnaryExpression) => x.fmt(f),
            TSFieldUpdateExpression(TSTypeUpdateExpression) => x.fmt(f),
        }
    }
}
enum TSTypeDeclaration {
    TSFieldAnnotationTypeDeclaration(TSTypeAnnotationTypeDeclaration),
    TSFieldClassDeclaration(TSTypeClassDeclaration),
    TSFieldEnumDeclaration(TSTypeEnumDeclaration),
    TSFieldImportDeclaration(TSTypeImportDeclaration),
    TSFieldInterfaceDeclaration(TSTypeInterfaceDeclaration),
    TSFieldModuleDeclaration(TSTypeModuleDeclaration),
    TSFieldPackageDeclaration(TSTypePackageDeclaration),
}
impl Display for TSTypeDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldAnnotationTypeDeclaration(TSTypeAnnotationTypeDeclaration) => x.fmt(f),
            TSFieldClassDeclaration(TSTypeClassDeclaration) => x.fmt(f),
            TSFieldEnumDeclaration(TSTypeEnumDeclaration) => x.fmt(f),
            TSFieldImportDeclaration(TSTypeImportDeclaration) => x.fmt(f),
            TSFieldInterfaceDeclaration(TSTypeInterfaceDeclaration) => x.fmt(f),
            TSFieldModuleDeclaration(TSTypeModuleDeclaration) => x.fmt(f),
            TSFieldPackageDeclaration(TSTypePackageDeclaration) => x.fmt(f),
        }
    }
}
enum TSTypeExpression {
    TSFieldUnaryExp(TSTypeUnaryExp),
    TSFieldAssignmentExpression(TSTypeAssignmentExpression),
    TSFieldBinaryExpression(TSTypeBinaryExpression),
    TSFieldInstanceofExpression(TSTypeInstanceofExpression),
    TSFieldLambdaExpression(TSTypeLambdaExpression),
    TSFieldTernaryExpression(TSTypeTernaryExpression),
}
impl Display for TSTypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldUnaryExp(TSTypeUnaryExp) => x.fmt(f),
            TSFieldAssignmentExpression(TSTypeAssignmentExpression) => x.fmt(f),
            TSFieldBinaryExpression(TSTypeBinaryExpression) => x.fmt(f),
            TSFieldInstanceofExpression(TSTypeInstanceofExpression) => x.fmt(f),
            TSFieldLambdaExpression(TSTypeLambdaExpression) => x.fmt(f),
            TSFieldTernaryExpression(TSTypeTernaryExpression) => x.fmt(f),
        }
    }
}
enum TSTypePrimaryExpression {
    TSFieldLiteral(TSTypeLiteral),
    TSFieldArrayAccess(TSTypeArrayAccess),
    TSFieldArrayCreationExpression(TSTypeArrayCreationExpression),
    TSFieldClassLiteral(TSTypeClassLiteral),
    TSFieldFieldAccess(TSTypeFieldAccess),
    TSFieldIdentifier(TSTypeIdentifier),
    TSFieldMethodInvocation(TSTypeMethodInvocation),
    TSFieldMethodReference(TSTypeMethodReference),
    TSFieldObjectCreationExpression(TSTypeObjectCreationExpression),
    TSFieldParenthesizedExpression(TSTypeParenthesizedExpression),
    TSFieldThis(TSTypeThis),
}
impl Display for TSTypePrimaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSFieldLiteral(TSTypeLiteral) => x.fmt(f),
            TSFieldArrayAccess(TSTypeArrayAccess) => x.fmt(f),
            TSFieldArrayCreationExpression(TSTypeArrayCreationExpression) => x.fmt(f),
            TSFieldClassLiteral(TSTypeClassLiteral) => x.fmt(f),
            TSFieldFieldAccess(TSTypeFieldAccess) => x.fmt(f),
            TSFieldIdentifier(TSTypeIdentifier) => x.fmt(f),
            TSFieldMethodInvocation(TSTypeMethodInvocation) => x.fmt(f),
            TSFieldMethodReference(TSTypeMethodReference) => x.fmt(f),
            TSFieldObjectCreationExpression(TSTypeObjectCreationExpression) => x.fmt(f),
            TSFieldParenthesizedExpression(TSTypeParenthesizedExpression) => x.fmt(f),
            TSFieldThis(TSTypeThis) => x.fmt(f),
        }
    }
}
enum TSTypeStatement {
    TSField0(TSType0),
    TSFieldAssertStatement(TSTypeAssertStatement),
    TSFieldBlock(TSTypeBlock),
    TSFieldBreakStatement(TSTypeBreakStatement),
    TSFieldContinueStatement(TSTypeContinueStatement),
    TSFieldDeclaration(TSTypeDeclaration),
    TSFieldDoStatement(TSTypeDoStatement),
    TSFieldEnhancedForStatement(TSTypeEnhancedForStatement),
    TSFieldExpressionStatement(TSTypeExpressionStatement),
    TSFieldForStatement(TSTypeForStatement),
    TSFieldIfStatement(TSTypeIfStatement),
    TSFieldLabeledStatement(TSTypeLabeledStatement),
    TSFieldLocalVariableDeclaration(TSTypeLocalVariableDeclaration),
    TSFieldReturnStatement(TSTypeReturnStatement),
    TSFieldSwitchStatement(TSTypeSwitchStatement),
    TSFieldSynchronizedStatement(TSTypeSynchronizedStatement),
    TSFieldThrowStatement(TSTypeThrowStatement),
    TSFieldTryStatement(TSTypeTryStatement),
    TSFieldTryWithResourcesStatement(TSTypeTryWithResourcesStatement),
    TSFieldWhileStatement(TSTypeWhileStatement),
    TSFieldYieldStatement(TSTypeYieldStatement),
}
impl Display for TSTypeStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSField0(TSType0) => x.fmt(f),
            TSFieldAssertStatement(TSTypeAssertStatement) => x.fmt(f),
            TSFieldBlock(TSTypeBlock) => x.fmt(f),
            TSFieldBreakStatement(TSTypeBreakStatement) => x.fmt(f),
            TSFieldContinueStatement(TSTypeContinueStatement) => x.fmt(f),
            TSFieldDeclaration(TSTypeDeclaration) => x.fmt(f),
            TSFieldDoStatement(TSTypeDoStatement) => x.fmt(f),
            TSFieldEnhancedForStatement(TSTypeEnhancedForStatement) => x.fmt(f),
            TSFieldExpressionStatement(TSTypeExpressionStatement) => x.fmt(f),
            TSFieldForStatement(TSTypeForStatement) => x.fmt(f),
            TSFieldIfStatement(TSTypeIfStatement) => x.fmt(f),
            TSFieldLabeledStatement(TSTypeLabeledStatement) => x.fmt(f),
            TSFieldLocalVariableDeclaration(TSTypeLocalVariableDeclaration) => x.fmt(f),
            TSFieldReturnStatement(TSTypeReturnStatement) => x.fmt(f),
            TSFieldSwitchStatement(TSTypeSwitchStatement) => x.fmt(f),
            TSFieldSynchronizedStatement(TSTypeSynchronizedStatement) => x.fmt(f),
            TSFieldThrowStatement(TSTypeThrowStatement) => x.fmt(f),
            TSFieldTryStatement(TSTypeTryStatement) => x.fmt(f),
            TSFieldTryWithResourcesStatement(TSTypeTryWithResourcesStatement) => x.fmt(f),
            TSFieldWhileStatement(TSTypeWhileStatement) => x.fmt(f),
            TSFieldYieldStatement(TSTypeYieldStatement) => x.fmt(f),
        }
    }
}
enum TSChildrenAnnotatedType {
    TSTypeUnannotatedType(TSTypeUnannotatedType),
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeAnnotatedType {
    _children: Vec<TSChildrenAnnotatedType>,
}
impl Display for TSTypeAnnotatedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeAnnotation {
    TSFieldArguments: TSFieldAnnotationArguments,
    TSFieldName: TSFieldAnnotationName,
}
impl Display for TSTypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenAnnotationArgumentList {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeElementValueArrayInitializer(TSTypeElementValueArrayInitializer),
    TSTypeElementValuePair(TSTypeElementValuePair),
    TSTypeExpression(TSTypeExpression),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeAnnotationArgumentList {
    _children: Option<Vec<TSChildrenAnnotationArgumentList>>,
}
impl Display for TSTypeAnnotationArgumentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenAnnotationTypeBody {
    TSTypeAnnotationTypeDeclaration(TSTypeAnnotationTypeDeclaration),
    TSTypeAnnotationTypeElementDeclaration(TSTypeAnnotationTypeElementDeclaration),
    TSTypeClassDeclaration(TSTypeClassDeclaration),
    TSTypeConstantDeclaration(TSTypeConstantDeclaration),
    TSTypeEnumDeclaration(TSTypeEnumDeclaration),
    TSTypeInterfaceDeclaration(TSTypeInterfaceDeclaration),
}
struct TSTypeAnnotationTypeBody {
    _children: Option<Vec<TSChildrenAnnotationTypeBody>>,
}
impl Display for TSTypeAnnotationTypeBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenAnnotationTypeDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeAnnotationTypeDeclaration {
    _children: Option<TSChildrenAnnotationTypeDeclaration>,
    TSFieldBody: TSFieldAnnotationTypeDeclarationBody,
    TSFieldName: TSFieldAnnotationTypeDeclarationName,
}
impl Display for TSTypeAnnotationTypeDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenAnnotationTypeElementDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeAnnotationTypeElementDeclaration {
    _children: Option<TSChildrenAnnotationTypeElementDeclaration>,
    TSFieldDimensions: Option<TSFieldAnnotationTypeElementDeclarationDimensions>,
    TSFieldName: TSFieldAnnotationTypeElementDeclarationName,
    TSFieldType: TSFieldAnnotationTypeElementDeclarationType,
    TSFieldValue: Option<TSFieldAnnotationTypeElementDeclarationValue>,
}
impl Display for TSTypeAnnotationTypeElementDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenArgumentList {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeArgumentList {
    _children: Option<Vec<TSChildrenArgumentList>>,
}
impl Display for TSTypeArgumentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeArrayAccess {
    TSFieldArray: TSFieldArrayAccessArray,
    TSFieldIndex: TSFieldArrayAccessIndex,
}
impl Display for TSTypeArrayAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenArrayCreationExpression {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeArrayCreationExpression {
    _children: Option<TSChildrenArrayCreationExpression>,
    TSFieldDimensions: Vec<TSFieldArrayCreationExpressionDimensions>,
    TSFieldType: TSFieldArrayCreationExpressionType,
    TSFieldValue: Option<TSFieldArrayCreationExpressionValue>,
}
impl Display for TSTypeArrayCreationExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenArrayInitializer {
    TSTypeArrayInitializer(TSTypeArrayInitializer),
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeArrayInitializer {
    _children: Option<Vec<TSChildrenArrayInitializer>>,
}
impl Display for TSTypeArrayInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeArrayType {
    TSFieldDimensions: TSFieldArrayTypeDimensions,
    TSFieldElement: TSFieldArrayTypeElement,
}
impl Display for TSTypeArrayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenAssertStatement {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeAssertStatement {
    _children: Vec<TSChildrenAssertStatement>,
}
impl Display for TSTypeAssertStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeAssignmentExpression {
    TSFieldLeft: TSFieldAssignmentExpressionLeft,
    TSFieldOperator: TSFieldAssignmentExpressionOperator,
    TSFieldRight: TSFieldAssignmentExpressionRight,
}
impl Display for TSTypeAssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeAsterisk {}
impl Display for TSTypeAsterisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeBinaryExpression {
    TSFieldLeft: TSFieldBinaryExpressionLeft,
    TSFieldOperator: TSFieldBinaryExpressionOperator,
    TSFieldRight: TSFieldBinaryExpressionRight,
}
impl Display for TSTypeBinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenBlock {
    TSTypeStatement(TSTypeStatement),
}
struct TSTypeBlock {
    _children: Option<Vec<TSChildrenBlock>>,
}
impl Display for TSTypeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenBreakStatement {
    TSTypeIdentifier(TSTypeIdentifier),
}
struct TSTypeBreakStatement {
    _children: Option<TSChildrenBreakStatement>,
}
impl Display for TSTypeBreakStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeCastExpression {
    TSFieldType: Vec<TSFieldCastExpressionType>,
    TSFieldValue: TSFieldCastExpressionValue,
}
impl Display for TSTypeCastExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenCatchClause {
    TSTypeCatchFormalParameter(TSTypeCatchFormalParameter),
}
struct TSTypeCatchClause {
    _children: TSChildrenCatchClause,
    TSFieldBody: TSFieldCatchClauseBody,
}
impl Display for TSTypeCatchClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenCatchFormalParameter {
    TSTypeCatchType(TSTypeCatchType),
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeCatchFormalParameter {
    _children: Vec<TSChildrenCatchFormalParameter>,
    TSFieldDimensions: Option<TSFieldCatchFormalParameterDimensions>,
    TSFieldName: TSFieldCatchFormalParameterName,
}
impl Display for TSTypeCatchFormalParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenCatchType {
    TSTypeUnannotatedType(TSTypeUnannotatedType),
}
struct TSTypeCatchType {
    _children: Vec<TSChildrenCatchType>,
}
impl Display for TSTypeCatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenClassBody {
    TSTypeAnnotationTypeDeclaration(TSTypeAnnotationTypeDeclaration),
    TSTypeBlock(TSTypeBlock),
    TSTypeClassDeclaration(TSTypeClassDeclaration),
    TSTypeConstructorDeclaration(TSTypeConstructorDeclaration),
    TSTypeEnumDeclaration(TSTypeEnumDeclaration),
    TSTypeFieldDeclaration(TSTypeFieldDeclaration),
    TSTypeInterfaceDeclaration(TSTypeInterfaceDeclaration),
    TSTypeMethodDeclaration(TSTypeMethodDeclaration),
    TSTypeRecordDeclaration(TSTypeRecordDeclaration),
    TSTypeStaticInitializer(TSTypeStaticInitializer),
}
struct TSTypeClassBody {
    _children: Option<Vec<TSChildrenClassBody>>,
}
impl Display for TSTypeClassBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenClassDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeClassDeclaration {
    _children: Option<TSChildrenClassDeclaration>,
    TSFieldBody: TSFieldClassDeclarationBody,
    TSFieldInterfaces: Option<TSFieldClassDeclarationInterfaces>,
    TSFieldName: TSFieldClassDeclarationName,
    TSFieldSuperclass: Option<TSFieldClassDeclarationSuperclass>,
    TSFieldTypeParameters: Option<TSFieldClassDeclarationTypeParameters>,
}
impl Display for TSTypeClassDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenClassLiteral {
    TSTypeUnannotatedType(TSTypeUnannotatedType),
}
struct TSTypeClassLiteral {
    _children: TSChildrenClassLiteral,
}
impl Display for TSTypeClassLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenConstantDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeConstantDeclaration {
    _children: Option<TSChildrenConstantDeclaration>,
    TSFieldDeclarator: Vec<TSFieldConstantDeclarationDeclarator>,
    TSFieldType: TSFieldConstantDeclarationType,
}
impl Display for TSTypeConstantDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenConstructorBody {
    TSTypeExplicitConstructorInvocation(TSTypeExplicitConstructorInvocation),
    TSTypeStatement(TSTypeStatement),
}
struct TSTypeConstructorBody {
    _children: Option<Vec<TSChildrenConstructorBody>>,
}
impl Display for TSTypeConstructorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenConstructorDeclaration {
    TSTypeModifiers(TSTypeModifiers),
    TSTypeThrows(TSTypeThrows),
}
struct TSTypeConstructorDeclaration {
    _children: Option<Vec<TSChildrenConstructorDeclaration>>,
    TSFieldBody: TSFieldConstructorDeclarationBody,
    TSFieldName: TSFieldConstructorDeclarationName,
    TSFieldParameters: TSFieldConstructorDeclarationParameters,
    TSFieldTypeParameters: Option<TSFieldConstructorDeclarationTypeParameters>,
}
impl Display for TSTypeConstructorDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenContinueStatement {
    TSTypeIdentifier(TSTypeIdentifier),
}
struct TSTypeContinueStatement {
    _children: Option<TSChildrenContinueStatement>,
}
impl Display for TSTypeContinueStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenDimensions {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeDimensions {
    _children: Option<Vec<TSChildrenDimensions>>,
}
impl Display for TSTypeDimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenDimensionsExpr {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeExpression(TSTypeExpression),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeDimensionsExpr {
    _children: Vec<TSChildrenDimensionsExpr>,
}
impl Display for TSTypeDimensionsExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeDoStatement {
    TSFieldBody: TSFieldDoStatementBody,
    TSFieldCondition: TSFieldDoStatementCondition,
}
impl Display for TSTypeDoStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenElementValueArrayInitializer {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeElementValueArrayInitializer(TSTypeElementValueArrayInitializer),
    TSTypeExpression(TSTypeExpression),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeElementValueArrayInitializer {
    _children: Option<Vec<TSChildrenElementValueArrayInitializer>>,
}
impl Display for TSTypeElementValueArrayInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeElementValuePair {
    TSFieldKey: TSFieldElementValuePairKey,
    TSFieldValue: TSFieldElementValuePairValue,
}
impl Display for TSTypeElementValuePair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenEnhancedForStatement {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeEnhancedForStatement {
    _children: Option<TSChildrenEnhancedForStatement>,
    TSFieldBody: TSFieldEnhancedForStatementBody,
    TSFieldDimensions: Option<TSFieldEnhancedForStatementDimensions>,
    TSFieldName: TSFieldEnhancedForStatementName,
    TSFieldType: TSFieldEnhancedForStatementType,
    TSFieldValue: TSFieldEnhancedForStatementValue,
}
impl Display for TSTypeEnhancedForStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenEnumBody {
    TSTypeEnumBodyDeclarations(TSTypeEnumBodyDeclarations),
    TSTypeEnumConstant(TSTypeEnumConstant),
}
struct TSTypeEnumBody {
    _children: Option<Vec<TSChildrenEnumBody>>,
}
impl Display for TSTypeEnumBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenEnumBodyDeclarations {
    TSTypeAnnotationTypeDeclaration(TSTypeAnnotationTypeDeclaration),
    TSTypeBlock(TSTypeBlock),
    TSTypeClassDeclaration(TSTypeClassDeclaration),
    TSTypeConstructorDeclaration(TSTypeConstructorDeclaration),
    TSTypeEnumDeclaration(TSTypeEnumDeclaration),
    TSTypeFieldDeclaration(TSTypeFieldDeclaration),
    TSTypeInterfaceDeclaration(TSTypeInterfaceDeclaration),
    TSTypeMethodDeclaration(TSTypeMethodDeclaration),
    TSTypeRecordDeclaration(TSTypeRecordDeclaration),
    TSTypeStaticInitializer(TSTypeStaticInitializer),
}
struct TSTypeEnumBodyDeclarations {
    _children: Option<Vec<TSChildrenEnumBodyDeclarations>>,
}
impl Display for TSTypeEnumBodyDeclarations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenEnumConstant {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeEnumConstant {
    _children: Option<TSChildrenEnumConstant>,
    TSFieldArguments: Option<TSFieldEnumConstantArguments>,
    TSFieldBody: Option<TSFieldEnumConstantBody>,
    TSFieldName: TSFieldEnumConstantName,
}
impl Display for TSTypeEnumConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenEnumDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeEnumDeclaration {
    _children: Option<TSChildrenEnumDeclaration>,
    TSFieldBody: TSFieldEnumDeclarationBody,
    TSFieldInterfaces: Option<TSFieldEnumDeclarationInterfaces>,
    TSFieldName: TSFieldEnumDeclarationName,
}
impl Display for TSTypeEnumDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeExplicitConstructorInvocation {
    TSFieldArguments: TSFieldExplicitConstructorInvocationArguments,
    TSFieldConstructor: TSFieldExplicitConstructorInvocationConstructor,
    TSFieldObject: Option<TSFieldExplicitConstructorInvocationObject>,
    TSFieldTypeArguments: Option<TSFieldExplicitConstructorInvocationTypeArguments>,
}
impl Display for TSTypeExplicitConstructorInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenExpressionStatement {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeExpressionStatement {
    _children: TSChildrenExpressionStatement,
}
impl Display for TSTypeExpressionStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenExtendsInterfaces {
    TSTypeInterfaceTypeList(TSTypeInterfaceTypeList),
}
struct TSTypeExtendsInterfaces {
    _children: TSChildrenExtendsInterfaces,
}
impl Display for TSTypeExtendsInterfaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenFieldAccess {
    TSTypeSuper(TSTypeSuper),
}
struct TSTypeFieldAccess {
    _children: Option<TSChildrenFieldAccess>,
    TSFieldField: TSFieldFieldAccessField,
    TSFieldObject: TSFieldFieldAccessObject,
}
impl Display for TSTypeFieldAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenFieldDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeFieldDeclaration {
    _children: Option<TSChildrenFieldDeclaration>,
    TSFieldDeclarator: Vec<TSFieldFieldDeclarationDeclarator>,
    TSFieldType: TSFieldFieldDeclarationType,
}
impl Display for TSTypeFieldDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenFinallyClause {
    TSTypeBlock(TSTypeBlock),
}
struct TSTypeFinallyClause {
    _children: TSChildrenFinallyClause,
}
impl Display for TSTypeFinallyClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeFloatingPointType {}
impl Display for TSTypeFloatingPointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeForStatement {
    TSFieldBody: TSFieldForStatementBody,
    TSFieldCondition: Option<TSFieldForStatementCondition>,
    TSFieldInit: Option<Vec<TSFieldForStatementInit>>,
    TSFieldUpdate: Option<Vec<TSFieldForStatementUpdate>>,
}
impl Display for TSTypeForStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenFormalParameter {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeFormalParameter {
    _children: Option<TSChildrenFormalParameter>,
    TSFieldDimensions: Option<TSFieldFormalParameterDimensions>,
    TSFieldName: TSFieldFormalParameterName,
    TSFieldType: TSFieldFormalParameterType,
}
impl Display for TSTypeFormalParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenFormalParameters {
    TSTypeFormalParameter(TSTypeFormalParameter),
    TSTypeReceiverParameter(TSTypeReceiverParameter),
    TSTypeSpreadParameter(TSTypeSpreadParameter),
}
struct TSTypeFormalParameters {
    _children: Option<Vec<TSChildrenFormalParameters>>,
}
impl Display for TSTypeFormalParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenGenericType {
    TSTypeScopedTypeIdentifier(TSTypeScopedTypeIdentifier),
    TSTypeTypeArguments(TSTypeTypeArguments),
    TSTypeTypeIdentifier(TSTypeTypeIdentifier),
}
struct TSTypeGenericType {
    _children: Vec<TSChildrenGenericType>,
}
impl Display for TSTypeGenericType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeIfStatement {
    TSFieldAlternative: Option<TSFieldIfStatementAlternative>,
    TSFieldCondition: TSFieldIfStatementCondition,
    TSFieldConsequence: TSFieldIfStatementConsequence,
}
impl Display for TSTypeIfStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenImportDeclaration {
    TSTypeAsterisk(TSTypeAsterisk),
    TSTypeIdentifier(TSTypeIdentifier),
    TSTypeScopedIdentifier(TSTypeScopedIdentifier),
}
struct TSTypeImportDeclaration {
    _children: Vec<TSChildrenImportDeclaration>,
}
impl Display for TSTypeImportDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenInferredParameters {
    TSTypeIdentifier(TSTypeIdentifier),
}
struct TSTypeInferredParameters {
    _children: Vec<TSChildrenInferredParameters>,
}
impl Display for TSTypeInferredParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeInstanceofExpression {
    TSFieldLeft: TSFieldInstanceofExpressionLeft,
    TSFieldRight: TSFieldInstanceofExpressionRight,
}
impl Display for TSTypeInstanceofExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeIntegralType {}
impl Display for TSTypeIntegralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenInterfaceBody {
    TSTypeAnnotationTypeDeclaration(TSTypeAnnotationTypeDeclaration),
    TSTypeClassDeclaration(TSTypeClassDeclaration),
    TSTypeConstantDeclaration(TSTypeConstantDeclaration),
    TSTypeEnumDeclaration(TSTypeEnumDeclaration),
    TSTypeInterfaceDeclaration(TSTypeInterfaceDeclaration),
    TSTypeMethodDeclaration(TSTypeMethodDeclaration),
}
struct TSTypeInterfaceBody {
    _children: Option<Vec<TSChildrenInterfaceBody>>,
}
impl Display for TSTypeInterfaceBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenInterfaceDeclaration {
    TSTypeExtendsInterfaces(TSTypeExtendsInterfaces),
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeInterfaceDeclaration {
    _children: Option<Vec<TSChildrenInterfaceDeclaration>>,
    TSFieldBody: TSFieldInterfaceDeclarationBody,
    TSFieldName: TSFieldInterfaceDeclarationName,
    TSFieldTypeParameters: Option<TSFieldInterfaceDeclarationTypeParameters>,
}
impl Display for TSTypeInterfaceDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenInterfaceTypeList {
    TSTypeType(TSTypeType),
}
struct TSTypeInterfaceTypeList {
    _children: Vec<TSChildrenInterfaceTypeList>,
}
impl Display for TSTypeInterfaceTypeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenLabeledStatement {
    TSTypeIdentifier(TSTypeIdentifier),
    TSTypeStatement(TSTypeStatement),
}
struct TSTypeLabeledStatement {
    _children: Vec<TSChildrenLabeledStatement>,
}
impl Display for TSTypeLabeledStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeLambdaExpression {
    TSFieldBody: TSFieldLambdaExpressionBody,
    TSFieldParameters: TSFieldLambdaExpressionParameters,
}
impl Display for TSTypeLambdaExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenLocalVariableDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeLocalVariableDeclaration {
    _children: Option<TSChildrenLocalVariableDeclaration>,
    TSFieldDeclarator: Vec<TSFieldLocalVariableDeclarationDeclarator>,
    TSFieldType: TSFieldLocalVariableDeclarationType,
}
impl Display for TSTypeLocalVariableDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeMarkerAnnotation {
    TSFieldName: TSFieldMarkerAnnotationName,
}
impl Display for TSTypeMarkerAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenMethodDeclaration {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
    TSTypeModifiers(TSTypeModifiers),
    TSTypeThrows(TSTypeThrows),
}
struct TSTypeMethodDeclaration {
    _children: Option<Vec<TSChildrenMethodDeclaration>>,
    TSFieldBody: Option<TSFieldMethodDeclarationBody>,
    TSFieldDimensions: Option<TSFieldMethodDeclarationDimensions>,
    TSFieldName: TSFieldMethodDeclarationName,
    TSFieldParameters: TSFieldMethodDeclarationParameters,
    TSFieldType: TSFieldMethodDeclarationType,
    TSFieldTypeParameters: Option<TSFieldMethodDeclarationTypeParameters>,
}
impl Display for TSTypeMethodDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenMethodInvocation {
    TSTypeSuper(TSTypeSuper),
}
struct TSTypeMethodInvocation {
    _children: Option<TSChildrenMethodInvocation>,
    TSFieldArguments: TSFieldMethodInvocationArguments,
    TSFieldName: TSFieldMethodInvocationName,
    TSFieldObject: Option<TSFieldMethodInvocationObject>,
    TSFieldTypeArguments: Option<TSFieldMethodInvocationTypeArguments>,
}
impl Display for TSTypeMethodInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenMethodReference {
    TSTypeType(TSTypeType),
    TSTypePrimaryExpression(TSTypePrimaryExpression),
    TSTypeSuper(TSTypeSuper),
    TSTypeTypeArguments(TSTypeTypeArguments),
}
struct TSTypeMethodReference {
    _children: Vec<TSChildrenMethodReference>,
}
impl Display for TSTypeMethodReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenModifiers {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeModifiers {
    _children: Option<Vec<TSChildrenModifiers>>,
}
impl Display for TSTypeModifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenModuleBody {
    TSTypeModuleDirective(TSTypeModuleDirective),
}
struct TSTypeModuleBody {
    _children: Option<Vec<TSChildrenModuleBody>>,
}
impl Display for TSTypeModuleBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenModuleDeclaration {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
}
struct TSTypeModuleDeclaration {
    _children: Option<Vec<TSChildrenModuleDeclaration>>,
    TSFieldBody: TSFieldModuleDeclarationBody,
    TSFieldName: TSFieldModuleDeclarationName,
}
impl Display for TSTypeModuleDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenModuleDirective {
    TSTypeIdentifier(TSTypeIdentifier),
    TSTypeRequiresModifier(TSTypeRequiresModifier),
    TSTypeScopedIdentifier(TSTypeScopedIdentifier),
}
struct TSTypeModuleDirective {
    _children: Vec<TSChildrenModuleDirective>,
}
impl Display for TSTypeModuleDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenObjectCreationExpression {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeClassBody(TSTypeClassBody),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
    TSTypePrimaryExpression(TSTypePrimaryExpression),
}
struct TSTypeObjectCreationExpression {
    _children: Option<Vec<TSChildrenObjectCreationExpression>>,
    TSFieldArguments: TSFieldObjectCreationExpressionArguments,
    TSFieldType: TSFieldObjectCreationExpressionType,
    TSFieldTypeArguments: Option<TSFieldObjectCreationExpressionTypeArguments>,
}
impl Display for TSTypeObjectCreationExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenPackageDeclaration {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeIdentifier(TSTypeIdentifier),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
    TSTypeScopedIdentifier(TSTypeScopedIdentifier),
}
struct TSTypePackageDeclaration {
    _children: Vec<TSChildrenPackageDeclaration>,
}
impl Display for TSTypePackageDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenParenthesizedExpression {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeParenthesizedExpression {
    _children: TSChildrenParenthesizedExpression,
}
impl Display for TSTypeParenthesizedExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenProgram {
    TSTypeStatement(TSTypeStatement),
}
struct TSTypeProgram {
    _children: Option<Vec<TSChildrenProgram>>,
}
impl Display for TSTypeProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenReceiverParameter {
    TSTypeUnannotatedType(TSTypeUnannotatedType),
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeIdentifier(TSTypeIdentifier),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
    TSTypeThis(TSTypeThis),
}
struct TSTypeReceiverParameter {
    _children: Vec<TSChildrenReceiverParameter>,
}
impl Display for TSTypeReceiverParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenRecordDeclaration {
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeRecordDeclaration {
    _children: Option<TSChildrenRecordDeclaration>,
    TSFieldBody: TSFieldRecordDeclarationBody,
    TSFieldName: TSFieldRecordDeclarationName,
    TSFieldParameters: TSFieldRecordDeclarationParameters,
}
impl Display for TSTypeRecordDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeRequiresModifier {}
impl Display for TSTypeRequiresModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenResource {
    TSTypeFieldAccess(TSTypeFieldAccess),
    TSTypeIdentifier(TSTypeIdentifier),
    TSTypeModifiers(TSTypeModifiers),
}
struct TSTypeResource {
    _children: Option<TSChildrenResource>,
    TSFieldDimensions: Option<TSFieldResourceDimensions>,
    TSFieldName: Option<TSFieldResourceName>,
    TSFieldType: Option<TSFieldResourceType>,
    TSFieldValue: Option<TSFieldResourceValue>,
}
impl Display for TSTypeResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenResourceSpecification {
    TSTypeResource(TSTypeResource),
}
struct TSTypeResourceSpecification {
    _children: Vec<TSChildrenResourceSpecification>,
}
impl Display for TSTypeResourceSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenReturnStatement {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeReturnStatement {
    _children: Option<TSChildrenReturnStatement>,
}
impl Display for TSTypeReturnStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeScopedIdentifier {
    TSFieldName: TSFieldScopedIdentifierName,
    TSFieldScope: TSFieldScopedIdentifierScope,
}
impl Display for TSTypeScopedIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenScopedTypeIdentifier {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeGenericType(TSTypeGenericType),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
    TSTypeScopedTypeIdentifier(TSTypeScopedTypeIdentifier),
    TSTypeTypeIdentifier(TSTypeTypeIdentifier),
}
struct TSTypeScopedTypeIdentifier {
    _children: Vec<TSChildrenScopedTypeIdentifier>,
}
impl Display for TSTypeScopedTypeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSpreadParameter {
    TSTypeUnannotatedType(TSTypeUnannotatedType),
    TSTypeModifiers(TSTypeModifiers),
    TSTypeVariableDeclarator(TSTypeVariableDeclarator),
}
struct TSTypeSpreadParameter {
    _children: Vec<TSChildrenSpreadParameter>,
}
impl Display for TSTypeSpreadParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenStaticInitializer {
    TSTypeBlock(TSTypeBlock),
}
struct TSTypeStaticInitializer {
    _children: TSChildrenStaticInitializer,
}
impl Display for TSTypeStaticInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSuperInterfaces {
    TSTypeInterfaceTypeList(TSTypeInterfaceTypeList),
}
struct TSTypeSuperInterfaces {
    _children: TSChildrenSuperInterfaces,
}
impl Display for TSTypeSuperInterfaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSuperclass {
    TSTypeType(TSTypeType),
}
struct TSTypeSuperclass {
    _children: TSChildrenSuperclass,
}
impl Display for TSTypeSuperclass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSwitchBlock {
    TSTypeSwitchBlockStatementGroup(TSTypeSwitchBlockStatementGroup),
    TSTypeSwitchRule(TSTypeSwitchRule),
}
struct TSTypeSwitchBlock {
    _children: Option<Vec<TSChildrenSwitchBlock>>,
}
impl Display for TSTypeSwitchBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSwitchBlockStatementGroup {
    TSTypeStatement(TSTypeStatement),
    TSTypeSwitchLabel(TSTypeSwitchLabel),
}
struct TSTypeSwitchBlockStatementGroup {
    _children: Vec<TSChildrenSwitchBlockStatementGroup>,
}
impl Display for TSTypeSwitchBlockStatementGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeSwitchExpression {
    TSFieldBody: TSFieldSwitchExpressionBody,
    TSFieldCondition: TSFieldSwitchExpressionCondition,
}
impl Display for TSTypeSwitchExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSwitchLabel {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeSwitchLabel {
    _children: Option<Vec<TSChildrenSwitchLabel>>,
}
impl Display for TSTypeSwitchLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSwitchRule {
    TSTypeBlock(TSTypeBlock),
    TSTypeExpressionStatement(TSTypeExpressionStatement),
    TSTypeSwitchLabel(TSTypeSwitchLabel),
    TSTypeThrowStatement(TSTypeThrowStatement),
}
struct TSTypeSwitchRule {
    _children: Vec<TSChildrenSwitchRule>,
}
impl Display for TSTypeSwitchRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeSwitchStatement {
    TSFieldBody: TSFieldSwitchStatementBody,
    TSFieldCondition: TSFieldSwitchStatementCondition,
}
impl Display for TSTypeSwitchStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenSynchronizedStatement {
    TSTypeParenthesizedExpression(TSTypeParenthesizedExpression),
}
struct TSTypeSynchronizedStatement {
    _children: TSChildrenSynchronizedStatement,
    TSFieldBody: TSFieldSynchronizedStatementBody,
}
impl Display for TSTypeSynchronizedStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeTernaryExpression {
    TSFieldAlternative: TSFieldTernaryExpressionAlternative,
    TSFieldCondition: TSFieldTernaryExpressionCondition,
    TSFieldConsequence: TSFieldTernaryExpressionConsequence,
}
impl Display for TSTypeTernaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenThrowStatement {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeThrowStatement {
    _children: TSChildrenThrowStatement,
}
impl Display for TSTypeThrowStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenThrows {
    TSTypeType(TSTypeType),
}
struct TSTypeThrows {
    _children: Vec<TSChildrenThrows>,
}
impl Display for TSTypeThrows {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenTryStatement {
    TSTypeCatchClause(TSTypeCatchClause),
    TSTypeFinallyClause(TSTypeFinallyClause),
}
struct TSTypeTryStatement {
    _children: Vec<TSChildrenTryStatement>,
    TSFieldBody: TSFieldTryStatementBody,
}
impl Display for TSTypeTryStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenTryWithResourcesStatement {
    TSTypeCatchClause(TSTypeCatchClause),
    TSTypeFinallyClause(TSTypeFinallyClause),
}
struct TSTypeTryWithResourcesStatement {
    _children: Option<Vec<TSChildrenTryWithResourcesStatement>>,
    TSFieldBody: TSFieldTryWithResourcesStatementBody,
    TSFieldResources: TSFieldTryWithResourcesStatementResources,
}
impl Display for TSTypeTryWithResourcesStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenTypeArguments {
    TSTypeType(TSTypeType),
    TSTypeWildcard(TSTypeWildcard),
}
struct TSTypeTypeArguments {
    _children: Option<Vec<TSChildrenTypeArguments>>,
}
impl Display for TSTypeTypeArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenTypeBound {
    TSTypeType(TSTypeType),
}
struct TSTypeTypeBound {
    _children: Vec<TSChildrenTypeBound>,
}
impl Display for TSTypeTypeBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenTypeParameter {
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeIdentifier(TSTypeIdentifier),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
    TSTypeTypeBound(TSTypeTypeBound),
}
struct TSTypeTypeParameter {
    _children: Vec<TSChildrenTypeParameter>,
}
impl Display for TSTypeTypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenTypeParameters {
    TSTypeTypeParameter(TSTypeTypeParameter),
}
struct TSTypeTypeParameters {
    _children: Vec<TSChildrenTypeParameters>,
}
impl Display for TSTypeTypeParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeUnaryExpression {
    TSFieldOperand: TSFieldUnaryExpressionOperand,
    TSFieldOperator: TSFieldUnaryExpressionOperator,
}
impl Display for TSTypeUnaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenUpdateExpression {
    TSTypeUnaryExp(TSTypeUnaryExp),
}
struct TSTypeUpdateExpression {
    _children: TSChildrenUpdateExpression,
}
impl Display for TSTypeUpdateExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeVariableDeclarator {
    TSFieldDimensions: Option<TSFieldVariableDeclaratorDimensions>,
    TSFieldName: TSFieldVariableDeclaratorName,
    TSFieldValue: Option<TSFieldVariableDeclaratorValue>,
}
impl Display for TSTypeVariableDeclarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSTypeWhileStatement {
    TSFieldBody: TSFieldWhileStatementBody,
    TSFieldCondition: TSFieldWhileStatementCondition,
}
impl Display for TSTypeWhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenWildcard {
    TSTypeType(TSTypeType),
    TSTypeAnnotation(TSTypeAnnotation),
    TSTypeMarkerAnnotation(TSTypeMarkerAnnotation),
    TSTypeSuper(TSTypeSuper),
}
struct TSTypeWildcard {
    _children: Option<Vec<TSChildrenWildcard>>,
}
impl Display for TSTypeWildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
enum TSChildrenYieldStatement {
    TSTypeExpression(TSTypeExpression),
}
struct TSTypeYieldStatement {
    _children: TSChildrenYieldStatement,
}
impl Display for TSTypeYieldStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSType32 {}
impl Display for TSType32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("!")
    }
}
struct TSType13 {}
impl Display for TSType13 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("!=")
    }
}
struct TSType14 {}
impl Display for TSType14 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("%")
    }
}
struct TSType1 {}
impl Display for TSType1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("%=")
    }
}
struct TSType15 {}
impl Display for TSType15 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("&")
    }
}
struct TSType16 {}
impl Display for TSType16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("&&")
    }
}
struct TSType2 {}
impl Display for TSType2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("&=")
    }
}
struct TSType34 {}
impl Display for TSType34 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")
    }
}
struct TSType35 {}
impl Display for TSType35 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(")")
    }
}
struct TSType17 {}
impl Display for TSType17 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("*")
    }
}
struct TSType3 {}
impl Display for TSType3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("*=")
    }
}
struct TSType18 {}
impl Display for TSType18 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("+")
    }
}
struct TSType36 {}
impl Display for TSType36 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("++")
    }
}
struct TSType4 {}
impl Display for TSType4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("+=")
    }
}
struct TSType37 {}
impl Display for TSType37 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(",")
    }
}
struct TSType19 {}
impl Display for TSType19 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("-")
    }
}
struct TSType38 {}
impl Display for TSType38 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("--")
    }
}
struct TSType5 {}
impl Display for TSType5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("-=")
    }
}
struct TSType39 {}
impl Display for TSType39 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("->")
    }
}
struct TSType40 {}
impl Display for TSType40 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(".")
    }
}
struct TSType41 {}
impl Display for TSType41 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("...")
    }
}
struct TSType20 {}
impl Display for TSType20 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("/")
    }
}
struct TSType6 {}
impl Display for TSType6 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("/=")
    }
}
struct TSType42 {}
impl Display for TSType42 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(":")
    }
}
struct TSType43 {}
impl Display for TSType43 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("::")
    }
}
struct TSType0 {}
impl Display for TSType0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(";")
    }
}
struct TSType21 {}
impl Display for TSType21 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<")
    }
}
struct TSType22 {}
impl Display for TSType22 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<<")
    }
}
struct TSType7 {}
impl Display for TSType7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<<=")
    }
}
struct TSType23 {}
impl Display for TSType23 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<=")
    }
}
struct TSType8 {}
impl Display for TSType8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("=")
    }
}
struct TSType24 {}
impl Display for TSType24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("==")
    }
}
struct TSType25 {}
impl Display for TSType25 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(">")
    }
}
struct TSType26 {}
impl Display for TSType26 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(">=")
    }
}
struct TSType27 {}
impl Display for TSType27 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(">>")
    }
}
struct TSType9 {}
impl Display for TSType9 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(">>=")
    }
}
struct TSType28 {}
impl Display for TSType28 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(">>>")
    }
}
struct TSType10 {}
impl Display for TSType10 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(">>>=")
    }
}
struct TSType44 {}
impl Display for TSType44 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("?")
    }
}
struct TSType45 {}
impl Display for TSType45 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("@")
    }
}
struct TSType46 {}
impl Display for TSType46 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("@interface")
    }
}
struct TSType47 {}
impl Display for TSType47 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")
    }
}
struct TSType48 {}
impl Display for TSType48 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("]")
    }
}
struct TSType29 {}
impl Display for TSType29 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("^")
    }
}
struct TSType11 {}
impl Display for TSType11 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("^=")
    }
}
struct TSType49 {}
impl Display for TSType49 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("abstract")
    }
}
struct TSType50 {}
impl Display for TSType50 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("assert")
    }
}
struct TSTypeBinaryIntegerLiteral {}
impl Display for TSTypeBinaryIntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("binary_integer_literal")
    }
}
struct TSTypeBooleanType {}
impl Display for TSTypeBooleanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("boolean_type")
    }
}
struct TSType51 {}
impl Display for TSType51 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("break")
    }
}
struct TSType52 {}
impl Display for TSType52 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("byte")
    }
}
struct TSType53 {}
impl Display for TSType53 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("case")
    }
}
struct TSType54 {}
impl Display for TSType54 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("catch")
    }
}
struct TSType55 {}
impl Display for TSType55 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("char")
    }
}
struct TSTypeCharacterLiteral {}
impl Display for TSTypeCharacterLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("character_literal")
    }
}
struct TSType56 {}
impl Display for TSType56 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("class")
    }
}
struct TSTypeComment {}
impl Display for TSTypeComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("comment")
    }
}
struct TSType57 {}
impl Display for TSType57 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("continue")
    }
}
struct TSTypeDecimalFloatingPointLiteral {}
impl Display for TSTypeDecimalFloatingPointLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("decimal_floating_point_literal")
    }
}
struct TSTypeDecimalIntegerLiteral {}
impl Display for TSTypeDecimalIntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("decimal_integer_literal")
    }
}
struct TSType58 {}
impl Display for TSType58 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("default")
    }
}
struct TSType59 {}
impl Display for TSType59 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("do")
    }
}
struct TSType60 {}
impl Display for TSType60 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("double")
    }
}
struct TSType61 {}
impl Display for TSType61 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("else")
    }
}
struct TSType62 {}
impl Display for TSType62 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("enum")
    }
}
struct TSType63 {}
impl Display for TSType63 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("exports")
    }
}
struct TSType64 {}
impl Display for TSType64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("extends")
    }
}
struct TSTypeFalse {}
impl Display for TSTypeFalse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("false")
    }
}
struct TSType65 {}
impl Display for TSType65 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("final")
    }
}
struct TSType66 {}
impl Display for TSType66 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("finally")
    }
}
struct TSType67 {}
impl Display for TSType67 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("float")
    }
}
struct TSType68 {}
impl Display for TSType68 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("for")
    }
}
struct TSTypeHexFloatingPointLiteral {}
impl Display for TSTypeHexFloatingPointLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("hex_floating_point_literal")
    }
}
struct TSTypeHexIntegerLiteral {}
impl Display for TSTypeHexIntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("hex_integer_literal")
    }
}
struct TSTypeIdentifier {}
impl Display for TSTypeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("identifier")
    }
}
struct TSType69 {}
impl Display for TSType69 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("if")
    }
}
struct TSType70 {}
impl Display for TSType70 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("implements")
    }
}
struct TSType71 {}
impl Display for TSType71 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("import")
    }
}
struct TSType72 {}
impl Display for TSType72 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("instanceof")
    }
}
struct TSType73 {}
impl Display for TSType73 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("int")
    }
}
struct TSType74 {}
impl Display for TSType74 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("interface")
    }
}
struct TSType75 {}
impl Display for TSType75 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("long")
    }
}
struct TSType76 {}
impl Display for TSType76 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("module")
    }
}
struct TSType77 {}
impl Display for TSType77 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("native")
    }
}
struct TSType78 {}
impl Display for TSType78 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("new")
    }
}
struct TSTypeNullLiteral {}
impl Display for TSTypeNullLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("null_literal")
    }
}
struct TSTypeOctalIntegerLiteral {}
impl Display for TSTypeOctalIntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("octal_integer_literal")
    }
}
struct TSType79 {}
impl Display for TSType79 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("open")
    }
}
struct TSType80 {}
impl Display for TSType80 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("opens")
    }
}
struct TSType81 {}
impl Display for TSType81 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("package")
    }
}
struct TSType82 {}
impl Display for TSType82 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("private")
    }
}
struct TSType83 {}
impl Display for TSType83 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("protected")
    }
}
struct TSType84 {}
impl Display for TSType84 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provides")
    }
}
struct TSType85 {}
impl Display for TSType85 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("public")
    }
}
struct TSType86 {}
impl Display for TSType86 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("record")
    }
}
struct TSType87 {}
impl Display for TSType87 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("requires")
    }
}
struct TSType88 {}
impl Display for TSType88 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("return")
    }
}
struct TSType89 {}
impl Display for TSType89 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("short")
    }
}
struct TSType90 {}
impl Display for TSType90 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("static")
    }
}
struct TSType91 {}
impl Display for TSType91 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("strictfp")
    }
}
struct TSTypeStringLiteral {}
impl Display for TSTypeStringLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("string_literal")
    }
}
struct TSTypeSuper {}
impl Display for TSTypeSuper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("super")
    }
}
struct TSType92 {}
impl Display for TSType92 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("switch")
    }
}
struct TSType93 {}
impl Display for TSType93 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("synchronized")
    }
}
struct TSTypeThis {}
impl Display for TSTypeThis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("this")
    }
}
struct TSType94 {}
impl Display for TSType94 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("throw")
    }
}
struct TSType95 {}
impl Display for TSType95 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("throws")
    }
}
struct TSType96 {}
impl Display for TSType96 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("to")
    }
}
struct TSType97 {}
impl Display for TSType97 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("transient")
    }
}
struct TSType98 {}
impl Display for TSType98 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("transitive")
    }
}
struct TSTypeTrue {}
impl Display for TSTypeTrue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("true")
    }
}
struct TSType99 {}
impl Display for TSType99 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("try")
    }
}
struct TSTypeTypeIdentifier {}
impl Display for TSTypeTypeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("type_identifier")
    }
}
struct TSType100 {}
impl Display for TSType100 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("uses")
    }
}
struct TSTypeVoidType {}
impl Display for TSTypeVoidType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("void_type")
    }
}
struct TSType101 {}
impl Display for TSType101 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("volatile")
    }
}
struct TSType102 {}
impl Display for TSType102 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("while")
    }
}
struct TSType103 {}
impl Display for TSType103 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("with")
    }
}
struct TSType104 {}
impl Display for TSType104 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("yield")
    }
}
struct TSType105 {}
impl Display for TSType105 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{")
    }
}
struct TSType30 {}
impl Display for TSType30 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("|")
    }
}
struct TSType12 {}
impl Display for TSType12 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("|=")
    }
}
struct TSType31 {}
impl Display for TSType31 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("||")
    }
}
struct TSType106 {}
impl Display for TSType106 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("}")
    }
}
struct TSType33 {}
impl Display for TSType33 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("~")
    }
}
