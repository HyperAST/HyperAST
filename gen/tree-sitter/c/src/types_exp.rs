macro_rules! make_type {
    (
        $(#[$km0:meta])*
        Keyword {$(
            $(#[$km:meta])*
            $ka:ident
        ),* $(,)?}
        $(#[$cm0:meta])*
        Concrete {$(
            $(#[$cm:meta])*
            $ca:ident$({$($cl:expr),+ $(,)*})?$(($($co:ident),+ $(,)*))?$([$($cx:ident),+ $(,)*])?
        ),* $(,)?}
        $(#[$wm0:meta])*
        WithFields {$(
            $(#[$wm:meta])*
            $wa:ident{$($wb:tt)*}
        ),* $(,)?}
        $(#[$am0:meta])*
        Abstract {$(
            $(#[$am:meta])*
            $aa:ident($($ab:ident),* $(,)?)
        ),* $(,)?}
    ) => {

        // #[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
        // #[strum(serialize_all = "snake_case")]
        #[derive(Hash, Clone, Copy, PartialEq, Eq)]
        pub enum Type {
            // Keywords
        $(
            // $( #[$km] )*
            $ka,
        )*
            // Concrete
        $(
            $ca,
        )*
            // WithFields
        $(
            // $( #[$wm] )*
            $wa,
        )*
        }
        enum Abstract {
            $(
                $aa,
            )*
        }

        // #[strum(props(Teacher="Ms.Frizzle", Room="201"))]
        // pub enum WithFields {}

        pub struct Factory {
            map: Box<[u16]>,
        }

        pub struct Language;
        // impl hyperast::types::Lang for Language {
        //     type Factory = Factory;
        //     type Type = Type;
        // }
    };
}

make_type! {
    Keyword {
        #[strum(serialize = "\n")]
        TS0,
        #[strum(serialize = "!")]
        Bang,
        #[strum(serialize = "!=")]
        BangEq,
        #[strum(serialize = "\"")]
        TS1,
        #[strum(serialize = "\"\"")]
        TS2,
        #[strum(serialize = "#define")]
        HashDefine,
        #[strum(serialize = "#elif")]
        HashElif,
        #[strum(serialize = "#else")]
        HashElse,
        #[strum(serialize = "#endif")]
        HashEndif,
        #[strum(serialize = "#if")]
        HashIf,
        #[strum(serialize = "#ifdef")]
        HashIfdef,
        #[strum(serialize = "#ifndef")]
        HashIfndef,
        #[strum(serialize = "#include")]
        HashInclude,
        #[strum(serialize = "%")]
        Percent,
        #[strum(serialize = "%=")]
        PercentEq,
        #[strum(serialize = "&")]
        Amp,
        #[strum(serialize = "&&")]
        AmpAmp,
        #[strum(serialize = "&=")]
        AmpEq,
        #[strum(serialize = "'")]
        TS3,
        #[strum(serialize = "(")]
        LParen,
        #[strum(serialize = "()")]
        TS4,
        #[strum(serialize = ")")]
        RParen,
        #[strum(serialize = "*")]
        Star,
        #[strum(serialize = "*=")]
        StarEq,
        #[strum(serialize = "+")]
        Plus,
        #[strum(serialize = "++")]
        PlusPlus,
        #[strum(serialize = "+=")]
        PlusEq,
        #[strum(serialize = ",")]
        Comma,
        #[strum(serialize = "-")]
        Dash,
        #[strum(serialize = "--")]
        DashDash,
        #[strum(serialize = "-=")]
        DashEq,
        #[strum(serialize = "->")]
        DashGt,
        #[strum(serialize = "->*")]
        DashGtStar,
        #[strum(serialize = ".")]
        Dot,
        #[strum(serialize = "...")]
        DotDotDot,
        #[strum(serialize = "/")]
        Slash,
        #[strum(serialize = "/=")]
        SlashEq,
        #[strum(serialize = ":")]
        Colon,
        #[strum(serialize = "::")]
        ColonColon,
        #[strum(serialize = ";")]
        SemiColon,
        #[strum(serialize = "<")]
        LT,
        #[strum(serialize = "<<")]
        LtLt,
        #[strum(serialize = "<<=")]
        LtLtEq,
        #[strum(serialize = "<=")]
        LTEq,
        #[strum(serialize = "=")]
        Eq,
        #[strum(serialize = "==")]
        EqEq,
        #[strum(serialize = ">")]
        GT,
        #[strum(serialize = ">=")]
        GTEq,
        #[strum(serialize = ">>")]
        GtGt,
        #[strum(serialize = ">>=")]
        GtGtEq,
        #[strum(serialize = "?")]
        QMark,
        #[strum(serialize = "L\"")]
        TS5,
        #[strum(serialize = "L'")]
        TS6,
        #[strum(serialize = "U\"")]
        TS7,
        #[strum(serialize = "U'")]
        TS8,
        #[strum(serialize = "[")]
        LBracket,
        #[strum(serialize = "[[")]
        TS9,
        #[strum(serialize = "[]")]
        TS10,
        #[strum(serialize = "]")]
        RBracket,
        #[strum(serialize = "]]")]
        TS11,
        #[strum(serialize = "^")]
        Caret,
        #[strum(serialize = "^=")]
        CaretEq,
        #[strum(serialize = "_Atomic")]
        TS12,
        #[strum(serialize = "__attribute__")]
        TS13,
        #[strum(serialize = "__based")]
        TS14,
        #[strum(serialize = "__cdecl")]
        TS15,
        #[strum(serialize = "__clrcall")]
        TS16,
        #[strum(serialize = "__declspec")]
        TS17,
        #[strum(serialize = "__fastcall")]
        TS18,
        #[strum(serialize = "__stdcall")]
        TS19,
        #[strum(serialize = "__thiscall")]
        TS20,
        #[strum(serialize = "__unaligned")]
        TS21,
        #[strum(serialize = "__vectorcall")]
        TS22,
        #[strum(serialize = "_unaligned")]
        TS23,
        Break,
        Case,
        Catch,
        Class,
        CoAwait,
        CoReturn,
        CoYield,
        Const,
        Constexpr,
        Continue,
        Decltype,
        Default,
        Defined,
        Delete,
        Do,
        Else,
        Enum,
        Explicit,
        Extern,
        Final,
        For,
        Friend,
        Goto,
        If,
        Inline,
        Long,
        Mutable,
        Namespace,
        New,
        Noexcept,
        Operator,
        Override,
        Private,
        Protected,
        Public,
        Register,
        Restrict,
        Return,
        Short,
        Signed,
        Sizeof,
        Static,
        StaticAssert,
        Struct,
        Switch,
        Template,
        ThreadLocal,
        Throw,
        Try,
        Typedef,
        Typename,
        #[strum(serialize = "u\"")]
        TS24,
        #[strum(serialize = "u'")]
        TS25,
        #[strum(serialize = "u8\"")]
        TS26,
        #[strum(serialize = "u8'")]
        TS27,
        Union,
        Unsigned,
        Using,
        Virtual,
        Volatile,
        While,
        #[strum(serialize = "{")]
        LBrace,
        #[strum(serialize = "|")]
        Pipe,
        #[strum(serialize = "|=")]
        PipeEq,
        #[strum(serialize = "||")]
        PipePipe,
        #[strum(serialize = "}")]
        RBrace,
        #[strum(serialize = "~")]
        Tilde,
    }
    /// Type of nodes actually stored
    /// ie. what should be stored on CST nodes
    /// but anyway encode it as a number
    /// and it would be better to take the smallest numbers for concrete nodes
    /// to facilitate convertion
    Concrete {
        AbstractParenthesizedDeclarator(AbstractDeclarator),
        AbstractReferenceDeclarator(AbstractDeclarator),
        AccessSpecifier,
        ArgumentList(Expression, InitializerList, PreprocDefined),
        AttributeDeclaration(Attribute),
        AttributeSpecifier(ArgumentList),
        AttributedDeclarator(
            Declarator,
            FieldDeclarator,
            TypeDeclarator,
            AttributeDeclaration,
        ),
        AttributedStatement(Statement, AttributeDeclaration),
        Auto,
        BaseClassClause(QualifiedIdentifier, TemplateType, TypeIdentifier),
        BitfieldClause(Expression),
        BreakStatement,
        CharLiteral(EscapeSequence),
        CoReturnStatement(Expression),
        CoYieldStatement(Expression),
        Comment,
        CompoundStatement(
            Statement,
            TypeSpecifier,
            AliasDeclaration,
            AttributedStatement,
            Declaration,
            FunctionDefinition,
            LinkageSpecification,
            NamespaceDefinition,
            PreprocCall,
            PreprocDef,
            PreprocFunctionDef,
            PreprocIf,
            PreprocIfdef,
            PreprocInclude,
            StaticAssertDeclaration,
            TemplateDeclaration,
            TemplateInstantiation,
            TypeDefinition,
            UsingDeclaration,
        ),
        ConcatenatedString(RawStringLiteral, StringLiteral),
        ContinueStatement,
        DeclarationList(
            Statement,
            TypeSpecifier,
            AliasDeclaration,
            AttributedStatement,
            Declaration,
            FunctionDefinition,
            LinkageSpecification,
            NamespaceDefinition,
            PreprocCall,
            PreprocDef,
            PreprocFunctionDef,
            PreprocIf,
            PreprocIfdef,
            PreprocInclude,
            StaticAssertDeclaration,
            TemplateDeclaration,
            TemplateInstantiation,
            TypeDefinition,
            UsingDeclaration,
        ),
        DefaultMethodClause,
        DeleteExpression(Expression),
        DeleteMethodClause,
        DependentName(TemplateFunction, TemplateMethod, TemplateType),
        DependentType(TypeSpecifier),
        DestructorName(Identifier),
        EnumeratorList(Enumerator),
        EscapeSequence,
        ExplicitFunctionSpecifier(Expression),
        ExpressionStatement(Expression, CommaExpression),
        False,
        FieldDeclarationList(
            AccessSpecifier,
            AliasDeclaration,
            Declaration,
            FieldDeclaration,
            FriendDeclaration,
            FunctionDefinition,
            PreprocCall,
            PreprocDef,
            PreprocFunctionDef,
            PreprocIf,
            PreprocIfdef,
            StaticAssertDeclaration,
            TemplateDeclaration,
            TypeDefinition,
            UsingDeclaration,
        ),
        FieldDesignator(FieldIdentifier),
        FieldIdentifier,
        FieldInitializer(
            ArgumentList,
            FieldIdentifier,
            InitializerList,
            QualifiedIdentifier,
            TemplateMethod,
        ),
        FieldInitializerList(FieldInitializer),
        FriendDeclaration(
            Declaration,
            FunctionDefinition,
            QualifiedIdentifier,
            TemplateType,
            TypeIdentifier,
        ),
        Identifier,
        InitializerList(Expression, InitializerList, InitializerPair),
        LambdaCaptureSpecifier(Expression, LambdaDefaultCapture),
        LambdaDefaultCapture,
        LiteralSuffix,
        MsBasedModifier(ArgumentList),
        MsCallModifier,
        MsDeclspecModifier(Identifier),
        MsPointerModifier(
            MsRestrictModifier,
            MsSignedPtrModifier,
            MsUnalignedPtrModifier,
            MsUnsignedPtrModifier,
        ),
        MsRestrictModifier,
        MsSignedPtrModifier,
        MsUnalignedPtrModifier,
        MsUnsignedPtrModifier,
        NamespaceDefinitionName(Identifier, NamespaceDefinitionName),
        NamespaceIdentifier,
        Null,
        Nullptr,
        NumberLiteral,
        OperatorName(Identifier),
        ParameterList(
            OptionalParameterDeclaration,
            ParameterDeclaration,
            VariadicParameterDeclaration,
        ),
        ParenthesizedDeclarator(Declarator, FieldDeclarator, TypeDeclarator),
        ParenthesizedExpression(Expression, CommaExpression, PreprocDefined),
        PreprocArg,
        PreprocDefined(Identifier),
        PreprocDirective,
        PreprocElse(
            Statement,
            TypeSpecifier,
            AccessSpecifier,
            AliasDeclaration,
            AttributedStatement,
            Declaration,
            FieldDeclaration,
            FriendDeclaration,
            FunctionDefinition,
            LinkageSpecification,
            NamespaceDefinition,
            PreprocCall,
            PreprocDef,
            PreprocFunctionDef,
            PreprocIf,
            PreprocIfdef,
            PreprocInclude,
            StaticAssertDeclaration,
            TemplateDeclaration,
            TemplateInstantiation,
            TypeDefinition,
            UsingDeclaration,
        ),
        PreprocParams(Identifier),
        PrimitiveType,
        RawStringLiteral,
        RefQualifier,
        ReferenceDeclarator(Declarator, FieldDeclarator, VariadicDeclarator),
        ReturnStatement(Expression, CommaExpression, InitializerList),
        StatementIdentifier,
        StorageClassSpecifier,
        StringLiteral(EscapeSequence),
        StructuredBindingDeclarator(Identifier),
        SubscriptDesignator(Expression),
        SystemLibString,
        TemplateArgumentList(Expression, TypeDescriptor),
        TemplateParameterList(
            OptionalParameterDeclaration,
            OptionalTypeParameterDeclaration,
            ParameterDeclaration,
            TemplateTemplateParameterDeclaration,
            TypeParameterDeclaration,
            VariadicParameterDeclaration,
            VariadicTypeParameterDeclaration,
        ),
        This,
        ThrowSpecifier(TypeDescriptor),
        ThrowStatement(Expression),
        TrailingReturnType(AbstractDeclarator, TypeSpecifier, TypeQualifier),
        TranslationUnit(
            Statement,
            TypeSpecifier,
            AliasDeclaration,
            AttributedStatement,
            Declaration,
            FunctionDefinition,
            LinkageSpecification,
            NamespaceDefinition,
            PreprocCall,
            PreprocDef,
            PreprocFunctionDef,
            PreprocIf,
            PreprocIfdef,
            PreprocInclude,
            StaticAssertDeclaration,
            TemplateDeclaration,
            TemplateInstantiation,
            TypeDefinition,
            UsingDeclaration,
        ),
        True,
        TypeIdentifier,
        TypeParameterDeclaration(TypeIdentifier),
        TypeQualifier,
        UserDefinedLiteral(
            CharLiteral,
            ConcatenatedString,
            LiteralSuffix,
            NumberLiteral,
            RawStringLiteral,
            StringLiteral,
        ),
        UsingDeclaration(Identifier, QualifiedIdentifier),
        VariadicDeclarator(Identifier),
        VariadicTypeParameterDeclaration(TypeIdentifier),
        VirtualFunctionSpecifier,
        VirtualSpecifier,
    }
    WithFields {
        AbstractArrayDeclarator {
            size: (Star, Expression),
            declarator: (AbstractDeclarator,),
            _cs: (TypeQualifier,),
        },
        AbstractFunctionDeclarator {
            parameters: (ParameterList,),
            declarator: (AbstractDeclarator,),
            _cs: (Noexcept, RefQualifier, ThrowSpecifier, TrailingReturnType, TypeQualifier),
        },
        AbstractPointerDeclarator {
            declarator: (AbstractDeclarator,),
            _cs: (TypeQualifier,),
        },
        AliasDeclaration { r#type: (TypeDescriptor,), name: (TypeIdentifier,) },
        ArrayDeclarator {
            declarator: (Declarator, FieldDeclarator, TypeDeclarator),
            size: (Star, Expression),
            _cs: (TypeQualifier,),
        },
        AssignmentExpression {
            operator: (
                PercentEq,
                AmpEq,
                StarEq,
                PlusEq,
                DashEq,
                SlashEq,
                LtLtEq,
                Eq,
                GtGtEq,
                CaretEq,
                PipeEq,
            ),
            left: (
                CallExpression,
                FieldExpression,
                Identifier,
                ParenthesizedExpression,
                PointerExpression,
                QualifiedIdentifier,
                SubscriptExpression,
            ),
            right: (Expression,),
        },
        Attribute { name: (Identifier,), prefix: (Identifier,), _cs: (ArgumentList,) },
        BinaryExpression {
            right: (Expression, PreprocDefined),
            left: (Expression, PreprocDefined),
            operator: (
                BangEq,
                Percent,
                Amp,
                AmpAmp,
                Star,
                Plus,
                Dash,
                Slash,
                LT,
                LtLt,
                LTEq,
                EqEq,
                GT,
                GTEq,
                GtGt,
                Caret,
                Pipe,
                PipePipe,
            ),
        },
        CallExpression { function: (Expression, PrimitiveType), arguments: (ArgumentList,) },
        CaseStatement {
            value: (Expression,),
            _cs: (
                AttributedStatement,
                BreakStatement,
                CoReturnStatement,
                CoYieldStatement,
                CompoundStatement,
                ContinueStatement,
                Declaration,
                DoStatement,
                ExpressionStatement,
                ForRangeLoop,
                ForStatement,
                GotoStatement,
                IfStatement,
                LabeledStatement,
                ReturnStatement,
                SwitchStatement,
                ThrowStatement,
                TryStatement,
                TypeDefinition,
                WhileStatement,
            ),
        },
        CastExpression { value: (Expression,), r#type: (TypeDescriptor,) },
        CatchClause { body: (CompoundStatement,), parameters: (ParameterList,) },
        ClassSpecifier {
            body: (FieldDeclarationList,),
            name: (QualifiedIdentifier, TemplateType, TypeIdentifier),
            _cs: (BaseClassClause, MsDeclspecModifier, VirtualSpecifier),
        },
        CoAwaitExpression { argument: (Expression,), operator: (CoAwait,) },
        CommaExpression { left: (Expression,), right: (Expression, CommaExpression) },
        CompoundLiteralExpression {
            value: (InitializerList,),
            r#type: (QualifiedIdentifier, TemplateType, TypeDescriptor, TypeIdentifier),
        },
        ConditionClause {
            value: (Expression, CommaExpression, Declaration),
            initializer: (Declaration, ExpressionStatement),
        },
        ConditionalExpression {
            alternative: (Expression,),
            condition: (Expression,),
            consequence: (Expression,),
        },
        Declaration {
            value: (Expression, InitializerList),
            declarator: (Declarator, InitDeclarator, OperatorCast),
            default_value: (Expression,),
            r#type: (TypeSpecifier,),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                ExplicitFunctionSpecifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        DoStatement { condition: (ParenthesizedExpression,), body: (Statement,) },
        EnumSpecifier {
            base: (QualifiedIdentifier, SizedTypeSpecifier, TypeIdentifier),
            body: (EnumeratorList,),
            name: (QualifiedIdentifier, TemplateType, TypeIdentifier),
        },
        Enumerator { name: (Identifier,), value: (Expression,) },
        FieldDeclaration {
            r#type: (TypeSpecifier,),
            default_value: (Expression, InitializerList),
            declarator: (FieldDeclarator,),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                BitfieldClause,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        FieldExpression {
            argument: (Expression,),
            field: (DependentName, DestructorName, FieldIdentifier, TemplateMethod),
            operator: (DashGt, Dot),
        },
        ForRangeLoop {
            right: (Expression, InitializerList),
            declarator: (Declarator,),
            body: (Statement,),
            r#type: (TypeSpecifier,),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        ForStatement {
            condition: (Expression,),
            update: (Expression, CommaExpression),
            initializer: (Expression, CommaExpression, Declaration),
            _cs: (Statement,),
        },
        FunctionDeclarator {
            declarator: (Declarator, FieldDeclarator, TypeDeclarator),
            parameters: (ParameterList,),
            _cs: (
                AttributeSpecifier,
                Noexcept,
                RefQualifier,
                ThrowSpecifier,
                TrailingReturnType,
                TypeQualifier,
                VirtualSpecifier,
            ),
        },
        FunctionDefinition {
            body: (CompoundStatement,),
            r#type: (TypeSpecifier,),
            declarator: (Declarator, FieldDeclarator, OperatorCast),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                DefaultMethodClause,
                DeleteMethodClause,
                ExplicitFunctionSpecifier,
                FieldInitializerList,
                MsCallModifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        GotoStatement { label: (StatementIdentifier,) },
        IfStatement {
            alternative: (Statement,),
            condition: (ConditionClause,),
            consequence: (Statement,),
        },
        InitDeclarator {
            declarator: (Declarator,),
            value: (Expression, ArgumentList, InitializerList),
        },
        InitializerPair {
            value: (Expression, InitializerList),
            designator: (FieldDesignator, SubscriptDesignator),
        },
        LabeledStatement { label: (StatementIdentifier,), _cs: (Statement,) },
        LambdaExpression {
            body: (CompoundStatement,),
            declarator: (AbstractFunctionDeclarator,),
            captures: (LambdaCaptureSpecifier,),
        },
        LinkageSpecification {
            value: (StringLiteral,),
            body: (Declaration, DeclarationList, FunctionDefinition),
        },
        NamespaceDefinition {
            name: (Identifier, NamespaceDefinitionName),
            body: (DeclarationList,),
        },
        NewDeclarator { length: (Expression,), _cs: (NewDeclarator,) },
        NewExpression {
            placement: (ArgumentList,),
            arguments: (ArgumentList, InitializerList),
            declarator: (NewDeclarator,),
            r#type: (TypeSpecifier,),
        },
        OperatorCast {
            r#type: (TypeSpecifier,),
            declarator: (AbstractDeclarator,),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        OptionalParameterDeclaration {
            r#type: (TypeSpecifier,),
            declarator: (Declarator,),
            default_value: (Expression,),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        OptionalTypeParameterDeclaration {
            name: (TypeIdentifier,),
            default_type: (TypeSpecifier,),
        },
        ParameterDeclaration {
            declarator: (AbstractDeclarator, Declarator),
            r#type: (TypeSpecifier,),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        ParameterPackExpansion { pattern: (Expression, TypeDescriptor) },
        PointerDeclarator {
            declarator: (Declarator, FieldDeclarator, TypeDeclarator),
            _cs: (MsBasedModifier, MsPointerModifier, TypeQualifier),
        },
        PointerExpression { argument: (Expression,), operator: (Amp, Star) },
        PreprocCall { directive: (PreprocDirective,), argument: (PreprocArg,) },
        PreprocDef { value: (PreprocArg,), name: (Identifier,) },
        PreprocElif {
            alternative: (PreprocElif, PreprocElse),
            condition: (
                BinaryExpression,
                CallExpression,
                CharLiteral,
                Identifier,
                NumberLiteral,
                ParenthesizedExpression,
                PreprocDefined,
                UnaryExpression,
            ),
            _cs: (
                Statement,
                TypeSpecifier,
                AccessSpecifier,
                AliasDeclaration,
                AttributedStatement,
                Declaration,
                FieldDeclaration,
                FriendDeclaration,
                FunctionDefinition,
                LinkageSpecification,
                NamespaceDefinition,
                PreprocCall,
                PreprocDef,
                PreprocFunctionDef,
                PreprocIf,
                PreprocIfdef,
                PreprocInclude,
                StaticAssertDeclaration,
                TemplateDeclaration,
                TemplateInstantiation,
                TypeDefinition,
                UsingDeclaration,
            ),
        },
        PreprocFunctionDef {
            name: (Identifier,),
            value: (PreprocArg,),
            parameters: (PreprocParams,),
        },
        PreprocIf {
            condition: (
                BinaryExpression,
                CallExpression,
                CharLiteral,
                Identifier,
                NumberLiteral,
                ParenthesizedExpression,
                PreprocDefined,
                UnaryExpression,
            ),
            alternative: (PreprocElif, PreprocElse),
            _cs: (
                Statement,
                TypeSpecifier,
                AccessSpecifier,
                AliasDeclaration,
                AttributedStatement,
                Declaration,
                FieldDeclaration,
                FriendDeclaration,
                FunctionDefinition,
                LinkageSpecification,
                NamespaceDefinition,
                PreprocCall,
                PreprocDef,
                PreprocFunctionDef,
                PreprocIf,
                PreprocIfdef,
                PreprocInclude,
                StaticAssertDeclaration,
                TemplateDeclaration,
                TemplateInstantiation,
                TypeDefinition,
                UsingDeclaration,
            ),
        },
        PreprocIfdef {
            name: (Identifier,),
            alternative: (PreprocElif, PreprocElse),
            _cs: (
                Statement,
                TypeSpecifier,
                AccessSpecifier,
                AliasDeclaration,
                AttributedStatement,
                Declaration,
                FieldDeclaration,
                FriendDeclaration,
                FunctionDefinition,
                LinkageSpecification,
                NamespaceDefinition,
                PreprocCall,
                PreprocDef,
                PreprocFunctionDef,
                PreprocIf,
                PreprocIfdef,
                PreprocInclude,
                StaticAssertDeclaration,
                TemplateDeclaration,
                TemplateInstantiation,
                TypeDefinition,
                UsingDeclaration,
            ),
        },
        PreprocInclude {
            path: (CallExpression, Identifier, StringLiteral, SystemLibString),
        },
        QualifiedIdentifier {
            scope: (DependentName, NamespaceIdentifier, TemplateType),
            name: (
                DependentName,
                DestructorName,
                FieldIdentifier,
                Identifier,
                OperatorCast,
                OperatorName,
                QualifiedIdentifier,
                TemplateFunction,
                TemplateMethod,
                TemplateType,
                TypeIdentifier,
            ),
        },
        SizedTypeSpecifier { r#type: (PrimitiveType, TypeIdentifier) },
        SizeofExpression { value: (Expression,), r#type: (TypeDescriptor,) },
        StaticAssertDeclaration {
            condition: (Expression,),
            message: (ConcatenatedString, RawStringLiteral, StringLiteral),
        },
        StructSpecifier {
            body: (FieldDeclarationList,),
            name: (QualifiedIdentifier, TemplateType, TypeIdentifier),
            _cs: (BaseClassClause, MsDeclspecModifier, VirtualSpecifier),
        },
        SubscriptExpression {
            index: (Expression, InitializerList),
            argument: (Expression,),
        },
        SwitchStatement { body: (CompoundStatement,), condition: (ConditionClause,) },
        TemplateDeclaration {
            parameters: (TemplateParameterList,),
            _cs: (
                TypeSpecifier,
                AliasDeclaration,
                Declaration,
                FunctionDefinition,
                TemplateDeclaration,
            ),
        },
        TemplateFunction { arguments: (TemplateArgumentList,), name: (Identifier,) },
        TemplateInstantiation {
            declarator: (Declarator,),
            r#type: (TypeSpecifier,),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        TemplateMethod { name: (FieldIdentifier,), arguments: (TemplateArgumentList,) },
        TemplateTemplateParameterDeclaration {
            parameters: (TemplateParameterList,),
            _cs: (
                OptionalTypeParameterDeclaration,
                TypeParameterDeclaration,
                VariadicTypeParameterDeclaration,
            ),
        },
        TemplateType { name: (TypeIdentifier,), arguments: (TemplateArgumentList,) },
        TryStatement { body: (CompoundStatement,), _cs: (CatchClause,) },
        TypeDefinition {
            declarator: (TypeDeclarator,),
            r#type: (TypeSpecifier,),
            _cs: (TypeQualifier,),
        },
        TypeDescriptor {
            r#type: (TypeSpecifier,),
            declarator: (AbstractDeclarator,),
            _cs: (TypeQualifier,),
        },
        UnaryExpression {
            argument: (Expression, PreprocDefined),
            operator: (Bang, Plus, Dash, Tilde),
        },
        UnionSpecifier {
            name: (QualifiedIdentifier, TemplateType, TypeIdentifier),
            body: (FieldDeclarationList,),
            _cs: (BaseClassClause, MsDeclspecModifier, VirtualSpecifier),
        },
        UpdateExpression { operator: (PlusPlus, DashDash), argument: (Expression,) },
        VariadicParameterDeclaration {
            r#type: (TypeSpecifier,),
            declarator: (ReferenceDeclarator, VariadicDeclarator),
            _cs: (
                AttributeDeclaration,
                AttributeSpecifier,
                MsDeclspecModifier,
                StorageClassSpecifier,
                TypeQualifier,
                VirtualFunctionSpecifier,
            ),
        },
        WhileStatement { condition: (ConditionClause,), body: (Statement,) },
    }
    Abstract {
        #[strum(serialize = "_abstract_declarator")]
        AbstractDeclarator(),
        #[strum(serialize = "_declarator")]
        Declarator(),
        #[strum(serialize = "_expression")]
        Expression(),
        #[strum(serialize = "_field_declarator")]
        FieldDeclarator(),
        #[strum(serialize = "_statement")]
        Statement(),
        #[strum(serialize = "_type_declarator")]
        TypeDeclarator(),
        #[strum(serialize = "_type_specifier")]
        TypeSpecifier(),
    }


}
