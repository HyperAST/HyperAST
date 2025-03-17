use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;

use lending::NodeStore as _;
use num::ToPrimitive;
use strum_macros::AsRefStr;
use strum_macros::Display;
use strum_macros::EnumCount;
use strum_macros::EnumIter;
use strum_macros::EnumString;

use crate::PrimInt;

pub trait HashKind: Copy + std::ops::Deref {
    fn structural() -> Self;
    fn label() -> Self;
}

/// TODO handle roles in a polyglote way
macro_rules! role_impl {
    (
        $( $t:ident => $s:expr, )+
    ) => {
        #[derive(PartialEq, Eq, Clone, Copy, Debug)]
        pub enum Role {
            $( $t, )+
        }

        impl<'a> TryFrom<&'a str> for Role {
            type Error = ();
            fn try_from(value: &'a str) -> Result<Self, Self::Error> {
                match value {
                    $( $s => Ok(Self::$t), )*
                    _ => Err(()),
                }
            }
        }

        impl Display for Role {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    $( Self::$t => $s, )*
                })
            }
        }
    };
}

role_impl!(
    Name => "name",
    Scope => "scope",
    Body => "body",
    SuperType => "super_type",
    Interfaces => "interfaces",
    Constructor => "constructor",
    Object => "object",
    Arguments => "arguments",
    TypeArguments => "type_arguments",
    Type => "type",
    Declarator => "declarator",
    Value => "value",
    TypeParameters => "type_parameters",
    Parameters => "parameters",
    Condition => "condition",
    Init => "init",
    Update => "update",
    Alternative => "alternative",
    Resources => "resources",
    Field => "field",
    Left => "left",
    Right => "right",
    Superclass => "superclass",
    Element => "element",
    Consequence => "consequence",
    Key => "key",
);

#[allow(unused)]
mod exp {
    use super::*;

    // keywords (leafs with a specific unique serialized form)
    // and concrete types (concrete rules) should definitely be stored.
    // But hidden nodes are can either be supertypes or nodes that are just deemed uninteresting (but still useful to for example the treesitter internal repr.)
    // The real important difference is the (max) number of children (btw an it cannot be a leaf (at least one child)),
    // indeed, with a single child it is possible to easily implement optimization that effectively reduce the number of nodes.
    // - a supertype should only have a single child
    // - in tree-sitter repeats (star and plus patterns) are binary nodes (sure balanced?)
    // - in tree-sitter other nodes can be hidden (even when they have fields), it can be espetially useful to add more structure without breaking existing queries !
    // Anyway lets wait for better type generation, this way it should be possible to explicitely/completely handle optimizable cases (supertypes,...)

    #[repr(transparent)]
    pub struct T(u16);

    #[repr(u16)]
    pub enum T2 {
        Java(u16),
        Cpp(u16),
    }

    // pub trait Lang {
    //     type Factory;
    //     type Type;
    // }

    trait TypeFactory {
        fn new() -> Self
        where
            Self: Sized;
    }

    mod polyglote {
        /// has statements
        struct Block;
        /// has a name
        struct Member;
    }

    // WARN order of fields matter in java for instantiation
    // stuff where order does not matter should be sorted before erasing anything

    pub enum TypeMapElement<Concrete, Abstract> {
        Keyword(Keyword),
        Concrete(Concrete),
        Abstract(Abstract),
    }

    pub enum ConvertResult<Concrete, Abstract> {
        Keyword(Keyword),
        Concrete(Concrete),
        Abstract(Abstract),
        Missing,
    }

    trait KeywordProvider: Sized {
        fn parse(&self, s: &str) -> Option<Self>;
        fn as_str(&'static self) -> &'static str;
        fn len(&self) -> usize;
    }

    /// only contains keywords such as
    #[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
    #[strum(serialize_all = "snake_case")]
    #[derive(Hash, Clone, Copy, PartialEq, Eq)]
    pub enum Keyword {
        // While,
        // For,
        // #[strum(serialize = ";")]
        // SemiColon,
        // #[strum(serialize = ".")]
        // Dot,
        // #[strum(serialize = "{")]
        // LeftCurly,
        // #[strum(serialize = "}")]
        // RightCurly,
    }

    impl KeywordProvider for Keyword {
        fn parse(&self, s: &str) -> Option<Self> {
            Keyword::from_str(s).ok()
        }

        fn as_str(&'static self) -> &'static str {
            Keyword::as_ref(&self)
        }

        fn len(&self) -> usize {
            <Keyword as strum::EnumCount>::COUNT
        }
    }

