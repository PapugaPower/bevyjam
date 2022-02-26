use heron::prelude::{PhysicsLayer};

#[derive(PhysicsLayer)]
pub enum PhysLayer{
    Player,
    World,
    Bullets,
    Enemies
}