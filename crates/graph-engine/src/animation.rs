use crate::{NodeId, SceneGraph};
use core_math::{Quat, Transform, Vec3};
use std::collections::HashMap;

/// Interpolation curve for animation channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterpolationMode {
    #[default]
    Linear,
    Cubic,
    Step,
}

/// A discrete keyframe in time for a value T
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe<T> {
    pub frame: f32,
    pub value: T,
    pub interpolation: InterpolationMode,
}

impl<T> Keyframe<T> {
    pub fn new(frame: f32, value: T, interpolation: InterpolationMode) -> Self {
        Self {
            frame,
            value,
            interpolation,
        }
    }
}

/// An animated channel track (sequence of sorted keyframes)
#[derive(Debug, Clone)]
pub struct AnimationTrack<T> {
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T> Default for AnimationTrack<T> {
    fn default() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }
}

impl<T: Copy + PartialEq> AnimationTrack<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Insert or replace a keyframe at `frame` maintaining sorted order.
    pub fn insert_keyframe(&mut self, frame: f32, value: T, interpolation: InterpolationMode) {
        if let Some(pos) = self.keyframes.iter().position(|k| (k.frame - frame).abs() < 1e-3) {
            self.keyframes[pos].value = value;
            self.keyframes[pos].interpolation = interpolation;
        } else {
            let pos = self.keyframes.partition_point(|k| k.frame < frame);
            self.keyframes.insert(pos, Keyframe::new(frame, value, interpolation));
        }
    }

    /// Remove a keyframe matching `frame` within tolerance.
    pub fn remove_keyframe(&mut self, frame: f32, tolerance: f32) -> bool {
        let initial_len = self.keyframes.len();
        self.keyframes.retain(|k| (k.frame - frame).abs() > tolerance);
        self.keyframes.len() != initial_len
    }

    /// Returns sorted frame times for this track.
    pub fn frames(&self) -> Vec<f32> {
        self.keyframes.iter().map(|k| k.frame).collect()
    }
}

impl AnimationTrack<Vec3> {
    /// Evaluate 3D translation / scale at the specified frame.
    pub fn evaluate(&self, frame: f32) -> Option<Vec3> {
        if self.keyframes.is_empty() {
            return None;
        }
        if self.keyframes.len() == 1 || frame <= self.keyframes[0].frame {
            return Some(self.keyframes[0].value);
        }
        if frame >= self.keyframes.last().unwrap().frame {
            return Some(self.keyframes.last().unwrap().value);
        }

        // Binary search for interval [k0, k1]
        let idx = self.keyframes.partition_point(|k| k.frame <= frame);
        let k0 = &self.keyframes[idx - 1];
        let k1 = &self.keyframes[idx];

        let span = k1.frame - k0.frame;
        let t = if span > 1e-4 {
            (frame - k0.frame) / span
        } else {
            0.0
        };

        match k0.interpolation {
            InterpolationMode::Step => Some(k0.value),
            InterpolationMode::Linear => Some(k0.value.lerp(k1.value, t)),
            InterpolationMode::Cubic => {
                let s = t * t * (3.0 - 2.0 * t);
                Some(k0.value.lerp(k1.value, s))
            }
        }
    }
}

impl AnimationTrack<Quat> {
    /// Evaluate orientation quaternion at the specified frame using spherical linear interpolation (Slerp).
    pub fn evaluate(&self, frame: f32) -> Option<Quat> {
        if self.keyframes.is_empty() {
            return None;
        }
        if self.keyframes.len() == 1 || frame <= self.keyframes[0].frame {
            return Some(self.keyframes[0].value);
        }
        if frame >= self.keyframes.last().unwrap().frame {
            return Some(self.keyframes.last().unwrap().value);
        }

        let idx = self.keyframes.partition_point(|k| k.frame <= frame);
        let k0 = &self.keyframes[idx - 1];
        let k1 = &self.keyframes[idx];

        let span = k1.frame - k0.frame;
        let t = if span > 1e-4 {
            (frame - k0.frame) / span
        } else {
            0.0
        };

        match k0.interpolation {
            InterpolationMode::Step => Some(k0.value),
            InterpolationMode::Linear => Some(k0.value.slerp(k1.value, t)),
            InterpolationMode::Cubic => {
                let s = t * t * (3.0 - 2.0 * t);
                Some(k0.value.slerp(k1.value, s))
            }
        }
    }
}