    mod macro_test {
        macro_rules! parse_unitary_variants {
        (@as_expr $e:expr) => {$e};
        (@as_item $($i:item)+) => {$($i)+};

        // Exit rules.
        (
            @collect_unitary_variants ($callback:ident ( $($args:tt)* )),
            ($(,)*) -> ($($var_names:ident,)*)
        ) => {
            parse_unitary_variants! {
                @as_expr
                $callback!{ $($args)* ($($var_names),*) }
            }
        };

        (
            @collect_unitary_variants ($callback:ident { $($args:tt)* }),
            ($(,)*) -> ($($var_names:ident,)*)
        ) => {
            parse_unitary_variants! {
                @as_item
                $callback!{ $($args)* ($($var_names),*) }
            }
        };

        // Consume an attribute.
        (
            @collect_unitary_variants $fixed:tt,
            (#[$_attr:meta] $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            parse_unitary_variants! {
                @collect_unitary_variants $fixed,
                ($($tail)*) -> ($($var_names)*)
            }
        };

        // Handle a variant, optionally with an with initialiser.
        (
            @collect_unitary_variants $fixed:tt,
            ($var:ident $(= $_val:expr)*, $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            parse_unitary_variants! {
                @collect_unitary_variants $fixed,
                ($($tail)*) -> ($($var_names)* $var,)
            }
        };

        // Abort on variant with a payload.
        (
            @collect_unitary_variants $fixed:tt,
            ($var:ident $_struct:tt, $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            const _error: () = "cannot parse unitary variants from enum with non-unitary variants";
        };

        // Entry rule.
        (enum $name:ident {$($body:tt)*} => $callback:ident $arg:tt) => {
            parse_unitary_variants! {
                @collect_unitary_variants
                ($callback $arg), ($($body)*,) -> ()
            }
        };
    }

        macro_rules! coucou {
            ( f(C, D)) => {
                struct B {}
            };
        }
        parse_unitary_variants! {
            enum A {
                C,D,
            } => coucou{ f}
        }
    }

    macro_rules! make_type {
        (
            Keyword {$(
                $(#[$km:meta])*
                $ka:ident
            ),* $(,)?}
            Concrete {$(
                $(#[$cm:meta])*
                $ca:ident$({$($cl:expr),+ $(,)*})?$(($($co:ident),+ $(,)*))?$([$($cx:ident),+ $(,)*])?
            ),* $(,)?}
            WithFields {$(
                $(#[$wm:meta])*
                $wa:ident{$($wb:tt)*}
            ),* $(,)?}
            Abstract {$(
                $(#[$am:meta])*
                $aa:ident($($ab:ident),* $(,)?)
            ),* $(,)?}
        ) => {
            #[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
            #[strum(serialize_all = "snake_case")]
            #[derive(Hash, Clone, Copy, PartialEq, Eq)]
            pub enum Type {
                // Keywords
            $(
                $( #[$km] )*
                $ka,
            )*
                // Concrete
            $(
                $ca,
            )*
                // WithFields
            $(
                $( #[$wm] )*
                $wa,
            )*
            }
            enum Abstract {
                $(
                    $aa,
                )*
            }

            pub struct Factory {
                map: Box<[u16]>,
            }

            pub struct Language;
        };
    }

    macro_rules! make_type_store {
    ($kw:ty, $sh:ty, $($a:ident($l:ty)),* $(,)?) => {

        #[repr(u16)]
        pub enum CustomTypeStore {$(
            $a(u16),
        )*}

        impl CustomTypeStore {
            // fn lang<L: Lang>(&self) -> Option<L> {
            //     todo!()
            // }
            fn eq_keyword(kw: &$kw) -> bool {
                todo!()
            }
            fn eq_shared(kw: &$sh) -> bool {
                todo!()
            }
        }
    };
}

    make_type_store!(Keyword, Shared, Java(java::Language), Cpp(cpp::Language),);

    pub mod java {
        use super::*;

        pub enum Field {
            Name,
            Body,
            Expression,
            Condition,
            Then,
            Else,
            Block,
            Type,
        }

        make_type! {
            Keyword{
                While,
                For,
                Public,
                Private,
                Protected,
                #[strum(serialize = ";")]
                SemiColon,
                #[strum(serialize = ".")]
                Dot,
                #[strum(serialize = "{")]
                LeftCurly,
                #[strum(serialize = "}")]
                RightCurly,
                #[strum(serialize = "(")]
                LeftParen,
                #[strum(serialize = ")")]
                RightParen,
                #[strum(serialize = "[")]
                LeftBracket,
                #[strum(serialize = "]")]
                RightBracket,
            }
            Concrete {
                Comment{r"//.\*$",r"/\*.*\*/"},
                Identifier{r"[a-zA-Z].*"},
                ExpressionStatement(Statement, Semicolon),
                ReturnStatement(Return, Expression, Semicolon),
                TryStatement(Try, Paren, Block),
            }
            WithFields {
                Class {
                    name(Identifier),
                    body(ClassBody),
                },
                Interface {
                    name(Identifier),
                    body(InterfaceBody),
                },
            }
            Abstract {
                Statement(
                    StatementExpression,
                    TryStatement,
                ),
                Expression(
                    BinaryExpression,
                    UnaryExpression,
                ),
            }
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
enum Abstract {
    Expression,
    Statement,
    Executable,
    Declaration,
    Literal,
}

#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Shared {
    Comment,
    // ExpressionStatement,
    // ReturnStatement,
    // TryStatement,
    Identifier,
    TypeDeclaration,
    Branch,
    Other,
    // WARN do not include Abtract type/rules (should go in Abstract) ie.
    // Expression,
    // Statement,
}

pub trait Lang<T>: LangRef<T> {
    fn make(t: TypeInternalSize) -> &'static T;
    fn to_u16(t: T) -> TypeInternalSize;
}

pub trait LangRef<T> {
    fn name(&self) -> &'static str;
    fn make(&self, t: TypeInternalSize) -> &'static T;
    fn to_u16(&self, t: T) -> TypeInternalSize;
    fn ts_symbol(&self, t: T) -> u16;
}

pub struct LangWrapper<T: 'static + ?Sized>(&'static dyn LangRef<T>);

impl<T> From<&'static (dyn LangRef<T> + 'static)> for LangWrapper<T> {
    fn from(value: &'static (dyn LangRef<T> + 'static)) -> Self {
        LangWrapper(value)
    }
}

impl<T> LangRef<T> for LangWrapper<T> {
    fn make(&self, t: TypeInternalSize) -> &'static T {
        self.0.make(t)
    }

    fn to_u16(&self, t: T) -> TypeInternalSize {
        self.0.to_u16(t)
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn ts_symbol(&self, t: T) -> u16 {
        self.0.ts_symbol(t)
    }
}

// trait object used to facilitate erasing node types
pub trait HyperType: Display + Debug {
    fn as_shared(&self) -> Shared;
    fn as_any(&self) -> &dyn std::any::Any;
    // returns the same address for the same type
    fn as_static(&self) -> &'static dyn HyperType;
    fn as_static_str(&self) -> &'static str;
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + Sized;
    fn is_file(&self) -> bool;
    fn is_directory(&self) -> bool;
    fn is_spaces(&self) -> bool;
    fn is_syntax(&self) -> bool;
    fn is_hidden(&self) -> bool;
    fn is_named(&self) -> bool;
    fn is_supertype(&self) -> bool;
    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized;
    fn lang_ref(&self) -> LangWrapper<AnyType>;
}

// experiment
// NOTE: it might actually be a good way to share types between languages.
// EX on a u16: lang on 4 bits, supertypes on 4 bits, concrete and hidden on the 8 remaining bits.
// lets also say the super types are precomputed on shared types.
// TODO still need to think about it

impl HyperType for u8 {
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + PartialEq + Sized,
    {
        // Do a type-safe casting. If the types are different,
        // return false, otherwise test the values for equality.
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |a| self == a)
    }

    fn as_shared(&self) -> Shared {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }

    fn as_static(&self) -> &'static dyn HyperType {
        todo!()
    }

    fn as_static_str(&self) -> &'static str {
        todo!()
    }

    fn is_file(&self) -> bool {
        todo!()
    }

    fn is_directory(&self) -> bool {
        todo!()
    }

    fn is_spaces(&self) -> bool {
        todo!()
    }

    fn is_syntax(&self) -> bool {
        todo!()
    }

    fn is_hidden(&self) -> bool {
        todo!()
    }

    fn is_supertype(&self) -> bool {
        todo!()
    }

    fn is_named(&self) -> bool {
        todo!()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        todo!()
    }
    fn lang_ref(&self) -> LangWrapper<AnyType> {
        todo!()
    }
}

// blanket impl for all TStore such that TypeTrait can be implemented on TypeU16
// impl<L> Lang<TypeU16<L>> for L
// where
//     L: LangRef<TypeU16<L>>, //for L
//     L: LLang<TypeU16<L>, I = u16>
// {
//     fn make(t: u16) -> &'static TypeU16<L> {
//         <L as Lang<L::E>>::make(t)
//     }
//     fn to_u16(t: TypeU16<L>) -> u16 {
//         <L as Lang<L::E>>::to_u16(t)
//     }
// }

pub trait TypeTrait: HyperType + Hash + Copy + Eq + Send + Sync {
    type Lang: Lang<Self>;
    fn is_fork(&self) -> bool;

    fn is_literal(&self) -> bool;
    fn is_primitive(&self) -> bool;
    fn is_type_declaration(&self) -> bool;
    fn is_identifier(&self) -> bool;
    fn is_instance_ref(&self) -> bool;

    fn is_type_body(&self) -> bool;

    fn is_value_member(&self) -> bool;

    fn is_executable_member(&self) -> bool;

    fn is_statement(&self) -> bool;

    fn is_declarative_statement(&self) -> bool;

    fn is_structural_statement(&self) -> bool;

    fn is_block_related(&self) -> bool;

    fn is_simple_statement(&self) -> bool;

    fn is_local_declare(&self) -> bool;

    fn is_parameter(&self) -> bool;

    fn is_parameter_list(&self) -> bool;

    fn is_argument_list(&self) -> bool;

    fn is_expression(&self) -> bool;
    fn is_comment(&self) -> bool;
}

pub trait Node {}

pub trait AsTreeRef<T> {
    fn as_tree_ref(&self) -> T;
}

pub trait Stored: Node {
    type TreeId: NodeId;
}

pub trait MarkedT: Stored
where
    // Self: for<'t> crate::types::NLending<
    //     't,
    //     Self::TreeId,
    //     N = <Self as crate::types::AstLending<'t>>::RT,
    // >,
    Self: for<'t> crate::types::AstLending<
        't,
        IdN = Self::TreeId,
        Idx = <Self as MarkedT>::ChildIdx,
        Label = <Self as MarkedT>::Label,
    >,
{
    type Label;
    type ChildIdx;
}

pub trait Typed {
    type Type: HyperType + Eq + Copy + Send + Sync; // todo try remove Copy
    fn get_type(&self) -> Self::Type; // TODO add TypeTrait bound on Self::Type to forbid AnyType from being given
    fn try_get_type(&self) -> Option<Self::Type> {
        Some(self.get_type())
    }
}

pub trait CLending<'a, Idx, IdN, __ImplBound = &'a Self> {
    type Children: Children<Idx, IdN>;
}

pub type LendC<'n, S, Idx, IdN> = <S as CLending<'n, Idx, IdN>>::Children;

pub trait Childrn<T>: Iterator<Item = T> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn iter_children(&self) -> Self;
}
pub trait Children<IdX, T>: std::ops::Index<IdX, Output = T> + Childrn<T> {
    fn child_count(&self) -> IdX;
    fn get(&self, i: IdX) -> Option<&T>;
    fn rev(&self, i: IdX) -> Option<&T>;
    fn after(&self, i: IdX) -> Self;
    fn before(&self, i: IdX) -> Self;
    fn between(&self, start: IdX, end: IdX) -> Self;
    fn inclusive(&self, start: IdX, end: IdX) -> Self;
}

pub trait WithChildren:
    Node + Stored + for<'a> CLending<'a, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>
{
    type ChildIdx: PrimInt;

    fn child_count(&self) -> Self::ChildIdx {
        self.children()
            .map_or(num::zero(), |cs| num::cast(cs.count()).unwrap())
    }
    fn child(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        let mut cs = self.children()?;
        cs.nth(idx.to_usize().unwrap())
    }
    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        let cs = self.children()?;
        let cs: Vec<_> = cs.collect();
        cs.get(cs.len() - idx.to_usize().unwrap())
            .cloned()
            .map(|x| x)
    }
    fn children(&self) -> Option<LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>>;
}

pub trait WithRoles: WithChildren {
    fn role_at<Role: 'static + Copy + std::marker::Sync + std::marker::Send>(
        &self,
        at: Self::ChildIdx,
    ) -> Option<Role>;
}

pub trait WithPrecompQueries {
    fn wont_match_given_precomputed_queries(&self, needed: u16) -> bool;
}

// pub trait WithChildrenSameLang: WithChildren {
//     type TChildren<'a>: Children<Self::ChildIdx, Self::TreeId> + ?Sized
//     where
//         Self: 'a;

//     fn child_count(&self) -> Self::ChildIdx;
//     fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId>;
//     fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId>;
//     fn children(&self) -> Option<LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>>;
// }

// pub trait AsSlice<'a, IdX, T: 'a> {
//     type Slice: std::ops::Index<IdX, Output = [T]> + ?Sized;

//     fn as_slice(&self) -> &Self::Slice;
// }

// impl<T> IterableChildren<T> for [T] {
//     type ChildrenIter<'a>
//         = core::slice::Iter<'a, T>
//     where
//         T: 'a;

//     fn iter_children(&self) -> Self::ChildrenIter<'_> {
//         <[T]>::iter(&self)
//     }

//     fn is_empty(&self) -> bool {
//         <[T]>::is_empty(&self)
//     }
// }

// impl<IdX: num::NumCast, T> Children<IdX, T> for [T]
// where
//     IdX: std::slice::SliceIndex<[T], Output = T>,
// {
//     fn child_count(&self) -> IdX {
//         IdX::from(<[T]>::len(&self)).unwrap()
//         // num::cast::<_, IdX>(<[T]>::len(&self)).unwrap()
//     }

//     fn get(&self, i: IdX) -> Option<&T> {
//         self.get(i.to_usize()?)
//     }

//     fn rev(&self, idx: IdX) -> Option<&T> {
//         let c = <[T]>::len(&self);
//         let c = c.checked_sub(idx.to_usize()?.checked_add(1)?)?;
//         self.get(c.to_usize()?)
//     }

//     fn after(&self, i: IdX) -> &Self {
//         (&self[i.to_usize().unwrap()..]).into()
//     }

//     fn before(&self, i: IdX) -> &Self {
//         (&self[..i.to_usize().unwrap()]).into()
//     }

//     fn between(&self, start: IdX, end: IdX) -> &Self {
//         (&self[start.to_usize().unwrap()..end.to_usize().unwrap()]).into()
//     }

//     fn inclusive(&self, start: IdX, end: IdX) -> &Self {
//         (&self[start.to_usize().unwrap()..=end.to_usize().unwrap()]).into()
//     }
// }

// #[derive(ref_cast::RefCast)]
// #[repr(transparent)]
// pub struct MySlice<T>(pub [T]);

// impl<'a, T> From<&'a [T]> for &'a MySlice<T> {
//     fn from(value: &'a [T]) -> Self {
//         use ref_cast::RefCast;
//         // NOTE it makes compile time layout assertions
//         MySlice::ref_cast(value)
//     }
// }

// impl<'a, T, const N: usize> From<&'a [T; N]> for &'a MySlice<T> {
//     fn from(value: &'a [T; N]) -> Self {
//         use ref_cast::RefCast;
//         let value: &'a [T] = value;
//         // NOTE it makes compile time layout assertions
//         MySlice::ref_cast(value)
//     }
// }

// impl<T> std::ops::Index<u16> for MySlice<T> {
//     type Output = T;

//     fn index(&self, index: u16) -> &Self::Output {
//         &self.0[index as usize]
//     }
// }

// impl<T> std::ops::Index<u8> for MySlice<T> {
//     type Output = T;

//     fn index(&self, index: u8) -> &Self::Output {
//         &self.0[index as usize]
//     }
// }

// impl<T> std::ops::Index<usize> for MySlice<T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.0[index]
//     }
// }

// impl<T: Clone> From<&MySlice<T>> for Vec<T> {
//     fn from(value: &MySlice<T>) -> Self {
//         value.0.to_vec()
//     }
// }

// impl<T: Debug> Debug for MySlice<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Debug::fmt(&self.0, f)
//     }
// }

// impl<T: Debug> Default for &MySlice<T> {
//     fn default() -> Self {
//         let r: &[T] = &[];
//         r.into()
//     }
// }

// impl<T> IterableChildren<T> for MySlice<T> {
//     type ChildrenIter<'a>
//         = core::slice::Iter<'a, T>
//     where
//         T: 'a;

//     fn iter_children(&self) -> Self::ChildrenIter<'_> {
//         <[T]>::iter(&self.0)
//     }

//     fn is_empty(&self) -> bool {
//         <[T]>::is_empty(&self.0)
//     }
// }

// impl<T> Children<u16, T> for MySlice<T> {
//     fn child_count(&self) -> u16 {
//         <[T]>::len(&self.0).to_u16().unwrap()
//     }

//     fn get(&self, i: u16) -> Option<&T> {
//         self.0.get(usize::from(i))
//     }

//     fn rev(&self, idx: u16) -> Option<&T> {
//         let c: u16 = self.child_count();
//         let c = c.checked_sub(idx.checked_add(1)?)?;
//         self.get(c)
//     }

//     fn after(&self, i: u16) -> &Self {
//         (&self.0[i.into()..]).into()
//     }

//     fn before(&self, i: u16) -> &Self {
//         (&self.0[..i.into()]).into()
//     }

//     fn between(&self, start: u16, end: u16) -> &Self {
//         (&self.0[start.into()..end.into()]).into()
//     }

//     fn inclusive(&self, start: u16, end: u16) -> &Self {
//         (&self.0[start.into()..=end.into()]).into()
//     }

//     fn is_empty(&self) -> bool {
//         <[T]>::is_empty(&self.0)
//     }
// }

// impl<T> Children<u8, T> for MySlice<T> {
//     fn child_count(&self) -> u8 {
//         <[T]>::len(&self.0).to_u8().unwrap()
//     }

//     fn get(&self, i: u8) -> Option<&T> {
//         self.0.get(usize::from(i))
//     }

//     fn rev(&self, idx: u8) -> Option<&T> {
//         let c: u8 = self.child_count();
//         let c = c.checked_sub(idx.checked_add(1)?)?;
//         self.get(c)
//     }

//     fn after(&self, i: u8) -> &Self {
//         (&self.0[i.into()..]).into()
//     }

//     fn before(&self, i: u8) -> &Self {
//         (&self.0[..i.into()]).into()
//     }

//     fn between(&self, start: u8, end: u8) -> &Self {
//         (&self.0[start.into()..end.into()]).into()
//     }

//     fn inclusive(&self, start: u8, end: u8) -> &Self {
//         (&self.0[start.into()..=end.into()]).into()
//     }

//     fn is_empty(&self) -> bool {
//         <[T]>::is_empty(&self.0)
//     }
// }

pub struct ChildrenSlice<'a, T>(pub &'a [T]);

impl<'a, T> From<&'a [T]> for ChildrenSlice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self(value)
    }
}

