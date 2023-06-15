use std::{
    future::{IntoFuture},
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

/// Memoized Code highlighting
pub fn highlight0(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob {
            self.highlight(theme, code, lang)
        }
    }

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
        fn spawn(&self, ctx: &egui::Context, (theme, code, lang): (&CodeTheme, &str, &str)) -> Incremental {
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
    impl cache::IncrementalComputer<IncrementalSpawner, (&CodeTheme, &str, &str), LayoutJob> for Incremental {
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

        let mut hh = hh.clone();
        let mut that = this.clone();

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
pub(crate) mod cache {
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

#[derive(Clone, Copy, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum SyntectTheme {
    Base16EightiesDark,
    Base16MochaDark,
    Base16OceanDark,
    Base16OceanLight,
    InspiredGitHub,
    SolarizedDark,
    SolarizedLight,
}

impl SyntectTheme {
    fn all() -> impl ExactSizeIterator<Item = Self> {
        [
            Self::Base16EightiesDark,
            Self::Base16MochaDark,
            Self::Base16OceanDark,
            Self::Base16OceanLight,
            Self::InspiredGitHub,
            Self::SolarizedDark,
            Self::SolarizedLight,
        ]
        .iter()
        .copied()
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "Base16 Eighties (dark)",
            Self::Base16MochaDark => "Base16 Mocha (dark)",
            Self::Base16OceanDark => "Base16 Ocean (dark)",
            Self::Base16OceanLight => "Base16 Ocean (light)",
            Self::InspiredGitHub => "InspiredGitHub (light)",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    fn syntect_key_name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "base16-eighties.dark",
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
            Self::InspiredGitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            Self::Base16EightiesDark
            | Self::Base16MochaDark
            | Self::Base16OceanDark
            | Self::SolarizedDark => true,

            Self::Base16OceanLight | Self::InspiredGitHub | Self::SolarizedLight => false,
        }
    }
}

#[derive(Clone, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeTheme {
    dark_mode: bool,

    syntect_theme: SyntectTheme,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn from_style(style: &egui::Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }

    pub fn from_memory(ctx: &egui::Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.data_mut(|d| {
                d.get_persisted(egui::Id::new("dark"))
                    .unwrap_or_else(CodeTheme::dark)
            })
        } else {
            ctx.data_mut(|d| {
                d.get_persisted(egui::Id::new("light"))
                    .unwrap_or_else(CodeTheme::light)
            })
        }
    }

    pub fn store_in_memory(self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.data_mut(|d| d.insert_persisted(egui::Id::new("dark"), self));
        } else {
            ctx.data_mut(|d| d.insert_persisted(egui::Id::new("light"), self));
        }
    }
}

impl CodeTheme {
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            syntect_theme: SyntectTheme::Base16MochaDark,
        }
    }

    pub fn light() -> Self {
        Self {
            dark_mode: false,
            syntect_theme: SyntectTheme::SolarizedLight,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_buttons(ui);

        for theme in SyntectTheme::all() {
            if theme.is_dark() == self.dark_mode {
                ui.radio_value(&mut self.syntect_theme, theme, theme.name());
            }
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

use lazy_static::__Deref;
use poll_promise::Promise;
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
        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };
        let lines = LinesWithEndings::from(text);
        (job, lines)
    }

    fn init_h<'a>(&'a self, language: &str, theme: &CodeTheme) -> Option<HighlightLines<'_>> {
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
        let theme = theme;
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

struct IncrementalHighlightLayout {
    highlight_state: syntect::highlighting::HighlightState,
    parse_state: syntect::parsing::ParseState,
    text: String,
    sections: Vec<LayoutSection>,
    i: usize,
}

