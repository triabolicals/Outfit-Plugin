pub use engage::{
	gameuserdata::GameUserData,
	menu::*,
	proc::{Bindable, ProcInstFields},
	resourcemanager::*,
	unityengine::*,
	util::{get_instance, try_get_instance},
};
use engage_il2cpp::{
	app::{
		BasicMenuItem_Attribute,
		IUnitMethods, IPersonDataMethods,
		ISingletonProcInst_1Methods, ISingletonMonoBehaviourList_1Methods,
		accessoryshopchangemenu::*,
		AccessoryDetailInfoWindow,
		AccessoryEquipmentInfo,
		AccessoryShopChangeMenuContent,
		AccessoryShopChangeRoot,
		BasicMenuContent,
		BasicMenu_Result,
		IAccessoryShopChangeRoot,
		IBasicMenuMethods,
		IBasicMenuSelectMethods,
		IGameUserDataMethods,
		IProcInstMethods,
		ISingletonClass_1Methods,
		ISortieSequenceUnitSelect,
		ResourceManager_2,
		IUnitInfo,
		IUnitInfo_Window,
		IUnitInfoWindowCharaModel,
		UnitInfoCharaImageMaskOffset,
		IUnitInfoCharaImageMaskOffset,
		IUnitInfoCharaImageMaskOffsetMethods,
		UnitInfo_Side,
		BackgroundManager,
		UnitInfo,
		ISortieSequenceUnitSelectMethods, ISortieSelectionUnitManager,
		UnitSelectMenu,
		IUnitSelectMenuMethods, IBasicMenuItemMethods, IBasicMenuItem,
		SortieSelectionUnitManager,
		IMapMindMethods, IBasicMenu,
		AssetTable_Result,
		IAssetTable_ResultMethods, IStructData_1Methods, IStructBase,
		SortieUtil,
		IAssetTable_Result, IPad
	},
	List_1Ext,
	nn::hid::NpadButton,
	prelude::List_1,
	system::{
		collections::generic::IList_1Methods,
		self,
		IObjectMethods
	},
	tm_pro::ITMP_TextMethods,
	unity_engine::{
		IAnimatorMethods,
		IComponentMethods,
		IGameObjectMethods,
		IMaterialMethods,
		IObject_2,
		IRectTransformMethods,
		IRenderTextureMethods,
		ITransformMethods,
	}
};
use engage::gamesound::GameSound;
use engage::titlebar::TitleBar;
use engage_il2cpp::tm_pro::{ITMP_Text, TextMeshProUGUI};
use engage_il2cpp::unity_engine::{IObject_2Methods, Transform};
use unity2::{Cast, ClassIdentity, FromIlInstance, IlNull, SystemType};
use unity::prelude::*;
pub use crate::{unitasset::*, localize::{MenuText, MenuTextCommand}, get_outfit_data, UnitAssetMenuData};

mod menuitem;
mod equipment_box;
pub(crate) mod items;
mod icons;
mod keyhelp;
mod control;

pub use menuitem::*;
pub use items::*;
pub use equipment_box::*;
pub use keyhelp::*;
pub use control::*;
use crate::data::room::hub_room_set_by_result;
use crate::room::ReloadType;

