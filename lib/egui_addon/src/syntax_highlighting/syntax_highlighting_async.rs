use std::{
    ops::Range,
    sync::{Arc, Mutex, RwLock},
};

use egui::text::LayoutJob;

/// View some code with syntax highlighting and selection.
pub fn code_view_ui(ui: &mut egui::Ui, mut code: &str) {
    let language = "rs";
    let theme = CodeTheme::from_memory(ui.ctx());

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string, language);
        // layout_job.wrap.max_width = wrap_width; // no wrapping
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        egui::TextEdit::multiline(&mut code)
            .font(egui::TextStyle::Monospace) // for cursor height
            .code_editor()
            .desired_rows(1)
            .lock_focus(true)
            .layouter(&mut layouter),
    );
}

impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highlighter {
    fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob {
        self.highlight(theme, code, lang)
    }
}

/// Memoized Code highlighting
pub fn highlight0(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    ctx.memory_mut(|mem| {
        mem.caches
            .cache::<HighlightCache>()
            .get((theme, code, language))
    })
}

/// Memoized Code highlighting
pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    // async fn something_async() {
    //     wasm_rs_dbg::dbg!("aaa");
    // }
    // let future = async move { something_async().await };
    // let promise = async_exec::spawn_stuff(future);

    // // let aaa = async_exec::hello();
    // impl cache::Spawner<(&CodeTheme, &str, &str), IncrementalHighlightLayout> for Highlighter {
    //     fn spawn(
    //         &self,
    //         (theme, code, lang): (&CodeTheme, &str, &str),
    //     ) -> IncrementalHighlightLayout {
    //         self.incremental(theme, code, lang)
    //     }
    // }
    // impl cache::IncrementalComputer<Highlighter, (&CodeTheme, &str, &str), LayoutJob>
    //     for IncrementalHighlightLayout
    // {
    //     fn increment(
    //         &mut self,
    //         hh: &Highlighter,
    //         (theme, code, _lang): (&CodeTheme, &str, &str),
    //     ) -> LayoutJob {
    //         wasm_rs_dbg::dbg!(self.i);
    //         let theme = theme.syntect_theme.syntect_key_name();
    //         let theme = &hh.ts.themes[theme];
    //         let highlighter = syntect::highlighting::Highlighter::new(theme);

    //         LayoutJob {
    //             text: code.into(),
    //             sections: self.inc(hh, &highlighter, code, 100),
    //             ..Default::default()
    //         }
    //     }
    // }

    #[derive(Default)]
    struct IncrementalSpawner(Arc<Highlighter>);

    struct Incremental {
        mt: Vec<Arc<Mutex<async_exec::TimeoutHandle>>>,
        h: Arc<IncrementalHighlightLayout2>,
        job: LayoutJob,
        i: usize,
    }

    impl cache::Spawner<(&CodeTheme, &str, &str), Incremental> for IncrementalSpawner {
        fn spawn(
            &self,
            ctx: &egui::Context,
            (theme, code, lang): (&CodeTheme, &str, &str),
        ) -> Incremental {
            Incremental {
                mt: vec![],
                h: Arc::new(self.0.incremental(ctx.clone(), theme, code, lang)),
                job: LayoutJob {
                    text: code.to_string(),
                    ..Default::default()
                },
                i: 0,
            }
        }
    }
    impl cache::IncrementalComputer<IncrementalSpawner, (&CodeTheme, &str, &str), LayoutJob>
        for Incremental
    {
        fn increment(&mut self, hh: &IncrementalSpawner, x: (&CodeTheme, &str, &str)) -> LayoutJob {
            if self.i < self.job.text.len() {
                // self.0 = Some(increment(&self.h, &hh.0, x));
                // if self.i == 0 {
                //     for _ in 0..40 {
                //         self.mt
                //             .push(Arc::new(Mutex::new(increment(self.h.clone(), &hh.0, x, 2))));
                //     }
                // } else {
                //     for _ in 0..3 {
                //         self.mt.push(Arc::new(Mutex::new(increment(
                //             self.h.clone(),
                //             &hh.0,
                //             x,
                //             100,
                //         ))));
                //     }
                // }
                {
                    let theme = x.0.syntect_theme.syntect_key_name();
                    let hh = hh.0.clone();
                    let this = self.h.clone();
                    let fut = move || {
                        IncrementalHighlightLayout2::highlight_n_auto(
                            this.clone(),
                            hh.clone(),
                            &theme,
                            10,
                        )
                    };
                    self.mt
                        .push(Arc::new(Mutex::new(async_exec::spawn_macrotask(Box::new(
                            fut,
                        )))));
                }
                // self.h.read().unwrap().job.clone()
                for _ in 0..self.h.as_ref().sections.len() {
                    let sections = self.h.as_ref().sections.pop();
                    if let Some(sections) = sections {
                        self.job.sections.extend_from_slice(&sections);
                    }
                }
                self.i = self.h.as_ref().inner.read().unwrap().i;
                let mut job = self.job.clone();
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: self.i..job.text.len(),
                    format: TextFormat {
                        font_id: egui::FontId::monospace(12.0),
                        ..Default::default()
                    },
                });
                job
            } else if !self.mt.is_empty() {
                self.mt.clear();
                self.job.clone()
            } else {
                self.job.clone()
            }
        }
    }

    fn increment(
        this: Arc<IncrementalHighlightLayout2>,
        hh: &Arc<Highlighter>,
        (theme, _code, _lang): (&CodeTheme, &str, &str),
        n: usize,
    ) -> TimeoutHandle {
        let theme = theme.syntect_theme.syntect_key_name();

        let hh = hh.clone();
        // let that = this.clone();

        // async fn something_async() {
        //     wasm_rs_dbg::dbg!("aaa");
        // }
        let future = move || {
            // let mut t = that.write().unwrap();

            // t.async_aux(hh.as_ref(), &highlighter).await
            IncrementalHighlightLayout2::highlight_n(this.clone(), hh.clone(), theme, n)
            // .into_future()
            // .await
        };
        async_exec::spawn_macrotask(Box::new(future))
    }

    // type HighlightCache = cache::IncrementalCache<IncrementalHighlightLayout, Highlighter>;
    type HighlightCache = cache::IncrementalCache<Incremental, IncrementalSpawner>;

    let res = ctx.memory_mut(|mem| {
        mem.caches
            .cache::<HighlightCache>()
            .get(ctx, (theme, code, language))
    });

    // drop(aaa);
    res
}

