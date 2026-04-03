use engage::menu::BasicMenuResult;
use unity::prelude::Il2CppString;
use crate::{CustomAssetMenuItem, EquipmentBoxMode};
use crate::menu::icons::CustomMenuIcon;

mod flags;
mod asset;
mod profile;
mod menus;
mod item;
mod data;

pub use flags::*;
pub use asset::*;
pub use profile::*;
pub use menus::{CustomAssetMenuKind, *};
pub use item::*;
pub use data::*;
pub trait CustomMenuItem {
    fn get_icon(&self, menu_item: &CustomAssetMenuItem) -> CustomMenuIcon;
    fn get_equipment_box_type(&self, menu_item: &CustomAssetMenuItem) -> EquipmentBoxMode;
    fn get_name(&self, menu_item: &CustomAssetMenuItem) -> &'static Il2CppString { menu_item.name }
    fn get_detail_box_name(&self, _menu_item: &CustomAssetMenuItem) -> Option<&'static Il2CppString> { None }
    fn get_help(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString { "".into() }
    fn get_body(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString { "".into() }
    fn a_call(&self, _menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
    fn x_call(&self, _menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
    fn minus_call(&self, _menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
    fn custom_call(&self, _menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
}