#[unity2::inject(namespace = "App", name = "CustomAssetMenu", parent = AccessoryShopChangeMenu)]
pub struct CustomAssetMenu {
	pub unit_name: TextMeshProUGUI,
	pub equipment: AccessoryEquipmentInfo,
	pub detail_box: AccessoryDetailInfoWindow,
	pub menu_kind_v: i32,
	pub next_menu_kind_v: i32,
	pub pause: bool,
}
impl CustomAssetMenu {
	pub fn set_vtable(self) {
		let klass = self.get_class().raw_mut();
		let vtable = klass.get_vtable_mut();
		/*
		vtable[10].method_ptr = Self::on_dispose as _;
		vtable[24].method_ptr = Self::on_build as _;
		vtable[39].method_ptr = Self::tick_input as _;
		vtable[42].method_ptr = Self::key_left as _;
		vtable[43].method_ptr = Self::key_right as _;
		vtable[51].method_ptr = Self::b_call as _;
		vtable[56].method_ptr = Self::plus_call as _;

		 */
	}
	pub fn get_menu_item_kind(self) -> CustomAssetMenuItemKind {
		let item = self.get_menu_item(self.m_select_index());
		if !item.is_null() {
			unsafe { item.cast::<CustomAssetMenuItem3>().menu_item_kind()}
		}
		else { NoItem }
	}
	pub fn menu_kind(self) -> CustomAssetMenuKind { CustomAssetMenuKind::from_index(self.menu_kind_v()) }
	pub fn set_menu_kind(self, menu_kind_v: CustomAssetMenuKind)  { self.set_menu_kind_v(menu_kind_v.to_index()); }
	pub fn next(self) -> Option<CustomAssetMenuKind> {
		let next = self.next_menu_kind_v();
		if next <= 0 { None } else { Some(CustomAssetMenuKind::from_index(next)) }
	}
	pub fn set_next(self, next: Option<CustomAssetMenuKind>) {
		if let Some(next) = next { self.set_next_menu_kind_v(next.to_index()); }
		else { self.set_next_menu_kind_v(-1); }
	}
	pub fn adjust_content(content: AccessoryShopChangeRoot) {
		content.get_components_in_children::<TextMeshProUGUI>(true).iter()
			.filter(|t| t.get_name().to_rust_string().starts_with("Name"))
			.for_each(|t| {
				t.set_m_min_font_size(22.0);
				t.set_m_max_font_size(24.0);
				t.set_m_font_size_max(24.0);
				t.set_m_font_size_min(22.0);
			});
		let transform = content.get_transform();
		crate::utils::change_rect_transform_in_children_size(transform, "AccName", 100.0, 0.0);
		crate::utils::change_rect_transform_in_child_anchor(transform, "BodyParts", 100.0, 0.0);
	}
	pub fn create_bind_unit_info(proc: impl Into<engage_il2cpp::app::ProcInst>, unit: engage_il2cpp::app::Unit){
		let menu_data = UnitAssetMenuData::get();
		if let Some(root) = Self::get_root() {
			let content: AccessoryShopChangeMenuContent = unsafe { root.get_component_in_children(SystemType::from_il2cpp_type(AccessoryShopChangeMenuContent::class().raw().get_type()).unwrap(), true).cast() };
			if content.is_null() { return; }
			let equipment = root.get_component_in_children_3::<AccessoryEquipmentInfo>();
			let detail_box = root.get_component_in_children_3::<AccessoryDetailInfoWindow>();
			menu_data.is_preview = true;
			menu_data.is_shop_combat = true;
			menu_data.mode = MenuMode::UnitInfo;
			UnitAssetMenuData::set_unit(unit);
			let menu = Self::new(content);
			let x_max = 1390.0;
			let x_min = 450.0;
			if !equipment.is_null() {
				menu.set_equipment(equipment);
				build_equipment_window(equipment, false);
			}
			if !detail_box.is_null() { menu.set_detail_box(detail_box); }
			let name = root.m_unit_name();
			if !name.is_null() {
				name.set_text_2(unit.get_name(), true);
				menu.set_unit_name(name);
			}
			BackgroundManager::bind_2();
			let descs = menu.create_default_desc();
			menu.create_bind(proc, descs, "OutfitMenu");
			menu_data.control.initialize(MenuMode::UnitInfo);
			UnitInfo::chara_only_on(false);
			if engage_il2cpp::app::GameUserData::get_instance().get_sequence().value != 3 { engage_il2cpp::app::UnitStatus::close(); }
			let sortie: engage_il2cpp::app::SortieSequenceUnitSelect = engage_il2cpp::app::SortieSequenceUnitSelect::get_instance();
			if !sortie.is_null() { sortie.m_unit_select_menu().m_menu_content().get_game_object().set_active(false); }
			let render_texture = UnitInfo::get_instance().m_windows().get(0).m_unit_info_window_chara_model().m_render_texture();
			start_key_help(OutfitMenuKind::UnitInfo);
			UnitInfoCharaImageMaskOffset::get_instance().iter().for_each(|mask| {
				if IObject_2Methods::equals(mask.m_texture(), render_texture) && mask.is_visible() {
					let mut pos = mask.m_rect_transform().get_position();
					if menu_data.menu_adj == 0.0 { menu_data.menu_adj = pos.x; }
					unit_info_char_mask_setup(mask, false);
					pos.x = (x_max + x_min) * 0.5;
					mask.m_rect_transform().set_position(pos);
				}
			});
			Self::adjust_content(root);
		}
	}
	pub fn create_photo_graph_bind(proc: impl Into<engage_il2cpp::app::ProcInst>) {
		let menu_data = UnitAssetMenuData::get();
		UnitAssetMenuData::init_photo_profiles();
		if let Some(root) = Self::get_root() {
			let content =  root.get_component_in_children_3::<AccessoryShopChangeMenuContent>();
			if content.is_null() { return; }
			let equipment = root.get_component_in_children_3::<AccessoryEquipmentInfo>();
			let detail_box = root.get_component_in_children_3::<AccessoryDetailInfoWindow>();
			if !equipment.is_null() {
				let go = equipment.get_game_object();
				engage_il2cpp::unity_engine::Object_2::destroy_2(go);
			}
			if !detail_box.is_null() {
				let go = detail_box.get_game_object();
				engage_il2cpp::unity_engine::Object_2::destroy_2(go);
			}
			let menu = Self::new(content);
			menu_data.mode = MenuMode::PhotoGraph;
			menu_data.is_preview = true;
			menu_data.is_shop_combat = false;
			let descs = menu.create_default_desc();
			menu_data.control.initialize(MenuMode::PhotoGraph);
			start_key_help(OutfitMenuKind::Photo);
			menu.create_bind(proc, descs, "OutfitPhotographMenu");
		}
	}
	pub fn get_root() -> Option<AccessoryShopChangeRoot> {
		AccessoryShopChangeRoot::load_prefab_async();
		if AccessoryShopChangeRoot::is_loading_prefab() { return None; }
		let canvas = BasicMenuContent::get_canvas();
		let obj = ResourceManager_2::instantiate_2("UI/Hub/Shop/Prefabs/ShopAccChangeRoot", canvas.get_transform());
		if !obj.is_null() {
			let obj: AccessoryShopChangeRoot = obj.get_component_in_children_3();
			if !obj.is_null() { Some(obj) }
			else { None }
		}
		else { None }
	}
	pub fn new(menu_content: AccessoryShopChangeMenuContent) -> Self {
		let menu = Self::instantiate().unwrap();
		menu.set_vtable();
		let items = List_1::<engage_il2cpp::app::BasicMenuItem>::new();
		MainShop.add_menu_items(items);
		IBasicMenuMethods::ctor(menu, items, menu_content);
		let selects = unity2::Array::<engage_il2cpp::app::BasicMenuSelect>::new(engage_il2cpp::app::BasicMenuSelect::class().raw(), CustomAssetMenuKind::SAVE_SELECT_COUNT).unwrap();
		for x in 0..CustomAssetMenuKind::SAVE_SELECT_COUNT { selects.set(x,engage_il2cpp::app::BasicMenuSelect::new()); }
		menu.set_m_selects(selects);
		menu.set_menu_kind(MainShop);
		menu.set_next(None);
		menu.set_pause(false);
		menu.set_m_reserved_show_row_num(12);
		menu.set_m_show_row_num(12);
		menu
	}
	pub fn save_select(self) {
		let menu_kind = self.menu_kind();
		if let Some(select) = menu_kind.get_save_select_index().map(|v| self.m_selects().get(v)){
			if !select.is_null() {
				select.set_scroll(self.get_scroll_index());
				select.set_index(self.get_select_index());
			}
		}
	}
	pub fn rebuild_menu(self, menu: CustomAssetMenuKind, save_select: bool){
		let items = self.m_full_menu_item_list();
		items.clear();
		menu.add_menu_items(items);

		if save_select { self.save_select(); }
		else {
			self.m_selects().iter()
				.for_each(|i|{
					i.set_index(0);
					i.set_scroll(0);
				});
		}
		let select =
		menu.get_save_select_index()
			.map(|i| self.m_selects().get(i))
			.unwrap_or({
				let s = engage_il2cpp::app::BasicMenuSelect::new();
				s.set_scroll(0);
				s.set_index(0);
				s
			});
		self.set_menu_kind(menu);
		self.rebuild_instant_2(select);
		IBasicMenuMethods::after_build(self);
		self.restore_select(select);
		if menu == FaceSelection { self.toggle_ui(); }
	}
	pub fn toggle_ui(self) {
		if !self.unit_name().is_null() { Self::toggle_animator_open_close_state(self.unit_name().get_game_object()); }
		if !self.detail_box().is_null() { Self::toggle_animator_open_close_state(self.detail_box().get_game_object()); }
		if !self.equipment().is_null() { Self::toggle_animator_open_close_state(self.equipment().get_game_object()); }
	}
	pub fn toggle_animator_open_close_state(go: engage_il2cpp::unity_engine::GameObject) {
		if !go.is_null() {
			let anim = go.get_component::<engage_il2cpp::unity_engine::Animator>();
			if !anim.is_null() {
				let closed = anim.get_bool("isClosed");
				if closed { anim.play_2("Open"); }
				else { anim.play_2("Close"); }
			}
		}
	}
	pub fn key_base(self, trigger: bool, right: bool) {
		if trigger {
			let menu_kind = self.menu_kind();
			let new_menu = if right { menu_kind.get_right() } else { menu_kind.get_left() };
			if let Some(new_menu) = new_menu {
				self.rebuild_menu(new_menu, true);
				GameSound::post_event("Category_Change", None);
			}
		}
	}
	fn lr_base(self, right: bool) {
		let sortie = SortieSelectionUnitManager::get_instance();
		if sortie.is_null() { return; }
		let unit = sortie.m_unit();
		let next = if right { SortieUtil::get_next_unit_loop(unit) } else { SortieUtil::get_prev_unit_loop(unit) };
		UnitAssetMenuData::commit();
		UnitInfo::set_unit(UnitInfo_Side::left(), next, false, false, false, system::Action::null());
		sortie.set_m_unit(next);
		UnitAssetMenuData::set_unit(next);
		self.rebuild_menu(MainShop, false);
		let result = AssetTable_Result::get_for_unit_info(next);
		result.set_right_hand("null");
		result.set_left_hand("null");
		result.set_body_anim(result.m_hub_anim());
		hub_room_set_by_result(Some(result), ReloadType::All);
		EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets).update();
		if !self.unit_name().is_null() { self.unit_name().set_text(next.get_name()); }
		GameSound::post_event("Chara_Change", None);
	}
}
#[unity2::injected_methods]
impl CustomAssetMenu{
	#[override_virtual(name = "BCall")]
	pub fn b_call(self) -> BasicMenu_Result {
		let menu_kind = self.menu_kind();
		if let Some(previous) = menu_kind.get_previous() {
			menu_kind.b_call();
			self.rebuild_menu(previous, true);
			BasicMenu_Result::se_cursor()
		}
		else {
			UnitAssetMenuData::commit();
			if !UnitAssetMenuData::is_unit_info() {
				let request_close = self.m_request_close_event_handler();
				if !request_close.is_null() { request_close.invoke(); }
			}
			BasicMenu_Result{value: 513}
		}
	}
	#[override_virtual(name = "PlusCall")]
	pub fn plus_call(self) -> BasicMenu_Result {
		if UnitAssetMenuData::is_unit_info() {
			self.toggle_ui();
			BasicMenu_Result::se_cursor()
		}
		else { BasicMenu_Result::pass() }
	}
	#[override_virtual(name = "OnDispose")]
	pub fn on_dispose(self){
		let menu = UnitAssetMenuData::get();
		menu.control.reset_all();
		match menu.mode {
			MenuMode::UnitInfo => {
				menu.is_preview = false;
				let render_texture = UnitInfo::get_instance().m_windows().get(0).m_unit_info_window_chara_model().m_render_texture();
				UnitInfoCharaImageMaskOffset::get_instance().iter().for_each(|mask|{
					if IObject_2Methods::equals(mask.m_texture(), render_texture) && mask.is_visible() {
						let mut pos = mask.m_rect_transform().get_position();
						pos.x = menu.menu_adj;
						unit_info_char_mask_setup(mask, true);
						mask.m_rect_transform().set_position(pos);
					}
				});
				if !self.detail_box().is_null() {
					let go = self.detail_box().get_game_object();
					if !go.is_null() { engage_il2cpp::unity_engine::Object_2::destroy_2(go); }
				}
				if !self.equipment().is_null() {
					let go = self.equipment().get_game_object();
					if !go.is_null() { engage_il2cpp::unity_engine::Object_2::destroy_2(go); }
				}
				if !self.unit_name().is_null() {
					let go = self.unit_name().get_game_object();
					if !go.is_null() { engage_il2cpp::unity_engine::Object_2::destroy_2(go); }
				}
				AccessoryShopChangeRoot::unload_prefab();
				UnitInfo::chara_only_off();
				UnitInfo::set_unit(UnitInfo_Side::left(), engage_il2cpp::app::Unit::null(), false, false, false, system::Action::null());
				let sortie = engage_il2cpp::app::SortieSequenceUnitSelect::get_instance();
				if !sortie.is_null() {
					sortie.m_window().get_game_object().set_active(true);
					sortie.m_unit_select_menu().m_menu_content().get_game_object().set_active(true);
					sortie.disp_all();
					let unit_manager = SortieSelectionUnitManager::get_instance();
					if !unit_manager.is_null() {
						let unit = unit_manager.m_unit();
						let sortie_unit_select_menu: UnitSelectMenu = unsafe { sortie.m_unit_select_menu().cast() };
						let current_select = sortie_unit_select_menu.get_select_index();
						sortie_unit_select_menu.set_select_index_from_unit(unit);
						let new_select = sortie_unit_select_menu.get_select_index();
						if current_select != new_select {
							let old_item = sortie_unit_select_menu.get_menu_item(current_select);
							if !old_item.is_null() { old_item.on_deselect(); }
							let new_item = sortie_unit_select_menu.get_menu_item(new_select);
							if !new_item.is_null() { new_item.on_select(); }
						}
						sortie_unit_select_menu.adjust_scroll_index();
						sortie_unit_select_menu.scroll_instant();
						sortie_unit_select_menu.open_anime_all();
						UnitInfo::set_unit(UnitInfo_Side::left(), unit, false, false, false, system::Action::null());
					}
				}
				BackgroundManager::unbind();
			}
			_ => {}
		}
	}
	#[override_virtual(name = "OnBuild")]
	pub fn on_build(self) {
		let go = engage_il2cpp::unity_engine::GameObject::find("Category");
		if !go.is_null() {
			go.set_active(false);
			engage_il2cpp::unity_engine::Object_2::destroy_2(go);
		}
	}
	#[override_virtual(name = "KeyLeft")]
	pub fn key_left(self, trigger: bool) { self.key_base(trigger, false) }