impl<'a, T> Default for ChildrenSlice<'a, T> {
    fn default() -> Self {
        Self(&[])
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for ChildrenSlice<'a, T> {
    fn from(value: &'a [T; N]) -> Self {
        Self(value)
    }
}

impl<T: Clone> From<ChildrenSlice<'_, T>> for Vec<T> {
    fn from(value: ChildrenSlice<'_, T>) -> Self {
        value.0.to_vec()
    }
}

impl<T> std::ops::Index<u16> for ChildrenSlice<'_, T> {
    type Output = T;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> std::ops::Index<u8> for ChildrenSlice<'_, T> {
    type Output = T;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> std::ops::Index<usize> for ChildrenSlice<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Debug> Debug for ChildrenSlice<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<'a, T: Clone> Iterator for ChildrenSlice<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.0.first()?.clone();
        self.0 = &self.0[1..];
        Some(r.clone())
    }
}

impl<'a, T: Clone> DoubleEndedIterator for ChildrenSlice<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let r = self.0.last()?.clone();
        self.0 = &self.0[..self.0.len() - 1];
        Some(r.clone())
    }
}

impl<'a, T: Clone> Children<u16, T> for ChildrenSlice<'a, T> {
    fn child_count(&self) -> u16 {
        <[T]>::len(self.0).to_u16().unwrap()
    }

