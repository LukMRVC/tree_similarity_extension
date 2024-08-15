pub struct NodeId(std::num::NonZeroUsize);

impl NodeId {
    pub fn index(&self) -> usize {
        self.0.get() - 1
    }
}
