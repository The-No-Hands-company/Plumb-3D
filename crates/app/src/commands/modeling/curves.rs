use geo_kernel::{EdgeKey, ModifierStack, ProfileShape, ShadingMode, Spline};
use graph_engine::{Command, Material, NodeData, NodeId, SceneGraph, SceneNode};

#[derive(Debug)]
pub struct ConvertCurveToMeshCommand {
    pub node_id: NodeId,
    pub old_spline: Option<Spline>,
    pub old_bevel_radius: f32,
    pub old_bevel_resolution: usize,
    pub old_profile_shape: ProfileShape,
}


impl Command for ConvertCurveToMeshCommand {
    fn name(&self) -> &str {
        "Convert Curve to Mesh"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        let (mesh, spline, r, res, shape) = match &node.data {
            NodeData::Curve { spline, bevel_radius, bevel_resolution, profile_shape, .. } => {
                let r = (*bevel_radius).max(0.01);
                let res = (*bevel_resolution).max(3);
                let cs = profile_shape.generate_cross_section(r, res);
                let sampled = spline.sample(12);
                let mesh = Spline::sweep(&cs, &sampled, spline.is_closed)
                    .map_err(|e| format!("Failed to sweep curve: {:?}", e))?;
                (mesh, spline.clone(), *bevel_radius, *bevel_resolution, *profile_shape)
            }
            _ => return Err("Node is not a curve".into()),
        };

        self.old_spline = Some(spline);
        self.old_bevel_radius = r;
        self.old_bevel_resolution = res;
        self.old_profile_shape = shape;

        node.data = NodeData::Mesh {
            base_mesh: mesh,
            modifier_stack: ModifierStack::default(),
            shading_mode: ShadingMode::Smooth,
            cached_render_mesh: None,
            materials: vec![Material::default()],
        };
        scene.mark_dirty(self.node_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(spline) = self.old_spline.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            node.data = NodeData::Curve {
                spline,
                bevel_radius: self.old_bevel_radius,
                bevel_resolution: self.old_bevel_resolution,
                cached_render_mesh: None,
                profile_shape: self.old_profile_shape,
            };
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("No old curve data to restore".into())
        }
    }
}


#[derive(Debug)]
pub struct ExtractCurveFromEdgesCommand {
    pub source_node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub created_node_id: Option<NodeId>,
}