    fn get(&self, i: u16) -> Option<&T> {
        self.0.get(usize::from(i))
    }

    fn rev(&self, idx: u16) -> Option<&T> {
        let c: u16 = self.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        self.get(c)
    }

    fn after(&self, i: u16) -> Self {
        Self(&self.0[i.into()..])
    }

    fn before(&self, i: u16) -> Self {
        Self(&self.0[..i.into()])
    }

    fn between(&self, start: u16, end: u16) -> Self {
        Self(&self.0[start.into()..end.into()])
    }

    fn inclusive(&self, start: u16, end: u16) -> Self {
        Self(&self.0[start.into()..=end.into()])
    }
}

impl<'a, T> ChildrenSlice<'a, T> {
    fn is_empty(&self) -> bool {
        <[T]>::is_empty(self.0)
    }
}

impl<'a, T: Clone> Childrn<T> for ChildrenSlice<'a, T> {
    fn len(&self) -> usize {
        <[T]>::len(self.0)
    }
    fn is_empty(&self) -> bool {
        <[T]>::is_empty(self.0)
    }

    fn iter_children(&self) -> Self {
        Self(&self.0[..])
    }
}

impl<'a, T: Clone> Children<u8, T> for ChildrenSlice<'a, T> {
    fn child_count(&self) -> u8 {
        <[T]>::len(self.0).to_u8().unwrap()
    }

    fn get(&self, i: u8) -> Option<&T> {
        self.0.get(usize::from(i))
    }

    fn rev(&self, idx: u8) -> Option<&T> {
        let c: u8 = self.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        self.get(c)
    }

    fn after(&self, i: u8) -> Self {
        Self(&self.0[i.into()..])
    }

    fn before(&self, i: u8) -> Self {
        Self(&self.0[..i.into()])
    }

    fn between(&self, start: u8, end: u8) -> Self {
        Self(&self.0[start.into()..end.into()])
    }

    fn inclusive(&self, start: u8, end: u8) -> Self {
        Self(&self.0[start.into()..=end.into()])
    }
}

/// just to show that it is not efficient
/// NOTE: it might prove necessary for ecs like hecs
mod owned {
    use std::cell::{Ref, RefMut};

    use super::*;

    pub trait WithChildren: Node {
        type ChildIdx: PrimInt;

        fn child_count(&self) -> Self::ChildIdx;
        fn get_child(&self, idx: &Self::ChildIdx) -> RefMut<Self>;
        fn get_child_mut(&mut self, idx: &Self::ChildIdx) -> Ref<Self>;
    }
    pub trait WithParent: Node {
        fn get_parent(&self) -> Ref<Self>;
        fn get_parent_mut(&mut self) -> RefMut<Self>;
    }
}

pub trait WithStats {
    fn size(&self) -> usize;
    fn height(&self) -> usize;
    fn line_count(&self) -> usize;
}
pub trait WithMetaData<C> {
    fn get_metadata(&self) -> Option<&C>;
}

pub trait WithSerialization {
    fn try_bytes_len(&self) -> Option<usize>;
}

pub trait WithHashs {
    type HK: HashKind;
    type HP: PrimInt + PartialEq + Eq;
    fn hash<'a>(&'a self, kind: impl std::ops::Deref<Target = Self::HK>) -> Self::HP;
}

pub trait Labeled {
    type Label: Eq;
    fn get_label_unchecked<'a>(&'a self) -> &'a Self::Label;
    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label>;
}
pub trait Tree: Labeled + WithChildren + ErasedHolder {
    fn has_children(&self) -> bool;
    fn has_label(&self) -> bool;
}

pub trait TypedTree: Typed + Tree {}

impl<T> TypedTree for T where Self: Typed + Tree {}

pub trait DeCompressedTree<T: PrimInt>: Tree {
    fn get_parent(&self) -> T;
}

pub trait TreePath {}

pub trait GenericItem<'a> {
    type Item;
}

pub trait NStore {
    type IdN; //: NodeId<IdN = Self::IdN>;
    type Idx: PrimInt;
}

pub mod assoc {

    pub trait NodStore<IdN> {
        type R<'a>;
    }

    pub trait NodeStore<IdN>: NodStore<IdN> {
        fn resolve(&self, id: &IdN) -> Self::R<'_>;
    }
}

pub mod lending {
    pub trait NLending<'a, IdN, __ImplBound = &'a Self> {
        type N: 'a + crate::types::Stored<TreeId = IdN>;
    }

    pub type LendN<'n, S, IdN> = <S as NLending<'n, IdN>>::N;

