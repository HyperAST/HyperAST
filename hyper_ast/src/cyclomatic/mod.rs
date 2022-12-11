use legion::world::ComponentError;

use crate::{
    store::nodes::legion::HashedNodeRef,
    types::{Type, Typed},
};

pub fn is_cyclomatic_persisted(t: &Type) -> bool {
    t == &Type::ClassDeclaration
        || t == &Type::InterfaceDeclaration
        || t == &Type::EnumDeclaration
        || t == &Type::AnnotationTypeDeclaration
        || t == &Type::MethodDeclaration
        || t == &Type::ConstructorDeclaration
        || t == &Type::Program
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
pub struct Mcc {
    value: u32,
}

impl Mcc {
    pub fn new(kind: &Type) -> Self {
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

    pub fn persist(kind: &Type) -> bool {
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

impl MetaData for Mcc {
    type R = Result<u32, ComponentError>;

    fn retrieve(node: &HashedNodeRef) -> Self::R {
        let kind = node.get_type();
        if Mcc::persist(&kind) {
            node.get_component::<Mcc>().map(|x| x.value + 1)
        } else {
            Ok(0)
        }
    }
}

pub trait MetaData {
    type R;
    fn retrieve(node: &HashedNodeRef) -> Self::R;
}

/// considering https://github.com/jacoco/jacoco/blob/b68fe1a0a7fb86f12cda689ec473fd6633699b55/org.jacoco.doc/docroot/doc/counters.html#L102
///
/// v(G) = b - d + 1 where b is the number of branches and d the number of dessision points
struct MccJacoco {
    value: u32,
}

/// v(G) = e - n + p
impl MccJacoco {
    pub fn new(kind: &Type) -> Self {
        Self {
            value: if kind.is_fork() { 1 } else { 0 },
        }
    }

    pub fn acc(self, kind: &Type, acc: &mut Self) {
        todo!()
    }
}

struct McCabe {
    value: u32,
}

/// v(G) = e - n + p
impl McCabe {
    pub fn new(kind: &Type) -> Self {
        Self {
            value: if kind.is_fork() { 1 } else { 0 },
        }
    }

    pub fn acc(self, kind: &Type, acc: &mut Self) {
        todo!()
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
