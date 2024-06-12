use crate::search::steped::{Query, TSTreeCursor};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                eprintln!("{}:{} {} - {}", file, line, record.level(), record.args());
            } else {
                eprintln!("{} - {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

#[test]
fn convert() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let language = tree_sitter_java::language();
    let source = r#"(class_declaration
        name: (identifier) @name
        body: (class_body) @class_body)"#;
    let source = r#"(class_declaration
        (identifier) @name
        (class_body) @class_body)"#;
    let source = "(_
        (expression_statement)
        .
        (statement)
    ) @a";
    // Compile the query.
    let query = Query::new(source, language).unwrap();
    let language = tree_sitter_java::language();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let text = "class A {}";
    let text = "class A {
        B f() {
            a++;
            return null;
        }
    }";
    let tree = parser.parse(text, None).unwrap();

    let cursor = TSTreeCursor::new(text.as_bytes(), tree.root_node().walk());

    let mut qcursor = super::QueryCursor::<'_> {
        halted: false,
        ascending: false,
        states: vec![],
        capture_list_pool: super::CaptureListPool::default(),
        finished_states: Default::default(),
        max_start_depth: u32::MAX,
        did_exceed_match_limit: false,
        // .did_exceed_match_limit = false,
        // .ascending = false,
        // .halted = false,
        // .states = array_new(),
        // .finished_states = array_new(),
        // .capture_list_pool = capture_list_pool_new(),
        // .max_start_depth = UINT32_MAX,
        depth: 0,
        on_visible_node: true,
        query: &query,
        cursor,
        next_state_id: 0,
        //   array_clear(&self->states);
        //   array_clear(&self->finished_states);
        //   ts_tree_cursor_reset(&self->cursor, node);
        //   capture_list_pool_reset(&self->capture_list_pool);
        //   self->on_visible_node = true;
        //   self->next_state_id = 0;
        //   self->depth = 0;
        //   self->ascending = false;
        //   self->halted = false;
        //   self->query = query;
        //   self->did_exceed_match_limit = false;

        // .start_byte = 0,
        // .end_byte = UINT32_MAX,
        // .start_point = {0, 0},
        // .end_point = POINT_MAX,
    };
    let mut matched = false;
    while let Some(m) = qcursor.next_match() {
        dbg!(m.pattern_index);
        dbg!(m.captures.len());
        for c in &m.captures {
            let i = c.index;
            dbg!(i);
            let name = query.capture_name(i);
            dbg!(name);
            dbg!(c.node.utf8_text(text.as_bytes()).unwrap());
        }
        matched = true;
    }
    assert!(matched)
}