    pub trait NodeStore<IdN>: for<'a> NLending<'a, IdN> {
        fn resolve(&self, id: &IdN) -> LendN<'_, Self, IdN>;
        fn scoped<R>(&self, id: &IdN, f: impl Fn(&LendN<'_, Self, IdN>) -> R) -> R {
            f(&self.resolve(id))
        }
        fn scoped_mut<R>(&self, id: &IdN, mut f: impl FnMut(&LendN<'_, Self, IdN>) -> R) -> R {
            f(&self.resolve(id))
        }
    }
}

pub use lending::*;

pub mod inner_ref {
    pub trait NodeStore<IdN> {
        type Ref;
        fn scoped<R>(&self, id: &IdN, f: impl Fn(&Self::Ref) -> R) -> R;
        fn scoped_mut<R>(&self, id: &IdN, f: impl FnMut(&Self::Ref) -> R) -> R;
        fn multi<R, const N: usize>(&self, id: &[IdN; N], f: impl Fn(&[Self::Ref; N]) -> R) -> R;
    }
}

pub trait NodeStoreLean<IdN> {
    type R;
    fn resolve(&self, id: &IdN) -> Self::R;
}

pub trait NodeStoreLife<'store, IdN> {
    type R<'s>
    where
        Self: 's,
        Self: 'store;
    fn resolve(&'store self, id: &IdN) -> Self::R<'store>;
}

pub trait NodeId: Eq + Clone + 'static {
    type IdN: Eq + AAAA;
    fn as_id(&self) -> &Self::IdN;
    // fn as_ty(&self) -> &Self::Ty;
    unsafe fn from_id(id: Self::IdN) -> Self;
    unsafe fn from_ref_id(id: &Self::IdN) -> &Self;
}

impl AAAA for u16 {}

impl NodeId for u16 {
    type IdN = u16;
    fn as_id(&self) -> &Self::IdN {
        self
    }
    unsafe fn from_id(id: Self::IdN) -> Self {
        id
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        id
    }
}

pub trait AAAA: NodeId<IdN = Self> {}

pub trait TypedNodeId: NodeId {
    type Ty: HyperType + Hash + Copy + Eq + Send + Sync;
    type TyErazed: Compo + Clone;
    fn unerase(ty: Self::TyErazed) -> Self::Ty;
}
pub trait TyNodeStore<IdN: TypedNodeId> {
    type R<'a>: Typed<Type = IdN::Ty>;
}

pub trait TypedNodeStore<IdN: TypedNodeId>: TyNodeStore<IdN> {
    fn try_typed(&self, id: &IdN::IdN) -> Option<IdN>;
    fn try_resolve(&self, id: &IdN::IdN) -> Option<(Self::R<'_>, IdN)> {
        self.try_typed(id).map(|x| (self.resolve(&x), x))
    }
    fn resolve(&self, id: &IdN) -> Self::R<'_>;
}

pub trait TypedNodeStoreLean<IdN: TypedNodeId> {
    type R: Typed<Type = IdN::Ty>;
    fn try_typed(&self, id: &IdN::IdN) -> Option<IdN>;
    fn try_resolve(&self, id: &IdN::IdN) -> Option<(Self::R, IdN)> {
        self.try_typed(id).map(|x| (self.resolve(&x), x))
    }
    fn resolve(&self, id: &IdN) -> Self::R;
}

pub trait DecompressedSubtree<IdN> {
    type Out: DecompressedSubtree<IdN>;
    fn decompress(self, id: &IdN) -> Self::Out;
}

pub trait DecompressedFrom<HAST: HyperASTShared> {
    type Out: DecompressedFrom<HAST>;
    fn decompress(store: HAST, id: &HAST::IdN) -> Self::Out;
}

// pub trait DecompressibleNodeStore<IdN: NodeId>: NodeStore<IdN> {
//     fn decompress<D>(&self, id: &IdN) -> (&Self, D::Out)
//     where
//         Self: Sized,
//         D: DecompressedSubtree<IdN>,
//     {
//         (self, D::decompress(self, id))
//     }

//     fn decompress_pair<D1, D2>(&self, id1: &IdN, id2: &IdN) -> (&Self, (D1::Out, D2::Out))
//     where
//         Self: Sized,
//         D1: DecompressedSubtree<IdN>,
//         D2: DecompressedSubtree<IdN>,
//     {
//         (self, (D1::decompress(self, id1), D2::decompress(self, id2)))
//     }
// }

// impl<IdN: NodeId, S> DecompressibleNodeStore<IdN> for S where S: NodeStore<IdN> {}

pub trait NodeStoreMut<T: Stored> {
    fn get_or_insert(&mut self, node: T) -> T::TreeId;
}
pub trait NodeStoreExt<T: TypedTree> {
    fn build_then_insert(
        &mut self,
        i: T::TreeId,
        t: T::Type,
        l: Option<T::Label>,
        cs: Vec<T::TreeId>,
    ) -> T::TreeId;
}

pub trait VersionedNodeStore<'a, IdN: NodeId>: NodeStore<IdN> {
    fn resolve_root(&self, version: (u8, u8, u8), node: IdN);
}

pub trait VersionedNodeStoreMut<'a, T: Stored>: NodeStoreMut<T>
where
    T::TreeId: Clone,
{
    // fn insert_as_root(&mut self, version: (u8, u8, u8), node: T) -> T::TreeId;
    //  {
    //     let r = self.get_or_insert(node);
    //     self.as_root(version, r.clone());
    //     r
    // }

    fn as_root(&mut self, version: (u8, u8, u8), node: T::TreeId);
}

pub type OwnedLabel = String;
pub type SlicedLabel = str;

pub trait LStore {
    type I;
}

pub trait LabelStore<L: ?Sized> {
    type I: Copy + Eq;

    fn get_or_insert<T: Borrow<L>>(&mut self, node: T) -> Self::I;

    fn get<T: Borrow<L>>(&self, node: T) -> Option<Self::I>;

    fn resolve(&self, id: &Self::I) -> &L;
}

type TypeInternalSize = u16;

pub trait TypeStore {
    type Ty: 'static
        + HyperType
        + Eq
        + std::hash::Hash
        + Copy
        + std::marker::Send
        + std::marker::Sync;

    fn type_to_u16(t: Self::Ty) -> TypeInternalSize {
        t.get_lang().to_u16(t)
    }
    fn ts_symbol(t: Self::Ty) -> TypeInternalSize {
        t.get_lang().ts_symbol(t)
    }
    fn decompress_type(erazed: &impl ErasedHolder, tid: std::any::TypeId) -> Self::Ty {
        *erazed
            .unerase_ref::<Self::Ty>(tid)
            .unwrap_or_else(|| unimplemented!("override 'decompress_type'"))
    }
}

pub trait TTypeStore: TypeStore {
    type TTy: Compo + Copy;
    fn decompress_ttype(erazed: &impl ErasedHolder, tid: std::any::TypeId) -> Self::TTy;
}

pub trait ETypeStore: TypeStore + Copy {
    type Ty2;
    fn intern(ty: Self::Ty2) -> Self::Ty;
}

impl<T> TTypeStore for T
where
    T: TypeStore,
    T::Ty: Compo,
{
    type TTy = Self::Ty;
    fn decompress_ttype(erazed: &impl ErasedHolder, tid: std::any::TypeId) -> Self::TTy {
        *unsafe {
            erazed
                .unerase_ref_unchecked::<Self::TTy>(tid)
                .unwrap_or_else(|| unimplemented!("override 'decompress_type'"))
        }
    }
}

pub trait LLang<T>: Lang<Self::E> {
    type I;
    type E: 'static + Copy + Display;
    const TE: &[Self::E];

    fn as_lang_wrapper() -> LangWrapper<T>;
}

struct LLangTest;

#[derive(Clone, Copy, Display, strum_macros::EnumCount)]
#[repr(u8)]
enum TyTest {
    A,
    B,
    C,
}

impl Lang<TyTest> for LLangTest {
    fn make(t: TypeInternalSize) -> &'static TyTest {
        todo!()
    }

    fn to_u16(t: TyTest) -> TypeInternalSize {
        todo!()
    }
}

impl LangRef<TyTest> for LLangTest {
    fn name(&self) -> &'static str {
        todo!()
    }

    fn make(&self, t: TypeInternalSize) -> &'static TyTest {
        todo!()
    }

    fn to_u16(&self, t: TyTest) -> TypeInternalSize {
        todo!()
    }

    fn ts_symbol(&self, t: TyTest) -> u16 {
        todo!()
    }
}

impl LLang<TypeU16<Self>> for LLangTest {
    type I = u16;

    type E = TyTest;

    const TE: &[Self::E] = &[TyTest::A, TyTest::B, TyTest::C];

    fn as_lang_wrapper() -> LangWrapper<TypeU16<Self>> {
        unimplemented!("not important here")
    }
}

pub trait SizedIndex<I> {
    fn len(&self) -> I;
}

impl<T> SizedIndex<u16> for [T] {
    fn len(&self) -> u16 {
        self.len().to_u16().unwrap()
    }
}

#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
pub struct TypeU16<L: LLang<Self, I = u16>>(u16, std::marker::PhantomData<L>);

unsafe impl<L: LLang<Self, I = u16>> Send for TypeU16<L> {}
unsafe impl<L: LLang<Self, I = u16>> Sync for TypeU16<L> {}

impl<L: LLang<Self, I = u16>> Debug for TypeU16<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypeU16")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl<L: LLang<Self, I = u16>> PartialEq for TypeU16<L> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<L: LLang<Self, I = u16>> Eq for TypeU16<L> {}

impl<L: LLang<Self, I = u16>> Copy for TypeU16<L> {}

impl<L: LLang<Self, I = u16>> Clone for TypeU16<L> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<L: LLang<Self, I = u16>> Hash for TypeU16<L> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}
impl<L: LLang<Self, I = u16>> Display for TypeU16<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.e())
    }
}

