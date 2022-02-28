// FIXME how to deal with removed audio? There doesn't seem to be a Remove filter or similar.

use bevy::prelude::*;
use bevy_kira_audio::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use bevy::utils::HashMap;

/// Pool of audio channels because I suspect Kira leaks it.
///
/// It is also necessary to properly stop audio on component removal.
#[derive(Default)]
pub struct AudioChannelPool {
	free: Vec<Arc<AudioChannel>>,
	used: HashMap<Entity, Arc<AudioChannel>>,
}

impl AudioChannelPool {
	/// Get or create an audio channel
	fn take(&mut self, entity: Entity) -> Arc<AudioChannel> {
		let channel = self.free.pop().unwrap_or_else(|| {
			static COUNTER: AtomicUsize = AtomicUsize::new(0);
			// Japanese to avoid having someone mess with this channel from outside
			//let channel = format!("audio2d_鳥_{:x}", COUNTER.fetch_add(1, Ordering::Relaxed));
			let channel = format!("audio2d_{:x}", COUNTER.fetch_add(1, Ordering::Relaxed));
			Arc::new(AudioChannel::new(channel))
		});
		let c = self.used.insert(entity, channel.clone());
		assert!(c.is_none(), "entity already has audio");
		channel
	}

	/// Give back an audio channel belonging to a certain entity and make it stop playing.
	fn give(&mut self, entity: Entity, audio: &Audio) {
		let channel = self.used.remove(&entity).unwrap();
		audio.stop_channel(&channel);
		self.free.push(channel)
	}
}

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
    /// Whether the audio is looping.
    looping: bool,
    /// Whether this source is playing.
    playing: bool,
	/// The channel the audio is playing on.
    channel: Arc<AudioChannel>,
	/// Whether this source was previously looping.
	was_looping: bool,
	/// Whether this source was previously playing.
	was_playing: bool,
}

impl SpatialAudio {
	/// Make the audio loop or not.
	pub fn set_looping(&mut self, looping: bool) {
		self.looping = looping
	}

	/// Whether to play or not.
	pub fn set_playing(&mut self, playing: bool) {
		self.playing = playing
	}
}

impl Default for SpatialAudio {
    fn default() -> Self {
        Self {
			// Tuned for 2D
            attenuation: Attenuation::InverseSquareDistance(300.0),
            max_volume: 1.0,
            volume: 1.0,
            playback_rate: 1.0,
            source: Default::default(),
            looping: false,
            playing: true,
            channel: Default::default(),
			was_playing: false,
			was_looping: false,
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
    mut sources: Query<&mut SpatialAudio, Changed<SpatialAudio>>,
) {
    sources.for_each_mut(|mut source| {
		// hack to prevent endless Changed
		if source.playing == source.was_playing && source.looping == source.was_looping {
			return;
		}
		dbg!(&source.channel);
		let source = &mut *source;

		if !source.playing && source.was_playing {
			dbg!("stop");
			audio.stop_channel(&source.channel);
		} else if source.was_looping != source.looping {
			dbg!("play");
			if source.looping {
				audio.play_looped_in_channel(source.source.clone(), &source.channel);
			} else {
				audio.play_in_channel(source.source.clone(), &source.channel);
			}
			audio.set_playback_rate_in_channel(source.playback_rate, &source.channel);
		}
		source.was_looping = source.looping;
		source.was_playing = source.playing;
    })
}

pub fn spatial_audio_added(audio: Res<Audio>, mut pool: ResMut<AudioChannelPool>, mut sources: Query<(Entity, &mut SpatialAudio), Added<SpatialAudio>>) {
    sources.for_each_mut(|(entity, mut source)| {
		let source = &mut *source;
		source.channel = pool.take(entity);
		dbg!(&source.channel, source.playing);
		if source.playing {
			if source.looping {
				audio.play_looped_in_channel(source.source.clone(), &source.channel);
			} else {
				audio.play_in_channel(source.source.clone(), &source.channel);
			}
		}
        audio.set_playback_rate_in_channel(source.playback_rate, &source.channel);
		source.was_looping = source.looping;
		source.was_playing = source.playing;
    })
}

pub fn spatial_audio_removed(audio: Res<Audio>, mut pool: ResMut<AudioChannelPool>, sources: RemovedComponents<SpatialAudio>) {
    sources.iter().for_each(|entity| pool.give(entity, &audio))
}
