use super::NodeIdentifier;
use super::ANA;
use crate::impact::partial_analysis::PartialAnalysis;
use crate::types::TIdN;
use crate::types::Type;
use hyperast::filter::BF;
use hyperast::filter::{Bloom, BloomSize};
use hyperast::impact::BulkHasher;
use hyperast::store::labels::LabelStore;
use hyperast::store::nodes::legion::PendingInsert;
use hyperast::store::nodes::EntityBuilder;
use hyperast::types::LabelStore as _;
use hyperast::types::Tree;
use hyperast::types::TypeTrait;
use hyperast::types::Typed as _;

pub(crate) fn build_ana(kind: &Type, label_store: &mut LabelStore) -> Option<PartialAnalysis> {
    if kind == &Type::ClassBody
        || kind == &Type::PackageDeclaration
        || kind == &Type::ClassDeclaration
        || kind == &Type::EnumDeclaration
        || kind == &Type::InterfaceDeclaration
        || kind == &Type::AnnotationTypeDeclaration
        || kind == &Type::Program
    {
        Some(PartialAnalysis::init(kind, None, |x| {
            label_store.get_or_insert(x)
        }))
    } else if kind == &Type::TypeParameter {
        Some(PartialAnalysis::init(kind, None, |x| {
            label_store.get_or_insert(x)
        }))
    } else {
        None
    }
}

