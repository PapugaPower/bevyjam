use bevy::prelude::*;
use std::any::Any;

/// Collection of items. Can be used on a player, chest ...
#[derive(Component, Default)]
pub struct Inventory {
    items: Vec<InventoryItem>,
}

impl Inventory {
    /// Insert a single item.
    pub fn insert(&mut self, item: InventoryItem) {
        self.items.push(item);
    }
}

/// Trait that must be implemented by all inventory items.
pub trait IsInventoryItem: Any + Send + Sync {}

/// Single item in inventory.
pub struct InventoryItem {
    // TODO what do we actually need in here?
    pub name: String,
    pub item: Box<dyn IsInventoryItem>,
}

impl InventoryItem {
    /// Create a new inventory item.
    pub fn new(name: impl Into<String>, item: impl IsInventoryItem) -> Self {
        Self {
            item: Box::new(item),
            name: name.into(),
        }
    }
}

/// Component that can hold a single item. This uses an Option so we can move items with
/// [`Option::Take`].
#[derive(Component)]
pub struct InventoryItemHolder {
    pub item: Option<InventoryItem>,
    /// Whether to despawn the entity if the item is picked up.
    pub despawn_on_pickup: bool,
}

impl InventoryItemHolder {
    /// Create new holder.
    pub fn new(item: InventoryItem) -> Self {
        Self {
            item: Some(item),
            despawn_on_pickup: true,
        }
    }
}

/// A component that allows automatically picking up inventory items.
#[derive(Component)]
pub struct InventoryItemAttractor {
    // squared is a wee bit faster because it avoids sqrt()
    pub radius_squared: f32,
}

impl InventoryItemAttractor {
    pub fn with_radius(r: f32) -> Self {
        Self {
            radius_squared: r * r,
        }
    }
}

pub fn pickup_items(
    mut cmd: Commands,
    mut pickup: Query<(&mut Inventory, &Transform, &InventoryItemAttractor)>,
    mut items: Query<(Entity, &Transform, &mut InventoryItemHolder), Without<Inventory>>,
) {
    pickup.for_each_mut(|(mut inv, p_trf, attr)| {
        items.for_each_mut(|(e, i_trf, mut holder)| {
            if p_trf.translation.distance_squared(i_trf.translation) < attr.radius_squared {
                if holder.despawn_on_pickup {
                    cmd.entity(e).despawn_recursive();
                }
                holder.item.take().map(|item| inv.insert(item));
            }
        })
    })
}