impl Command for ExtractCurveFromEdgesCommand {
    fn name(&self) -> &str {
        "Extract Curve from Edges"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (spline, transform, name) = {
            let node = scene.nodes.get(self.source_node_id).ok_or("Source node not found")?;
            match &node.data {
                NodeData::Mesh { base_mesh, .. } => {
                    let spline = base_mesh.extract_curve_from_edges(&self.edges)
                        .map_err(|e| format!("Extract curve failed: {:?}", e))?;
                    (spline, node.local_transform, node.name.clone())
                }
                _ => return Err("Source node is not a mesh".into()),
            }
        };

        let new_name = format!("{}_curve", name);
        let curve_node = SceneNode::new_curve(new_name, transform, spline);
        let id = scene.add_node(curve_node);
        self.created_node_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node_id.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}

/// Command to create a new Bézier curve node in the scene graph (Curve Pen Tool).
#[derive(Debug)]
pub struct CreateCurveNodeCommand {
    pub name: String,
    pub spline: Spline,
    pub created_node_id: Option<NodeId>,
}

impl Command for CreateCurveNodeCommand {
    fn name(&self) -> &str {
        "Draw Bézier Curve (Pen Tool)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = SceneNode::new_curve(self.name.clone(), core_math::Transform::IDENTITY, self.spline.clone());
        let id = scene.add_node(node);
        self.created_node_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node_id.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoftCurvesCommand {
    pub curve_node_ids: Vec<NodeId>,
    pub samples_per_curve: usize,
    pub closed_u: bool,
    pub closed_v: bool,
    pub created_node_id: Option<NodeId>,
}

impl Command for LoftCurvesCommand {
    fn name(&self) -> &str {
        "Loft Curves (Ctrl+L)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if self.curve_node_ids.len() < 2 {
            return Err("Loft requires at least 2 curve profiles".into());
        }

        let samples = self.samples_per_curve.max(4);
        let mut profile_loops = Vec::with_capacity(self.curve_node_ids.len());

        for &nid in &self.curve_node_ids {
            let node = scene.nodes.get(nid).ok_or("Curve node not found")?;
            match &node.data {
                NodeData::Curve { spline, .. } => {
                    let sampled = spline.sample(samples);
                    let world_pts: Vec<core_math::Vec3> = sampled
                        .iter()
                        .map(|&p| node.world_matrix.transform_point3(p))
                        .collect();
                    profile_loops.push(world_pts);
                }
                _ => return Err(format!("Node {:?} is not a curve", nid)),
            }
        }

        let loft_mesh = Spline::loft(&profile_loops, self.closed_u, self.closed_v)
            .map_err(|e| format!("Loft failed: {:?}", e))?;

        let loft_node = SceneNode::new_mesh(
            "Loft_Surface",
            core_math::Transform::IDENTITY,
            loft_mesh,
        );
        let new_id = scene.add_node(loft_node);
        self.created_node_id = Some(new_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node_id.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}

/// CAD Fillet curve corners command.
#[derive(Debug)]
pub struct FilletCurveCommand {
    pub node_id: NodeId,
    pub radius: f32,
    pub selected_knots: Option<Vec<usize>>,
    pub old_spline: Option<Spline>,
}

impl FilletCurveCommand {
    pub fn all(node_id: NodeId, radius: f32) -> Self {
        Self {
            node_id,
            radius,
            selected_knots: None,
            old_spline: None,
        }
    }

    pub fn selected(node_id: NodeId, radius: f32, selected_knots: Vec<usize>) -> Self {
        Self {
            node_id,
            radius,
            selected_knots: Some(selected_knots),
            old_spline: None,
        }
    }
}

impl Command for FilletCurveCommand {
    fn name(&self) -> &str {
        "Fillet Curve Corners"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Curve node not found")?;
        match &mut node.data {
            NodeData::Curve { spline, cached_render_mesh, .. } => {
                self.old_spline = Some(spline.clone());
                if let Some(knots) = &self.selected_knots {
                    *spline = spline.fillet_selected_corners(knots, self.radius);
                } else {
                    *spline = spline.fillet_all_corners(self.radius);
                }
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a curve".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_spline.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Curve node not found")?;
            match &mut node.data {
                NodeData::Curve { spline, cached_render_mesh, .. } => {
                    *spline = old;
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a curve".into()),
            }
        } else {
            Err("No previous spline data to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct ChamferCurveCommand {
    pub node_id: NodeId,
    pub distance: f32,
    pub selected_knots: Vec<usize>,
    pub old_spline: Option<Spline>,
}

impl ChamferCurveCommand {
    pub fn new(node_id: NodeId, distance: f32, selected_knots: Vec<usize>) -> Self {
        Self {
            node_id,
            distance,
            selected_knots,
            old_spline: None,
        }
    }
}

impl Command for ChamferCurveCommand {
    fn name(&self) -> &str {
        "Chamfer Curve Corners"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Curve node not found")?;
        match &mut node.data {
            NodeData::Curve { spline, cached_render_mesh, .. } => {
                self.old_spline = Some(spline.clone());
                *spline = spline.chamfer_selected_corners(&self.selected_knots, self.distance);
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a curve".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_spline.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Curve node not found")?;
            match &mut node.data {
                NodeData::Curve { spline, cached_render_mesh, .. } => {
                    *spline = old;
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a curve".into()),
            }
        } else {
            Err("No previous spline data to restore".into())
        }
    }
}

/// Sets Bézier knot handle constraint type (Free, Aligned, Vector, Auto).
#[derive(Debug)]
pub struct SetKnotHandleTypeCommand {
    pub node_id: NodeId,
    pub knot_idx: usize,
    pub handle_type: geo_kernel::HandleType,
    pub old_spline: Option<Spline>,
}

impl Command for SetKnotHandleTypeCommand {
    fn name(&self) -> &str {
        "Set Curve Knot Handle Type"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Curve node not found")?;
        match &mut node.data {
            NodeData::Curve { spline, cached_render_mesh, .. } => {
                if self.old_spline.is_none() {
                    self.old_spline = Some(spline.clone());
                }
                spline.set_handle_type(self.knot_idx, self.handle_type);
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a curve".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_spline.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Curve node not found")?;
            match &mut node.data {
                NodeData::Curve { spline, cached_render_mesh, .. } => {
                    *spline = old;
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a curve".into()),
            }
        } else {
            Err("No previous spline data to restore".into())
        }
    }
}

/// Revolve / Lathe curve profile around Y-axis command.
#[derive(Debug)]
pub struct LatheCurveCommand {
    pub curve_node_id: NodeId,
    pub segments: u32,
    pub angle_degrees: f32,
    pub created_node_id: Option<NodeId>,
}

impl Command for LatheCurveCommand {
    fn name(&self) -> &str {
        "Revolve / Lathe Curve"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (spline, transform, name) = {
            let node = scene.nodes.get(self.curve_node_id).ok_or("Curve node not found")?;
            match &node.data {
                NodeData::Curve { spline, .. } => (spline.clone(), node.local_transform, node.name.clone()),
                _ => return Err("Node is not a curve".into()),
            }
        };

        let angle_rad = self.angle_degrees.to_radians();
        let lathe_mesh = spline
            .lathe_spline(12, self.segments, angle_rad)
            .map_err(|e| format!("Lathe failed: {:?}", e))?;

        let new_name = format!("{}_revolve", name);
        let mesh_node = SceneNode::new_mesh(new_name, transform, lathe_mesh);
        let new_id = scene.add_node(mesh_node);
        self.created_node_id = Some(new_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node_id.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}

/// Trims a curve with another cutting curve (CAD Multi-Segment Boolean Trimming).
#[derive(Debug)]
pub struct TrimCurveCommand {
    pub target_curve_id: NodeId,
    pub cutter_curve_id: NodeId,
    pub keep_inside: bool,
    pub tolerance: f32,
    pub old_spline: Option<Spline>,
    pub created_node_ids: Vec<NodeId>,
}

impl TrimCurveCommand {
    pub fn new(target_curve_id: NodeId, cutter_curve_id: NodeId, keep_inside: bool, tolerance: f32) -> Self {
        Self {
            target_curve_id,
            cutter_curve_id,
            keep_inside,
            tolerance,
            old_spline: None,
            created_node_ids: Vec::new(),
        }
    }
}

impl Command for TrimCurveCommand {
    fn name(&self) -> &str {
        if self.keep_inside {
            "Trim Curve (Keep Inside)"
        } else {
            "Trim Curve (Keep Outside)"
        }
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (target_spline, transform, name) = {
            let node = scene.nodes.get(self.target_curve_id).ok_or("Target curve node not found")?;
            match &node.data {
                NodeData::Curve { spline, .. } => (spline.clone(), node.local_transform, node.name.clone()),
                _ => return Err("Target node is not a curve".into()),
            }
        };

        let cutter_spline = {
            let node = scene.nodes.get(self.cutter_curve_id).ok_or("Cutter curve node not found")?;
            match &node.data {
                NodeData::Curve { spline, .. } => spline.clone(),
                _ => return Err("Cutter node is not a curve".into()),
            }
        };

        let trimmed_sub_splines = target_spline.trim_with_spline(&cutter_spline, self.keep_inside, self.tolerance);
        if trimmed_sub_splines.is_empty() {
            return Err("Trimming resulted in an empty curve".into());
        }

        self.old_spline = Some(target_spline);
        self.created_node_ids.clear();

        // 1. Replace target curve with first trimmed piece
        if let Some(node) = scene.nodes.get_mut(self.target_curve_id) {
            if let NodeData::Curve { spline, cached_render_mesh, .. } = &mut node.data {
                *spline = trimmed_sub_splines[0].clone();
                *cached_render_mesh = None;
            }
            scene.mark_dirty(self.target_curve_id);
        }

        // 2. If multiple pieces resulted, spawn remaining pieces as sibling curve nodes
        for (i, sub_spline) in trimmed_sub_splines.iter().enumerate().skip(1) {
            let sub_name = format!("{}_trim_{}", name, i + 1);
            let sub_node = SceneNode::new_curve(sub_name, transform, sub_spline.clone());
            let sub_id = scene.add_node(sub_node);
            self.created_node_ids.push(sub_id);
        }

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        for id in self.created_node_ids.drain(..) {
            scene.remove_node(id);
        }

        if let Some(old) = self.old_spline.take() {
            if let Some(node) = scene.nodes.get_mut(self.target_curve_id) {
                if let NodeData::Curve { spline, cached_render_mesh, .. } = &mut node.data {
                    *spline = old;
                    *cached_render_mesh = None;
                }
                scene.mark_dirty(self.target_curve_id);
            }
        }

        Ok(())
    }
}

/// Connects two curve endpoints or knots with a G1 circular tangent arc (Plasticity Tangent Arc).
#[derive(Debug)]
pub struct ConnectTangentArcCommand {
    pub curve_a_id: NodeId,
    pub knot_a: usize,
    pub curve_b_id: NodeId,
    pub knot_b: usize,
    pub created_arc_node_id: Option<NodeId>,
}

impl ConnectTangentArcCommand {
    pub fn new(curve_a_id: NodeId, knot_a: usize, curve_b_id: NodeId, knot_b: usize) -> Self {
        Self {
            curve_a_id,
            knot_a,
            curve_b_id,
            knot_b,
            created_arc_node_id: None,
        }
    }
}

impl Command for ConnectTangentArcCommand {
    fn name(&self) -> &str {
        "Connect Tangent Arc"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (pt_a, dir_a, transform_a) = {
            let node = scene.nodes.get(self.curve_a_id).ok_or("Curve A not found")?;
            match &node.data {
                NodeData::Curve { spline, .. } => {
                    let k = self.knot_a.min(spline.points.len().saturating_sub(1));
                    let pt = spline.points[k].position;
                    let dir = if spline.points[k].handle_out.length_squared() > 1e-4 {
                        spline.points[k].handle_out.normalize()
                    } else if k > 0 {
                        (pt - spline.points[k - 1].position).normalize_or_zero()
                    } else {
                        core_math::Vec3::X
                    };
                    (pt, dir, node.local_transform)
                }
                _ => return Err("Node A is not a curve".into()),
            }
        };

        let (pt_b, dir_b) = {
            let node = scene.nodes.get(self.curve_b_id).ok_or("Curve B not found")?;
            match &node.data {
                NodeData::Curve { spline, .. } => {
                    let k = self.knot_b.min(spline.points.len().saturating_sub(1));
                    let pt = spline.points[k].position;
                    let dir = if spline.points[k].handle_in.length_squared() > 1e-4 {
                        spline.points[k].handle_in.normalize()
                    } else if k + 1 < spline.points.len() {
                        (spline.points[k + 1].position - pt).normalize_or_zero()
                    } else {
                        core_math::Vec3::Z
                    };
                    (pt, dir)
                }
                _ => return Err("Node B is not a curve".into()),
            }
        };

        let arc = Spline::compute_tangent_arc(pt_a, dir_a, pt_b, dir_b)
            .ok_or("Could not compute valid tangent arc between points")?;

        let arc_spline = arc.to_spline();
        let arc_node = SceneNode::new_curve("TangentArc", transform_a, arc_spline);
        let new_id = scene.add_node(arc_node);
        self.created_arc_node_id = Some(new_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_arc_node_id.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}