/// slight modifications to egui's Framecache
pub mod cache {
    use std::{
        collections::HashMap,
        hash::{BuildHasher, Hasher},
    };

    use egui::util::cache::CacheTrait;

    pub trait Spawner<Key, Value>: 'static + Send + Sync {
        fn spawn(&self, ctx: &egui::Context, key: Key) -> Value;
    }

    pub trait IncrementalComputer<Computer, Key, Value>: 'static + Send + Sync {
        fn increment(&mut self, computer: &Computer, key: Key) -> Value;
    }

    /// Caches the results of a computation for one frame.
    /// If it is still used next frame, it is not recomputed.
    /// If it is not used next frame, it is evicted from the cache to save memory.
    pub struct IncrementalCache<IncState, Computer> {
        generation: u32,
        computer: Computer,
        cache: HashMap<u64, (u32, IncState)>, //nohash_hasher::IntMap<u64, (u32, Value)>,
    }

    impl<IncState, Computer> Default for IncrementalCache<IncState, Computer>
    where
        Computer: Default,
    {
        fn default() -> Self {
            Self::new(Computer::default())
        }
    }

    impl<IncState, Computer> IncrementalCache<IncState, Computer> {
        pub fn new(computer: Computer) -> Self {
            Self {
                generation: 0,
                computer,
                cache: Default::default(),
            }
        }

        /// Must be called once per frame to clear the cache.
        pub fn evice_cache(&mut self) {
            let current_generation = self.generation;
            self.cache.retain(|_key, cached| {
                current_generation.abs_diff(cached.0) < 50
                // cached.0 == current_generation // only keep those that were used this frame
            });
            self.generation = self.generation.wrapping_add(1);
        }
    }

    impl<IncState, Computer> IncrementalCache<IncState, Computer> {
        /// Get from cache (if the same key was used last frame)
        /// or recompute and store in the cache.
        pub fn get<Key, Value>(&mut self, ctx: &egui::Context, key: Key) -> Value
        where
            Key: Clone + std::hash::Hash,
            IncState: IncrementalComputer<Computer, Key, Value>,
            Computer: Spawner<Key, IncState>,
        {
            // let hash = crate::util::hash(key);
            let hash = {
                let ref this = self.cache.hasher();
                let mut hasher = this.build_hasher();
                (&key).hash(&mut hasher);
                hasher.finish()
            };

            match self.cache.entry(hash) {
                std::collections::hash_map::Entry::Occupied(entry) => {
                    let cached = entry.into_mut();
                    cached.0 = self.generation;
                    cached.1.increment(&self.computer, key)
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let mut incremental = self.computer.spawn(ctx, key.clone());
                    let value = incremental.increment(&self.computer, key);
                    entry.insert((self.generation, incremental));
                    value
                }
            }
        }
    }

    impl<IncState: 'static + Send + Sync, Computer: 'static + Send + Sync> CacheTrait
        for IncrementalCache<IncState, Computer>
    {
        fn update(&mut self) {
            self.evice_cache();
        }

        fn len(&self) -> usize {
            self.cache.len()
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
}

struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

use syntect::easy::HighlightLines;
use syntect::highlighting::FontStyle;
use syntect::util::LinesWithEndings;

impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, code: &str, lang: &str) -> LayoutJob {
        self.highlight_impl(theme, code, lang).unwrap_or_else(|| {
            // Fallback:
            LayoutJob::simple(
                code.into(),
                egui::FontId::monospace(12.0),
                if theme.dark_mode {
                    egui::Color32::LIGHT_GRAY
                } else {
                    egui::Color32::DARK_GRAY
                },
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(&self, theme: &CodeTheme, text: &str, language: &str) -> Option<LayoutJob> {
        let mut h = self.init_h(language, theme)?;
        let (mut job, lines) = self.init_processing(text);
        for line in lines {
            self.process_line(&mut h, line, text, &mut job)?;
        }
        Some(job)
    }

    fn init_processing<'a>(&self, text: &'a str) -> (LayoutJob, LinesWithEndings<'a>) {
        let job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };
        let lines = LinesWithEndings::from(text);
        (job, lines)
    }

    fn init_h<'a>(&'a self, language: &str, theme: &CodeTheme) -> Option<HighlightLines<'a>> {
        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;
        let theme = theme.syntect_theme.syntect_key_name();
        let theme = &self.ts.themes[theme];
        let h = HighlightLines::new(syntax, theme);
        Some(h)
    }

    fn process_line<'a, 's>(
        &'a self,
        h: &'a mut HighlightLines<'s>,
        line: &'s str,
        text: &'s str,
        job: &mut LayoutJob,
    ) -> Option<()> {
        let syntect_highlighted_line = h.highlight_line(line, &self.ps).ok()?;
        Some(for (style, range) in syntect_highlighted_line {
            let byte_range = as_byte_range(text, range);
            let format = convert_syntect_style(style);
            let section = LayoutSection {
                leading_space: 0.0,
                byte_range,
                format,
            };
            job.sections.push(section);
        })
    }

    fn incremental(
        &self,
        ctx: egui::Context,
        theme: &CodeTheme,
        code: &str,
        lang: &str,
    ) -> IncrementalHighlightLayout2 {
        let syntax = self
            .ps
            .find_syntax_by_name(lang)
            .or_else(|| self.ps.find_syntax_by_extension(lang))
            .or_else(|| {
                wasm_rs_dbg::dbg!(lang);
                self.ps.find_syntax_by_extension("java")
            })
            .unwrap();
        let theme = theme.syntect_theme.syntect_key_name();
        let theme = &self.ts.themes[theme];
        let highlighter = syntect::highlighting::Highlighter::new(theme);
        let highlight_state = syntect::highlighting::HighlightState::new(
            &highlighter,
            syntect::parsing::ScopeStack::new(),
        );
        let parse_state = syntect::parsing::ParseState::new(syntax);
        // let job = LayoutJob {
        //     text: code.into(),
        //     ..Default::default()
        // };

        IncrementalHighlightLayout2 {
            ctx,
            inner: RwLock::new(IncrementalHighlightLayout2Inner {
                macrotask: None,
                highlight_state,
                parse_state,
                i: 0,
            }),
            text: code.into(),
            sections: SegQueue::default(),
        }
    }
}

