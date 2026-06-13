use engage_il2cpp::app::{AssetTable_Accessory, IAssetTable_AccessoryMethods};
use super::*;

pub fn new_asset_table_accessory<T: Into<unity2::Il2CppString>>(model: T, loc: T) -> AssetTable_Accessory {
    let accessory = AssetTable_Accessory::new();
    accessory.set_model(model.into());
    accessory.set_locator(loc.into());
    accessory
}