// FIXME how to deal with removed audio? There doesn't seem to be a Remove filter or similar.

use bevy::prelude::*;
use bevy_kira_audio::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Component)]
pub struct SpatialAudio {
    /// How quickly the audio mutes by distance
    pub attenuation: Attenuation,
    /// The maximum volume of the audio to avoid blasting ears.
    pub max_volume: f32,
    /// The volume at 1.0 distance
    pub volume: f32,
    /// How quickly the audio plays. Also affects pitch.
    pub playback_rate: f32,
    /// The audio to play.
    pub source: Handle<AudioSource>,
    /// Whether the audio looped.
    pub looped: bool,
    /// Whether this source is playing.
    pub playing: bool,
    /// The channel the audio is playing on
	// TODO would be nice to avoid pub
    pub channel: AudioChannel,
}

impl Default for SpatialAudio {
    fn default() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        // Japanese to avoid having someone mess with this channel from outside
        let channel = format!("audio2d_鳥_{:x}", COUNTER.fetch_add(1, Ordering::Relaxed));
        let channel = AudioChannel::new(channel);
        Self {
			// Tuned for 2D
            attenuation: Attenuation::InverseSquareDistance(300.0),
            max_volume: 1.0,
            volume: 1.0,
            playback_rate: 1.0,
            source: Default::default(),
            looped: false,
            playing: true,
            channel,
        }
    }
}

/// A single audio receptor.
///
/// Only one entity should have this component at all times.
#[derive(Component)]
pub struct SpatialAudioReceptor;

#[derive(Clone, Copy)]
pub enum Attenuation {
    /// f(x) = c ^ 2 / x ^ 2
    InverseSquareDistance(f32),
}

pub fn spatial_audio(
    audio: Res<Audio>,
    receptor: Query<&Transform, (With<SpatialAudioReceptor>, Without<SpatialAudio>)>,
    sources: Query<(&SpatialAudio, &Transform)>,
) {
    let receptor = if let Ok(r) = receptor.get_single() {
        r
    } else {
        return;
    }
    .translation;
    sources.for_each(|(source, tr)| {
        let tr = tr.translation;
        let dir = tr - receptor;
        let dir_norm = dir.normalize_or_zero();
        // cross product
        let panning = Vec2::Y.x * dir_norm.y - dir_norm.x * Vec2::Y.y;
        let f = match source.attenuation {
            Attenuation::InverseSquareDistance(c) => c * c / dir.length_squared(),
        };
        audio.set_panning_in_channel((1.0 - panning) / 2.0, &source.channel);
        audio.set_volume_in_channel((source.volume * f).min(source.max_volume), &source.channel)
    })
}

pub fn spatial_audio_changed(
    audio: Res<Audio>,
    sources: Query<&SpatialAudio, Changed<SpatialAudio>>,
) {
    sources.for_each(|source| {
        // TODO check looped, playing, source
        audio.set_playback_rate_in_channel(source.playback_rate, &source.channel);
    })
}

pub fn spatial_audio_added(audio: Res<Audio>, sources: Query<&SpatialAudio, Added<SpatialAudio>>) {
    sources.for_each(|source| {
		dbg!("playing shit & stuff");
        if source.looped {
            audio.play_looped_in_channel(source.source.clone(), &source.channel);
        } else {
            audio.play_in_channel(source.source.clone(), &source.channel);
        }
    })
}
