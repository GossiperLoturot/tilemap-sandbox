use glam::*;
use native_core::dataflow::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionKey {
    None,
    Tile(TileKey),
    Block(BlockKey),
    Entity(EntityKey),
}

// resource (client only)

#[derive(Debug)]
pub struct SelectionResource {
    selection_key: SelectionKey,
}

impl SelectionResource {
    pub fn new() -> Self {
        Self {
            selection_key: SelectionKey::None,
        }
    }
}

impl Resource for SelectionResource {}

// system (client only)

pub struct SelectionSystem;

impl SelectionSystem {
    pub const FLAG_NONE: i32 = 0;
    pub const FLAG_TILE: i32 = 1;
    pub const FLAG_BLOCK: i32 = 2;
    pub const FLAG_ENTITY: i32 = 3;

    pub fn set_selection(
        dataflow: &mut Dataflow,
        location: Vec2,
        scroll: i32,
        flag: i32,
    ) -> Result<(), DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let mut resource = resource.borrow_mut()?;

        let mut selection_keys = vec![];
        match flag {
            Self::FLAG_NONE => {}
            Self::FLAG_TILE => {
                let point = Vec2::new(location.x, location.y).floor().as_ivec2();
                if let Some(selection_key) = dataflow.get_tile_key_by_point(point) {
                    selection_keys.push(SelectionKey::Tile(selection_key));
                }
            }
            Self::FLAG_BLOCK => {
                let point = Vec2::new(location.x, location.y);
                for selection_key in dataflow.get_block_keys_by_hint_point(point) {
                    selection_keys.push(SelectionKey::Block(selection_key));
                }
            }
            Self::FLAG_ENTITY => {
                let point = Vec2::new(location.x, location.y);
                for entities in dataflow.get_entity_keys_by_hint_point(point) {
                    selection_keys.push(SelectionKey::Entity(entities));
                }
            }
            _ => panic!("Invalid selection flag"),
        }

        if !selection_keys.is_empty() {
            let index = scroll.div_euclid(selection_keys.len() as i32) as usize;
            resource.selection_key = selection_keys[index];
        } else {
            resource.selection_key = SelectionKey::None;
        }

        Ok(())
    }

    pub fn clear_selection(dataflow: &mut Dataflow) -> Result<(), DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let mut resource = resource.borrow_mut()?;

        resource.selection_key = SelectionKey::None;
        Ok(())
    }

    pub fn has_selection(dataflow: &Dataflow) -> Result<bool, DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let resource = resource.borrow()?;

        Ok(resource.selection_key != SelectionKey::None)
    }

    pub fn get_selection_tile(dataflow: &Dataflow) -> Result<Option<TileKey>, DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let resource = resource.borrow()?;

        match resource.selection_key {
            SelectionKey::Tile(key) => Ok(Some(key)),
            _ => Ok(None),
        }
    }

    pub fn get_selection_block(dataflow: &Dataflow) -> Result<Option<BlockKey>, DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let resource = resource.borrow()?;

        match resource.selection_key {
            SelectionKey::Block(key) => Ok(Some(key)),
            _ => Ok(None),
        }
    }

    pub fn get_selection_entity(dataflow: &Dataflow) -> Result<Option<EntityKey>, DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let resource = resource.borrow()?;

        match resource.selection_key {
            SelectionKey::Entity(key) => Ok(Some(key)),
            _ => Ok(None),
        }
    }

    pub fn get_selection_display_name(
        dataflow: &Dataflow,
    ) -> Result<Option<String>, DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let resource = resource.borrow()?;

        let display_name = match resource.selection_key {
            SelectionKey::None => None,
            SelectionKey::Tile(selection_key) => {
                let display_name = dataflow.get_tile_display_name(selection_key).unwrap();
                Some(display_name.to_string())
            }
            SelectionKey::Block(selection_key) => {
                let display_name = dataflow.get_block_display_name(selection_key).unwrap();
                Some(display_name.to_string())
            }
            SelectionKey::Entity(selection_key) => {
                let display_name = dataflow.get_entity_display_name(selection_key).unwrap();
                Some(display_name.to_string())
            }
        };
        Ok(display_name)
    }

    pub fn get_selection_description(dataflow: &Dataflow) -> Result<Option<String>, DataflowError> {
        let resource = dataflow.find_resources::<SelectionResource>()?;
        let resource = resource.borrow()?;

        let description = match resource.selection_key {
            SelectionKey::None => None,
            SelectionKey::Tile(selection_key) => {
                let description = dataflow.get_tile_description(selection_key).unwrap();
                Some(description.to_string())
            }
            SelectionKey::Block(selection_key) => {
                let description = dataflow.get_block_description(selection_key).unwrap();
                Some(description.to_string())
            }
            SelectionKey::Entity(selection_key) => {
                let description = dataflow.get_entity_description(selection_key).unwrap();
                Some(description.to_string())
            }
        };
        Ok(description)
    }
}
