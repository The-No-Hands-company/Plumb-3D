use geo_nodes::{nodes::*, GeoNodeGraph};

/// Action triggered by the node creation palette.
pub enum SpawnNodeAction {
    Cube,
    UvSphere,
    Cylinder,
    Plane,
    Torus,
    Transform,
    Subdivide,
    Extrude,
    Boolean,
    Solidify,
    Decimate,
    VoxelRemesh,
    JoinGeometry,
    SetMaterial,
    MeshToPoints,
    SplineCircle,
    SplineSpiral,
    CurveFillet,
    CurveRevolve,
    Math,
    VectorMath,
    CombineXyz,
    SeparateXyz,
    Value,
    Vector,
    DistributePoints,
    InstanceOnPoints,
    NoiseTexture,
    DisplaceGeometry,
    GroupInput,
    GroupOutput,
}

impl SpawnNodeAction {
    pub fn spawn_into(self, graph: &mut GeoNodeGraph, pos: [f32; 2]) {
        match self {
            Self::Cube => {
                graph.add_node(Box::new(CubeNode), pos);
            }
            Self::UvSphere => {
                graph.add_node(Box::new(SphereNode), pos);
            }
            Self::Cylinder => {
                graph.add_node(Box::new(CylinderNode), pos);
            }
            Self::Plane => {
                graph.add_node(Box::new(PlaneNode), pos);
            }
            Self::Torus => {
                graph.add_node(Box::new(TorusNode), pos);
            }
            Self::Transform => {
                graph.add_node(Box::new(TransformGeometryNode), pos);
            }
            Self::Subdivide => {
                graph.add_node(Box::new(SubdivideNode), pos);
            }
            Self::Extrude => {
                graph.add_node(Box::new(ExtrudeNode), pos);
            }
            Self::Boolean => {
                graph.add_node(Box::new(BooleanNode), pos);
            }
            Self::Solidify => {
                graph.add_node(Box::new(SolidifyNode), pos);
            }
            Self::Decimate => {
                graph.add_node(Box::new(DecimateNode), pos);
            }
            Self::VoxelRemesh => {
                graph.add_node(Box::new(VoxelRemeshNode), pos);
            }
            Self::JoinGeometry => {
                graph.add_node(Box::new(JoinGeometryNode), pos);
            }
            Self::SetMaterial => {
                graph.add_node(Box::new(SetMaterialNode), pos);
            }
            Self::MeshToPoints => {
                graph.add_node(Box::new(MeshToPointsNode), pos);
            }
            Self::SplineCircle => {
                graph.add_node(Box::new(SplineCircleNode), pos);
            }
            Self::SplineSpiral => {
                graph.add_node(Box::new(SplineSpiralNode), pos);
            }
            Self::CurveFillet => {
                graph.add_node(Box::new(CurveFilletNode), pos);
            }
            Self::CurveRevolve => {
                graph.add_node(Box::new(CurveRevolveNode), pos);
            }
            Self::Math => {
                graph.add_node(Box::new(MathNode::default()), pos);
            }
            Self::VectorMath => {
                graph.add_node(Box::new(VectorMathNode::default()), pos);
            }
            Self::CombineXyz => {
                graph.add_node(Box::new(CombineXyzNode), pos);
            }
            Self::SeparateXyz => {
                graph.add_node(Box::new(SeparateXyzNode), pos);
            }
            Self::Value => {
                graph.add_node(Box::new(ValueNode::default()), pos);
            }
            Self::Vector => {
                graph.add_node(Box::new(VectorNode::default()), pos);
            }
            Self::DistributePoints => {
                graph.add_node(Box::new(DistributePointsOnFacesNode), pos);
            }
            Self::InstanceOnPoints => {
                graph.add_node(Box::new(InstanceOnPointsNode), pos);
            }
            Self::NoiseTexture => {
                graph.add_node(Box::new(NoiseTextureNode), pos);
            }
            Self::DisplaceGeometry => {
                graph.add_node(Box::new(DisplaceGeometryNode), pos);
            }
            Self::GroupInput => {
                graph.add_node(Box::new(GroupInputNode::default()), pos);
            }
            Self::GroupOutput => {
                graph.add_node(Box::new(GroupOutputNode), pos);
            }
        }
    }
}