use crossbeam_queue::SegQueue;

struct IncrementalHighlightLayout2 {
    ctx: egui::Context,
    inner: RwLock<IncrementalHighlightLayout2Inner>,
    text: String,
    sections: SegQueue<Vec<LayoutSection>>,
}

struct IncrementalHighlightLayout2Inner {
    macrotask: Option<Arc<Mutex<async_exec::TimeoutHandle>>>,
    highlight_state: syntect::highlighting::HighlightState,
    parse_state: syntect::parsing::ParseState,
    pub i: usize,
}

impl IncrementalHighlightLayout2 {
    fn highlight_n_auto(
        this: Arc<Self>,
        hh: Arc<Highlighter>,
        // highlighter: &syntect::highlighting::Highlighter<'_>,
        theme: &'static str,
        mut n: usize,
    ) {
        // let old_n = n;
        let th = &hh.ts.themes[theme];
        let highlighter = syntect::highlighting::Highlighter::new(th);
        let mut sections = vec![];
        while n > 0 {
            n -= 1;
            let whole = &this.text;
            let mut inner = this.inner.write().unwrap();
            let Some(line) = whole.get(inner.i..) else {
                return;
            };
            let i = inner.i;
            if let Some(i) = line.find("\n") {
                inner.i += i + 1;
            } else {
                inner.i = whole.len();
                return;
            }
            let range = i..inner.i;
            drop(inner);
            this.highlight_line(hh.as_ref(), &highlighter, range, &mut sections);
        }
        this.sections.push(sections);
        let aaa = this.clone();
        // let future = move || {
        //     // let mut t = that.write().unwrap();

        //     // t.async_aux(hh.as_ref(), &highlighter).await
        //     IncrementalHighlightLayout2::highlight_n(
        //         this.clone(),
        //         hh.clone(),
        //         theme,
        //         (old_n * 2).min(500),
        //     )
        //     // .into_future()
        //     // .await
        // };
        aaa.ctx
            .request_repaint_after(std::time::Duration::from_millis(50));
        // let value = async_exec::spawn_macrotask(Box::new(future));
        // let a = aaa
        //     .inner
        //     .write()
        //     .unwrap()
        //     .macrotask
        //     .replace(Arc::new(Mutex::new(value)));
    }
    fn highlight_n(
        this: Arc<Self>,
        hh: Arc<Highlighter>,
        // highlighter: &syntect::highlighting::Highlighter<'_>,
        theme: &'static str,
        mut n: usize,
    ) {
        let theme = &hh.ts.themes[theme];
        let highlighter = syntect::highlighting::Highlighter::new(theme);
        let mut sections = vec![];
        while n > 0 {
            n -= 1;
            let whole = &this.text;
            let mut inner = this.inner.write().unwrap();
            let Some(line) = whole.get(inner.i..) else {
                return;
            };
            let i = inner.i;
            if let Some(i) = line.find("\n") {
                inner.i += i + 1;
            } else {
                inner.i = whole.len();
                return;
            }
            let range = i..inner.i;
            drop(inner);
            this.highlight_line(hh.as_ref(), &highlighter, range, &mut sections);
        }
        this.sections.push(sections);
    }
    fn highlight_line(
        &self,
        hh: &Highlighter,
        highlighter: &syntect::highlighting::Highlighter<'_>,
        range: Range<usize>,
        sections: &mut Vec<LayoutSection>,
    ) {
        let whole = &self.text;
        let i = range.start;
        let line = &whole[range];
        let mut inner = self.inner.write().unwrap();
        let ops = inner.parse_state.parse_line(line, &hh.ps).unwrap();
        let highlighted = syntect::highlighting::HighlightIterator::new(
            &mut inner.highlight_state,
            &ops[..],
            line,
            &highlighter,
        );
        for (style, range) in highlighted {
            let byte_range = as_byte_range(line, range);
            let byte_range = i + byte_range.start..i + byte_range.end;
            let format = convert_syntect_style(style);
            let section = LayoutSection {
                leading_space: 0.0,
                byte_range,
                format,
            };
            sections.push(section);
        }
    }
}

use egui::text::{LayoutSection, TextFormat};

use super::syntect::CodeTheme;

fn convert_syntect_style(style: syntect::highlighting::Style) -> TextFormat {
    let fg = style.foreground;
    let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
    let italics = style.font_style.contains(FontStyle::ITALIC);
    let underline = style.font_style.contains(FontStyle::ITALIC);
    let underline = if underline {
        egui::Stroke::new(1.0, text_color)
    } else {
        egui::Stroke::NONE
    };
    let format = TextFormat {
        font_id: egui::FontId::monospace(12.0),
        color: text_color,
        italics,
        underline,
        ..Default::default()
    };
    format
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

use crate::async_exec;
pub use async_exec::TimeoutHandle;