impl<L: LLang<Self, I = u16>> TypeU16<L> {
    pub fn e(&self) -> L::E {
        debug_assert!(L::TE.len() <= u16::MAX as usize);
        L::TE[self.0 as usize]
    }
    fn s(&self) -> &'static L::E {
        debug_assert!(L::TE.len() <= u16::MAX as usize);
        &L::TE[self.0 as usize]
    }
    pub fn new(e: L::E) -> Self {
        Self(<L as Lang<L::E>>::to_u16(e), std::marker::PhantomData)
    }
}

impl<L: LLang<Self, I = u16> + std::fmt::Debug> HyperType for TypeU16<L>
where
    L::E: HyperType,
{
    fn as_shared(&self) -> Shared {
        self.e().as_shared()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.s().as_any()
    }

    fn as_static(&self) -> &'static dyn HyperType {
        self.e().as_static()
    }

    fn as_static_str(&self) -> &'static str {
        self.e().as_static_str()
    }

    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + Sized,
    {
        self.e().generic_eq(other)
    }

    fn is_file(&self) -> bool {
        self.e().is_file()
    }

    fn is_directory(&self) -> bool {
        self.e().is_directory()
    }

    fn is_spaces(&self) -> bool {
        self.e().is_spaces()
    }

    fn is_syntax(&self) -> bool {
        self.e().is_syntax()
    }

    fn is_hidden(&self) -> bool {
        self.e().is_hidden()
    }

    fn is_named(&self) -> bool {
        self.e().is_named()
    }

    fn is_supertype(&self) -> bool {
        self.e().is_supertype()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        L::as_lang_wrapper()
    }

    fn lang_ref(&self) -> LangWrapper<AnyType> {
        self.e().lang_ref()
    }
}

// impl<L: LLang<Self, I = u16> + std::fmt::Debug> TypeTrait for TypeU16<L>
// where
//     L::E: TypeTrait<Lang = L>,
//     L: Lang<Self>,
// {
//     type Lang = L;

// }

#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
pub struct TypeU8<L: LLang<Self>>(u8, std::marker::PhantomData<L>);

#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
pub enum TypeEnumCommon<J: Lang<Self>, X: Lang<Self>, C: Lang<Self>, M: Lang<Self>> {
    Java(u16, std::marker::PhantomData<J>),
    Xml(u16, std::marker::PhantomData<X>),
    C(u16, std::marker::PhantomData<C>),
    Make(u16, std::marker::PhantomData<M>),
}

pub trait CompressedCompo {
    fn decomp(ptr: impl ErasedHolder, tid: std::any::TypeId) -> Self
    where
        Self: Sized;

    // fn compressed_insert(self, e: &mut EntityWorldMut<'_>);
    // fn components(world: &mut World) -> Vec<ComponentId>;
}

pub trait ErasedHolder {
    /// made unsafe because mixed-up args could return corrupted memory for certain impls
    unsafe fn unerase_ref_unchecked<T: 'static + Compo>(
        &self,
        tid: std::any::TypeId,
    ) -> Option<&T> {
        self.unerase_ref(tid)
    }
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T>;
}

impl ErasedHolder for &dyn std::any::Any {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        if tid == std::any::TypeId::of::<T>() {
            self.downcast_ref()
        } else {
            None
        }
    }
}

#[cfg(all(feature = "bevy_ecs", feature = "legion"))]
pub trait Compo: bevy_ecs::component::Component + legion::storage::Component {}

#[cfg(all(feature = "bevy_ecs", feature = "legion"))]
impl<T> Compo for T where T: bevy_ecs::component::Component + legion::storage::Component {}

#[cfg(all(not(feature = "bevy_ecs"), feature = "legion"))]
pub trait Compo: legion::storage::Component {}

#[cfg(all(not(feature = "bevy_ecs"), feature = "legion"))]
impl<T> Compo for T where T: legion::storage::Component {}

#[cfg(all(not(feature = "bevy_ecs"), not(feature = "legion")))]
pub trait Compo: Send + Sync {}

#[cfg(all(not(feature = "bevy_ecs"), not(feature = "legion")))]
impl<T: Send + Sync> Compo for T {}

pub trait ErasedInserter {
    fn insert<T: 'static + Compo>(&mut self, t: T);
}

pub trait CompoRegister {
    type Id;
    fn register_compo<T: 'static + Compo>(&mut self) -> Self::Id;
}

pub trait SpecializedTypeStore<T: Typed>: TypeStore {}

pub trait RoleStore: TypeStore {
    type IdF: 'static + Copy + Default + PartialEq;
    type Role: 'static + Copy + PartialEq + std::marker::Sync + std::marker::Send;
    fn resolve_field(lang: LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role;
    fn intern_role(lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF;
}

// trait HyperAST2: HyperAST<TM = MT> {
//     type MT: MarkedT;
// }

pub trait HyperAST: for<'a> AstLending<'a> {
    type NS: for<'a> NLending<'a, Self::IdN, N = <Self as AstLending<'a>>::RT>
        + lending::NodeStore<Self::IdN>;
    fn node_store(&self) -> &Self::NS;

    type LS: LabelStore<str, I = Self::Label>;
    fn label_store(&self) -> &Self::LS;

    type TS: TypeStore;

    fn decompress<D>(&self, id: &Self::IdN) -> (&Self, D)
    where
        Self: Sized,
        D: for<'a> From<&'a Self>,
        D: DecompressedSubtree<Self::IdN, Out = D>, // + for<'t> lending::NLending<'t, Self::IdN, N = <Self as AstLending<'t>>::RT>
    {
        (self, D::from(self).decompress(id))
    }

    fn decompress_pair<D1, D2>(&self, id1: &Self::IdN, id2: &Self::IdN) -> (&Self, (D1, D2))
    where
        Self: Sized,
        D1: for<'a> From<&'a Self>,
        D1: DecompressedSubtree<Self::IdN, Out = D1>,
        D2: for<'a> From<&'a Self>,
        D2: DecompressedSubtree<Self::IdN, Out = D2>,
    {
        (
            self,
            (
                D1::from(self).decompress(id1),
                D2::from(self).decompress(id2),
            ),
        )
    }

    fn decompress_pair2<D1, D2>(self, id1: &Self::IdN, id2: &Self::IdN) -> (Self, (D1, D2))
    where
        Self: Sized + Copy,
        D1: DecompressedFrom<Self, Out = D1>,
        D2: DecompressedFrom<Self, Out = D2>,
    {
        (self, (D1::decompress(self, id1), D2::decompress(self, id2)))
    }
    fn resolve_type(&self, id: &Self::IdN) -> <Self::TS as TypeStore>::Ty {
        let _ns = self.node_store();
        // // SAFETY: ns is released at the end of the function
        // let ns: &Self::NS = unsafe { std::mem::transmute(_ns) };
        let ns = _ns;
        let n: <Self as AstLending<'_>>::RT = ns.resolve(id);
        Self::TS::decompress_type(&n, std::any::TypeId::of::<<Self::TS as TypeStore>::Ty>())
    }
    fn resolve<'a>(&self, id: &'a Self::IdN) -> <Self as AstLending<'_>>::RT {
        let _ns = self.node_store();
        // // SAFETY: ns is released at the end of the function
        // let ns: &Self::NS = unsafe { std::mem::transmute(_ns) };
        let ns = _ns;
        let n = ns.resolve(id);
        n
    }
    fn resolve_ttype(&self, id: &Self::IdN) -> <Self::TS as TypeStore>::Ty
    where
        Self::TS: TypeStore<Ty = AnyType>,
    {
        todo!()
        // let _ns = self.node_store();
        // // SAFETY: ns is released at the end of the function
        // let ns: &Self::NS = unsafe { std::mem::transmute(_ns) };
        // let n = ns.resolve(id);
        // Self::TS::decompress_ttype(&n, std::any::TypeId::of::<<Self::TS as TypeStore>::Ty>())
    }
    // fn resolve_ttype(&'store self, id: &Self::IdN) -> <Self::TS as TypeStore>::Ty {
    //     let ns = self.node_store();
    //     let n = ns.resolve(id);
    //     Self::TS::decompress_ttype(&n, std::any::TypeId::of::<<Self::TS as TypeStore>::Ty>())
    // }
    fn resolve_lang(
        &self,
        n: &<Self as AstLending<'_>>::RT,
    ) -> LangWrapper<<Self::TS as TypeStore>::Ty> {
        Self::TS::decompress_type(n, std::any::TypeId::of::<<Self::TS as TypeStore>::Ty>())
            .get_lang()
    }
    fn type_eq(&self, n: &<Self as AstLending<'_>>::RT, m: &<Self as AstLending<'_>>::RT) -> bool {
        Self::TS::decompress_type(n, std::any::TypeId::of::<<Self::TS as TypeStore>::Ty>())
            .generic_eq(
                Self::TS::decompress_type(m, std::any::TypeId::of::<<Self::TS as TypeStore>::Ty>())
                    .as_static(),
            )
    }
}

