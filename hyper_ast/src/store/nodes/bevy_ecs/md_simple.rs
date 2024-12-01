use super::*;

pub fn compute_byte_len_aux<'r>(e: EntityRef<'r>) -> Result<usize, &'r [Entity]> {
    let r = match (e.get::<Lab>(), e.get::<Children>()) {
        (None, None) => e.get::<Type>().unwrap().0.as_bytes().len(),
        (Some(Lab(l)), None) => l.as_bytes().len(),
        (_, Some(Children(cs))) => return Err(cs),
    };
    Ok(r)
}


pub fn compute_rec_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => cs
            .iter()
            .map(|x| compute_rec_byte_len(w, w.entity(*x)).to_usize())
            .sum(),
    };
    ByteLen(r)
}

#[test]
fn test_compute_byte_len() {
    let mut world = World::new();

    // construction
    let l_42 = world.spawn(Leaf {
        ty: Type("number"),
        label: Lab("42"),
    });
    let l_42 = l_42.id();
    let op_plus = world.spawn(Type("+"));
    let op_plus = op_plus.id();
    let l_x = world.spawn(Leaf {
        ty: Type("identifier"),
        label: Lab("x"),
    });
    let l_x = l_x.id();
    let expr_bin = world.spawn(Node {
        ty: Type("binary_expr"),
        cs: Children(vec![l_42, op_plus, l_x].into()),
    });
    let expr_bin = expr_bin.id();

    // usage
    assert_eq!(ByteLen(4), compute_rec_byte_len(&world, world.entity(expr_bin)));
}


pub fn precompute_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => w
            .get_many_entities_dynamic(&cs)
            .expect("should all be there")
            .iter()
            .map(|x| x.get::<ByteLen>().unwrap().to_usize())
            .sum(),
    };
    ByteLen(r)
}

#[test]
fn test_precompute_byte_len() {
    // Create a new empty World to hold our Entities and Components
    let mut world = World::new();

    // construction
    let mut l_42 = world.spawn(Leaf {
        ty: Type("number"),
        label: Lab("42"),
    });
    precompute_md(&mut l_42, precompute_byte_len);
    let l_42 = l_42.id();
    let mut op_plus = world.spawn(Type("+"));
    precompute_md(&mut op_plus, precompute_byte_len);
    let op_plus = op_plus.id();
    let mut l_x = world.spawn(Leaf {
        ty: Type("identifier"),
        label: Lab("x"),
    });
    precompute_md(&mut l_x, precompute_byte_len);
    let l_x = l_x.id();
    let mut expr_bin = world.spawn(Node {
        ty: Type("binary_expr"),
        cs: Children(vec![l_42, op_plus, l_x].into()),
    });
    precompute_md(&mut expr_bin, precompute_byte_len);
    let expr_bin = expr_bin.id();

    // usage
    assert_eq!(Some(&ByteLen(4)), world.entity(expr_bin).get::<ByteLen>());
}

// WIP

pub fn compute_hybr_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => w
            .get_many_entities_dynamic(&cs)
            .expect("should all be there")
            .iter()
            .map(|x| {
                x.get::<ByteLen>()
                    .map_or_else(|| compute_rec_byte_len(w, *x).to_usize(), |x| x.to_usize())
            })
            .sum(),
    };
    ByteLen(r)
}

pub fn compute_hybrec_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => w
            .get_many_entities_dynamic(&cs)
            .expect("should all be there")
            .iter()
            .map(|x| compute_hybr_byte_len(w, *x).to_usize())
            .sum(),
    };
    ByteLen(r)
}