	#[override_virtual(name = "KeyRight")]
	pub fn key_right(self, trigger: bool) { self.key_base(trigger, true) }

	#[override_virtual(name = "TickInput")]
	pub fn tick_input(self) -> bool {
		let left = engage_il2cpp::app::Pad::is_trigger(NpadButton::left());
		let right = engage_il2cpp::app::Pad::is_trigger(NpadButton::right());
		let unit_info = UnitAssetMenuData::is_unit_info();
		if (left || right) && left != right {
			if self.get_menu_item_kind().can_facial() && self.menu_kind().can_facial() {
				hub_room_set_by_result(None, ReloadType::Facial(right));
				GameSound::post_event("Category_Change", None);
			}
		}
		let menu = UnitAssetMenuData::get();
		if !UnitAssetMenuData::is_shop() {
			let menu_item_index = self.get_menu_item_kind().to_index();
			let rgb = menu_item_index >= 100 && menu_item_index < 120;
			let stick = model_camera_control(rgb);
			let trigger = engage_il2cpp::app::Pad::is_trigger(NpadButton::plus());
			let menu_kind = self.menu_kind();
			if self.pause() {
				if engage_il2cpp::app::Pad::is_trigger(NpadButton::minus()) && unit_info {
					crate::capture::capture_unit_info(self, false, false);
				}
				if trigger {
					self.m_menu_content().get_game_object().set_active(true);
					self.toggle_ui();
					self.set_pause(false);
					TitleBar::show_header();
					menu_kind.key_help_update(false);
				}
				else if engage_il2cpp::app::Pad::is_trigger(NpadButton::x()) {
					let title = TitleBar::get_instance();
					if title.is_show_header { TitleBar::hide_header(); } else { TitleBar::show_header(); }
				}
				return true;
			}
			else if engage_il2cpp::app::Pad::is_trigger(NpadButton::plus()) {
				self.toggle_ui();
				self.m_menu_content().get_game_object().set_active(false);
				self.set_pause(true);
				menu_kind.key_help_update(true);
				return true;
			}
			if stick { return true; }
			if menu_kind == MainShop && unit_info {
				let l = engage_il2cpp::app::Pad::is_trigger(NpadButton::l());
				let r = engage_il2cpp::app::Pad::is_trigger(NpadButton::r());
				if (l || r) && l != r { self.lr_base(r); }
			}
		}
		if let Some(next) = self.next() {
			self.rebuild_menu(next, true);
			self.set_next(None);
		}
		else if let Some(reload) = menu.reload_type{
			if !engage_il2cpp::app::Pad::is_button(NpadButton::up()) && !engage_il2cpp::app::Pad::is_button(NpadButton::down()) { menu.reload_delay = false; }
			if !menu.reload_delay { UnitAssetMenuData::reload_unit(reload); }
		}
		unsafe { tick_input_base(self, None) }
	}
}
#[skyline::from_offset(0x245ed80)]
fn tick_input_base(custom: CustomAssetMenu, optional_method: OptionalMethod) -> bool;
pub fn is_button_pressed(button: i64, check: NpadButton) -> bool { button & check.value != 0 }