pub trait StoreLending<'a, __ImplBound = &'a Self>: AstLending<'a, __ImplBound> + HyperAST {
    type S: 'a
        + Copy
        + HyperAST<
            TS = <Self as HyperAST>::TS,
            IdN = <Self as HyperASTShared>::IdN,
            Label = <Self as HyperASTShared>::Label,
            Idx = <Self as HyperASTShared>::Idx,
        >
        + AstLending<'a, RT = <Self as AstLending<'a, __ImplBound>>::RT>;
}

pub trait StoreLending2: HyperAST {
    type S<'a>: Copy
        + HyperAST<
            TS = <Self as HyperAST>::TS,
            IdN = <Self as HyperASTShared>::IdN,
            Label = <Self as HyperASTShared>::Label,
            Idx = <Self as HyperASTShared>::Idx,
        > + for<'t> AstLending<'t, RT = <Self as AstLending<'t>>::RT>;
}

// trait HyperASTLender: for<'a> StoreLending<'a> {}

pub trait NodeStorage<IdN> {}

pub trait HyperASTShared {
    type IdN: NodeId;
    type Idx: PrimInt;
    type Label;
}

pub trait AstLending<'a, __ImplBound = &'a Self>:
    HyperASTShared + NLending<'a, Self::IdN, __ImplBound, N = Self::RT>
{
    type RT: 'a + Tree<Label = Self::Label, TreeId = Self::IdN, ChildIdx = Self::Idx>;
}

pub type LendT<'t, HAST> = <HAST as AstLending<'t>>::RT;

pub trait TypedLending<'a, Ty: HyperType + Hash + Copy + Eq + Send + Sync, __ImplBound = &'a Self>:
    AstLending<'a, __ImplBound>
{
    type TT: Deref<Target = <Self as AstLending<'a, __ImplBound>>::RT> + Typed<Type = Ty>;
}

// impl<T> HyperASTShared for &T
// where
//     T: HyperASTShared,
// {
//     type IdN = T::IdN;
//     type Idx = T::Idx;
//     type Label = T::Label;
// }

// pub trait HyperASTLean: HyperASTShared {
//     type T: Tree<Label = Self::Label, TreeId = Self::IdN, ChildIdx = Self::Idx>;

//     type NS;
//     fn node_store(&self) -> &Self::NS
//     where
//         for<'a,'t> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T<'t>>;

//     type LS: LabelStore<str, I = Self::Label>;
//     fn label_store(&self) -> &Self::LS;

//     type TS: TypeStore;

//     fn resolve_type(&self, id: &Self::IdN) -> <Self::TS as TypeStore>::Ty
//     where
//         for<'a,'t> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T<'t>>,
//     {
//         let ns = self.node_store();
//         let n = ns.resolve(id);
//         todo!()
//         // self.type_store().resolve_type(&n).clone()
//     }
// }

// pub trait HyperASTAsso: HyperASTShared {
//     type T<'store>: Tree<Label = Self::Label, TreeId = Self::IdN, ChildIdx = Self::Idx>
//     where
//         Self: 'store;

//     type NS<'store>: NodeStore<Self::IdN, R<'store> = Self::T<'store>>
//     where
//         Self: 'store,
//         Self::T<'store>: 'store;
//     fn node_store<'a>(&'a self) -> &'a Self::NS<'a>;
//     fn node_store2<'a, 'b>(&'a self) -> Self::NS<'b> {
//         panic!()
//     }
//     fn node_store3(&self) -> Self::NS<'_> {
//         panic!()
//     }

//     type LS: LabelStore<str, I = Self::Label>;
//     fn label_store(&self) -> &Self::LS;

//     type TS<'store>: TypeStore
//     where
//         Self: 'store;

//     fn resolve_type(&self, id: &Self::IdN) -> <Self::TS<'_> as TypeStore>::Ty {
//         todo!()
//         // let ns = self.node_store();
//         // let n = ns.resolve(id);
//         // Self::TS::decompress_ttype(
//         //     &n,
//         //     std::any::TypeId::of::<<Self::TS<'_> as TypeStore>::Ty>(),
//         // )
//         // // let ns = self.node_store();
//         // // let n = ns.resolve(id);
//         // // todo!()
//         // // self.type_store().resolve_type(&n).clone()
//     }
// }

// impl<T> HyperASTLean for &T
// where
//     T: HyperASTLean,
// {
//     type T = T::T;

//     type NS = T::NS;
//     fn node_store(&self) -> &T::NS
//     where
//         for<'a> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T>,
//     {
//         (*self).node_store()
//     }

//     type LS = T::LS;
//     fn label_store(&self) -> &Self::LS {
//         (*self).label_store()
//     }

//     type TS = T::TS;

//     fn resolve_type(&self, id: &Self::IdN) -> <Self::TS as TypeStore>::Ty
//     where
//         for<'a> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T>,
//     {
//         (*self).resolve_type(id)
//     }
// }

// pub trait TypedHyperASTLean<TIdN: TypedNodeId<IdN = Self::IdN>>: HyperASTLean {
//     type TT: TypedTree<
//         Type = TIdN::Ty,
//         TreeId = TIdN::IdN,
//         Label = Self::Label,
//         ChildIdx = <<Self as HyperASTLean>::T as WithChildren>::ChildIdx,
//     >;
//     // type TNS<'a> where &'a Self::TNS<'a>: TypedNodeStoreLean<Self::IdN, R = Self::T>, Self: 'a;
//     // fn typed_node_store(&self) -> Self::TNS<'_>;
// }

pub trait TypedHyperAST<TIdN: TypedNodeId<
// Ty = <<Self as HyperAST>::TS as TypeStore>::Ty
>>:
    HyperAST + for<'a> TypedLending<'a, TIdN::Ty, IdN = TIdN::IdN>
{
    // type TT<'t>: TypedTree<
    //     Type = TIdN::Ty,
    //     TreeId = TIdN::IdN,
    //     Label = Self::Label,
    //     ChildIdx = <Self as HyperASTShared>::Idx,
    // >;
    // type TNS: for<'t> TypedNodeStore<TIdN, R<'t> = Self::TT>;
    // fn typed_node_store(&self) -> &Self::TNS;
    fn try_typed(&self, id: &Self::IdN) -> Option<TIdN>;
    fn try_resolve(
        &self,
        id: &Self::IdN,
    ) -> Option<(<Self as TypedLending<'_, TIdN::Ty>>::TT, TIdN)> {
        self.try_typed(id)
            .map(|x| (TypedHyperAST::resolve_typed(self, &x), x))
    }
    fn resolve_typed(&self, id: &TIdN) -> <Self as TypedLending<'_, TIdN::Ty>>::TT;
}

pub struct SimpleHyperAST<T, TS, NS, LS> {
    pub node_store: NS,
    pub label_store: LS,
    pub _phantom: std::marker::PhantomData<(T, TS)>,
}

