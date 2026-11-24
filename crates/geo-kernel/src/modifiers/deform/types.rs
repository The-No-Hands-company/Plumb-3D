/// Deformation axis along which a curve or spline is tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeformAxis {
    X,
    #[default]
    Y,
    Z,
}

/// Method for projecting vertices onto target surface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShrinkwrapMethod {
    #[default]
    NearestSurfacePoint,
    ProjectNormal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleDeformMethod {
    Twist,
    Bend,
    Taper,
    Stretch,
}
