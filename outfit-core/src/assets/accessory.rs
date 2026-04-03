use super::*;

pub fn new_asset_table_accessory(model: impl Into<&'static Il2CppString>, loc: impl Into<&'static Il2CppString>) -> &'static mut AssetTableAccessory {
    let accessory_class = Il2CppClass::from_name("App", "AssetTable").unwrap().get_nested_types().iter().find(|x| x.get_name() == "Accessory").unwrap();
    let new_accessory = Il2CppObject::<AssetTableAccessory>::from_class( accessory_class ).unwrap();
    new_accessory.model = Some(model.into() );
    new_accessory.locator = Some(loc.into());
    new_accessory
}