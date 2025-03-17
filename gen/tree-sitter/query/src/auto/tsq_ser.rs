use hyperast::nodes::Space;
use hyperast::types;
use hyperast::types::AstLending;
use hyperast::types::Childrn;
use hyperast::types::HyperType;
use hyperast::types::NodeId;
use std::fmt::{Debug, Display, Write};

pub struct TreeToQuery<
    'a,
    HAST: types::HyperAST,
    F: Fn(&<HAST as AstLending<'_>>::RT) -> bool,
    const TY: bool = true,
    const LABELS: bool = false,
    const IDS: bool = false,
    const SPC: bool = false,
> {
    stores: &'a HAST,
    root: HAST::IdN,
    pred: F, // TODO use a TS query, the list. Could even validate at compile time with proc macro
}

pub fn to_query<'store, HAST: types::HyperAST>(
    stores: &'store HAST,
    root: HAST::IdN,
) -> TreeToQuery<
    'store,
    HAST,
    impl for<'a> Fn(&'a <HAST as AstLending<'_>>::RT) -> bool,
    true,
    true,
    false,
    false,
> {
    TreeToQuery::with_pred(stores, root, |_| true)
}

impl<
        'store,
        HAST: types::HyperAST,
        F: Fn(&<HAST as AstLending<'_>>::RT) -> bool,
        const TY: bool,
        const LABELS: bool,
        const IDS: bool,
        const SPC: bool,
    > TreeToQuery<'store, HAST, F, TY, LABELS, IDS, SPC>
{
    pub fn with_pred(stores: &'store HAST, root: HAST::IdN, pred: F) -> Self {
        Self { stores, root, pred }
    }
}

impl<
        'store,
        HAST: types::HyperAST,
        F: Fn(&<HAST as AstLending<'_>>::RT) -> bool,
        const TY: bool,
        const LABELS: bool,
        const IDS: bool,
        const SPC: bool,
    > Display for TreeToQuery<'store, HAST, F, TY, LABELS, IDS, SPC>
where
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::IdN: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(&self.root, &mut 0, f).map(|_| ())
    }
}

impl<
        'store,
        HAST: types::HyperAST,
        F: Fn(&<HAST as AstLending<'_>>::RT) -> bool,
        const TY: bool,
        const LABELS: bool,
        const IDS: bool,
        const SPC: bool,
    > TreeToQuery<'store, HAST, F, TY, LABELS, IDS, SPC>
where
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    // pub fn tree_syntax_with_ids(
    fn serialize(
        &self,
        id: &HAST::IdN,
        count: &mut usize,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        const LABELS0: bool = false;
        use types::LabelStore;
        use types::Labeled;
        use types::NodeStore;
        use types::WithChildren;
        let b = self.stores.node_store().resolve(id);
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.resolve_type(&id);
        let label = b.try_get_label();
        let children = b.children();

        if kind.is_spaces() {
            if SPC {
                let s = LabelStore::resolve(self.stores.label_store(), &label.unwrap());
                let b: String = Space::format_indentation(s.as_bytes())
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                write!(out, "(")?;
                if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                    if TY {
                        write!(out, "_",)
                    } else {
                        Ok(x)
                    }
                })?;
                if LABELS0 {
                    write!(out, " {:?}", Space::format_indentation(b.as_bytes()))?;
                }
                write!(out, ")")?;
            }
            return Ok(());
        }

        let w_kind = |out: &mut std::fmt::Formatter<'_>| {
            if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                if TY {
                    write!(out, "{}", kind.to_string())
                } else {
                    Ok(x)
                }
            })
        };

        match (label, children) {
            (None, None) => {
                // w_kind(out)?;
                if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                    if TY {
                        write!(out, "\"{}\"", kind.to_string())
                    } else {
                        Ok(x)
                    }
                })?;
            }
            (label, Some(children)) => {
                if let Some(label) = label {
                    let s = self.stores.label_store().resolve(label);
                    if LABELS0 {
                        write!(out, " {:?}", Space::format_indentation(s.as_bytes()))?;
                    }
                }
                if !children.is_empty() {
                    let it = children.iter_children();
                    write!(out, "(")?;
                    w_kind(out)?;
                    for id in it {
                        let kind = self.stores.resolve_type(&id);
                        if !kind.is_spaces() {
                            write!(out, " ")?;
                        }
                        self.serialize(&id, count, out)?;
                    }
                    write!(out, ")")?;
                }
            }
            (Some(label), None) => {
                write!(out, "(")?;
                w_kind(out)?;
                if LABELS0 {
                    let s = self.stores.label_store().resolve(label);
                    if s.len() > 20 {
                        write!(out, "='{}...'", &s[..20])?;
                    } else {
                        write!(out, "='{}'", s)?;
                    }
                }
                write!(out, ")")?;
                if LABELS && (self.pred)(&b) {
                    let s = self.stores.label_store().resolve(label);
                    write!(out, " @id{} (#eq? @id{} \"{}\")", count, count, escape(s))?;
                    *count += 1;
                }
            }
        }
        return Ok(());
    }
}

fn escape(src: &str) -> String {
    let mut escaped = String::with_capacity(src.len());
    let mut utf16_buf = [0u16; 2];
    for c in src.chars() {
        match c {
            ' ' => escaped += " ",
            '\x08' => escaped += "\\b",
            '\x0c' => escaped += "\\f",
            '\n' => escaped += "\\n",
            '\r' => escaped += "\\r",
            '\t' => escaped += "\\t",
            '"' => escaped += "\\\"",
            '\\' => escaped += "\\\\",
            c if c.is_ascii_graphic() => escaped.push(c),
            c => {
                let encoded = c.encode_utf16(&mut utf16_buf);
                for utf16 in encoded {
                    write!(&mut escaped, "\\u{:04X}", utf16).unwrap();
                }
            }
        }
    }
    escaped
}
