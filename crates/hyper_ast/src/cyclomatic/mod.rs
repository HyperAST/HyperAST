use crate::types::{TypeTrait, Typed, WithMetaData};

pub fn is_cyclomatic_persisted<K: TypeTrait>(t: &K) -> bool {
    t.is_type_declaration() // TODO EnumConstant might not be appropriate here
    || t.is_executable_member()
    || t.is_file()
    // || t == &Type::ClassDeclaration
    // || t == &Type::InterfaceDeclaration
    // || t == &Type::EnumDeclaration
    // || t == &Type::AnnotationTypeDeclaration
    // || t == &Type::MethodDeclaration
    // || t == &Type::ConstructorDeclaration
    // || t == &Type::Program
}

// TODO look at https://crates.io/crates/complexity
// and also https://github.com/jacoco/jacoco/blob/b68fe1a0a7fb86f12cda689ec473fd6633699b55/org.jacoco.doc/docroot/doc/counters.html#L102

/// An analysis which computes McCabe's cyclomatic complexity of any vertex
///
/// The McCabes complexity is calculated simply by counting all branching
/// statements + 1. This is not correct in the strictest sense of the original
/// conception of cyclomatic complexity or even McCabe's complexity (which can
/// only be computed this way on the machine code level) but this is what most
/// modern tools do.
/// From https://bitbucket.org/sealuzh/lisa/src/master/lisa-module/src/main/scala/ch/uzh/ifi/seal/lisa/module/analysis/object-oriented/MccAnalysis.scala
/// same POV https://github.com/qxo/eclipse-metrics-plugin/blob/08e51bd48725494aaa82023716ce659504948610/net.sourceforge.metrics/src/net/sourceforge/metrics/calculators/McCabe.java
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
pub struct Mcc {
    value: u32,
}

impl Mcc {
    pub fn new<K: TypeTrait>(kind: &K) -> Self {
        // TODO also consider || and && as forks
        // we would need to check the operand ie. the children
        Self {
            value: if kind.is_fork() { 1 } else { 0 },
        }
    }

    /// TODO reverse &mut and self
    pub fn acc(self, acc: &mut Self) {
        acc.value += self.value
    }

    pub fn persist<K: TypeTrait>(kind: &K) -> bool {
        is_cyclomatic_persisted(kind)
    }

    // pub fn persist(&self, kind: &Type) -> Option<Self> {
    //     if is_cyclomatic_persisted(kind) {
    //         Some(Self {
    //             value: self.value + 1,
    //         })
    //     } else {
    //         None
    //     }
    // }
}

impl<T: Typed + WithMetaData<Mcc>> MetaData<T> for Mcc
where
    T::Type: TypeTrait,
{
    type R = u32;

    fn retrieve(node: &T) -> Self::R {
        let kind = node.get_type();
        if Mcc::persist(&kind) {
            node.get_metadata()
                .map(|x| x.value + 1)
                .expect("missing mcc")
        } else {
            0
        }
    }
}

pub trait MetaData<T> {
    type R;
    fn retrieve(node: &T) -> Self::R;
}

/// considering https://github.com/jacoco/jacoco/blob/b68fe1a0a7fb86f12cda689ec473fd6633699b55/org.jacoco.doc/docroot/doc/counters.html#L102
///
/// v(G) = b - d + 1 where b is the number of branches and d the number of dessision points
pub struct MccJacoco {
    value: u32,
}

/// v(G) = e - n + p
impl MccJacoco {
    pub fn new<K: TypeTrait>(kind: &K) -> Self {
        // TODO also consider || and && as forks
        // we would need to check the operand ie. the children
        Self {
            value: if kind.is_fork() { 1 } else { 0 },
        }
    }

    /// TODO reverse &mut and self
    pub fn acc(self, acc: &mut Self) {
        acc.value += self.value
    }
}

#[cfg(test)]
pub mod tests {
    // *v = e - n + 2p
    // a -> b
    // v = 1 - 2 + 2 = 1

    // if x {a} else {b} ; c
    // v = 4 - 4 + 2 = 2

    // while x {a} ; b
    // v = 3 - 3 + 2 = 2

    // do {a} while x ; b
    // v = 3 - 3 + 2 = 2
}
