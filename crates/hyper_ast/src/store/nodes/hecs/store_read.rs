use num::ToPrimitive;

use super::*;
use std::fmt::Debug;
impl Debug for NodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("hecs::NodeStore")
            .field("count", &self.count)
            .field("errors", &self.errors)
            .field("internal_len", &self.internal.len())
            // .field("internal", &self.internal)
            .finish()
    }
}

impl<'a> crate::types::NLending<'a, NodeIdentifier> for NodeStore {
    type N = HashedNodeRef<'a, NodeIdentifier>;
}

impl crate::types::NodeStore<NodeIdentifier> for NodeStore {
    fn resolve(&self, id: &NodeIdentifier) -> <Self as crate::types::NLending<'_, NodeIdentifier>>::N {
        self.internal
            .entity(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }

}


// impl crate::types::NodStore<NodeIdentifier> for NodeStore {
//     type R<'a> = HashedNodeRef<'a, NodeIdentifier>;
// }

// impl crate::types::NodeStore<NodeIdentifier> for NodeStore {
//     fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
//         self.internal
//             .entity(id.clone())
//             .map(|x| HashedNodeRef::new(x))
//             .unwrap()
//     }
// }

impl NodeStore {
    pub fn len(&self) -> usize {
        self.internal.len().to_usize().unwrap()
    }
}
impl Default for NodeStore {
    fn default() -> Self {
        Self::new()
    }
}