impl AnimationTrack<f32> {
    /// Evaluate scalar channel value at the specified frame using linear, cubic, or step interpolation.
    pub fn evaluate(&self, frame: f32) -> Option<f32> {
        if self.keyframes.is_empty() {
            return None;
        }
        if self.keyframes.len() == 1 || frame <= self.keyframes[0].frame {
            return Some(self.keyframes[0].value);
        }
        if frame >= self.keyframes.last().unwrap().frame {
            return Some(self.keyframes.last().unwrap().value);
        }

        let idx = self.keyframes.partition_point(|k| k.frame <= frame);
        let k0 = &self.keyframes[idx - 1];
        let k1 = &self.keyframes[idx];

        let span = k1.frame - k0.frame;
        let t = if span > 1e-4 {
            (frame - k0.frame) / span
        } else {
            0.0
        };

        match k0.interpolation {
            InterpolationMode::Step => Some(k0.value),
            InterpolationMode::Linear => Some(k0.value + (k1.value - k0.value) * t),
            InterpolationMode::Cubic => {
                let s = t * t * (3.0 - 2.0 * t);
                Some(k0.value + (k1.value - k0.value) * s)
            }
        }
    }
}

/// Combined 3D Transform animation track for a scene graph node
#[derive(Debug, Clone, Default)]
pub struct TransformTrack {
    pub translation: AnimationTrack<Vec3>,
    pub rotation: AnimationTrack<Quat>,
    pub scale: AnimationTrack<Vec3>,
}

impl TransformTrack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.translation.is_empty() && self.rotation.is_empty() && self.scale.is_empty()
    }

    pub fn evaluate(&self, frame: f32, default_transform: Transform) -> Transform {
        Transform {
            translation: self.translation.evaluate(frame).unwrap_or(default_transform.translation),
            rotation: self.rotation.evaluate(frame).unwrap_or(default_transform.rotation),
            scale: self.scale.evaluate(frame).unwrap_or(default_transform.scale),
        }
    }

    /// Returns all unique frame timestamps that have keyframes in this track.
    pub fn keyframe_frames(&self) -> Vec<f32> {
        let mut frames = Vec::new();
        for k in &self.translation.keyframes {
            frames.push(k.frame);
        }
        for k in &self.rotation.keyframes {
            frames.push(k.frame);
        }
        for k in &self.scale.keyframes {
            frames.push(k.frame);
        }
        frames.sort_by(|a, b| a.partial_cmp(b).unwrap());
        frames.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
        frames
    }
}

