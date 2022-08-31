//! Describes the available SDF operations

/// The operations combining SDFs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SDFOperators {
    /// A hard union between two SDFs
    Union,
    /// A hard subtraction between two SDFs - subtracting R from L
    Subtraction,
    /// A hard intersection between two SDFs
    Intersection,
}

impl SDFOperators {
    /// Process the value from two SDFs using the operator
    pub fn value(&self, left: &f32, right: &f32) -> f32 {
        match self {
            SDFOperators::Union => union(left, right),
            SDFOperators::Subtraction => subtraction(left, right),
            SDFOperators::Intersection => intersection(left, right),
        }
    }
}

fn union(left: &f32, right: &f32) -> f32 {
    left.min(*right)
}

fn subtraction(left: &f32, right: &f32) -> f32 {
    left.max(-1. * right)
}

fn intersection(left: &f32, right: &f32) -> f32 {
    left.max(*right)
}
