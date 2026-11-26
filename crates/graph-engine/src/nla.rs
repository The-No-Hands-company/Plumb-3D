use crate::{NodeId, TransformTrack};
use core_math::{Quat, Transform, Vec3};
use std::collections::HashMap;

/// Blending mode for Non-Linear Animation (NLA) clips
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NlaBlendMode {
    #[default]
    Replace,
    Add,
    Multiply,
}

/// An NLA Clip containing animated node tracks with start/end frames, cross-fades, and influence weights.
#[derive(Debug, Clone)]
pub struct NlaClip {
    pub name: String,
    pub start_frame: f32,
    pub end_frame: f32,
    pub blend_in: f32,
    pub blend_out: f32,
    pub weight: f32,
    pub playback_speed: f32,
    pub blend_mode: NlaBlendMode,
    pub node_tracks: HashMap<NodeId, TransformTrack>,
}

impl NlaClip {
    pub fn new(name: impl Into<String>, start_frame: f32, end_frame: f32) -> Self {
        Self {
            name: name.into(),
            start_frame,
            end_frame,
            blend_in: 0.0,
            blend_out: 0.0,
            weight: 1.0,
            playback_speed: 1.0,
            blend_mode: NlaBlendMode::Replace,
            node_tracks: HashMap::new(),
        }
    }

    pub fn with_tracks(
        name: impl Into<String>,
        start_frame: f32,
        end_frame: f32,
        node_tracks: HashMap<NodeId, TransformTrack>,
    ) -> Self {
        Self {
            name: name.into(),
            start_frame,
            end_frame,
            blend_in: 0.0,
            blend_out: 0.0,
            weight: 1.0,
            playback_speed: 1.0,
            blend_mode: NlaBlendMode::Replace,
            node_tracks,
        }
    }

    /// Computes the effective cross-fade influence weight at the specified `frame`.
    pub fn compute_influence(&self, frame: f32) -> f32 {
        if frame < self.start_frame || frame > self.end_frame || self.weight <= 0.0 {
            return 0.0;
        }

        let mut influence = self.weight;

        // Blend-in ramp
        if self.blend_in > 1e-4 {
            let t = (frame - self.start_frame) / self.blend_in;
            if t < 1.0 {
                influence *= t.clamp(0.0, 1.0);
            }
        }

        // Blend-out ramp
        if self.blend_out > 1e-4 {
            let t = (self.end_frame - frame) / self.blend_out;
            if t < 1.0 {
                influence *= t.clamp(0.0, 1.0);
            }
        }

        influence
    }

    /// Evaluates the transform of a specific node at `frame`, mapped to the clip's local timeline.
    pub fn evaluate(&self, node_id: NodeId, frame: f32, default_transform: Transform) -> Option<(Transform, f32)> {
        let influence = self.compute_influence(frame);
        if influence <= 0.0 {
            return None;
        }

        let track = self.node_tracks.get(&node_id)?;
        if track.is_empty() {
            return None;
        }

        // Map global frame to clip-local relative frame based on playback speed
        let local_frame = (frame - self.start_frame) * self.playback_speed;
        let evaluated = track.evaluate(local_frame, default_transform);
        Some((evaluated, influence))
    }
}

/// An NLA Track stacking sequential or parallel NLA clips.
#[derive(Debug, Clone, Default)]
pub struct NlaTrack {
    pub name: String,
    pub is_muted: bool,
    pub is_solo: bool,
    pub clips: Vec<NlaClip>,
}

impl NlaTrack {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_muted: false,
            is_solo: false,
            clips: Vec::new(),
        }
    }

    pub fn add_clip(&mut self, clip: NlaClip) {
        self.clips.push(clip);
    }

    /// Evaluates active clips on this track at `frame`.
    pub fn evaluate(&self, node_id: NodeId, frame: f32, default_transform: Transform) -> Option<(Transform, f32, NlaBlendMode)> {
        if self.is_muted {
            return None;
        }

        for clip in &self.clips {
            if let Some((tf, influence)) = clip.evaluate(node_id, frame, default_transform) {
                return Some((tf, influence, clip.blend_mode));
            }
        }

        None
    }
}

