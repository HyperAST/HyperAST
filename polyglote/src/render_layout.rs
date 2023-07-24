
use layout::{backends::svg::SVGWriter, core::color::Color};
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::*;
use layout::core::utils::save_to_file;
use layout::std_shapes::shapes::*;
use layout::topo::layout::VisualGraph;
use std::collections::HashMap;
use std::ops::Deref;

use crate::preprocess::{TypeSys, T, SubTypes, Fields, Role, DChildren, SubType, Child};



pub(crate) fn render(mut w: TypeSys) {
    // Create a new graph:
    let mut vg = VisualGraph::new(Orientation::LeftToRight);
    let mut map = HashMap::with_capacity(w.types.len() as usize);

    let sz = Point::new(150., 100.);

    w.types.query_mut::<(&T,)>().with::<(&Child,)>().into_iter().for_each(|(e, (t,))| {
        let node = make_node(t, sz);
        let handle = vg.add_node(node);
        map.insert(e, handle);
    });
    w.types.query_mut::<(&T,)>().without::<(&Child,)>().with::<(&SubType,)>().into_iter().for_each(|(e, (t,))| {
        let node = make_node(t, sz);
        let handle = vg.add_node(node);
        map.insert(e, handle);
    });
    w.types.query_mut::<(&T,)>().without::<(&Child,)>().without::<(&SubType,)>().with::<(&SubTypes,)>().into_iter().for_each(|(e, (t,))| {
        let node = make_node(t, sz);
        let handle = vg.add_node(node);
        map.insert(e, handle);
    });
    w.types.query_mut::<(&T,)>().without::<(&Child,)>().without::<(&SubType,)>().with::<(&DChildren,)>().into_iter().for_each(|(e, (t,))| {
        let node = make_node(t, sz);
        let handle = vg.add_node(node);
        map.insert(e, handle);
    });
    w.types.query_mut::<(&T,)>().without::<(&Child,)>().without::<(&SubType,)>().with::<(&Fields,)>().into_iter().for_each(|(e, (t,))| {
        let node = make_node(t, sz);
        let handle = vg.add_node(node);
        map.insert(e, handle);
    });

    w.types.query_mut::<(&Role,)>().into_iter().for_each(|(e, (t,))| {
        let sp = ShapeKind::new_circle(&t.to_string());
        let look = StyleAttr::simple();
        let node = Element::create(sp, look, Orientation::TopToBottom, sz);
        let handle = vg.add_node(node);
        map.insert(e, handle);
    });

    w.types.query_mut::<(&SubTypes,)>().into_iter().for_each(|(e, (st,))| {
        for t in st.deref() {
            let mut arrow = Arrow::simple("st");
            arrow.look.line_color = Color::fast("blue");
            vg.add_edge(arrow, map[&e], map[&t]);
        }
    });

    w.types.query_mut::<(&Fields,)>().into_iter().for_each(|(e, (fi,))| {
        for t in fi.deref() {
            let mut arrow = Arrow::simple("fi");
            arrow.look.line_color = Color::fast("red");
            vg.add_edge(arrow, map[&e], map[&t]);
        }
    });

    w.types.query_mut::<(&DChildren,)>().into_iter().for_each(|(e, (fi,))| {
        for t in fi.deref() {
            let arrow = Arrow::simple("cs");
            vg.add_edge(arrow, map[&e], map[&t]);
        }
    });

    // Render the nodes to some rendering backend.
    let mut svg = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);

    // Save the output.
    let _ = save_to_file("/tmp/graph.svg", &svg.finalize());
}

fn make_node(t: &T, sz: Point) -> Element {
    let sp = ShapeKind::new_box(&t.to_string());
    let look = StyleAttr::simple();
    let node = Element::create(sp, look, Orientation::TopToBottom, sz);
    node
}