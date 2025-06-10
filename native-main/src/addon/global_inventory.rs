use native_core::dataflow::*;

// resource

#[derive(Debug, Default)]
pub struct GlobalInventoryResource {
    inventory_key: Option<InventoryKey>,
}

impl GlobalInventoryResource {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl Resource for GlobalInventoryResource {}

// system

pub struct GlobalInventorySystem;

impl GlobalInventorySystem {
    pub fn insert_inventory(dataflow: &mut Dataflow, id: u16) -> Result<(), DataflowError> {
        let resource = dataflow.find_resources::<GlobalInventoryResource>()?;
        let mut resource = resource.borrow_mut().map_err(DataflowError::from)?;

        let inventory_key = dataflow.insert_inventory(id)?;
        resource.inventory_key = Some(inventory_key);
        Ok(())
    }

    pub fn get_inventory_key(dataflow: &Dataflow) -> Result<InventoryKey, DataflowError> {
        let resource = dataflow.find_resources::<GlobalInventoryResource>()?;
        let resource = resource.borrow().map_err(DataflowError::from)?;

        let inventory_key = resource.inventory_key.unwrap();
        Ok(inventory_key)
    }
}