impl<T, TS, NS: Copy, LS> SimpleHyperAST<T, TS, NS, &LS> {
    pub fn change_type_store_ref<TS2>(&self, new: TS2) -> SimpleHyperAST<T, TS2, NS, &LS> {
        SimpleHyperAST {
            node_store: self.node_store,
            label_store: self.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, TS, NS, LS> SimpleHyperAST<T, TS, NS, LS> {
    pub fn change_type_store<TS2>(self, new: TS2) -> SimpleHyperAST<T, TS2, NS, LS> {
        SimpleHyperAST {
            node_store: self.node_store,
            label_store: self.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn change_tree_type<T2>(self) -> SimpleHyperAST<T2, TS, NS, LS> {
        SimpleHyperAST {
            node_store: self.node_store,
            label_store: self.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, TS: Default, NS: Default, LS: Default> Default for SimpleHyperAST<T, TS, NS, LS> {
    fn default() -> Self {
        Self {
            node_store: Default::default(),
            label_store: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<T, TS, NS, LS> assoc::NodStore<T::TreeId> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    NS: assoc::NodStore<T::TreeId>,
{
    type R<'a> = NS::R<'a>;
}

impl<T, TS, NS, LS> assoc::NodeStore<T::TreeId> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    NS: assoc::NodeStore<T::TreeId>,
{
    fn resolve(&self, id: &T::TreeId) -> Self::R<'_> {
        self.node_store.resolve(id)
    }
}

impl<T, TS, NS, LS> NodeStoreLean<T::TreeId> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    NS: NodeStoreLean<T::TreeId>,
{
    type R = NS::R;

    fn resolve(&self, id: &T::TreeId) -> Self::R {
        self.node_store.resolve(id)
    }
}

impl<'store, T, TS, NS, LS> LabelStore<str> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    LS: LabelStore<str, I = T::Label>,
    <T as Labeled>::Label: Copy,
{
    type I = LS::I;

    fn get_or_insert<U: Borrow<str>>(&mut self, node: U) -> Self::I {
        self.label_store.get_or_insert(node)
    }

    fn get<U: Borrow<str>>(&self, node: U) -> Option<Self::I> {
        self.label_store.get(node)
    }

    fn resolve(&self, id: &Self::I) -> &str {
        self.label_store.resolve(id)
    }
}

// impl<'store, T, TS, NS, LS> TypeStore for SimpleHyperAST<T, TS, NS, LS>
// where
//     T: TypedTree<Type = TS::Ty>,
//     T::TreeId: NodeId<IdN = T::TreeId>,
//     TS::Ty: 'static + std::hash::Hash,
//     TS: TypeStore,
//     // TS::Ty:CompressedCompo,
// {
//     type Ty = TS::Ty;

//     // fn resolve_lang(&self, n: &T) -> LangWrapper<Self::Ty> {
//     //     self.type_store.resolve_lang(n)
//     // }

//     // fn type_eq(&self, n: &T, m: &T) -> bool {
//     //     self.type_store.type_eq(n, m)
//     // }

//     // fn resolve_type(&self, n: &T) -> Self::Ty {
//     //     self.type_store.resolve_type(n)
//     // }

//     fn decompress_type(erazed: &impl ErasedHolder, tid: std::any::TypeId) -> Self::Ty where <TS as TypeStore>::Ty: Compo {
//         *unsafe {
//             erazed
//                 .unerase_ref::<Self::Ty>(tid)
//                 .unwrap_or_else(|| unimplemented!("override 'decompress_type'"))
//         }
//     }
// }

pub struct TypeIndex {
    pub lang: &'static str,
    pub ty: TypeInternalSize,
}

// impl<T, TS, NS, LS> HyperAST for SimpleHyperAST<T, TS, NS, LS>
// where
//     T: Stored,
//     T::TreeId: NodeId<IdN = T::TreeId>,
//     TS: TypeStore,
//     NS: inner_ref::NodeStore<T::TreeId, Ref = T>,
//     NS: lending::NodeStore<T::TreeId, N = T>,
//     LS: LabelStore<str, I = T::Label>,
//     T: for<'a> lending::NLending<'a, Self::IdN>,
// {
//     type NS = NS;

//     fn node_store(&self) -> &Self::NS {
//         &self.node_store
//     }

//     type LS = LS;

//     fn label_store(&self) -> &Self::LS {
//         &self.label_store
//     }

//     type TS = TS;
// }

// impl<'store, T, TS, NS, LS> HyperASTShared for SimpleHyperAST<T, TS, NS, LS>
// where
//     T: Stored,
//     T::TreeId: NodeId<IdN = T::TreeId>,
//     T: for<'a> lending::NLending<'a, Self::IdN>,
// {
//     type IdN = T::TreeId;

//     type Idx = T::N::ChildIdx;

//     type Label = T::N::Label;

//     type TM = T;
//     // crate::store::nodes::legion::TMarker<T::TreeId>;
// }

// impl<'store, T, TS, NS, LS> AstLending<'store> for SimpleHyperAST<T, TS, NS, LS>
// where
//     T: Stored,
//     T::TreeId: NodeId<IdN = T::TreeId>,
//     T: for<'a> lending::NLending<'a, Self::IdN>,
// {
//     type RT =
//         <T as lending::NLending<'store, <SimpleHyperAST<T, TS, NS, LS> as HyperASTShared>::IdN>>::N;
// }

// impl<T, TS, NS, LS> HyperASTAsso for SimpleHyperAST<T, TS, NS, LS>
// where
//     T: Tree,
//     T::TreeId: NodeId<IdN = T::TreeId>,
//     TS: TypeStore,
//     for<'s> NS: 's + NodeStore<T::TreeId, R<'s> = T>,
//     LS: LabelStore<str, I = T::Label>,
// {
//     type T<'s>
//         = T
//     where
//         Self: 's;

//     type NS<'s>
//         = NS
//     where
//         Self: 's;

//     fn node_store(&self) -> &Self::NS<'_> {
//         &self.node_store
//     }

//     type LS = LS;

//     fn label_store(&self) -> &Self::LS {
//         &self.label_store
//     }

//     type TS<'s>
//         = TS
//     where
//         Self: 's;
// }

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct AnyType(pub(crate) &'static dyn HyperType);

unsafe impl Send for AnyType {}
unsafe impl Sync for AnyType {}
impl PartialEq for AnyType {
    fn eq(&self, other: &Self) -> bool {
        self.generic_eq(other.0)
    }
}
// impl Default for AnyType {}
impl Eq for AnyType {}
impl Hash for AnyType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_shared().hash(state);
    }
}
impl Display for AnyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
impl From<&'static dyn HyperType> for AnyType {
    fn from(value: &'static dyn HyperType) -> Self {
        Self(value)
    }
}

impl HyperType for AnyType {
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + PartialEq + Sized,
    {
        // elegant solution leveraging the static nature of node types
        std::ptr::eq(self.as_static(), other.as_static())
    }

    fn is_file(&self) -> bool {
        self.0.is_file()
    }

    fn is_directory(&self) -> bool {
        self.0.is_directory()
    }

    fn is_spaces(&self) -> bool {
        self.0.is_spaces()
    }

    fn is_syntax(&self) -> bool {
        self.0.is_syntax()
    }

    fn as_shared(&self) -> Shared {
        self.0.as_shared()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.0.as_any()
    }

    fn as_static(&self) -> &'static dyn HyperType {
        self.0.as_static()
    }

    fn as_static_str(&self) -> &'static str {
        self.0.as_static_str()
    }

    fn is_hidden(&self) -> bool {
        self.0.is_hidden()
    }

    fn is_supertype(&self) -> bool {
        self.0.is_supertype()
    }

    fn is_named(&self) -> bool {
        self.0.is_named()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        // self.0.get_lang()
        // NOTE quite surprising Oo
        // the type inference is working in our favour
        // TODO post on https://users.rust-lang.org/t/understanding-trait-object-safety-return-types/73425 or https://stackoverflow.com/questions/54465400/why-does-returning-self-in-trait-work-but-returning-optionself-requires or https://www.reddit.com/r/rust/comments/lbbobv/3_things_to_try_when_you_cant_make_a_trait_object/
        self.0.lang_ref()
    }

    fn lang_ref(&self) -> LangWrapper<AnyType> {
        self.0.lang_ref()
    }
}