impl IncrementalHighlightLayout {
    fn inc(
        &mut self,
        hh: &Highlighter,
        highlighter: &syntect::highlighting::Highlighter<'_>,
        whole: &str,
        mut n: usize,
    ) -> Vec<LayoutSection> {
        debug_assert_eq!(&self.text, whole);
        while n > 0 {
            n -= 1;
            let Some(line) = whole.get(self.i..) else {
                return self.sections.clone()
            };
            let i = self.i;
            if let Some(i) = line.find("\n") {
                self.i += i + 1;
            } else {
                self.i = whole.len();
                return self.sections.clone();
            }
            let line = &whole[i..self.i];
            wasm_rs_dbg::dbg!(line);
            let ops = self.parse_state.parse_line(line, &hh.ps).unwrap();
            let highlighted = syntect::highlighting::HighlightIterator::new(
                &mut self.highlight_state,
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
                self.sections.push(section);
            }
        }
        return self.sections.clone();
    }
    async fn async_aux2(this: Arc<RwLock<Self>>, hh: Arc<Highlighter>, theme: &'static str) {
        // ) -> LayoutJob {
        loop {
            let this = this.clone();
            let hh = hh.clone();
            let fut = async move {
                let theme = &hh.ts.themes[theme];
                let highlighter = syntect::highlighting::Highlighter::new(theme);
                let mut this = this.write().unwrap();
                let whole = &this.text;
                let Some(line) = whole.get(this.i..) else {
                    return true
                };
                let i = this.i;
                if let Some(i) = line.find("\n") {
                    this.i += i + 1;
                } else {
                    this.i = whole.len();
                    return true;
                }
                this.async_aux_aux(&hh.clone(), &highlighter, i);
                false
            };
            async_exec::hello();
            if fut.into_future().await {
                break;
            };
        }
        // return self.job.clone();
    }
    async fn async_aux(
        &mut self,
        hh: &Highlighter,
        highlighter: &syntect::highlighting::Highlighter<'_>,
    ) {
        loop {
            let whole = &self.text;
            let Some(line) = whole.get(self.i..) else {
                    break
                };
            let i = self.i;
            if let Some(i) = line.find("\n") {
                self.i += i + 1;
            } else {
                self.i = whole.len();
                break;
            }
            self.async_aux_aux(hh, highlighter, i);
            async_exec::nop_await();
        }
    }

