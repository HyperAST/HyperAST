
pub struct ZsTree<
    IdC: PrimInt,
    IdD: PrimInt + Into<usize>,
> {
    id_compressed: Vec<IdC>,
    id_parent: Vec<IdD>,
    id_first_child: Vec<IdD>,
    llds: Vec<IdD>,
    // LR_keyroots(T) = {klthere exists no k’> k such that l(k)= l(k’)}.
    kr: Vec<IdD>,
}

pub trait ZsStore<IdC: PrimInt, IdD: PrimInt + Into<usize>> {
    fn lld(&self, last_row: IdD) -> IdD;
    fn tree(&self, row: IdD) -> IdC;
    fn get_node_count(&self) -> IdD;
    fn get_leaf_count(&self) -> IdD;
    fn kr(&self, x: IdD) -> IdD;
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> ZsStore<IdC, IdD>
    for ZsTree</*T, HK, HP, */ IdC, IdD>
{
    fn lld(&self, i: IdD) -> IdD {
        self.llds[i.into()] // + num_traits::one()
    }

    fn tree(&self, i: IdD) -> IdC {
        self.id_compressed[i.into()] // + num_traits::one()
    }

    fn get_node_count(&self) -> IdD {
        cast(self.id_compressed.len()).unwrap()
    }

    fn get_leaf_count(&self) -> IdD {
        cast(self.kr.len()).unwrap()
    }

    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.into()]
    }
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> DecompressedTreeStore<IdC, IdD>
    for ZsTree</*T,HK,HP,*/ IdC, IdD>
{
    fn new<
        T: Tree<TreeId = IdC> + WithHashs<HK = HK, HP = HP>,
        HK: HashKind,
        HP: PrimInt,
        S: NodeStore<T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self {
        let mut id_compressed: Vec<IdC> = vec![*root];
        let mut id_parent: Vec<IdD> = vec![num_traits::zero()];
        let mut id_first_child: Vec<IdD> = vec![];
        let mut i: usize = 0;
        let mut leaf_count = 0;
        let mut llds: Vec<IdD> = vec![];

        while i < id_compressed.len() {
            let ii = cast(i).unwrap();
            let node = store.get_node_at_id(&id_compressed[i]);
            let l = node.get_children();
            llds.push(ii);
            id_parent.extend(l.iter().map(|_| ii));
            if l.len() > 0 {
                id_first_child.push(cast(id_compressed.len()).unwrap());
            } else {
                // leaf
                leaf_count += 1;
                id_first_child.push(num_traits::zero());
                let mut a = ii;
                loop {
                    if a == num_traits::zero() {
                        break;
                    }
                    let p = id_parent[a.into()];
                    if id_first_child[p.into()] != a {
                        break;
                    }
                    llds[p.into()] = ii;
                    a = p;
                }
            };
            id_compressed.extend_from_slice(l);
            i += 1;
        }
        let node_count = id_compressed.len();
        let mut kr = vec![num_traits::zero(); leaf_count];
        let mut visited = vec![false; node_count];
        let mut k = 0;
        for i in (0..node_count) {
            if !visited[llds[i].into()] {
                kr[k] = cast(i).unwrap();
                visited[llds[i].into()] = true;
                k += 1;
            }
        }

        Self {
            id_compressed,
            id_parent,
            id_first_child,
            llds,
            kr,
        }
    }

    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: IdD) -> IdC {
        self.id_compressed[id.to_usize().unwrap()]
    }

    fn has_parent(&self, id: IdD) -> bool {
        self.parent(id) != None
    }

    fn parent(&self, id: IdD) -> Option<IdD> {
        let r = self.id_parent[id.to_usize().unwrap()];
        if r == num_traits::zero() {
            None
        } else {
            Some(r)
        }
    }

    fn has_children(&self, id: IdD) -> bool {
        self.first_child(id) != None
    }

    fn first_child(&self, id: IdD) -> Option<IdD> {
        let r = self.id_first_child[id.to_usize().unwrap()];
        if r == num_traits::zero() {
            None
        } else {
            Some(r)
        }
    }
}
impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> ZsTree<IdC, IdD> {}