pub fn add_md_ref_ana(
    dyn_builder: &mut impl EntityBuilder,
    children_is_empty: bool,
    ana: Option<&PartialAnalysis>,
) {
    if children_is_empty {
        dyn_builder.add(BloomSize::None);
    } else {
        macro_rules! bloom_aux {
            ( $t:ty ) => {{
                type B = $t;
                let it = ana.as_ref().unwrap().solver.iter_refs();
                let it = BulkHasher::<_, <B as BF<[u8]>>::S, <B as BF<[u8]>>::H>::from(it);
                let bloom = B::from(it);
                dyn_builder.add(B::SIZE);
                dyn_builder.add(bloom);
            }};
        }
        macro_rules! bloom {
            ( $t:ty ) => {{
                bloom_aux!(Bloom::<&'static [u8], $t>);
            }};
        }
        match ana.as_ref().map(|x| x.estimated_refs_count()).unwrap_or(0) {
            x if x > 2048 => {
                dyn_builder.add(BloomSize::Much);
            }
            x if x > 1024 => bloom!([u64; 64]),
            x if x > 512 => bloom!([u64; 32]),
            x if x > 256 => bloom!([u64; 16]),
            x if x > 150 => bloom!([u64; 8]),
            x if x > 100 => bloom!([u64; 4]),
            x if x > 30 => bloom!([u64; 2]),
            x if x > 15 => bloom!(u64),
            x if x > 8 => bloom!(u32),
            x if x > 0 => bloom!(u16),
            _ => {
                dyn_builder.add(BloomSize::None);
            } // TODO use the following after having tested the previous, already enough changes for now
              // 2048.. => {
              //     dyn_builder.add(BloomSize::Much);
              // }
              // 1024.. => bloom!([u64; 64]),
              // 512.. => bloom!([u64; 32]),
              // 256.. => bloom!([u64; 16]),
              // 150.. => bloom!([u64; 8]),
              // 100.. => bloom!([u64; 4]),
              // 32.. => bloom!([u64; 2]),
              // 16.. => bloom!(u64),
              // 8.. => bloom!(u32),
              // 1.. => bloom!(u16),
              // 0 => {
              //     dyn_builder.add(BloomSize::None);
              // }
        }
    }
}

pub(crate) fn make_partial_ana(
    kind: Type,
    ana: &mut Option<PartialAnalysis>,
    label: &Option<String>,
    children: &[legion::Entity],
    label_store: &mut LabelStore,
    insertion: &PendingInsert,
) {
    if !ANA {
        *ana = None;
        return;
    }
    *ana = partial_ana_extraction(kind, ana.take(), &label, children, label_store, insertion)
        .map(|ana| ana_resolve(kind, ana, label_store));
}

pub(crate) fn ana_resolve(
    kind: Type,
    ana: PartialAnalysis,
    label_store: &LabelStore,
) -> PartialAnalysis {
    if kind == Type::ClassBody
        || kind.is_type_declaration()
        || kind == Type::MethodDeclaration
        || kind == Type::ConstructorDeclaration
    {
        log::trace!("refs in {kind:?}");
        for x in ana.display_refs(label_store) {
            log::trace!("    {}", x);
        }
        log::trace!("decls in {kind:?}");
        for x in ana.display_decls(label_store) {
            log::trace!("    {}", x);
        }
        let ana = ana.resolve();
        log::trace!("refs in {kind:?} after resolution");

        for x in ana.display_refs(label_store) {
            log::trace!("    {}", x);
        }
        ana
    } else if kind == Type::Program {
        log::debug!("refs in {kind:?}");
        for x in ana.display_refs(label_store) {
            log::debug!("    {}", x);
        }
        log::debug!("decls in {kind:?}");
        for x in ana.display_decls(label_store) {
            log::debug!("    {}", x);
        }
        let ana = ana.resolve();
        log::debug!("refs in {kind:?} after resolve");
        for x in ana.display_refs(label_store) {
            log::debug!("    {}", x);
        }
        // TODO assert that ana.solver.refs does not contains mentions to ?.this
        ana
    } else {
        ana
    }
}

pub(crate) fn partial_ana_extraction(
    kind: Type,
    ana: Option<PartialAnalysis>,
    label: &Option<String>,
    children: &[legion::Entity],
    label_store: &mut LabelStore,
    insertion: &PendingInsert,
) -> Option<PartialAnalysis> {
    let is_possibly_empty = |kind| {
        kind == Type::ArgumentList
            || kind == Type::FormalParameters
            || kind == Type::AnnotationArgumentList
            || kind == Type::SwitchLabel
            || kind == Type::Modifiers
            || kind == Type::BreakStatement
            || kind == Type::ContinueStatement
            || kind == Type::Wildcard
            || kind == Type::ConstructorBody
            || kind == Type::InterfaceBody
            || kind == Type::SwitchBlock
            || kind == Type::ClassBody
            || kind == Type::EnumBody
            || kind == Type::ModuleBody
            || kind == Type::AnnotationTypeBody
            || kind == Type::TypeArguments
            || kind == Type::ArrayInitializer
            || kind == Type::ReturnStatement
            || kind == Type::ForStatement
            || kind == Type::RequiresModifier
            || kind == Type::ERROR
    };
    let mut make = |label| {
        Some(PartialAnalysis::init(&kind, label, |x| {
            label_store.get_or_insert(x)
        }))
    };
    if kind == Type::Program {
        ana
    } else if kind.is_comment() {
        None
    } else if let Some(label) = label.as_ref() {
        let label = if kind.is_literal() {
            kind.literal_type()
        } else {
            label.as_str()
        };
        make(Some(label))
    } else if kind.is_primitive() {
        let node = insertion.resolve::<TIdN<NodeIdentifier>>(children[0]);
        let ty = node.get_type();
        let label = ty.to_str();
        make(Some(label))
    } else if let Some(ana) = ana {
        // nothing to do, resolutions at the end of post ?
        Some(ana)
    } else if kind == Type::Static
        || kind == Type::Public
        || kind == Type::Asterisk
        || kind == Type::Dimensions
        || kind == Type::Block
        || kind == Type::ElementValueArrayInitializer
        || kind == Type::PackageDeclaration
        || kind == Type::TypeParameter
    {
        make(None)
    } else if is_possibly_empty(kind) {
        if kind == Type::ArgumentList
            || kind == Type::FormalParameters
            || kind == Type::AnnotationArgumentList
        {
            if !children
                .iter()
                .all(|x| !insertion.resolve::<TIdN<NodeIdentifier>>(*x).has_children())
            {
                // eg. an empty body/block/paramlist/...
                log::error!("{:?} should only contains leafs", &kind);
            }
            make(None)
        // } else if kind == Type::SwitchLabel || kind == Type::Modifiers {
        //     // TODO decls or refs ?
        //     None
        } else {
            None
        }
    } else {
        if !children.is_empty()
            && children
                .iter()
                .all(|x| !insertion.resolve::<TIdN<NodeIdentifier>>(*x).has_children())
        {
            // eg. an empty body/block/paramlist/...
            log::error!("{:?} should only contains leafs", kind);
        }
        None
    }
}
