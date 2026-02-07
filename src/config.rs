use crate::inventory::Inventory;
use serde_saphyr::from_reader;
pub struct InventoryLoader;

impl InventoryLoader {
    pub fn load<T: std::io::Read>(file: T) -> Inventory {
        //todo!("outsource yaml schema validation?");
        //Inventory::new(vec![])
        let parsed: Inventory = from_reader(file).unwrap();
        parsed
    }
}
