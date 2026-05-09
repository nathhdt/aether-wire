//! directional modes for TCP benchmark

pub use crate::protocol::messages::Direction;

impl Direction {
    /// human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Direction::Default => "default",
            Direction::Reverse => "reverse",
            Direction::Both => "both directions sequentially",
            Direction::Bidirectional => "both directions simultaneously",
        }
    }
}

// TODO: implement download, both, and bidirectional execution logic
