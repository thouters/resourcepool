use crate::respo::Inventory;
use std::path::PathBuf; //, Pool, Resource, ResourceRequest};

pub struct InventoryLoader;

impl InventoryLoader {
    pub fn load(_path: PathBuf) -> Inventory {
        //todo!("outsource yaml schema validation?");
        //Inventory::new(vec![])
        todo!("implement yaml parsing and creating of an inventory");
    }
}
