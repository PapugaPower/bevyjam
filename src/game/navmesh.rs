use navmesh::{NavVec3, NavQuery, NavPathMode};
use bevy::prelude::*;
use bevy::render::mesh::{VertexAttributeValues, Indices};

#[derive(Component)]
pub struct NavMesh(navmesh::NavMesh);

impl NavMesh {
	/// Create a NavMesh from a Bevy Mesh.
	///
	/// Use thicken to prevent the agent from being stupid and going to corners because it can't cross a 0.000000000001 border.
	pub fn new(mesh: &Mesh, thicken: f32) -> Self {
		let verts = if let VertexAttributeValues::Float32x3(v) = mesh.attribute(Mesh::ATTRIBUTE_POSITION).expect("vertices") { v } else { panic!("expected float32x3") };
		let verts = verts.into_iter().map(|&[a, b, c]: &[f32; 3]| NavVec3::new(a, b, c)).collect::<Vec<_>>();
		// TODO avoid collect
		let tris = match mesh.indices() {
			Some(Indices::U16(v)) => v.iter().map(|&i| i as usize).collect::<Vec<_>>(),
			Some(Indices::U32(v)) => v.iter().map(|&i| i as usize).collect::<Vec<_>>(),
			None => (0..verts.len()).collect::<Vec<_>>(),
		};
		let tris = tris.chunks_exact(3).map(|t| navmesh::NavTriangle { first: t[0] as u32, second: t[1] as u32, third: t[2] as u32 }).collect();
		Self(navmesh::NavMesh::new(verts, tris).expect("failed to build navmesh").thicken(thicken).expect("failed to thicken"))
	}
}

impl From<navmesh::NavMesh> for NavMesh {
	fn from(m: navmesh::NavMesh) -> Self {
		Self(m)
	}
}

#[derive(Component, Default)]
pub struct NavAgent {
	target: NavVec3,
	path: Vec<NavVec3>,
}

impl NavAgent {
	/// Set the target the agent should move to.
	pub fn set_target(&mut self, target: Vec3) {
		self.target = NavVec3::new(target.x, target.y, target.z);
	}

	/// Give the point at the given distance from the start point along the current path.
	///
	/// Is None if there is no path.
	pub fn point_on_path(&self, distance: f32) -> Option<Vec3> {
		dbg!(&self.path);
		navmesh::NavMesh::point_on_path(&self.path, distance)
			.or(self.path.last().copied()) 
			.and_then(map_vec)
	}
}

pub fn update_agent_path(mut agents: Query<(&mut NavAgent, &Transform), Changed<NavAgent>>, mesh: Query<&NavMesh>) {
	if let Ok(mesh) = mesh.get_single() {
		agents.for_each_mut(|(mut agent, tr)| {
			let start = tr.translation;
			let start = NavVec3::new(start.x, start.y, start.z);
			agent.path = mesh.0.find_path(start, agent.target, NavQuery::Accuracy, NavPathMode::Accuracy).unwrap_or_default();
		})
	}
}

/// Returns None if it contains NaN
fn map_vec(v: NavVec3) -> Option<Vec3> {
	if v.x.is_nan() || v.y.is_nan() || v.z.is_nan() {
		None
	} else {
		Some(Vec3::new(v.x, v.y, v.z))
	}
}
