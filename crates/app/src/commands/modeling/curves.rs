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