    fn async_aux_aux(
        &mut self,
        hh: &Highlighter,
        highlighter: &syntect::highlighting::Highlighter<'_>,
        i: usize,
    ) {
        let whole = &self.text;
        let line = &whole[i..self.i];
        wasm_rs_dbg::dbg!(line);
        let ops = self.parse_state.parse_line(line, &hh.ps).unwrap();
        let highlighted = syntect::highlighting::HighlightIterator::new(
            &mut self.highlight_state,
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
            self.sections.push(section);
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
        let old_n = n;
        let th = &hh.ts.themes[theme];
        let highlighter = syntect::highlighting::Highlighter::new(th);
        let mut sections = vec![];
        while n > 0 {
            n -= 1;
            let whole = &this.text;
            let mut inner = this.inner.write().unwrap();
            let Some(line) = whole.get(inner.i..) else {
                    return
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
        let future = move || {
            // let mut t = that.write().unwrap();

            // t.async_aux(hh.as_ref(), &highlighter).await
            IncrementalHighlightLayout2::highlight_n(this.clone(), hh.clone(), theme, (old_n * 2).min(500))
            // .into_future()
            // .await
        };
        aaa.ctx.request_repaint_after(std::time::Duration::from_millis(50));
        let value = async_exec::spawn_macrotask(Box::new(future));
        let a = aaa
            .inner
            .write()
            .unwrap()
            .macrotask
            .replace(Arc::new(Mutex::new(value)));
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
                    return
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
        let i: usize = range.start;
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

use crate::syntax_highlighting_async::async_exec::TimeoutHandle;

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

#[cfg(target_arch = "wasm32")]
pub(crate) mod async_exec {
    use js_sys::Function;
    use std::sync::{Arc, Mutex};
    use wasm_bindgen::prelude::*;

    #[derive(Debug)]
    pub enum Error {
        // #[error("JsValue {0:?}")]
        JsValue(JsValue),

        // #[error("Invalid interval handle")]
        InvalidIntervalHandle,

        // #[error("Invalid timeout handle")]
        InvalidTimeoutHandle,
    }

    impl From<JsValue> for Error {
        fn from(value: JsValue) -> Self {
            Error::JsValue(value)
        }
    }

    pub mod native {
        use super::*;

        #[wasm_bindgen]
        extern "C" {
            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`setInterval()`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval) function
            #[wasm_bindgen (catch, js_name = setInterval)]
            pub fn set_interval(
                closure: &Function,
                timeout: u32,
            ) -> std::result::Result<u32, JsValue>;

            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`clearInterval()`](https://developer.mozilla.org/en-US/docs/Web/API/clearInterval) function
            #[wasm_bindgen (catch, js_name = clearInterval)]
            pub fn clear_interval(interval: u32) -> std::result::Result<(), JsValue>;

            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`setTimeout()`](https://developer.mozilla.org/en-US/docs/Web/API/setTimeout) function
            #[wasm_bindgen (catch, js_name = setTimeout)]
            pub fn set_timeout(
                closure: &Function,
                timeout: u32,
            ) -> std::result::Result<u32, JsValue>;

            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`clearTimeout()`](https://developer.mozilla.org/en-US/docs/Web/API/clearTimeout) function
            #[wasm_bindgen (catch, js_name = clearTimeout)]
            pub fn clear_timeout(interval: u32) -> std::result::Result<(), JsValue>;

            #[wasm_bindgen(js_namespace = console)]
            pub fn log(s: &str);
        }
    }

    /// JavaScript interval handle dropping which stops and clears the associated interval
    #[derive(Clone, Debug)]
    pub struct IntervalHandle(Arc<Mutex<u32>>);

    impl Drop for IntervalHandle {
        fn drop(&mut self) {
            let handle = self.0.lock().unwrap();
            if *handle != 0 {
                native::clear_interval(*handle).expect("Unable to clear interval");
            }
        }
    }

    /// JavaScript timeout handle, droppping which cancels the associated timeout.
    #[derive(Clone)]
    pub struct TimeoutHandle0(Arc<Mutex<u32>>);

    impl Drop for TimeoutHandle0 {
        fn drop(&mut self) {
            let handle = self.0.lock().unwrap();
            if *handle != 0 {
                native::clear_timeout(*handle).expect("Unable to clear timeout");
            }
        }
    }

    /// Create JavaScript interval
    pub fn set_interval(
        closure: &Closure<dyn FnMut()>,
        timeout: u32,
    ) -> Result<IntervalHandle, Error> {
        let handle = native::set_interval(closure.as_ref().unchecked_ref(), timeout)?;
        Ok(IntervalHandle(Arc::new(Mutex::new(handle))))
    }

    /// Clear JavaScript interval using a handle returned by [`set_interval`]
    pub fn clear_interval(handle: &IntervalHandle) -> Result<(), Error> {
        let mut handle = handle.0.lock().unwrap();
        if *handle != 0 {
            native::clear_interval(*handle)?;
            *handle = 0;
            Ok(())
        } else {
            Err(Error::InvalidIntervalHandle)
        }
    }

    /// Create JavaScript timeout
    pub fn set_timeout(
        closure: &Closure<dyn FnMut()>,
        timeout: u32,
    ) -> Result<TimeoutHandle0, Error> {
        let handle = native::set_timeout(closure.as_ref().unchecked_ref(), timeout)?;
        Ok(TimeoutHandle0(Arc::new(Mutex::new(handle))))
    }

    /// Clear JavaScript timeout using a handle returns by [`set_timeout`]
    pub fn clear_timeout(handle: &TimeoutHandle0) -> Result<(), Error> {
        let mut handle = handle.0.lock().unwrap();
        if *handle != 0 {
            native::clear_timeout(*handle)?;
            *handle = 0;
            Ok(())
        } else {
            Err(Error::InvalidTimeoutHandle)
        }
    }

    // Keep logging "hello" every second until the resulting `Interval` is dropped.
    pub fn hello() -> IntervalHandle {
        native::log("hello0");
        let aa = Closure::new(|| {
            native::log("hello");
        });
        set_interval(&aa, 1).unwrap()
    }
    pub fn nop_await() -> TimeoutHandle0 {
        let aa = Closure::new(|| {});
        set_timeout(&aa, 4).unwrap()
    }

    pub struct TimeoutHandle(TimeoutHandle0, Closure<dyn FnMut()>);
    unsafe impl Send for TimeoutHandle {}

    pub(crate) fn spawn_macrotask(mut f: Box<dyn FnMut() + 'static>) -> TimeoutHandle {
        let aa = Closure::new(move || f());
        TimeoutHandle(set_timeout(&aa, 4).unwrap(), aa)
        // TimeoutHandle(Arc::new(Timeout::new(4, move || {
        //     f()
        // })))
    }
    // pub fn spawn_stuff<T: Send + 'static,F>(f:F)
    // where
    //     F: FnOnce() -> T + Send + 'static, {
    //     use poll_promise::Promise;
    //     let promise = Promise::spawn_async(f);
    // }

    use poll_promise::Promise;
    pub(crate) fn spawn_stuff<T: Send + 'static>(
        f: impl std::future::Future<Output = T> + 'static,
    ) -> poll_promise::Promise<T> {
        Promise::spawn_async(f)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod async_exec {

    pub struct IntervalHandle {}

    pub fn hello() -> IntervalHandle {
        IntervalHandle {}
    }
    pub struct TimeoutHandle {}
    pub fn nop_await() -> TimeoutHandle {
        TimeoutHandle {}
        // let aa = Closure::new(|| {});
        // set_timeout(&aa, 4).unwrap()
    }
    pub(crate) fn spawn_macrotask(mut _f: Box<dyn FnMut() + 'static>) -> TimeoutHandle {
        todo!()
    }

    // pub fn spawn_stuff<T: Send + 'static,F>(f:F)
    // where
    //     F: FnOnce() -> T + Send + 'static, {
    //     use poll_promise::Promise;
    //     let promise = Promise::spawn_thread("aaa", f);
    // }

    use poll_promise::Promise;
    pub(crate) fn spawn_stuff<T: Send + 'static>(
        f: impl std::future::Future<Output = T> + 'static,
    ) -> poll_promise::Promise<T> {
        // Promise::spawn_async(f)
        todo!()
    }
}
