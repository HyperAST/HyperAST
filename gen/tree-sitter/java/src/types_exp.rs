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
        impl hyperast::types::Lang for Language {
            type Factory = Factory;
            type Type = Type;
        }
    };
}
use paste::paste;
macro_rules! make_type2 {
    () => {};
    (@as_expr $($i:expr)+) => {$($i)+};
    (@as_item $($i:item)+) => {$($i)+};
    (@as_lit $var:ident) => {paste!{stringify!([<$var:snake>])}};
    (@as_lit $l:literal) => {$l};


    (
        @colct_disc ($(,)*) -> ($($var_names:ident -> ($($var_ser:tt)*),)*)
    ) => {
        make_type2! {
            @as_item
            pub enum Type {
                $($var_names),*
            }

            static ALL: &'static [Type] = &[$(Type::$var_names),*];

            impl Type {
                fn slice() -> &'static [Self] {
                    ALL
                }
            }
            impl Type {
                fn to_string(&self) -> String {
                    match self {$(
                        Self::$var_names => make_type2! { @as_lit $($var_ser)* },
                    )*}.to_string()
                }
            }
        }

    };

    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident,
    //                 $(#[$_attr1:meta])* $var1:ident,
    //                 $(#[$_attr2:meta])* $var2:ident,
    //                 $(#[$_attr3:meta])* $var3:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)* $var, $var1, $var3, $var2, )}
    // };

    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident, $(#[$_attr1:meta])* $var1:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)* $var, $var1, )}
    // };

    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)* $var, )}
    // };

    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident($_lit:literal),
    //     $(#[$_attr1:meta])* $var1:ident($_lit1:literal),
    //     $(#[$_attr2:meta])* $var2:ident($_lit2:literal),
    //     $(#[$_attr3:meta])* $var3:ident($_lit3:literal),
    //     $(#[$_attr4:meta])* $var4:ident($_lit4:literal),
    //     $(#[$_attr5:meta])* $var5:ident($_lit5:literal),
    //     $(#[$_attr6:meta])* $var6:ident($_lit6:literal),
    //     $(#[$_attr7:meta])* $var7:ident($_lit7:literal), $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)*
    //         $var -> ($_lit),
    //         $var1 -> ($_lit1),
    //         $var2 -> ($_lit2),
    //         $var3 -> ($_lit3),
    //         $var4 -> ($_lit4),
    //         $var5 -> ($_lit5),
    //         $var6 -> ($_lit6),
    //         $var7 -> ($_lit7), )}
    // };

    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident($_lit:literal),
    //     $(#[$_attr1:meta])* $var1:ident($_lit1:literal),
    //     $(#[$_attr2:meta])* $var2:ident($_lit2:literal),
    //     $(#[$_attr3:meta])* $var3:ident($_lit3:literal),
    //     $(#[$_attr4:meta])* $var4:ident($_lit4:literal),
    //     $(#[$_attr5:meta])* $var5:ident($_lit5:literal),
    //     $(#[$_attr6:meta])* $var6:ident($_lit6:literal),
    //     $(#[$_attr7:meta])* $var7:ident($_lit7:literal),
    //     $(#[$_attr8:meta])* $var8:ident($_lit8:literal),
    //     $(#[$_attr9:meta])* $var9:ident($_lit9:literal),
    //     $(#[$_attr10:meta])* $var10:ident($_lit10:literal),
    //     $(#[$_attr11:meta])* $var11:ident($_lit11:literal),
    //     $(#[$_attr12:meta])* $var12:ident($_lit12:literal),
    //     $(#[$_attr13:meta])* $var13:ident($_lit13:literal),
    //     $(#[$_attr14:meta])* $var14:ident($_lit14:literal),
    //     $(#[$_attr15:meta])* $var15:ident($_lit15:literal), $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)*
    //         $var -> ($_lit),
    //         $var1 -> ($_lit1),
    //         $var2 -> ($_lit2),
    //         $var3 -> ($_lit3),
    //         $var4 -> ($_lit4),
    //         $var5 -> ($_lit5),
    //         $var6 -> ($_lit6),
    //         $var7 -> ($_lit7), )}
    // };

    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident($_lit:literal),
    //                 $(#[$_attr1:meta])* $var1:ident($_lit1:literal),
    //                 $(#[$_attr2:meta])* $var2:ident($_lit2:literal),
    //                 $(#[$_attr3:meta])* $var3:ident($_lit3:literal), $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)*
    //         $var -> ($_lit),
    //         $var1 -> ($_lit1),
    //         $var2 -> ($_lit2),
    //         $var3 -> ($_lit3), )}
    // };


    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident,
    //                 $(#[$_attr1:meta])* $var1:ident,
    //                 $(#[$_attr2:meta])* $var2:ident,
    //                 $(#[$_attr3:meta])* $var3:ident,
    //                 $(#[$_attr4:meta])* $var4:ident,
    //                 $(#[$_attr5:meta])* $var5:ident,
    //                 $(#[$_attr6:meta])* $var6:ident,
    //                 $(#[$_attr7:meta])* $var7:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)*
    //         $var  -> ($var),
    //         $var1 -> ($var1),
    //         $var3 -> ($var2),
    //         $var3 -> ($var3),
    //         $var4 -> ($var4),
    //         $var5 -> ($var5),
    //         $var6 -> ($var6),
    //         $var7 -> ($var7), )}
    // };

    // (
    //     @colct_disc ($(#[$_attr:meta])* $var:ident,
    //                 $(#[$_attr1:meta])* $var1:ident,
    //                 $(#[$_attr2:meta])* $var2:ident,
    //                 $(#[$_attr3:meta])* $var3:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc ($($tail)*) -> ($($var_names)*
    //         $var  -> ($var),
    //         $var1 -> ($var1),
    //         $var2 -> ($var2),
    //         $var3 -> ($var3), )}
    // };

    (
        @colct_disc ($(#[$_attr:meta])* $var:ident($_lit:literal), $(#[$_attr1:meta])* $var1:ident($_lit1:literal), $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        make_type2!{@colct_disc ($($tail)*) -> ($($var_names)* $var -> ($_lit), $var1 -> ($_lit1), )}
    };


    (
        @colct_disc ($(#[$_attr:meta])* $var:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {

        make_type2!{@colct_disc ($($tail)*) -> ($($var_names)* $var -> ($var), )}
    };
    (
        @colct_disc ($(#[$_attr:meta])* $var:ident($_lit:literal), $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        make_type2!{@colct_disc ($($tail)*) -> ($($var_names)* $var -> ($_lit), )}
    };

    (
        @colct_disc2 ($(,)* ; $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        make_type2!{@colct_disc ($($tail)*) -> ($($var_names)*)}
    };

    // (
    //     @colct_disc2 {$($kkk:tt)*} ($(#[$_attr:meta])* $var:ident($_a:ty $(,)*),
    //     $(#[$_attr1:meta])* $var1:ident($_a1:ty $(,)*),
    //     $(#[$_attr2:meta])* $var2:ident($_a2:ty $(,)*),
    //     $(#[$_attr3:meta])* $var3:ident($_a3:ty $(,)*), $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc2 {$($kkk)*} ($($tail)*) -> ($($var_names)*
    //         $var -> ($var),
    //         $var1 -> ($var1),
    //         $var2 -> ($var2),
    //         $var3 -> ($var3), )}
    // };

    // (
    //     @colct_disc2 {$($kkk:tt)*} ($(#[$_attr:meta])* $var:ident($_a:ty $(,)*),
    //                                $(#[$_attr1:meta])* $var1:ident($_a1:ty $(,)*), $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     make_type2!{@colct_disc2 {$($kkk)*} ($($tail)*) -> ($($var_names)* $var -> ($var), $var1 -> ($var1), )}
    // };

    (
        @colct_disc2
            ($(#[$_attr:meta])* $var:ident($($_a:ty),* $(,)*), $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        make_type2!{@colct_disc2 ($($tail)*) -> ($($var_names)* $var -> ($var), )}
    };

    // (
    //     @colct_disc2 {$($kkk:tt)*} ($(#[$_attr:meta])* $var:ident, $(#[$_attr1:meta])* $var1:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {

    //     make_type2!{@colct_disc2 {$($kkk)*} ($($tail)*) -> ($($var_names)* $var -> ($var), $var1 -> ($var1), )}
    // };

    (
        @colct_disc2
            ($(#[$_attr:meta])* $var:ident, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {

        make_type2!{@colct_disc2 ($($tail)*) -> ($($var_names)* $var -> ($var), )}
    };

    (
        @colct_disc3 ($(,)* ; $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        make_type2!{@colct_disc2 ($($tail)*) -> ($($var_names)*)}
    };

    (
        @colct_disc3
            ($(#[$_attr:meta])* $var:ident{$($_a:tt)*}, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {

        make_type2!{@colct_disc3 ($($tail)*) -> ($($var_names)* $var -> ($var), )}
    };


    // (@t {$($kkk:tt)*} $($ccc:tt)*) => {
    //     make_type2!{@collect $($kkk)*}
    // };

    (@k $($kkk:tt)*) => {};
    (
        $(#[$km0:meta])*
        Keyword {$($kkk:tt)*}
        $(#[$cm0:meta])*
        Concrete {$($ccc:tt)*}
        $(#[$wm0:meta])*
        WithFields {$($www:tt)*}
        $(#[$am0:meta])*
        Abstract {$($aaa:tt)*}
    ) => {
        // make_type2!{@colct_disc ($($kkk)*,) -> ()}
        make_type2!{@colct_disc3 ($($www)* ; $($ccc)* ; $($kkk)*) -> ()}
        // make_type2!{@t {$($kkk)*} $($ccc)*}
        // make_type2!{@k $($kkk)*}
    };
}

// fn f () {
//     dbg!(std::mem::variant_count::<Type>());
// }

make_type2! {
    Keyword {
        Bang("!"),
        BangEq("!="),
        Percent("%"),
        PercentEq("%="),
        Amp("&"),
        AmpAmp("&&"),
        AmpEq("&="),
        LParen("("),
        RParen(")"),
        Star("*"),
        StarEq("*="),
        Plus("+"),
        PlusPlus("++"),
        PlusEq("+="),
        Comma(","),
        Dash("-"),
        DashDash("--"),
        DashEq("-="),
        DashGt("->"),
        Dot("."),
        DotDotDot("..."),
        Slash("/"),
        SlashEq("/="),
        Colon(":"),
        ColonColon("::"),
        SemiColon(";"),
        LtLt("<<"),
        LtLtEq("<<="),
        LTEq("<="),
        Eq("="),
        EqEq("=="),
        LT("<"),
        GT(">"),
        GTEq(">="),
        GtGt(">>"),
        GtGtEq(">>="),
        GtGtGt(">>>"),
        GtGtGtEq(">>>="),
        QMark("?"),
        At("@"),
        TS0("@interface"),
        LBracket("["),
        RBracket("]"),
        Caret("^"),
        CaretEq("^="),
        LBrace("{"),
        RBrace("}"),
        Pipe("|"),
        PipeEq("|="),
        PipePipe("||"),
        Tilde("~"),
        TS1("non-sealed"),
        Abstract,
        Assert,
        Break,
        Byte,
        Case,
        Catch,
        Char,
        Class,
        Continue,
        Default,
        Do,
        Double,
        Else,
        Enum,
        Exports,
        Extends,
        Final,
        Finally,
        Float,
        For,
        If,
        Implements,
        Import,
        Instanceof,
        Int,
        Interface,
        Long,
        Module,
        Native,
        New,
        Open,
        Opens,
        Package,
        Permits,
        Private,
        Protected,
        Provides,
        Public,
        Record,
        Requires,
        Return,
        Sealed,
        Short,
        Static,
        Strictfp,
        Switch,
        Synchronized,
        Throw,
        Throws,
        To,
        Transient,
        Transitive,
        Try,
        Uses,
        Volatile,
        While,
        With,
        Yield,
    }
    /// Type of nodes actually stored
    /// ie. what should be stored on CST nodes
    /// but anyway encode it as a number
    /// and it would be better to take the smallest numbers for concrete nodes
    /// to facilitate convertion
    Concrete {
        Asterisk,
        BinaryIntegerLiteral,
        AnnotatedType(MultReq<(UnannotatedType, Annotation, MarkerAnnotation,)>),
        CharacterLiteral,
        AnnotationArgumentList(Mult<(
            Annotation,
            ElementValueArrayInitializer,
            ElementValuePair,
            Expression,
            MarkerAnnotation,
        ),>,),
        AnnotationTypeBody(
            Mult<
                (
                    AnnotationTypeDeclaration,
                    AnnotationTypeElementDeclaration,
                    ClassDeclaration,
                    ConstantDeclaration,
                    EnumDeclaration,
                    InterfaceDeclaration,
                ),
            >,
        ),
        ArgumentList(Mult<(Expression,)>),
        ArrayInitializer(Mult<(ArrayInitializer, Expression)>),
        AssertStatement(MultReq<(Expression,)>),
        Block(Mult<(Statement,)>),
        BlockComment,
        BooleanType,
        BreakStatement(Identifier),
        CatchType(MultReq<(UnannotatedType,)>),
        ClassBody(
            Mult<
                (
                    AnnotationTypeDeclaration,
                    Block,
                    ClassDeclaration,
                    CompactConstructorDeclaration,
                    ConstructorDeclaration,
                    EnumDeclaration,
                    FieldDeclaration,
                    InterfaceDeclaration,
                    MethodDeclaration,
                    RecordDeclaration,
                    StaticInitializer,
                ),
            >,
        ),
        ClassLiteral(Req<(UnannotatedType,)>),
        ConstructorBody(Mult<(ExplicitConstructorInvocation, Statement)>),
        ContinueStatement(Identifier),
        DecimalFloatingPointLiteral,
        DecimalIntegerLiteral,
        Dimensions(Mult<(Annotation, MarkerAnnotation)>),
        DimensionsExpr(MultReq<(Annotation, Expression, MarkerAnnotation)>),
        ElementValueArrayInitializer(
            Mult<(Annotation, ElementValueArrayInitializer, Expression, MarkerAnnotation)>,
        ),
        EnumBody(Mult<(EnumBodyDeclarations, EnumConstant)>),
        EnumBodyDeclarations(
            Mult<
                (
                    AnnotationTypeDeclaration,
                    Block,
                    ClassDeclaration,
                    CompactConstructorDeclaration,
                    ConstructorDeclaration,
                    EnumDeclaration,
                    FieldDeclaration,
                    InterfaceDeclaration,
                    MethodDeclaration,
                    RecordDeclaration,
                    StaticInitializer,
                ),
            >,
        ),
        ExpressionStatement(Req<(Expression,)>),
        ExtendsInterfaces(Req<(TypeList,)>),
        False,
        FinallyClause(Req<(Block,)>),
        FloatingPointType,
        FormalParameters(Mult<(FormalParameter, ReceiverParameter, SpreadParameter)>),
        GenericType(MultReq<(ScopedTypeIdentifier, TypeArguments, TypeIdentifier)>),
        HexFloatingPointLiteral,
        HexIntegerLiteral,
        Identifier,
        ImportDeclaration(MultReq<(Asterisk, Identifier, ScopedIdentifier)>),
        InferredParameters(MultReq<(Identifier,)>),
        IntegralType,
        InterfaceBody(
            Mult<
                (
                    AnnotationTypeDeclaration,
                    ClassDeclaration,
                    ConstantDeclaration,
                    EnumDeclaration,
                    InterfaceDeclaration,
                    MethodDeclaration,
                    RecordDeclaration,
                ),
            >,
        ),
        LabeledStatement(MultReq<(Identifier, Statement)>),
        LineComment,
        MethodReference(MultReq<(Type, PrimaryExpression, Super, TypeArguments)>),
        Modifiers(Mult<(Annotation, MarkerAnnotation)>),
        ModuleBody(Mult<(ModuleDirective,)>),
        NullLiteral,
        OctalIntegerLiteral,
        PackageDeclaration(
            MultReq<(Annotation, Identifier, MarkerAnnotation, ScopedIdentifier)>,
        ),
        ParenthesizedExpression(Req<(Expression,)>),
        Program(Mult<(Statement,)>),
        ReceiverParameter(
            MultReq<(UnannotatedType, Annotation, Identifier, MarkerAnnotation, This)>,
        ),
        RequiresModifier,
        ResourceSpecification(MultReq<(Resource,)>),
        ReturnStatement(Expression),
        ScopedTypeIdentifier(
            MultReq<
                (
                    Annotation,
                    GenericType,
                    MarkerAnnotation,
                    ScopedTypeIdentifier,
                    TypeIdentifier,
                ),
            >,
        ),
        SpreadParameter(MultReq<(UnannotatedType, Modifiers, VariableDeclarator)>),
        StaticInitializer(Req<(Block,)>),
        StringLiteral,
        Super,
        SuperInterfaces(Req<(TypeList,)>),
        Superclass(Req<(Type,)>),
        SwitchBlock(Mult<(SwitchBlockStatementGroup, SwitchRule)>),
        SwitchBlockStatementGroup(MultReq<(Statement, SwitchLabel)>),
        SwitchLabel(Mult<(Expression,)>),
        SwitchRule(MultReq<(Block, ExpressionStatement, SwitchLabel, ThrowStatement)>),
        TextBlock,
        This,
        ThrowStatement(Req<(Expression,)>),
        True,
        TypeArguments(Mult<(Type, Wildcard)>),
        TypeBound(MultReq<(Type,)>),
        TypeIdentifier,
        TypeList(MultReq<(Type,)>),
        TypeParameter(MultReq<(Annotation, MarkerAnnotation, TypeBound, TypeIdentifier)>),
        TypeParameters(MultReq<(TypeParameter,)>),
        UpdateExpression(Req<(Expression,)>),
        VoidType,
        Wildcard(Mult<(Type, Annotation, MarkerAnnotation, Super)>),
        YieldStatement(Req<(Expression,)>),
    }
    WithFields {
        Annotation {
            name: Req<(Identifier, ScopedIdentifier)>,
            arguments: Req<(AnnotationArgumentList,)>,
        },
        AnnotationTypeDeclaration {
            name: Req<(Identifier,)>,
            body: Req<(AnnotationTypeBody,)>,
            _cs: (Modifiers,),
        },
        AnnotationTypeElementDeclaration {
            value: (Annotation, ElementValueArrayInitializer, Expression, MarkerAnnotation),
            dimensions: (Dimensions,),
            r#type: Req<(UnannotatedType,)>,
            name: Req<(Identifier,)>,
            _cs: (Modifiers,),
        },
        ArrayAccess { index: Req<(Expression,)>, array: Req<(PrimaryExpression,)> },
        ArrayCreationExpression {
            r#type: Req<(SimpleType,)>,
            dimensions: MultReq<(Dimensions, DimensionsExpr)>,
            value: (ArrayInitializer,),
            _cs: Mult<(Annotation, MarkerAnnotation)>,
        },
        ArrayType { element: Req<(UnannotatedType,)>, dimensions: Req<(Dimensions,)> },
        AssignmentExpression {
            right: Req<(Expression,)>,
            operator: Req<
                (
                    PercentEq,
                    AmpEq,
                    StarEq,
                    PlusEq,
                    DashEq,
                    SlashEq,
                    LtLtEq,
                    Eq,
                    GtGtEq,
                    GtGtGtEq,
                    CaretEq,
                    PipeEq,
                ),
            >,
            left: Req<(ArrayAccess, FieldAccess, Identifier)>,
        },
        BinaryExpression {
            left: Req<(Expression,)>,
            operator: Req<
                (
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
                    GtGtGt,
                    Caret,
                    Pipe,
                    PipePipe,
                ),
            >,
            right: Req<(Expression,)>,
        },
        CastExpression { r#type: MultReq<(Type,)>, value: Req<(Expression,)> },
        CatchClause { body: Req<(Block,)>, _cs: Req<(CatchFormalParameter,)> },
        CatchFormalParameter {
            dimensions: (Dimensions,),
            name: Req<(Identifier,)>,
            _cs: MultReq<(CatchType, Modifiers)>,
        },
        ClassDeclaration {
            interfaces: (SuperInterfaces,),
            name: Req<(Identifier,)>,
            permits: (Permits,),
            type_parameters: (TypeParameters,),
            body: Req<(ClassBody,)>,
            superclass: (Superclass,),
            _cs: (Modifiers,),
        },
        CompactConstructorDeclaration {
            body: Req<(Block,)>,
            name: Req<(Identifier,)>,
            _cs: (Modifiers,),
        },
        ConstantDeclaration {
            r#type: Req<(UnannotatedType,)>,
            declarator: MultReq<(VariableDeclarator,)>,
            _cs: (Modifiers,),
        },
        ConstructorDeclaration {
            body: Req<(ConstructorBody,)>,
            parameters: Req<(FormalParameters,)>,
            name: Req<(Identifier,)>,
            type_parameters: (TypeParameters,),
            _cs: Mult<(Modifiers, Throws)>,
        },
        DoStatement { condition: Req<(ParenthesizedExpression,)>, body: Req<(Statement,)> },
        ElementValuePair {
            key: Req<(Identifier,)>,
            value: Req<
                (Annotation, ElementValueArrayInitializer, Expression, MarkerAnnotation),
            >,
        },
        EnhancedForStatement {
            r#type: Req<(UnannotatedType,)>,
            dimensions: (Dimensions,),
            body: Req<(Statement,)>,
            value: Req<(Expression,)>,
            name: Req<(Identifier,)>,
            _cs: (Modifiers,),
        },
        EnumConstant {
            arguments: (ArgumentList,),
            body: (ClassBody,),
            name: Req<(Identifier,)>,
            _cs: (Modifiers,),
        },
        EnumDeclaration {
            interfaces: (SuperInterfaces,),
            body: Req<(EnumBody,)>,
            name: Req<(Identifier,)>,
            _cs: (Modifiers,),
        },
        ExplicitConstructorInvocation {
            arguments: Req<(ArgumentList,)>,
            constructor: Req<(Super, This)>,
            object: (PrimaryExpression,),
            type_arguments: (TypeArguments,),
        },
        ExportsModuleDirective {
            modules: Mult<(Identifier, ScopedIdentifier)>,
            package: Req<(Identifier, ScopedIdentifier)>,
        },
        FieldAccess {
            object: Req<(PrimaryExpression, Super)>,
            field: Req<(Identifier, This)>,
            _cs: (Super,),
        },
        FieldDeclaration {
            declarator: MultReq<(VariableDeclarator,)>,
            r#type: Req<(UnannotatedType,)>,
            _cs: (Modifiers,),
        },
        ForStatement {
            condition: (Expression,),
            init: Mult<(Expression, LocalVariableDeclaration)>,
            update: Mult<(Expression,)>,
            body: Req<(Statement,)>,
        },
        FormalParameter {
            r#type: Req<(UnannotatedType,)>,
            name: Req<(Identifier,)>,
            dimensions: (Dimensions,),
            _cs: (Modifiers,),
        },
        IfStatement {
            consequence: Req<(Statement,)>,
            alternative: (Statement,),
            condition: Req<(ParenthesizedExpression,)>,
        },
        InstanceofExpression {
            name: (Identifier,),
            left: Req<(Expression,)>,
            right: Req<(Type,)>,
        },
        InterfaceDeclaration {
            body: Req<(InterfaceBody,)>,
            permits: (Permits,),
            name: Req<(Identifier,)>,
            type_parameters: (TypeParameters,),
            _cs: Mult<(ExtendsInterfaces, Modifiers)>,
        },
        LambdaExpression {
            body: Req<(Block, Expression)>,
            parameters: Req<(FormalParameters, Identifier, InferredParameters)>,
        },
        LocalVariableDeclaration {
            r#type: Req<(UnannotatedType,)>,
            declarator: MultReq<(VariableDeclarator,)>,
            _cs: (Modifiers,),
        },
        MarkerAnnotation { name: Req<(Identifier, ScopedIdentifier)> },
        MethodDeclaration {
            name: Req<(Identifier,)>,
            parameters: Req<(FormalParameters,)>,
            r#type: Req<(UnannotatedType,)>,
            type_parameters: (TypeParameters,),
            dimensions: (Dimensions,),
            body: (Block,),
            _cs: Mult<(Annotation, MarkerAnnotation, Modifiers, Throws)>,
        },
        MethodInvocation {
            type_arguments: (TypeArguments,),
            arguments: Req<(ArgumentList,)>,
            name: Req<(Identifier,)>,
            object: (PrimaryExpression, Super),
            _cs: (Super,),
        },
        ModuleDeclaration {
            body: Req<(ModuleBody,)>,
            name: Req<(Identifier, ScopedIdentifier)>,
            _cs: Mult<(Annotation, MarkerAnnotation)>,
        },
        ObjectCreationExpression {
            type_arguments: (TypeArguments,),
            r#type: Req<(SimpleType,)>,
            arguments: Req<(ArgumentList,)>,
            _cs: Mult<(ClassBody, PrimaryExpression)>,
        },
        OpensModuleDirective {
            package: Req<(Identifier, ScopedIdentifier)>,
            modules: Mult<(Identifier, ScopedIdentifier)>,
        },
        ProvidesModuleDirective {
            provided: Req<(Identifier, ScopedIdentifier)>,
            provider: Mult<(Identifier, ScopedIdentifier)>,
            _cs: Req<(Identifier, ScopedIdentifier)>,
        },
        RecordDeclaration {
            body: Req<(ClassBody,)>,
            name: Req<(Identifier,)>,
            parameters: Req<(FormalParameters,)>,
            interfaces: (SuperInterfaces,),
            type_parameters: (TypeParameters,),
            _cs: (Modifiers,),
        },
        RequiresModuleDirective {
            modifiers: Mult<(RequiresModifier,)>,
            module: Req<(Identifier, ScopedIdentifier)>,
        },
        Resource {
            value: (Expression,),
            dimensions: (Dimensions,),
            name: (Identifier,),
            r#type: (UnannotatedType,),
            _cs: (FieldAccess, Identifier, Modifiers),
        },
        ScopedIdentifier {
            name: Req<(Identifier,)>,
            scope: Req<(Identifier, ScopedIdentifier)>,
        },
        SwitchExpression {
            condition: Req<(ParenthesizedExpression,)>,
            body: Req<(SwitchBlock,)>,
        },
        SynchronizedStatement { body: Req<(Block,)>, _cs: Req<(ParenthesizedExpression,)> },
        TernaryExpression {
            alternative: Req<(Expression,)>,
            condition: Req<(Expression,)>,
            consequence: Req<(Expression,)>,
        },
        TryStatement { body: Req<(Block,)>, _cs: MultReq<(CatchClause, FinallyClause)> },
        TryWithResourcesStatement {
            body: Req<(Block,)>,
            resources: Req<(ResourceSpecification,)>,
            _cs: Mult<(CatchClause, FinallyClause)>,
        },
        UnaryExpression {
            operator: Req<(Bang, Plus, Dash, Tilde)>,
            operand: Req<(Expression,)>,
        },
        UsesModuleDirective { r#type: Req<(Identifier, ScopedIdentifier)> },
        VariableDeclarator {
            name: Req<(Identifier,)>,
            dimensions: (Dimensions,),
            value: (ArrayInitializer, Expression),
        },
        WhileStatement {
            body: Req<(Statement,)>,
            condition: Req<(ParenthesizedExpression,)>,
        },
    }
    Abstract {
        Literal(
            Raw<"_literal">,
            BinaryIntegerLiteral,
            CharacterLiteral,
            DecimalFloatingPointLiteral,
            DecimalIntegerLiteral,
            False,
            HexFloatingPointLiteral,
            HexIntegerLiteral,
            NullLiteral,
            OctalIntegerLiteral,
            StringLiteral,
            TextBlock,
            True,
        ),
        SimpleType(
            Raw<"_simple_type">,
            BooleanType,
            FloatingPointType,
            GenericType,
            IntegralType,
            ScopedTypeIdentifier,
            TypeIdentifier,
            VoidType,
        ),
        Type(Raw<"_type">, UnannotatedType, AnnotatedType),
        UnannotatedType(Raw<"_unannotated_type">, SimpleType, ArrayType),
        Comment(BlockComment, LineComment),
        Declaration(
            AnnotationTypeDeclaration,
            ClassDeclaration,
            EnumDeclaration,
            ImportDeclaration,
            InterfaceDeclaration,
            ModuleDeclaration,
            PackageDeclaration,
            RecordDeclaration,
        ),
        Expression(
            AssignmentExpression,
            BinaryExpression,
            CastExpression,
            InstanceofExpression,
            LambdaExpression,
            PrimaryExpression,
            SwitchExpression,
            TernaryExpression,
            UnaryExpression,
            UpdateExpression,
        ),
        ModuleDirective(
            ExportsModuleDirective,
            OpensModuleDirective,
            ProvidesModuleDirective,
            RequiresModuleDirective,
            UsesModuleDirective,
        ),
        PrimaryExpression(
            Literal,
            ArrayAccess,
            ArrayCreationExpression,
            ClassLiteral,
            FieldAccess,
            Identifier,
            MethodInvocation,
            MethodReference,
            ObjectCreationExpression,
            ParenthesizedExpression,
            This,
        ),
        Statement(
            SemiColon,
            AssertStatement,
            Block,
            BreakStatement,
            ContinueStatement,
            Declaration,
            DoStatement,
            EnhancedForStatement,
            ExpressionStatement,
            ForStatement,
            IfStatement,
            LabeledStatement,
            LocalVariableDeclaration,
            ReturnStatement,
            SwitchExpression,
            SynchronizedStatement,
            ThrowStatement,
            TryStatement,
            TryWithResourcesStatement,
            WhileStatement,
            YieldStatement,
        ),
    }
}

// make_type2! {
//     Keyword {
//         #[strum(serialize = "!")]
//         Bang,
//         #[strum(serialize = "!=")]
//         BangEq,
//         #[strum(serialize = "%")]
//         Percent,
//         #[strum(serialize = "%=")]
//         PercentEq,
//         #[strum(serialize = "&")]
//         Amp,
//         #[strum(serialize = "&&")]
//         AmpAmp,
//         #[strum(serialize = "&=")]
//         AmpEq,
//         #[strum(serialize = "(")]
//         LParen,
//         #[strum(serialize = ")")]
//         RParen,
//         #[strum(serialize = "*")]
//         Star,
//         #[strum(serialize = "*=")]
//         StarEq,
//         #[strum(serialize = "+")]
//         Plus,
//         #[strum(serialize = "++")]
//         PlusPlus,
//         #[strum(serialize = "+=")]
//         PlusEq,
//         #[strum(serialize = ",")]
//         Comma,
//         #[strum(serialize = "-")]
//         Dash,
//         #[strum(serialize = "--")]
//         DashDash,
//         #[strum(serialize = "-=")]
//         DashEq,
//         #[strum(serialize = "->")]
//         DashGt,
//         #[strum(serialize = ".")]
//         Dot,
//         #[strum(serialize = "...")]
//         DotDotDot,
//         #[strum(serialize = "/")]
//         Slash,
//         #[strum(serialize = "/=")]
//         SlashEq,
//         #[strum(serialize = ":")]
//         Colon,
//         #[strum(serialize = "::")]
//         ColonColon,
//         #[strum(serialize = ";")]
//         SemiColon,
//         #[strum(serialize = "<")]
//         LT,
//         #[strum(serialize = "<<")]
//         LtLt,
//         #[strum(serialize = "<<=")]
//         LtLtEq,
//         #[strum(serialize = "<=")]
//         LTEq,
//         #[strum(serialize = "=")]
//         Eq,
//         #[strum(serialize = "==")]
//         EqEq,
//         #[strum(serialize = ">")]
//         GT,
//         #[strum(serialize = ">=")]
//         GTEq,
//         #[strum(serialize = ">>")]
//         GtGt,
//         #[strum(serialize = ">>=")]
//         GtGtEq,
//         #[strum(serialize = ">>>")]
//         GtGtGt,
//         #[strum(serialize = ">>>=")]
//         GtGtGtEq,
//         #[strum(serialize = "?")]
//         QMark,
//         #[strum(serialize = "@")]
//         At,
//         #[strum(serialize = "@interface")]
//         TS0,
//         #[strum(serialize = "[")]
//         LBracket,
//         #[strum(serialize = "]")]
//         RBracket,
//         #[strum(serialize = "^")]
//         Caret,
//         #[strum(serialize = "^=")]
//         CaretEq,
//         Abstract,
//         Assert,
//         Break,
//         Byte,
//         Case,
//         Catch,
//         Char,
//         Class,
//         Continue,
//         Default,
//         Do,
//         Double,
//         Else,
//         Enum,
//         Exports,
//         Extends,
//         Final,
//         Finally,
//         Float,
//         For,
//         If,
//         Implements,
//         Import,
//         Instanceof,
//         Int,
//         Interface,
//         Long,
//         Module,
//         Native,
//         New,
//         #[strum(serialize = "non-sealed")]
//         TS1,
//         Open,
//         Opens,
//         Package,
//         Permits,
//         Private,
//         Protected,
//         Provides,
//         Public,
//         Record,
//         Requires,
//         Return,
//         Sealed,
//         Short,
//         Static,
//         Strictfp,
//         Switch,
//         Synchronized,
//         Throw,
//         Throws,
//         To,
//         Transient,
//         Transitive,
//         Try,
//         Uses,
//         Volatile,
//         While,
//         With,
//         Yield,
//         #[strum(serialize = "{")]
//         LBrace,
//         #[strum(serialize = "|")]
//         Pipe,
//         #[strum(serialize = "|=")]
//         PipeEq,
//         #[strum(serialize = "||")]
//         PipePipe,
//         #[strum(serialize = "}")]
//         RBrace,
//         #[strum(serialize = "~")]
//         Tilde,
//     }
//     /// Type of nodes actually stored
//     /// ie. what should be stored on CST nodes
//     /// but anyway encode it as a number
//     /// and it would be better to take the smallest numbers for concrete nodes
//     /// to facilitate convertion
//     Concrete {
//         AnnotatedType(UnannotatedType, Annotation, MarkerAnnotation),
//         AnnotationArgumentList(
//             Annotation,
//             ElementValueArrayInitializer,
//             ElementValuePair,
//             Expression,
//             MarkerAnnotation,
//         ),
//         AnnotationTypeBody(
//             AnnotationTypeDeclaration,
//             AnnotationTypeElementDeclaration,
//             ClassDeclaration,
//             ConstantDeclaration,
//             EnumDeclaration,
//             InterfaceDeclaration,
//         ),
//         ArgumentList(Expression),
//         ArrayInitializer(ArrayInitializer, Expression),
//         AssertStatement(Expression),
//         Asterisk,
//         BinaryIntegerLiteral,
//         Block(Statement),
//         BlockComment,
//         BooleanType,
//         BreakStatement(Identifier),
//         CatchType(UnannotatedType),
//         CharacterLiteral,
//         ClassBody(
//             AnnotationTypeDeclaration,
//             Block,
//             ClassDeclaration,
//             CompactConstructorDeclaration,
//             ConstructorDeclaration,
//             EnumDeclaration,
//             FieldDeclaration,
//             InterfaceDeclaration,
//             MethodDeclaration,
//             RecordDeclaration,
//             StaticInitializer,
//         ),
//         ClassLiteral(UnannotatedType),
//         ConstructorBody(ExplicitConstructorInvocation, Statement),
//         ContinueStatement(Identifier),
//         DecimalFloatingPointLiteral,
//         DecimalIntegerLiteral,
//         Dimensions(Annotation, MarkerAnnotation),
//         DimensionsExpr(Annotation, Expression, MarkerAnnotation),
//         ElementValueArrayInitializer(
//             Annotation,
//             ElementValueArrayInitializer,
//             Expression,
//             MarkerAnnotation,
//         ),
//         EnumBody(EnumBodyDeclarations, EnumConstant),
//         EnumBodyDeclarations(
//             AnnotationTypeDeclaration,
//             Block,
//             ClassDeclaration,
//             CompactConstructorDeclaration,
//             ConstructorDeclaration,
//             EnumDeclaration,
//             FieldDeclaration,
//             InterfaceDeclaration,
//             MethodDeclaration,
//             RecordDeclaration,
//             StaticInitializer,
//         ),
//         ExpressionStatement(Expression),
//         ExtendsInterfaces(TypeList),
//         False,
//         FinallyClause(Block),
//         FloatingPointType,
//         FormalParameters(FormalParameter, ReceiverParameter, SpreadParameter),
//         GenericType(ScopedTypeIdentifier, TypeArguments, TypeIdentifier),
//         HexFloatingPointLiteral,
//         HexIntegerLiteral,
//         Identifier,
//         ImportDeclaration(Asterisk, Identifier, ScopedIdentifier),
//         InferredParameters(Identifier),
//         IntegralType,
//         InterfaceBody(
//             AnnotationTypeDeclaration,
//             ClassDeclaration,
//             ConstantDeclaration,
//             EnumDeclaration,
//             InterfaceDeclaration,
//             MethodDeclaration,
//             RecordDeclaration,
//         ),
//         LabeledStatement(Identifier, Statement),
//         LineComment,
//         MethodReference(Type, PrimaryExpression, Super, TypeArguments),
//         Modifiers(Annotation, MarkerAnnotation),
//         ModuleBody(ModuleDirective),
//         NullLiteral,
//         OctalIntegerLiteral,
//         PackageDeclaration(Annotation, Identifier, MarkerAnnotation, ScopedIdentifier),
//         ParenthesizedExpression(Expression),
//         Program(Statement),
//         ReceiverParameter(UnannotatedType, Annotation, Identifier, MarkerAnnotation, This),
//         RequiresModifier,
//         ResourceSpecification(Resource),
//         ReturnStatement(Expression),
//         ScopedTypeIdentifier(
//             Annotation,
//             GenericType,
//             MarkerAnnotation,
//             ScopedTypeIdentifier,
//             TypeIdentifier,
//         ),
//         SpreadParameter(UnannotatedType, Modifiers, VariableDeclarator),
//         StaticInitializer(Block),
//         StringLiteral,
//         Super,
//         SuperInterfaces(TypeList),
//         Superclass(Type),
//         SwitchBlock(SwitchBlockStatementGroup, SwitchRule),
//         SwitchBlockStatementGroup(Statement, SwitchLabel),
//         SwitchLabel(Expression),
//         SwitchRule(Block, ExpressionStatement, SwitchLabel, ThrowStatement),
//         TextBlock,
//         This,
//         ThrowStatement(Expression),
//         True,
//         TypeArguments(Type, Wildcard),
//         TypeBound(Type),
//         TypeIdentifier,
//         TypeList(Type),
//         TypeParameter(Annotation, MarkerAnnotation, TypeBound, TypeIdentifier),
//         TypeParameters(TypeParameter),
//         UpdateExpression(Expression),
//         VoidType,
//         Wildcard(Type, Annotation, MarkerAnnotation, Super),
//         YieldStatement(Expression),
//     }
//     WithFields {
//         Annotation {
//             name: (Identifier, ScopedIdentifier),
//             arguments: (AnnotationArgumentList,),
//         },
//         AnnotationTypeDeclaration {
//             body: (AnnotationTypeBody,),
//             name: (Identifier,),
//             _cs: (Modifiers,),
//         },
//         AnnotationTypeElementDeclaration {
//             name: (Identifier,),
//             r#type: (UnannotatedType,),
//             value: (Annotation, ElementValueArrayInitializer, Expression, MarkerAnnotation),
//             dimensions: (Dimensions,),
//             _cs: (Modifiers,),
//         },
//         ArrayAccess { array: (PrimaryExpression,), index: (Expression,) },
//         ArrayCreationExpression {
//             r#type: (SimpleType,),
//             value: (ArrayInitializer,),
//             dimensions: (Dimensions, DimensionsExpr),
//             _cs: (Annotation, MarkerAnnotation),
//         },
//         ArrayType { dimensions: (Dimensions,), element: (UnannotatedType,) },
//         AssignmentExpression {
//             left: (ArrayAccess, FieldAccess, Identifier),
//             operator: (
//                 PercentEq,
//                 AmpEq,
//                 StarEq,
//                 PlusEq,
//                 DashEq,
//                 SlashEq,
//                 LtLtEq,
//                 Eq,
//                 GtGtEq,
//                 GtGtGtEq,
//                 CaretEq,
//                 PipeEq,
//             ),
//             right: (Expression,),
//         },
//         BinaryExpression {
//             right: (Expression,),
//             left: (Expression,),
//             operator: (
//                 BangEq,
//                 Percent,
//                 Amp,
//                 AmpAmp,
//                 Star,
//                 Plus,
//                 Dash,
//                 Slash,
//                 LT,
//                 LtLt,
//                 LTEq,
//                 EqEq,
//                 GT,
//                 GTEq,
//                 GtGt,
//                 GtGtGt,
//                 Caret,
//                 Pipe,
//                 PipePipe,
//             ),
//         },
//         CastExpression { value: (Expression,), r#type: (Type,) },
//         CatchClause { body: (Block,), _cs: (CatchFormalParameter,) },
//         CatchFormalParameter {
//             name: (Identifier,),
//             dimensions: (Dimensions,),
//             _cs: (CatchType, Modifiers),
//         },
//         ClassDeclaration {
//             superclass: (Superclass,),
//             interfaces: (SuperInterfaces,),
//             body: (ClassBody,),
//             type_parameters: (TypeParameters,),
//             permits: (Permits,),
//             name: (Identifier,),
//             _cs: (Modifiers,),
//         },
//         CompactConstructorDeclaration {
//             name: (Identifier,),
//             body: (Block,),
//             _cs: (Modifiers,),
//         },
//         ConstantDeclaration {
//             r#type: (UnannotatedType,),
//             declarator: (VariableDeclarator,),
//             _cs: (Modifiers,),
//         },
//         ConstructorDeclaration {
//             type_parameters: (TypeParameters,),
//             body: (ConstructorBody,),
//             parameters: (FormalParameters,),
//             name: (Identifier,),
//             _cs: (Modifiers, Throws),
//         },
//         DoStatement { condition: (ParenthesizedExpression,), body: (Statement,) },
//         ElementValuePair {
//             key: (Identifier,),
//             value: (Annotation, ElementValueArrayInitializer, Expression, MarkerAnnotation),
//         },
//         EnhancedForStatement {
//             body: (Statement,),
//             name: (Identifier,),
//             r#type: (UnannotatedType,),
//             dimensions: (Dimensions,),
//             value: (Expression,),
//             _cs: (Modifiers,),
//         },
//         EnumConstant {
//             arguments: (ArgumentList,),
//             body: (ClassBody,),
//             name: (Identifier,),
//             _cs: (Modifiers,),
//         },
//         EnumDeclaration {
//             interfaces: (SuperInterfaces,),
//             body: (EnumBody,),
//             name: (Identifier,),
//             _cs: (Modifiers,),
//         },
//         ExplicitConstructorInvocation {
//             constructor: (Super, This),
//             arguments: (ArgumentList,),
//             type_arguments: (TypeArguments,),
//             object: (PrimaryExpression,),
//         },
//         ExportsModuleDirective {
//             package: (Identifier, ScopedIdentifier),
//             modules: (Identifier, ScopedIdentifier),
//         },
//         FieldAccess {
//             field: (Identifier, This),
//             object: (PrimaryExpression, Super),
//             _cs: (Super,),
//         },
//         FieldDeclaration {
//             r#type: (UnannotatedType,),
//             declarator: (VariableDeclarator,),
//             _cs: (Modifiers,),
//         },
//         ForStatement {
//             init: (Expression, LocalVariableDeclaration),
//             condition: (Expression,),
//             body: (Statement,),
//             update: (Expression,),
//         },
//         FormalParameter {
//             dimensions: (Dimensions,),
//             name: (Identifier,),
//             r#type: (UnannotatedType,),
//             _cs: (Modifiers,),
//         },
//         IfStatement {
//             condition: (ParenthesizedExpression,),
//             consequence: (Statement,),
//             alternative: (Statement,),
//         },
//         InstanceofExpression { left: (Expression,), right: (Type,), name: (Identifier,) },
//         InterfaceDeclaration {
//             type_parameters: (TypeParameters,),
//             body: (InterfaceBody,),
//             name: (Identifier,),
//             permits: (Permits,),
//             _cs: (ExtendsInterfaces, Modifiers),
//         },
//         LambdaExpression {
//             body: (Block, Expression),
//             parameters: (FormalParameters, Identifier, InferredParameters),
//         },
//         LocalVariableDeclaration {
//             declarator: (VariableDeclarator,),
//             r#type: (UnannotatedType,),
//             _cs: (Modifiers,),
//         },
//         MarkerAnnotation { name: (Identifier, ScopedIdentifier) },
//         MethodDeclaration {
//             name: (Identifier,),
//             dimensions: (Dimensions,),
//             type_parameters: (TypeParameters,),
//             body: (Block,),
//             parameters: (FormalParameters,),
//             r#type: (UnannotatedType,),
//             _cs: (Annotation, MarkerAnnotation, Modifiers, Throws),
//         },
//         MethodInvocation {
//             arguments: (ArgumentList,),
//             name: (Identifier,),
//             object: (PrimaryExpression, Super),
//             type_arguments: (TypeArguments,),
//             _cs: (Super,),
//         },
//         ModuleDeclaration {
//             name: (Identifier, ScopedIdentifier),
//             body: (ModuleBody,),
//             _cs: (Annotation, MarkerAnnotation),
//         },
//         ObjectCreationExpression {
//             arguments: (ArgumentList,),
//             r#type: (SimpleType,),
//             type_arguments: (TypeArguments,),
//             _cs: (ClassBody, PrimaryExpression),
//         },
//         OpensModuleDirective {
//             modules: (Identifier, ScopedIdentifier),
//             package: (Identifier, ScopedIdentifier),
//         },
//         ProvidesModuleDirective {
//             provided: (Identifier, ScopedIdentifier),
//             provider: (Identifier, ScopedIdentifier),
//             _cs: (Identifier, ScopedIdentifier),
//         },
//         RecordDeclaration {
//             body: (ClassBody,),
//             interfaces: (SuperInterfaces,),
//             name: (Identifier,),
//             type_parameters: (TypeParameters,),
//             parameters: (FormalParameters,),
//             _cs: (Modifiers,),
//         },
//         RequiresModuleDirective {
//             module: (Identifier, ScopedIdentifier),
//             modifiers: (RequiresModifier,),
//         },
//         Resource {
//             value: (Expression,),
//             name: (Identifier,),
//             dimensions: (Dimensions,),
//             r#type: (UnannotatedType,),
//             _cs: (FieldAccess, Identifier, Modifiers),
//         },
//         ScopedIdentifier { name: (Identifier,), scope: (Identifier, ScopedIdentifier) },
//         SwitchExpression { body: (SwitchBlock,), condition: (ParenthesizedExpression,) },
//         SynchronizedStatement { body: (Block,), _cs: (ParenthesizedExpression,) },
//         TernaryExpression {
//             consequence: (Expression,),
//             alternative: (Expression,),
//             condition: (Expression,),
//         },
//         TryStatement { body: (Block,), _cs: (CatchClause, FinallyClause) },
//         TryWithResourcesStatement {
//             resources: (ResourceSpecification,),
//             body: (Block,),
//             _cs: (CatchClause, FinallyClause),
//         },
//         UnaryExpression { operand: (Expression,), operator: (Bang, Plus, Dash, Tilde) },
//         UsesModuleDirective { r#type: (Identifier, ScopedIdentifier) },
//         VariableDeclarator {
//             dimensions: (Dimensions,),
//             value: (ArrayInitializer, Expression),
//             name: (Identifier,),
//         },
//         WhileStatement { body: (Statement,), condition: (ParenthesizedExpression,) },
//     }
//     Abstract {
//         #[strum(serialize = "_literal")]
//         Literal(
//             BinaryIntegerLiteral,
//             CharacterLiteral,
//             DecimalFloatingPointLiteral,
//             DecimalIntegerLiteral,
//             False,
//             HexFloatingPointLiteral,
//             HexIntegerLiteral,
//             NullLiteral,
//             OctalIntegerLiteral,
//             StringLiteral,
//             TextBlock,
//             True,
//         ),
//         #[strum(serialize = "_simple_type")]
//         SimpleType(
//             BooleanType,
//             FloatingPointType,
//             GenericType,
//             IntegralType,
//             ScopedTypeIdentifier,
//             TypeIdentifier,
//             VoidType,
//         ),
//         #[strum(serialize = "_type")]
//         Type(UnannotatedType, AnnotatedType),
//         #[strum(serialize = "_unannotated_type")]
//         UnannotatedType(SimpleType, ArrayType),
//         Comment(BlockComment, LineComment),
//         Declaration(
//             AnnotationTypeDeclaration,
//             ClassDeclaration,
//             EnumDeclaration,
//             ImportDeclaration,
//             InterfaceDeclaration,
//             ModuleDeclaration,
//             PackageDeclaration,
//             RecordDeclaration,
//         ),
//         Expression(
//             AssignmentExpression,
//             BinaryExpression,
//             CastExpression,
//             InstanceofExpression,
//             LambdaExpression,
//             PrimaryExpression,
//             SwitchExpression,
//             TernaryExpression,
//             UnaryExpression,
//             UpdateExpression,
//         ),
//         ModuleDirective(
//             ExportsModuleDirective,
//             OpensModuleDirective,
//             ProvidesModuleDirective,
//             RequiresModuleDirective,
//             UsesModuleDirective,
//         ),
//         PrimaryExpression(
//             Literal,
//             ArrayAccess,
//             ArrayCreationExpression,
//             ClassLiteral,
//             FieldAccess,
//             Identifier,
//             MethodInvocation,
//             MethodReference,
//             ObjectCreationExpression,
//             ParenthesizedExpression,
//             This,
//         ),
//         Statement(
//             SemiColon,
//             AssertStatement,
//             Block,
//             BreakStatement,
//             ContinueStatement,
//             Declaration,
//             DoStatement,
//             EnhancedForStatement,
//             ExpressionStatement,
//             ForStatement,
//             IfStatement,
//             LabeledStatement,
//             LocalVariableDeclaration,
//             ReturnStatement,
//             SwitchExpression,
//             SynchronizedStatement,
//             ThrowStatement,
//             TryStatement,
//             TryWithResourcesStatement,
//             WhileStatement,
//             YieldStatement,
//         ),
//     }
// }