/// Non-Linear Animation Mixer for multi-track clip layering and blending.
#[derive(Debug, Clone, Default)]
pub struct NlaMixer {
    pub tracks: Vec<NlaTrack>,
}

impl NlaMixer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_track(&mut self, track: NlaTrack) {
        self.tracks.push(track);
    }

    /// Blends active NLA tracks on top of `base_transform` for `node_id` at `frame`.
    pub fn blend_transforms(&self, node_id: NodeId, frame: f32, base_transform: Transform) -> Transform {
        if self.tracks.is_empty() {
            return base_transform;
        }

        let has_solo = self.tracks.iter().any(|t| t.is_solo);
        let mut result = base_transform;

        for track in &self.tracks {
            if (has_solo && !track.is_solo) || track.is_muted {
                continue;
            }

            if let Some((clip_tf, influence, mode)) = track.evaluate(node_id, frame, base_transform) {
                match mode {
                    NlaBlendMode::Replace => {
                        result.translation = result.translation.lerp(clip_tf.translation, influence);
                        result.rotation = result.rotation.slerp(clip_tf.rotation, influence);
                        result.scale = result.scale.lerp(clip_tf.scale, influence);
                    }
                    NlaBlendMode::Add => {
                        let add_trans = (clip_tf.translation - base_transform.translation) * influence;
                        let add_scale = (clip_tf.scale - base_transform.scale) * influence;
                        let add_rot = Quat::IDENTITY.slerp(clip_tf.rotation * base_transform.rotation.inverse(), influence);

                        result.translation += add_trans;
                        result.rotation = add_rot * result.rotation;
                        result.scale += add_scale;
                    }
                    NlaBlendMode::Multiply => {
                        let mult_trans = result.translation * (Vec3::ONE.lerp(clip_tf.translation, influence));
                        let mult_scale = result.scale * (Vec3::ONE.lerp(clip_tf.scale, influence));
                        result.translation = mult_trans;
                        result.scale = mult_scale;
                    }
                }
            }
        }

        result
    }
}

/// Audio Scrub Track supporting waveform audio scrubbing and visualization.
#[derive(Debug, Clone)]
pub struct AudioScrubTrack {
    pub name: String,
    pub start_frame: f32,
    pub sample_rate: u32,
    pub samples: Vec<f32>,
    pub volume: f32,
    pub is_muted: bool,
}

