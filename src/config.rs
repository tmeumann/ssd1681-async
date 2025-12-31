pub struct Ssd1681Config<const X: usize, const Y: usize> {
    pub busy_settle_ms: u32,
    pub dc_settle_us: u32,
}

impl<const X: usize, const Y: usize> Default for Ssd1681Config<X, Y> {
    fn default() -> Self {
        Self {
            busy_settle_ms: 20,
            dc_settle_us: 10,
        }
    }
}
