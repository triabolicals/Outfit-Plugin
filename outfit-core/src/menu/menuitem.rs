use std::sync::OnceLock;
use engage::gamedata::accessory::AccessoryData;
use engage::menu::{BasicMenuItemAttribute, BasicMenuResult};
use engage::menu::menu_item::accessory::*;
use unity::engine::Color;
use unity::engine::ui::IsImage;
use unity::prelude::*;

use engage::game::GameColor;
use engage::menu::menu_item::accessory::AccessoryMenuItemContent;
use engage::menu::menu_item::MenuItem;
use crate::AssetType;
use crate::AssetType::ColorPreset;
use super::{*, items::{CustomMenuItem, *}};

pub static CUSTOM_ASSET_MENU_ITEM: OnceLock<&'static Il2CppClass> = OnceLock::new();

#[unity::class("App", "AccessoryMenuItem")]
pub struct CustomAssetMenuItem {
	pub menu: &'static mut CustomAssetMenu,	//0
	pub menu_item_content: Option<&'static mut AccessoryMenuItemContent>,	//8
	pub name: &'static Il2CppString,	//16
	pub index: i32,	//24
	pub full_index: i32,	//28
	pub attribute: i32,	//32
	pub cursor_color: Color,	//36
	pub active_text: Color,	//52
	pub inactive_text: Color,		//	68
	pub hash: i32,	//84
	pub accessory_data: Option<&'static AccessoryData>,	//88
	pub kind: i32,	//96
	pub decided: bool,
	pub is_asset: bool,
	pub is_menu: bool,	// Type: 0 - Asset, 1 - Menu, 2 - Accessory
	pub original: bool,
	pub sub_kind: i32,	// 104
	pub padding: i32,	//	108
	pub select_event_handler: Option<&'static mut AccessoryMenuItemSelectHandler>,
	pub decide_event_handler:  Option<&'static mut AccessoryMenuItemDecideHandler>,
	pub menu_kind: CustomAssetMenuItemKind,
}
impl MenuItem for CustomAssetMenuItem {}
impl CustomAssetMenuItem {
	pub fn create_class() -> &'static mut Il2CppClass {
		let accessory_klass = Il2CppClass::from_name("App", "AccessoryMenuItem").unwrap().clone();
		accessory_klass._2.instance_size = size_of::<CustomAssetMenuItem>() as u32;
		accessory_klass._2.actual_size = size_of::<CustomAssetMenuItem>() as u32;
		let vtable = accessory_klass.get_vtable_mut();
		vtable[4].method_ptr = Self::get_name as _;
		vtable[8].method_ptr = Self::build_attribute as _;
		vtable[10].method_ptr = Self::on_build as _;
		vtable[11].method_ptr = Self::on_build_menu_item_content as _;
		vtable[12].method_ptr = Self::on_select as _;
		vtable[13].method_ptr = Self::on_deselect as _;
		vtable[18].method_ptr = Self::a_call as _;
		// 19 BCall
		vtable[20].method_ptr = Self::x_call as _;
		// 21 Y Call
		// 22 L
		// 23 R
		// 24 PlusCall
		vtable[25].method_ptr = Self::minus_call as _;
		vtable[26].method_ptr = Self::custom_call as _;
		accessory_klass
	}
	pub fn get_custom_class() -> &'static Il2CppClass {
		CUSTOM_ASSET_MENU_ITEM.get_or_init(|| Self::create_class() )
	}
	pub fn new_menu2(menu_type: CustomAssetMenuKind) -> &'static mut CustomAssetMenuItem {
		let item = Self::new(0, 0);
		//(item.kind, item.sub_kind) = (Menu(menu_type)).to_index();
		item.menu_kind = Menu(menu_type);
		item
	}
	pub fn new_menu3(menu_type: CustomAssetMenuKind, name: &'static Il2CppString) -> &'static mut CustomAssetMenuItem {
		let item = Self::new_menu2(menu_type);
		item.name = name;
		item.menu_kind = Menu(menu_type);
		item
	}
	pub fn new_type(ty: CustomAssetMenuItemKind) -> &'static mut CustomAssetMenuItem {
		let item = Self::new(0, 0);
		if ty == NoItem { item.name = "NONE".into(); }
		match ty {
			FlagMenuItem(flag) => { item.decided = flag.is_decided() }
			_ => {}
		}
		item.menu_kind = ty;
		item
	}
	pub fn new_asset(asset_type: AssetType, hash: i32, name: &'static Il2CppString, decided: bool, original: bool) -> &'static mut CustomAssetMenuItem {
		let item = Self::new(0, 0);
		item.hash = hash;
		item.name = name;
		item.decided = decided;
		item.original = original;
		item.menu_kind = Asset(asset_type);
		if item.original {
			if let Some(game_color) = GameColor::get() {
				item.active_text= game_color.yellow_text;
				item.inactive_text = game_color.yellow_text;
			}
		}
		item
	}
	pub fn new(kind: i32, sub: i32) -> &'static mut CustomAssetMenuItem {
		let item2 = Self::get_custom_class().instantiate_as::<CustomAssetMenuItem>().unwrap();
		item2.ctor_base();
		item2.kind = kind;
		item2.sub_kind = sub;
		item2.is_menu = true;
		item2.is_asset = false;
		item2
	}
	pub fn get_name(this: &CustomAssetMenuItem, _optional_method: OptionalMethod) -> &'static Il2CppString {
		this.menu_kind.get_name(this)
	}
	pub fn x_call(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.x_call(this)
	}
	pub fn minus_call(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.minus_call(this)
	}
	pub fn custom_call(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.custom_call(this)
	}
	pub fn on_select(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) {
		this.on_select_base();
		this.menu_kind.on_select(this);
		this.set_color();
	}
	pub fn build_attribute(this: &CustomAssetMenuItem, _optional_method: OptionalMethod) -> BasicMenuItemAttribute { this.menu_kind.build_attribute() }
	pub fn rebuild_text(&mut self) {
		if let Some(content) = self.menu_item_content.as_ref(){
			let menu_kind = self.menu_kind.clone();
			content.build_text_();
			self.on_build_menu_item_content();
			content.name_text.set_text(menu_kind.get_name(self), true);
			let icon = menu_kind.get_icon(self);
			if let Some(icon) = icon.get_icon(){
				content.kind_icon.set_active(true);
				content.kind_icon_image.set_sprite2(icon);
			}
		}
	}
	pub fn set_decided(&mut self, decided: bool) {
		let ami = unsafe { std::mem::transmute::<&CustomAssetMenuItem, &AccessoryMenuItem>(self) };
		if decided { ami.set_decide(); } else { ami.unset_decide(); }
		self.set_color();
	}
	pub fn on_deselect(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) {
		let original = this.original;
		if let Some(game_color) = GameColor::get() {
			if original {
				if let Some(content) = this.menu_item_content.as_mut() { content.name_text.set_color(game_color.yellow_text); }
				let kind = this.menu_kind.clone();
				match kind {
					Asset(ColorPreset(_)) => {
						let mut vv = [0.0, 0.0, 0.0];
						for x in 0..3 {
							let v = (this.hash >> (x * 8)) & 255;
							vv[x] = v as f32 / 255.0;
						}
						let color = Color { r: vv[0], g: vv[1], b: vv[2], a: 1.0 };
						this.cursor_color = color;
						return;
					}
					_ => {}
				}
			}
			else {
				if let Some(content) = this.menu_item_content.as_mut() {
					content.name_text.set_color(Color{r: 1.0, g: 1.0, b: 1.0, a: 1.0});
				}
			}
		}
	}
	fn set_color(&mut self) {
		let original = self.original;
		if let Some(game_color) = GameColor::get() {
			let kind = self.menu_kind.clone();
			match kind {
				Asset(ColorPreset(_)) => {
					let mut vv = [0.0, 0.0, 0.0];
					for x in 0..3 {
						let v = (self.hash >> (x * 8)) & 255;
						vv[x] = v as f32 / 255.0;
					}
					let color = Color { r: vv[0], g: vv[1], b: vv[2], a: 1.0 };
					self.cursor_color = color;
					if self.original {
						if let Some(content) = self.menu_item_content.as_mut() { content.name_text.set_color(game_color.yellow_text); }
					}
					return;
				}
				_ => {}
			}
			if original {
				self.cursor_color = game_color.yellow_text;
				self.inactive_text = game_color.yellow_text;
				self.active_text = game_color.yellow_text;
				if let Some(content) = self.menu_item_content.as_mut() { content.name_text.set_color(game_color.yellow_text); }
			}
			else {
				self.cursor_color = game_color.default_color;
				self.active_text = game_color.default_color;
			}
		}
	}
	pub fn a_call(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.a_call(this)
	}
	pub fn on_build(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) { this.set_color(); }
	pub fn on_build_menu_item_content(this: &mut CustomAssetMenuItem, _optional_method: OptionalMethod) {
		let custom_item = unsafe { std::mem::transmute::<&CustomAssetMenuItem, &AccessoryMenuItem>(this) };
		custom_item.on_build_menu_item_content_();
		let kind = this.menu_kind.clone();
		let name = kind.get_name(this);
		let icon = kind.get_icon(this);
		let decided = this.decided;
		let original = this.original;
		this.set_color();
		if let Some(content) = this.menu_item_content.as_mut() {
			content.name_text.set_text(name, true);
			content.fixed_cursor_object.set_active(decided);
			if original { if let Some(game_color) = GameColor::get() {
				content.name_text.set_color(game_color.yellow_text); } 
			}
			if let Some(icon) = icon.get_icon() {
				content.kind_icon.set_active(true);
				content.kind_icon_image.set_sprite2(icon);
			} else {
				content.kind_icon.set_active(false);
				content.fixed_cursor_object.set_active(false);
			}
		}
	}
}
pub fn accessory_menu_item_content_build_text(this: &AccessoryMenuItemContent, _optional_method: OptionalMethod) {
	this.build_text_();
	if !UnitAssetMenuData::get().is_preview { return; }
	let custom_item = unsafe { std::mem::transmute::<&BasicMenuItem, &CustomAssetMenuItem>(this.parent.menu_item) };
	this.name_text.set_text(custom_item.menu_kind.get_name(custom_item), true);
	this.fixed_cursor_object.set_active(custom_item.decided);
	if custom_item.original {
		if let Some(game_color) = GameColor::get() { this.name_text.set_color(game_color.yellow_text); }
	}
	return;
}