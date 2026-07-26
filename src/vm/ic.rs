/// Monomorphic and Polymorphic Inline Cache (IC) infrastructure for fast dynamic method & field access.
/// Inspired by Self, Strongtalk, and V8 inline caching techniques.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IcState {
    Uninitialized,
    Monomorphic { class_name: String, offset: usize },
    Polymorphic { slots: Vec<(String, usize)> },
    Megamorphic,
}

#[derive(Debug, Clone)]
pub struct InlineCache {
    pub slot: usize,
    pub state: IcState,
}

impl InlineCache {
    pub fn new(slot: usize) -> Self {
        InlineCache {
            slot,
            state: IcState::Uninitialized,
        }
    }

    pub fn lookup(&self, class_name: &str) -> Option<usize> {
        match &self.state {
            IcState::Monomorphic {
                class_name: cached_name,
                offset,
            } => {
                if cached_name == class_name {
                    Some(*offset)
                } else {
                    None
                }
            }
            IcState::Polymorphic { slots } => {
                for (name, offset) in slots {
                    if name == class_name {
                        return Some(*offset);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn update(&mut self, class_name: String, offset: usize) {
        match &mut self.state {
            IcState::Uninitialized => {
                self.state = IcState::Monomorphic { class_name, offset };
            }
            IcState::Monomorphic {
                class_name: prev_name,
                offset: prev_offset,
            } => {
                if prev_name != &class_name {
                    self.state = IcState::Polymorphic {
                        slots: vec![(prev_name.clone(), *prev_offset), (class_name, offset)],
                    };
                }
            }
            IcState::Polymorphic { slots } => {
                if slots.len() < 4 {
                    slots.push((class_name, offset));
                } else {
                    self.state = IcState::Megamorphic;
                }
            }
            IcState::Megamorphic => {}
        }
    }
}