fn model_camera_control(rgb: bool) -> bool {
	let menu_data = UnitAssetMenuData::get();
	let pad = engage_il2cpp::app::Pad::get_instance();
	let buttons = pad.m_npad_state().buttons.value;
	let fast = buttons & NpadButton::y().value != 0;
	let mut translation_change: [i32; 3] = [0; 3];
	let r_stick = engage_il2cpp::app::Pad::is_trigger(NpadButton::stick_r());
	let rotation_y =
		if is_button_pressed(buttons, NpadButton::stick_r_left()) { if fast { -7.5 } else { -2.5 } }
		else if is_button_pressed(buttons, NpadButton::stick_r_right())  { if fast { 7.5 } else { 2.5 } }
		else { 0.0 };

	if is_button_pressed(buttons, NpadButton::stick_r_up())  { translation_change[2] = -1; }
	else if is_button_pressed(buttons, NpadButton::stick_r_down())  { translation_change[2] = 1; }

	if is_button_pressed(buttons, NpadButton::stick_l_left()) { translation_change[0] = 1; }
	else if is_button_pressed(buttons, NpadButton::stick_l_right()) { translation_change[0] = -1; }

	if is_button_pressed(buttons, NpadButton::stick_l_down()) { translation_change[1] = -1; }
	else if is_button_pressed(buttons, NpadButton::stick_l_up())  { translation_change[1] = 1; }
	if fast { for x in 0..3 { translation_change[x] *= 3; } }
	if r_stick {
		menu_data.control.reset_character_position();
		menu_data.control.reset_character_rotation();
	}
	let rl_stick = translation_change.iter().any(|&x| x != 0) || rotation_y != 0.0;
	let mut rot_x = 0.0;
	let mut rot_z = 0.0;
	match menu_data.mode {
		MenuMode::UnitInfo => {
			menu_data.control.translate_character(translation_change);
			menu_data.control.character_rotation(rot_x, rotation_y, rot_z);
		}
		MenuMode::PhotoGraph => {
			if !rgb {
				if engage_il2cpp::app::Pad::is_button(NpadButton::zl()) { rot_x = -1.25; }
				else if engage_il2cpp::app::Pad::is_button(NpadButton::zr()) { rot_x = 1.25; }

				if engage_il2cpp::app::Pad::is_button(NpadButton::l()) { rot_z = -1.25; }
				else if engage_il2cpp::app::Pad::is_button(NpadButton::r()) { rot_z = 1.25; }
			}
			if rot_x != 0.0 || rot_z != 0.0 { menu_data.control.camera_rotation(rot_x, 0.0, rot_z); }
			if rotation_y != 0.0 { menu_data.control.character_rotation(0.0, rotation_y, 0.0); }
			if r_stick {
				menu_data.control.reset_camera_rotation();
				menu_data.control.reset_camera_position();
			}
			if rotation_y != 0.0 { translation_change[2] = 0; }
			menu_data.control.translate_camera(translation_change);
		}
		_ => {}
	}
	rl_stick
}
pub fn unit_item_y_call(this: engage_il2cpp::app::BasicMenuItem, _: unity2::OptionalMethod) -> BasicMenu_Result {
	let sortie = SortieSelectionUnitManager::get_instance();
	if !sortie.is_null() {
		if !sortie.m_unit().is_null() {
			CustomAssetMenu::create_bind_unit_info(this.m_menu(), sortie.m_unit());
			return BasicMenu_Result::close_decide();
		}
	}
	let map_mind = engage_il2cpp::app::MapMind::get_instance();
	if !map_mind.is_null() {
		if !map_mind.get_unit().is_null() {
			CustomAssetMenu::create_bind_unit_info(this.m_menu(), map_mind.get_unit());
			return BasicMenu_Result::close_decide();
		}
	}
	BasicMenu_Result::se_miss()
}

