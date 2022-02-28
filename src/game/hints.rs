use bevy::prelude::*;

/// Hints shown after the player fails a scenario
#[derive(Component, Default)]
pub struct Hints(Vec<String>);

impl Hints {
	/// Add a single hint.
	pub fn push(&mut self, hint: impl ToString) {
		let hint = hint.to_string();
		if !self.0.contains(&hint) {
			debug!("Adding hint '{}'", &hint);
			self.0.push(hint);
		}
	}

	/// Iterate over all hints.
	pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a str> + 'a {
		self.0.iter().map(|s| &**s)
	}
}

/// Add a single hints component.
pub fn init_hints(mut cmd: Commands) {
	dbg!();
	cmd.spawn_bundle((Hints::default(),));
}
