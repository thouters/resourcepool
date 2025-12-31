use crate::inventory::InnerInventory;
use serde_saphyr::from_reader;
pub struct InventoryLoader;

impl InventoryLoader {
    pub fn load<T: std::io::Read>(file: T) -> InnerInventory {
        //todo!("outsource yaml schema validation?");
        //Inventory::new(vec![])
        let parsed: InnerInventory = from_reader(file).unwrap();
        parsed
    }
}