pub fn add_sub_unit_menu_item(proc: engage_il2cpp::app::ProcInst) {
	/*
	let menu = proc.cast_mut::<BasicMenu<CustomAssetMenuItem>>();
	menu.full_menu_item_list.add(CustomAssetMenuItem::new_type(UnitInventorySubMenuItem));
	let len = menu.full_menu_item_list.len();
	menu.reserved_show_row_num = len as i32;
	menu.show_row_num = len as i32;
	menu.proc.desc_index = 0;
	menu.status_field.value |= 32;
	 */
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
#[skyline::hook(offset= 0x2b0ed80)]
pub fn appearance_create_from_result(this: AssetTable_Result, map_distance: i32, o: unity2::OptionalMethod) -> engage_il2cpp::combat::CharacterAppearance {
	let appearance:  engage_il2cpp::combat::CharacterAppearance = call_original!(this, map_distance, o);
	if !appearance.is_null() {
		if !this.get_pid().is_null() {
			let person = engage_il2cpp::app::PersonData::get(this.get_pid());
			if !person.is_null() {
				unity2::field_set_value_at_offset::<i32>(appearance, 0xd4, person.hash());
			}
		}
	}
	appearance
}
fn unit_info_char_mask_setup(mask: UnitInfoCharaImageMaskOffset, revert: bool) {
	let texture = UnitInfo::get_render_texture(UnitInfo_Side::left());
	mask.set_m_texture(texture);
	let rect = mask.m_rect_transform();
	let width = if revert { 640 } else { 1500 };
	rect.set_size_delta(engage_il2cpp::unity_engine::Vector2 { x: width as f32, y: 1080.0 });
	let texture = mask.m_texture();
	if texture.get_width() != width {
		texture.release();
		texture.set_width(width);
		texture.create();
		mask.update_camera(UnitInfo_Side::left());
		mask.set_m_mask_texture_old(mask.m_material().get_texture("_MaskTex"))
	}
}