/// Renders the Add Node dropdown or context menu.
pub fn show_add_node_menu(ui: &mut egui::Ui) -> Option<SpawnNodeAction> {
    let mut selected = None;

    ui.menu_button("➕ Add Node", |ui| {
        ui.menu_button("🧊 Primitives", |ui| {
            if ui.button("Cube").clicked() {
                selected = Some(SpawnNodeAction::Cube);
                ui.close_menu();
            }
            if ui.button("UV Sphere").clicked() {
                selected = Some(SpawnNodeAction::UvSphere);
                ui.close_menu();
            }
            if ui.button("Cylinder").clicked() {
                selected = Some(SpawnNodeAction::Cylinder);
                ui.close_menu();
            }
            if ui.button("Grid Plane").clicked() {
                selected = Some(SpawnNodeAction::Plane);
                ui.close_menu();
            }
            if ui.button("Torus").clicked() {
                selected = Some(SpawnNodeAction::Torus);
                ui.close_menu();
            }
        });

        ui.menu_button("⚡ Geometry Ops", |ui| {
            if ui.button("Transform Geometry").clicked() {
                selected = Some(SpawnNodeAction::Transform);
                ui.close_menu();
            }
            if ui.button("Subdivide (Catmull-Clark)").clicked() {
                selected = Some(SpawnNodeAction::Subdivide);
                ui.close_menu();
            }
            if ui.button("Extrude Mesh").clicked() {
                selected = Some(SpawnNodeAction::Extrude);
                ui.close_menu();
            }
            if ui.button("Boolean (CSG)").clicked() {
                selected = Some(SpawnNodeAction::Boolean);
                ui.close_menu();
            }
            if ui.button("Solidify").clicked() {
                selected = Some(SpawnNodeAction::Solidify);
                ui.close_menu();
            }
            if ui.button("Decimate").clicked() {
                selected = Some(SpawnNodeAction::Decimate);
                ui.close_menu();
            }
            if ui.button("Voxel Remesh (DynaMesh)").clicked() {
                selected = Some(SpawnNodeAction::VoxelRemesh);
                ui.close_menu();
            }
            if ui.button("Displace Geometry").clicked() {
                selected = Some(SpawnNodeAction::DisplaceGeometry);
                ui.close_menu();
            }
            if ui.button("Join Geometry").clicked() {
                selected = Some(SpawnNodeAction::JoinGeometry);
                ui.close_menu();
            }
            if ui.button("Set Material").clicked() {
                selected = Some(SpawnNodeAction::SetMaterial);
                ui.close_menu();
            }
        });

        ui.menu_button("✨ Points & Instances", |ui| {
            if ui.button("Mesh to Points").clicked() {
                selected = Some(SpawnNodeAction::MeshToPoints);
                ui.close_menu();
            }
            if ui.button("Distribute Points on Faces").clicked() {
                selected = Some(SpawnNodeAction::DistributePoints);
                ui.close_menu();
            }
            if ui.button("Instance on Points").clicked() {
                selected = Some(SpawnNodeAction::InstanceOnPoints);
                ui.close_menu();
            }
        });

        ui.menu_button("➰ Curves", |ui| {
            if ui.button("Spline Circle").clicked() {
                selected = Some(SpawnNodeAction::SplineCircle);
                ui.close_menu();
            }
            if ui.button("Spline Spiral").clicked() {
                selected = Some(SpawnNodeAction::SplineSpiral);
                ui.close_menu();
            }
            if ui.button("Fillet Curve").clicked() {
                selected = Some(SpawnNodeAction::CurveFillet);
                ui.close_menu();
            }
            if ui.button("Revolve Spline (Lathe)").clicked() {
                selected = Some(SpawnNodeAction::CurveRevolve);
                ui.close_menu();
            }
        });

        ui.menu_button("📐 Math & Procedural", |ui| {
            if ui.button("Math").clicked() {
                selected = Some(SpawnNodeAction::Math);
                ui.close_menu();
            }
            if ui.button("Vector Math").clicked() {
                selected = Some(SpawnNodeAction::VectorMath);
                ui.close_menu();
            }
            if ui.button("Combine XYZ").clicked() {
                selected = Some(SpawnNodeAction::CombineXyz);
                ui.close_menu();
            }
            if ui.button("Separate XYZ").clicked() {
                selected = Some(SpawnNodeAction::SeparateXyz);
                ui.close_menu();
            }
            if ui.button("Noise Texture (3D)").clicked() {
                selected = Some(SpawnNodeAction::NoiseTexture);
                ui.close_menu();
            }
        });

        ui.menu_button("🎛 Inputs & Output", |ui| {
            if ui.button("Group Input").clicked() {
                selected = Some(SpawnNodeAction::GroupInput);
                ui.close_menu();
            }
            if ui.button("Value (Float)").clicked() {
                selected = Some(SpawnNodeAction::Value);
                ui.close_menu();
            }
            if ui.button("Vector").clicked() {
                selected = Some(SpawnNodeAction::Vector);
                ui.close_menu();
            }
            if ui.button("Group Output").clicked() {
                selected = Some(SpawnNodeAction::GroupOutput);
                ui.close_menu();
            }
        });
    });

    selected
}