impl AudioScrubTrack {
    pub fn new(name: impl Into<String>, sample_rate: u32, samples: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            start_frame: 0.0,
            sample_rate,
            samples,
            volume: 1.0,
            is_muted: false,
        }
    }

    /// Generates a synthetic sinusoidal tone for audio playback and scrub testing.
    pub fn synthetic_sine_tone(name: impl Into<String>, sample_rate: u32, duration_secs: f32, freq_hz: f32) -> Self {
        let total_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(total_samples);
        let tau = std::f32::consts::TAU;

        for i in 0..total_samples {
            let t = i as f32 / sample_rate as f32;
            let val = (t * freq_hz * tau).sin();
            samples.push(val);
        }

        Self::new(name, sample_rate, samples)
    }

    /// Retrieves instantaneous audio amplitude at the given animation `frame`.
    pub fn sample_at_frame(&self, frame: f32, fps: f32) -> f32 {
        if self.is_muted || self.samples.is_empty() || frame < self.start_frame {
            return 0.0;
        }

        let elapsed_seconds = (frame - self.start_frame) / fps.max(1.0);
        let sample_idx = (elapsed_seconds * self.sample_rate as f32) as usize;
        if sample_idx < self.samples.len() {
            self.samples[sample_idx] * self.volume
        } else {
            0.0
        }
    }

    /// Computes RMS waveform amplitude bins across a frame interval for timeline waveform rendering.
    pub fn waveform_slice(&self, start_f: f32, end_f: f32, fps: f32, num_bins: usize) -> Vec<f32> {
        if self.samples.is_empty() || num_bins == 0 || end_f <= start_f {
            return vec![0.0; num_bins];
        }

        let fps = fps.max(1.0);
        let start_sec = ((start_f - self.start_frame) / fps).max(0.0);
        let end_sec = ((end_f - self.start_frame) / fps).max(0.0);

        let start_idx = (start_sec * self.sample_rate as f32) as usize;
        let end_idx = (end_sec * self.sample_rate as f32).min(self.samples.len() as f32) as usize;

        if start_idx >= end_idx {
            return vec![0.0; num_bins];
        }

        let total_span = end_idx - start_idx;
        let mut bins = Vec::with_capacity(num_bins);

        for bin_i in 0..num_bins {
            let bin_start = start_idx + (bin_i * total_span) / num_bins;
            let bin_end = start_idx + ((bin_i + 1) * total_span) / num_bins;
            let count = (bin_end - bin_start).max(1);

            let mut sum_sq = 0.0_f32;
            for i in bin_start..bin_end.min(self.samples.len()) {
                let s = self.samples[i];
                sum_sq += s * s;
            }
            let rms = (sum_sq / count as f32).sqrt() * self.volume;
            bins.push(rms.clamp(0.0, 1.0));
        }

        bins
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InterpolationMode;

    #[test]
    fn test_nla_clip_influence_and_crossfade() {
        let mut clip = NlaClip::new("WalkCycle", 10.0, 50.0);
        clip.blend_in = 10.0;
        clip.blend_out = 10.0;
        clip.weight = 1.0;

        // Outside clip range
        assert_eq!(clip.compute_influence(5.0), 0.0);
        assert_eq!(clip.compute_influence(55.0), 0.0);

        // Blend-in ramp (frame 10 -> 0.0, frame 15 -> 0.5, frame 20 -> 1.0)
        assert!((clip.compute_influence(10.0) - 0.0).abs() < 1e-4);
        assert!((clip.compute_influence(15.0) - 0.5).abs() < 1e-4);
        assert!((clip.compute_influence(20.0) - 1.0).abs() < 1e-4);

        // Full weight plateau
        assert_eq!(clip.compute_influence(30.0), 1.0);

        // Blend-out ramp (frame 40 -> 1.0, frame 45 -> 0.5, frame 50 -> 0.0)
        assert!((clip.compute_influence(40.0) - 1.0).abs() < 1e-4);
        assert!((clip.compute_influence(45.0) - 0.5).abs() < 1e-4);
        assert!((clip.compute_influence(50.0) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_nla_mixer_blending() {
        use crate::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        let nid = scene.add_node(SceneNode::new_transform("Box", Transform::IDENTITY));

        let mut clip1 = NlaClip::new("Walk", 0.0, 30.0);
        let mut track1 = TransformTrack::new();
        track1.translation.insert_keyframe(0.0, Vec3::new(10.0, 0.0, 0.0), InterpolationMode::Linear);
        clip1.node_tracks.insert(nid, track1);

        let mut clip2 = NlaClip::new("Jump", 10.0, 40.0);
        let mut track2 = TransformTrack::new();
        track2.translation.insert_keyframe(0.0, Vec3::new(0.0, 20.0, 0.0), InterpolationMode::Linear);
        clip2.blend_mode = NlaBlendMode::Add;
        clip2.node_tracks.insert(nid, track2);

        let mut t1 = NlaTrack::new("BaseLayer");
        t1.add_clip(clip1);

        let mut t2 = NlaTrack::new("AdditiveLayer");
        t2.add_clip(clip2);

        let mut mixer = NlaMixer::new();
        mixer.add_track(t1);
        mixer.add_track(t2);

        let base_tf = Transform::IDENTITY;
        let blended = mixer.blend_transforms(nid, 15.0, base_tf);
        // Base is at 10.0, Additive adds 20.0 -> total X = 10, Y = 20
        assert!((blended.translation.x - 10.0).abs() < 1e-3);
        assert!((blended.translation.y - 20.0).abs() < 1e-3);
    }

    #[test]
    fn test_audio_scrub_track_waveform() {
        let audio = AudioScrubTrack::synthetic_sine_tone("BeepTrack", 44100, 1.0, 440.0);
        assert_eq!(audio.samples.len(), 44100);

        let s0 = audio.sample_at_frame(0.0, 24.0);
        assert!((s0 - 0.0).abs() < 1e-3);

        let bins = audio.waveform_slice(0.0, 24.0, 24.0, 8);
        assert_eq!(bins.len(), 8);
        for b in bins {
            assert!(b > 0.0 && b <= 1.0);
        }
    }
}
