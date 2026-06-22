use engage::app::BasicMenu_Result;
use crate::{EquipmentBoxMode, menu::icons::CustomMenuIcon, CustomAssetMenuItem3};
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
    fn get_icon(&self, menu_item: CustomAssetMenuItem3) -> CustomMenuIcon;
    fn get_equipment_box_type(&self, menu_item: CustomAssetMenuItem3) -> EquipmentBoxMode;
    fn get_name(&self, _menu_item: CustomAssetMenuItem3) -> unity::Il2CppString { "".into() }
    fn get_detail_box_name(&self, _menu_item: CustomAssetMenuItem3) -> Option<unity::Il2CppString> { None }
    fn get_help(&self, _menu_item: CustomAssetMenuItem3) -> unity::Il2CppString { "".into() }
    fn get_body(&self, _menu_item: CustomAssetMenuItem3) -> unity::Il2CppString { "".into() }
    fn a_call(&self, _menu_item: CustomAssetMenuItem3) -> BasicMenu_Result{ BasicMenu_Result::pass() }
    fn x_call(&self, _menu_item: CustomAssetMenuItem3) -> BasicMenu_Result{ BasicMenu_Result::pass() }
    fn minus_call(&self, _menu_item: CustomAssetMenuItem3) -> BasicMenu_Result{ BasicMenu_Result::pass() }
    fn custom_call(&self, _menu_item: CustomAssetMenuItem3) -> BasicMenu_Result{ BasicMenu_Result::pass() }
}