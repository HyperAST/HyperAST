use std::collections::HashSet;

pub struct NodeStoreStats {
    pub(crate) height_counts: Vec<u32>,
    pub(crate) height_counts_non_dedup: Vec<u32>,
    pub(crate) height_counts_structural: Vec<u32>,
    pub(crate) structurals: HashSet<u32>,
    pub(crate) height_counts_label: Vec<u32>,
    pub(crate) labels: HashSet<u32>,
}

impl Default for NodeStoreStats {
    fn default() -> Self {
        Self {
            height_counts: Vec::with_capacity(100),
            height_counts_non_dedup: Vec::with_capacity(100),
            height_counts_structural: Vec::with_capacity(100),
            structurals: HashSet::with_capacity(100),
            height_counts_label: Vec::with_capacity(100),
            labels: HashSet::with_capacity(100),
        }
    }
}

impl std::fmt::Debug for NodeStoreStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = f.debug_struct("NodeStoreStats");
        fn lim<T>(v: &[T]) -> &[T] {
            &v[..v.len().min(30)]
        }
        r.field("height_counts", &lim(&self.height_counts_structural));
        r.field("height_counts", &lim(&self.height_counts_label));
        r.field("height_counts", &lim(&self.height_counts));
        r.field(
            "height_counts_non_dedup",
            &lim(&self.height_counts_non_dedup),
        );

        r.finish()
    }
}

impl NodeStoreStats {
    pub fn add_height_non_dedup(&mut self, height: u32) {
        Self::accumulate_height(&mut self.height_counts_non_dedup, height);
    }

    pub fn add_height_dedup(&mut self, height: u32, hashs: crate::hashed::SyntaxNodeHashs<u32>) {
        self.add_height(height);
        self.add_height_label(height, hashs.label);
        self.add_height_structural(height, hashs.structt);
    }

    pub(crate) fn add_height(&mut self, height: u32) {
        Self::accumulate_height(&mut self.height_counts, height);
    }

    pub(crate) fn add_height_structural(&mut self, height: u32, hash: u32) {
        if Self::not_there(&mut self.structurals, hash) {
            Self::accumulate_height(&mut self.height_counts_structural, height);
        }
    }

    pub(crate) fn add_height_label(&mut self, height: u32, hash: u32) {
        if Self::not_there(&mut self.labels, hash) {
            Self::accumulate_height(&mut self.height_counts_label, height);
        }
    }

    pub(crate) fn not_there(hash_set: &mut HashSet<u32>, hash: u32) -> bool {
        if hash_set.contains(&hash) {
            return false;
        }
        hash_set.insert(hash);
        true
    }

    pub(crate) fn accumulate_height(counts: &mut Vec<u32>, height: u32) {
        if counts.len() <= height as usize {
            counts.resize(height as usize + 1, 0);
        }
        counts[height as usize] += 1;
    }
}