/// Central timeline and keyframe manager (Maya Timeline / Blender Dope Sheet)
#[derive(Debug, Clone)]
pub struct AnimationManager {
    pub tracks: HashMap<NodeId, TransformTrack>,
    pub scalar_tracks: HashMap<(NodeId, String), AnimationTrack<f32>>,
    pub current_frame: f32,
    pub start_frame: f32,
    pub end_frame: f32,
    pub fps: f32,
    pub is_playing: bool,
    pub loop_playback: bool,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            tracks: HashMap::new(),
            scalar_tracks: HashMap::new(),
            current_frame: 0.0,
            start_frame: 0.0,
            end_frame: 100.0,
            fps: 24.0,
            is_playing: false,
            loop_playback: true,
        }
    }
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a keyframe for `node_id` at the given `frame`.
    pub fn insert_keyframe(
        &mut self,
        node_id: NodeId,
        frame: f32,
        transform: Transform,
        interpolation: InterpolationMode,
    ) {
        let track = self.tracks.entry(node_id).or_default();
        track.translation.insert_keyframe(frame, transform.translation, interpolation);
        track.rotation.insert_keyframe(frame, transform.rotation, interpolation);
        track.scale.insert_keyframe(frame, transform.scale, interpolation);
    }

    /// Insert a scalar keyframe (e.g. for Morpher target weights or material channels)
    pub fn insert_scalar_keyframe(
        &mut self,
        node_id: NodeId,
        channel: impl Into<String>,
        frame: f32,
        value: f32,
        interpolation: InterpolationMode,
    ) {
        let track = self.scalar_tracks.entry((node_id, channel.into())).or_default();
        track.insert_keyframe(frame, value, interpolation);
    }

    /// Check if a scalar channel has a keyframe at `frame` within tolerance
    pub fn has_scalar_keyframe(&self, node_id: NodeId, channel: &str, frame: f32) -> bool {
        if let Some(track) = self.scalar_tracks.get(&(node_id, channel.to_string())) {
            track.keyframes.iter().any(|k| (k.frame - frame).abs() < 1e-3)
        } else {
            false
        }
    }

    /// Remove a scalar keyframe at `frame`
    pub fn remove_scalar_keyframe(&mut self, node_id: NodeId, channel: &str, frame: f32) -> bool {
        if let Some(track) = self.scalar_tracks.get_mut(&(node_id, channel.to_string())) {
            track.remove_keyframe(frame, 1e-3)
        } else {
            false
        }
    }

    /// Evaluate scalar channel value at the specified frame
    pub fn evaluate_scalar(&self, node_id: NodeId, channel: &str, frame: f32) -> Option<f32> {
        self.scalar_tracks.get(&(node_id, channel.to_string())).and_then(|t| t.evaluate(frame))
    }

    /// Advance time during real-time playback by `delta_seconds` and update scene transforms.
    pub fn step(&mut self, delta_seconds: f32, scene: &mut SceneGraph) {
        if !self.is_playing {
            return;
        }

        let delta_frames = delta_seconds * self.fps;
        let mut next_frame = self.current_frame + delta_frames;

        if next_frame > self.end_frame {
            if self.loop_playback {
                let range = (self.end_frame - self.start_frame).max(1.0);
                next_frame = self.start_frame + (next_frame - self.start_frame) % range;
            } else {
                next_frame = self.end_frame;
                self.is_playing = false;
            }
        }

        self.current_frame = next_frame;
        self.evaluate_scene(scene);
    }

    /// Scrub to a specific frame and update scene node transforms and modifiers.
    pub fn scrub_to_frame(&mut self, frame: f32, scene: &mut SceneGraph) {
        self.current_frame = frame.clamp(self.start_frame, self.end_frame);
        self.evaluate_scene(scene);
    }

    /// Evaluate all animated tracks in the scene at `self.current_frame`.
    pub fn evaluate_scene(&self, scene: &mut SceneGraph) {
        let frame = self.current_frame;
        for (&node_id, track) in &self.tracks {
            if track.is_empty() {
                continue;
            }
            if let Some(node) = scene.nodes.get_mut(node_id) {
                let new_transform = track.evaluate(frame, node.local_transform);
                node.local_transform = new_transform;
                scene.mark_dirty(node_id);
            }
        }

        // Evaluate scalar tracks (e.g. morph targets, shape keys)
        for ((node_id, channel), track) in &self.scalar_tracks {
            if let Some(val) = track.evaluate(frame) {
                if channel.starts_with("morph:") {
                    let target_name = &channel[6..];
                    if let Some(node) = scene.nodes.get_mut(*node_id) {
                        if let crate::NodeData::Mesh { modifier_stack, cached_render_mesh, .. } = &mut node.data {
                            for modifier in modifier_stack.modifiers_mut() {
                                if let Some(morpher) = modifier.as_morpher_mut() {
                                    for t in &mut morpher.targets {
                                        if t.name == target_name {
                                            if (t.weight - val).abs() > 1e-4 {
                                                t.weight = val;
                                                *cached_render_mesh = None;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        scene.update_transforms();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Vec3;

    #[test]
    fn test_track_interpolation_linear_and_step() {
        let mut track = AnimationTrack::<Vec3>::new();
        track.insert_keyframe(0.0, Vec3::new(0.0, 0.0, 0.0), InterpolationMode::Linear);
        track.insert_keyframe(10.0, Vec3::new(10.0, 20.0, 30.0), InterpolationMode::Linear);

        // Before start
        assert_eq!(track.evaluate(-5.0), Some(Vec3::new(0.0, 0.0, 0.0)));
        // Exact start
        assert_eq!(track.evaluate(0.0), Some(Vec3::new(0.0, 0.0, 0.0)));
        // Halfway
        let mid = track.evaluate(5.0).unwrap();
        assert!((mid.x - 5.0).abs() < 1e-4);
        assert!((mid.y - 10.0).abs() < 1e-4);
        assert!((mid.z - 15.0).abs() < 1e-4);
        // Exact end
        assert_eq!(track.evaluate(10.0), Some(Vec3::new(10.0, 20.0, 30.0)));
        // Past end
        assert_eq!(track.evaluate(20.0), Some(Vec3::new(10.0, 20.0, 30.0)));

        // Step mode test
        let mut step_track = AnimationTrack::<Vec3>::new();
        step_track.insert_keyframe(0.0, Vec3::new(1.0, 1.0, 1.0), InterpolationMode::Step);
        step_track.insert_keyframe(10.0, Vec3::new(2.0, 2.0, 2.0), InterpolationMode::Step);

        assert_eq!(step_track.evaluate(4.9), Some(Vec3::new(1.0, 1.0, 1.0)));
        assert_eq!(step_track.evaluate(9.99), Some(Vec3::new(1.0, 1.0, 1.0)));
        assert_eq!(step_track.evaluate(10.0), Some(Vec3::new(2.0, 2.0, 2.0)));
    }

    #[test]
    fn test_animation_manager_playback_and_scrub() {
        let mut scene = SceneGraph::new();
        let node_id = scene.add_node(crate::SceneNode::new_transform("AnimatedBox", Transform::IDENTITY));

        let mut anim = AnimationManager::new();
        anim.start_frame = 0.0;
        anim.end_frame = 24.0;
        anim.fps = 24.0;

        let t0 = Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        };
        let t1 = Transform {
            translation: Vec3::new(24.0, 0.0, 0.0),
            ..Default::default()
        };

        anim.insert_keyframe(node_id, 0.0, t0, InterpolationMode::Linear);
        anim.insert_keyframe(node_id, 24.0, t1, InterpolationMode::Linear);

        anim.scrub_to_frame(12.0, &mut scene);
        assert_eq!(anim.current_frame, 12.0);
        let node = scene.nodes.get(node_id).unwrap();
        assert!((node.local_transform.translation.x - 12.0).abs() < 1e-4);

        // Test playback step
        anim.is_playing = true;
        anim.current_frame = 0.0;
        anim.step(0.5, &mut scene); // 0.5s * 24 fps = 12 frames
        assert!((anim.current_frame - 12.0).abs() < 1e-3);
        let node = scene.nodes.get(node_id).unwrap();
        assert!((node.local_transform.translation.x - 12.0).abs() < 1e-3);
    }

    #[test]
    fn test_scalar_and_morph_animation() {
        let mut scene = SceneGraph::new();
        let cube = geo_kernel::Primitives::create_cube(1.0);
        let node_id = scene.add_node(crate::SceneNode::new_mesh("Cube", Transform::IDENTITY, cube));

        // Add Morpher modifier with a "Smile" shape target
        let mut morpher = geo_kernel::MorpherModifier::new();
        morpher.add_target(geo_kernel::MorphTarget::new("Smile"));
        if let crate::NodeData::Mesh { modifier_stack, .. } = &mut scene.nodes.get_mut(node_id).unwrap().data {
            modifier_stack.add_modifier(Box::new(morpher));
        }

        let mut anim = AnimationManager::new();
        anim.insert_scalar_keyframe(node_id, "morph:Smile", 0.0, 0.0, InterpolationMode::Linear);
        anim.insert_scalar_keyframe(node_id, "morph:Smile", 20.0, 1.0, InterpolationMode::Linear);

        assert!(anim.has_scalar_keyframe(node_id, "morph:Smile", 0.0));
        assert!(anim.has_scalar_keyframe(node_id, "morph:Smile", 20.0));
        assert!(!anim.has_scalar_keyframe(node_id, "morph:Smile", 10.0));

        // Scrub to frame 10: weight should be 0.5
        anim.scrub_to_frame(10.0, &mut scene);
        if let crate::NodeData::Mesh { modifier_stack, .. } = &scene.nodes[node_id].data {
            let morph = modifier_stack.modifiers()[0].as_morpher().unwrap();
            assert!((morph.targets[0].weight - 0.5).abs() < 1e-3);
        } else {
            panic!("Expected mesh node");
        }
    }
}
