pub use engage::{
	unit::*,
	gameuserdata::GameUserData,
	menu::*,
	proc::{Bindable, ProcInstFields},
	resourcemanager::*,
	sortie::SortieSelectionUnitManager,
	unityengine::*,
	util::{get_instance, try_get_instance},
};
use engage::proc::ProcInst;
use unity::prelude::*;
pub use crate::unitasset::*;
pub use crate::localize::{MenuText, MenuTextCommand};

pub use crate::get_outfit_data;

mod menuitem;
mod assetmenu;
mod equipment_box;
pub(crate) mod items;
mod icons;
mod keyhelp;
mod control;

pub use menuitem::*;
pub use assetmenu::*;
pub use items::*;
pub use equipment_box::*;
pub use keyhelp::*;
pub use control::*;
use crate::data::room::hub_room_set_by_result;
use crate::room::ReloadType;
pub use crate::UnitAssetMenuData;


#[unity::class("App", "UnitSelectRoot")]
pub struct UnitSelectRoot {
	parent: u64,
	pub unit_list_root: &'static GameObject,
	pub god_image_object: &'static GameObject,
	pub unit: &'static Unit,
}

pub fn unit_item_y_call(this: &mut BasicMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
	if GameUserData::get_sequence() != 3 { CustomAssetMenu::create_unit_info_bind(this.menu, SortieSelectionUnitManager::get_unit()); }
	else { CustomAssetMenu::create_unit_info_bind(this.menu, engage::map::mind::MapMind::get_unit().unwrap()); }
	BasicMenuResult::close_decide()
}

pub fn add_sub_unit_menu_item(proc: &mut ProcInst) {
	let menu = proc.cast_mut::<BasicMenu<CustomAssetMenuItem>>();
	menu.full_menu_item_list.add(CustomAssetMenuItem::new_type(UnitInventorySubMenuItem));
	let len = menu.full_menu_item_list.len();
	menu.reserved_show_row_num = len as i32;
	menu.show_row_num = len as i32;
	menu.proc.desc_index = 0;
	menu.status_field.value |= 32;
}

pub fn change_selected_profile() -> bool {
	let emblem = UnitAssetMenuData::get().god_mode;
	if crate::r_l_press(true, false, true) {
		let limit = if emblem { 3 } else { 5 };
		let preview = UnitAssetMenuData::get_preview();
		let previous = preview.selected_profile;
		let new =  (limit + previous - 1) % limit;
		UnitAssetMenuData::set_profile(new);
		EquipmentBoxMode::CurrentProfile.update();
		hub_room_set_by_result(None, ReloadType::All);
		true
	}
	else if crate::r_l_press(false, true, true) {
		let limit = if emblem { 3 } else { 5 };
		let preview = UnitAssetMenuData::get_preview();
		let previous = preview.selected_profile;
		let new = (previous + 1) % limit;
		UnitAssetMenuData::set_profile(new);
		EquipmentBoxMode::CurrentProfile.update();
		hub_room_set_by_result(None, ReloadType::All);
		true
	}
	else { false }
}
