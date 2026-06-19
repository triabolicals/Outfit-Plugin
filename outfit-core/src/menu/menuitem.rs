use std::sync::OnceLock;
use engage_il2cpp::{
	app::{
		accessorymenuitem::*,
		IAccessoryMenuItemContent, IBasicMenuItem,
		IBasicMenuItemMethods, BasicMenu_Result, BasicMenuItem_Attribute, BasicMenuItem,
	},
	tm_pro::ITMP_Text,
	unity_engine::{
		IGameObjectMethods, IObject_2Methods,
		ui::{IGraphicMethods, IImageMethods}
	},
	app::{IAccessoryMenuItemContentMethods, IBasicMenuItemContentMethods},
	tm_pro::ITMP_TextMethods,
	app::{AccessoryMenuItemContent, ISpriteAtlasManager_2},
	system::collections::generic::IDictionary_2Methods,
	unity_engine::{IRectTransformMethods, RectTransform}
};
use unity::{prelude::*};
use unity2::{Cast, ClassIdentity, FromIlInstance, IlNull};
use crate::{AssetItem, AssetLabelTable, AssetType, OtherAssetItem, UnitAssetMenuData};
use crate::menu::icons::CustomMenuIcon;
use super::{CustomAssetMenu, items::{CustomMenuItem, *}};

pub static CUSTOM_ASSET_MENU_ITEM: OnceLock<&'static Il2CppClass> = OnceLock::new();

#[unity2::inject(
	namespace = "App",
	name = "CustomAssetMenuItem",
	parent= AccessoryMenuItem
)]
pub struct CustomAssetMenuItem3 {
	pub menu_item_v: i32,
	pub value: i32,
	pub value2: i32,
	pub is_original: bool,
}
impl CustomAssetMenuItem3 {
	pub fn set_vtable(self) {
		let klass = self.get_class();
		let table = klass.raw_mut().get_vtable_mut();
		// table[4].method_ptr = Self::get_name as _;
		table[8].method_ptr = Self::build_attribute as _;
		table[11].method_ptr = Self::on_build_menu_item_content as _;
		table[12].method_ptr = Self::on_select as _;
		table[13].method_ptr = Self::on_deselect as _;
		table[18].method_ptr = Self::a_call as _;
		table[20].method_ptr = Self::x_call as _;
		table[25].method_ptr = Self::minus_call as _;
		table[26].method_ptr = Self::custom_call as _;
	}
	pub fn set_menu_item_kind(self, v: CustomAssetMenuItemKind){ self.set_menu_item_v(v.to_index()); }
	pub fn menu_item_kind(self) -> CustomAssetMenuItemKind { CustomAssetMenuItemKind::from_index(self.menu_item_v()) }
	fn new_internal() -> Self {
		let item = Self::instantiate().unwrap();
		IBasicMenuItemMethods::ctor(item);
		item.set_m_inactive_text_color(engage_il2cpp::unity_engine::Color::get_white());
		item.set_m_active_text_color(engage_il2cpp::unity_engine::Color{r: 0.75, g: 0.95, b: 1.0, a: 1.0});
		item.set_vtable();
		item
	}
	pub fn new(menu_item_type: CustomAssetMenuItemKind) -> Self {
		let item = Self::new_internal();
		item.set_menu_item_kind(menu_item_type);
		item
	}
	pub fn new_menu(menu_type: CustomAssetMenuKind, name: unity2::Il2CppString) -> Self {
		let item = Self::new_internal();
		if !name.is_null() { IBasicMenuItemMethods::set_name(item, name); }
		item.set_menu_item_kind(Menu(menu_type));
		item
	}
	pub fn new_asset(kind: AssetType, hash: i32, name: unity2::Il2CppString, decided: bool, original: bool) -> Self {
		let item = Self::new_internal();
		IBasicMenuItemMethods::set_name(item, name);
		item.set_m_decided(decided);
		item.set_is_original(original);
		item.set_menu_item_kind(Asset(kind));
		item.set_value(hash);
		item
	}
	pub fn new_asset2(asset: &AssetItem, label: &str) -> Self {
		let item = Self::new_internal();
		let kind = asset.kind;
		item.set_m_decided(UnitAssetMenuData::get_current_unit_hash(asset.kind) == asset.hash);
		item.set_value(asset.hash);
		IBasicMenuItemMethods::set_name(item,  asset.get_name(label));
		item.set_menu_item_kind(Asset(asset.kind));
		let preview = UnitAssetMenuData::get_preview();
		let original =
			match kind {
				AssetType::Body => preview.original_assets[0],
				AssetType::Head => preview.original_assets[1],
				AssetType::Hair => preview.original_assets[2],
				AssetType::Acc(slot) => preview.original_assets[5+slot as usize],
				AssetType::AOC(slot) => preview.original_assets[10 + slot as usize],
				AssetType::Mount(slot) => preview.preview_data.mount[slot as usize],
				AssetType::Voice => preview.original_assets[14],
				AssetType::Rig => preview.original_assets[15],
				AssetType::ColorPreset(kind) => {
					let mut original = 0;
					for x in 0..3 { original += (preview.original_color[4*kind as usize + x] << 8*x) as i32; }
					original
				}
			};
		let is_original = original == asset.hash;
		item.set_is_original(is_original);
		if is_original {
			let yellow = engage_il2cpp::unity_engine::Color{ r: 1.0, g: 1.0, b: 0.0, a: 1.0};
			item.set_cursor_color(yellow);
			item.set_m_inactive_text_color(yellow);
			item.set_m_active_text_color(yellow);
		}
		item
	}
	pub fn new_asset3(other: &OtherAssetItem, labels: &AssetLabelTable, is_body: bool) -> Self {
		let item = Self::new_asset2(&other.asset, other.label.as_str());
		if !other.is_mess { IBasicMenuItemMethods::set_name(item, other.get_name(labels, is_body)); }
		item
	}
	pub fn as_basic_menu_item(self) -> BasicMenuItem { unsafe { self.cast() } }
	pub fn get_asset_menu(self) -> CustomAssetMenu { unsafe { IBasicMenuItemMethods::get_menu(self).cast() } }
	pub fn get_color(self) -> Option<engage_il2cpp::unity_engine::Color> {
		let kind = self.menu_item_kind();
		match kind {
			Asset(AssetType::ColorPreset(_))|ResetColor(_)|RGBA(_) => {
				let value = self.value();
				if value == 0 { None }
				else {
					let r = (value & 255) as f32 / 255.0;
					let g = ((value >> 8) & 255) as f32 / 255.0;
					let b = ((value >> 16) & 255) as f32 / 255.0;
					if r + g + b < 0.30 { None }
					else { Some(engage_il2cpp::unity_engine::Color{r, g, b, a: 1.0}) }
				}
			}
			_ => {
				if self.is_original() { Some(engage_il2cpp::unity_engine::Color{r: 1.0, g: 1.0, b: 0.0, a: 1.0}) }
				else { None }
			}
		}
	}
	pub fn get_item_content(self) -> Option<AccessoryMenuItemContent> {
		let content = IBasicMenuItemMethods::get_menu_item_content(self);
		if !content.is_null() { content.try_cast() } else { None }
	}
	pub fn rebuild_text(self) {
		self.on_build_menu_item_content();
		if let Some(content) = self.get_item_content() {
			let menu_item_kind = self.menu_item_kind();
			content.m_name_text().set_m_text(menu_item_kind.get_name(self));
		}
		self.set_icon();
	}
	pub fn set_icon(self) {
		let is_decided = self.get_m_decided();
		let menu_kind = self.menu_item_kind();
		let mut idx = menu_kind.to_index();
		if idx == -4 { // FaceThumb
			let name = IBasicMenuItemMethods::get_name(self).to_rust_string();
			let name_trimmed = name.trim_end_matches(".png").to_string();
			if let Some(content) = self.get_menu_item_content().try_cast::<AccessoryMenuItemContent>() {
				content.m_name_text().set_text_2(name_trimmed.as_str(), true);
				let index = self.get_index();
				let key = format!("LOAD_{}", index);
				let (found, sprite) = engage_il2cpp::app::FaceThumbnail::s_face_thumb().m_cache_table().try_get_value(key.into());
				if found && !sprite.is_null() {
					content.m_kind_icon_object().set_active(true);
					let rec = content.m_kind_icon_object().get_component::<RectTransform>();
					if !rec.is_null() {
						rec.set_anchored_position(engage_il2cpp::unity_engine::Vector2{x: 90.0, y: 90.0});
						rec.set_size_delta(engage_il2cpp::unity_engine::Vector2{x: 127.0, y: 50.0});
					}
					let rec_name = content.m_name_object().get_component::<RectTransform>();
					if rec_name.is_null() {
						rec.set_anchored_position(engage_il2cpp::unity_engine::Vector2{x: 160.0, y: -40.0});
					}
					content.m_kind_icon_image().set_sprite(sprite);
					return;
				}
			}
		}
		if let Some(content) = self.get_item_content() {
			content.m_fixed_cursor_object().set_active(is_decided);
			let icon = menu_kind.get_icon(self);
			let rect = content.m_name_object().get_component::<RectTransform>();
			if !rect.is_null() {
				rect.set_anchored_position(engage_il2cpp::unity_engine::Vector2{x: 104.0, y: -40.0});
			}
			let rect = content.m_kind_icon_object().get_component::<RectTransform>();
			if !rect.is_null() {
				rect.set_anchored_position(engage_il2cpp::unity_engine::Vector2{x: 70.0, y: 0.0});
				rect.set_size_delta(engage_il2cpp::unity_engine::Vector2{x: 48.0, y: 48.0});
			}
			if icon == CustomMenuIcon::Color {
				let mut rgb: Option<(u8, u8, u8)> = None;
				let preview = UnitAssetMenuData::get_preview();
				content.m_kind_icon_image().set_sprite(engage_il2cpp::unity_engine::Sprite::null());
				if idx >= 1140 {
					let kind = (idx - 1140) % 16;
					if preview.preview_data.colors[kind as usize].has_color() { idx = 100 + kind; }
					else { idx = 30 + kind; }
				}
				if idx >= 30 && idx < 46 {	// Default Color
					let k = (idx - 30) as usize;
					rgb = Some((preview.original_color[4 * k] , preview.original_color[4 * k + 1], preview.original_color[4 * k + 2]));
				}
				else if idx >= 100 && idx < 116 {	// Preview Color / Set Color
					let k = (idx - 100) as usize;
					rgb = Some((preview.color_preview[4 * k], preview.color_preview[4 * k + 1], preview.color_preview[4 * k + 2]));
				}
				else if idx >= 80 && idx < 96 {	// Color Preset
					let v = self.value();
					rgb = Some(((v & 255) as u8, ((v >> 8) & 255) as u8, ((v >> 16) & 255) as u8));
				}
				if let Some((r, g, b)) = rgb.filter(|(r, g, b)| *r > 0 || *g > 0 || *b > 0) {
					content.m_kind_icon_object().set_active(true);
					content.m_kind_icon_image().set_color(engage_il2cpp::unity_engine::Color{r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0});
				}
				else { content.m_kind_icon_object().set_active(false); }
			}
			else {
				content.m_kind_icon_image().set_color(engage_il2cpp::unity_engine::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
				if let Some(icon2) = icon.get_icon() {
					if let Some(icon_key) = icon.get_system_label() { println!("MenuItemIndex {}: {}", idx, icon_key); }
					content.m_kind_icon_image().set_sprite(icon2);
					content.m_kind_icon_object().set_active(true);
				}
			}
		}
	}
}
#[unity2::injected_methods]
impl CustomAssetMenuItem3 {
	/*
	#[override_virtual(name = "GetName")]
	pub fn get_name(self) -> unity2::Il2CppString {
		let name = self.menu_item_kind().get_name(self);
		name
	}
	 */
	#[override_virtual(name = "BuildAttribute")]
	pub fn build_attribute(self) -> BasicMenuItem_Attribute { self.menu_item_kind().build_attribute() }
	#[override_virtual(name = "OnSelect")]
	pub fn on_select(self) {
		IBasicMenuItemMethods::on_select(self);
		self.menu_item_kind().on_select(self);
		if let Some(c) = self.get_color() { self.set_cursor_color(c); }
	}
	#[override_virtual(name = "OnDeselect")]
	pub fn on_deselect(self) {
		IBasicMenuItemMethods::on_deselect(self);
		if let Some(c) = self.get_color() { IBasicMenuItemMethods::set_cursor_color(self, c); }
	}
	#[override_virtual(name = "ACall")] pub fn a_call(self) -> BasicMenu_Result { self.menu_item_kind().a_call(self) }
	#[override_virtual(name = "XCall")] pub fn x_call(self) -> BasicMenu_Result { self.menu_item_kind().x_call(self) }
	#[override_virtual(name = "MinusCall")] pub fn minus_call(self) -> BasicMenu_Result { self.menu_item_kind().minus_call(self) }
	#[override_virtual(name = "CustomCall")] pub fn custom_call(self) -> BasicMenu_Result { self.menu_item_kind().custom_call(self) }

	#[override_virtual(name = "OnBuildMenuItemContent")]
	pub fn on_build_menu_item_content(self) {
		if self.is_original() {
			let yellow = engage_il2cpp::unity_engine::Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
			self.set_cursor_color(yellow);
			self.set_text_color(yellow, false);
			self.set_text_color(yellow, true);
		}
		else {
			let original = engage_il2cpp::unity_engine::Color { r: 0.60, g: 0.95, b: 1.0, a: 1.0 };
			self.set_cursor_color(original);
			self.set_text_color(original , false);
			self.set_text_color(engage_il2cpp::unity_engine::Color::get_white(), true);
		}
		self.set_icon();
	}
}
/*
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
	pub decided: bool,	// 100
	pub is_asset: bool,	// 101
	pub is_menu: bool,	// 102
	pub original: bool,	// 103
	pub sub_kind: i32,	// 104
	pub padding: i32,	//	108
	pub select_event_handler: Option<&'static mut AccessoryMenuItemSelectHandler>,	// 112
	pub decide_event_handler:  Option<&'static mut AccessoryMenuItemDecideHandler>,	// 120
	pub menu_kind: CustomAssetMenuItemKind,	// 128
}

impl CustomAssetMenuItem {
	pub fn new_menu2(menu_type: CustomAssetMenuKind) -> &'static mut CustomAssetMenuItem {
		let item = Self::new(0, 0);
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
	pub fn new_asset3(other: &OtherAssetItem, labels: &AssetLabelTable, is_body: bool) -> &'static mut CustomAssetMenuItem {
		let item = Self::new_asset2(&other.asset, other.label.as_str());
		if !other.is_mess { item.name = other.get_name(labels, is_body); }
		item
	}
	pub fn new_asset2(asset: &AssetItem, label: &str) -> &'static mut CustomAssetMenuItem {
		let item = Self::new(0, 0);
		let kind = asset.kind;
		item.decided = UnitAssetMenuData::get_current_unit_hash(asset.kind) == asset.hash;
		item.hash = asset.hash;
		item.name = asset.get_name(label);
		item.menu_kind = Asset(asset.kind);
		let preview = UnitAssetMenuData::get_preview();
		let original =
			match kind {
				AssetType::Body => preview.original_assets[0],
				AssetType::Head => preview.original_assets[1],
				AssetType::Hair => preview.original_assets[2],
				AssetType::Acc(slot) => preview.original_assets[5+slot as usize],
				AssetType::AOC(slot) => preview.original_assets[10 + slot as usize],
				AssetType::Mount(slot) => preview.preview_data.mount[slot as usize],
				AssetType::Voice => preview.original_assets[14],
				AssetType::Rig => preview.original_assets[15],
				AssetType::ColorPreset(kind) => {
					let mut original = 0;
					for x in 0..3 { original += (preview.original_color[4*kind as usize + x] << 8*x) as i32; }
					original
				}
			};
		let is_original = original == item.hash;
		item.original = is_original;
		if let Some(game_color) = GameColor::get() {
			if item.original {
				item.active_text = game_color.yellow_text;
				item.cursor_color = game_color.yellow_text;
				item.inactive_text = game_color.yellow_text;
			}
		}
		item
	}
	pub fn new_asset(asset_type: AssetType, hash: i32, name: &'static Il2CppString, decided: bool, original: bool) -> &'static mut CustomAssetMenuItem {
		let item = Self::new(0, 0);
		item.hash = hash;
		item.name = name;
		item.decided = decided;
		item.original = original;
		item.menu_kind = Asset(asset_type);
		if let Some(game_color) = GameColor::get() {
			match asset_type {
				AssetType::ColorPreset(_) => {
					let mut vv = [0.0, 0.0, 0.0];
					for x in 0..3 {
						let v = (hash >> (x * 8)) & 255;
						vv[x] = v as f32 / 255.0;
					}
					let color = Color { r: vv[0], g: vv[1], b: vv[2], a: 1.0 };
					item.cursor_color = color;
				}
				_ => {
					if item.original {
						item.active_text = game_color.yellow_text;
						item.inactive_text = game_color.yellow_text;
						item.cursor_color = game_color.yellow_text;
					}
				}
			}
		}
		item
	}
	pub fn get_name(this: &CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) -> &'static Il2CppString {
		this.menu_kind.get_name(this)
	}
	pub fn x_call(this: &mut CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.x_call(this)
	}
	pub fn minus_call(this: &mut CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.minus_call(this)
	}
	pub fn custom_call(this: &mut CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.custom_call(this)
	}
	pub fn on_select(this: &mut CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) {
		this.on_select_base();
		let rgb: Option<(u8, u8, u8)>;
		match this.menu_kind {
			UnitInventorySubMenuItem => { return; }
			RGBA(kind) => {
				let k = (kind % 16) as usize;
				let preview = UnitAssetMenuData::get_preview();
				rgb = Some((preview.color_preview[4 * k], preview.color_preview[4 * k + 1], preview.color_preview[4 * k + 2]));
			}
			Asset(AssetType::ColorPreset(_)) => {
				rgb = Some(((this.hash & 255) as u8, (this.hash >> 8) as u8 & 255, (this.hash >> 16) as u8 & 255));
			}
			ResetColor(kind) => {
				let k = (kind % 16) as usize;
				let preview = UnitAssetMenuData::get_preview();
				rgb = Some((preview.original_color[4 * k], preview.original_color[4 * k + 1], preview.original_color[4 * k + 2]));
			}
			_ => { rgb = None; }
		}
		if let Some((r, g, b)) = rgb.filter(|(r, g, b)| r != g &&  r != b){
			let color = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
			this.cursor_color = color;
			this.menu.menu_content.set_cursor_color(color);
		}
		else {
			if let Some(color) = GameColor::get() {
				if this.original { this.menu.menu_content.set_cursor_color(color.yellow_text); }
				else { this.menu.menu_content.set_cursor_color(color.default_color); }
			}
		}
		this.menu_kind.on_select(this);
		this.set_color();
	}
	pub fn build_attribute(this: &CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) -> BasicMenuItemAttribute { this.menu_kind.build_attribute() }
	pub fn rebuild_text(&mut self) {
		Self::on_build_menu_item_content(self, None);
		let menu_kind = self.menu_kind.clone();
		if let Some(content) = self.menu_item_content.as_ref() {
			content.build_text_();
			content.name_text.set_text(menu_kind.get_name(self), true);
		}
		self.set_color();
	}
	pub fn set_decided(&mut self, decided: bool) {
		let ami = unsafe { std::mem::transmute::<&CustomAssetMenuItem, &AccessoryMenuItem>(self) };
		if decided { ami.set_decide(); } else { ami.unset_decide(); }
		self.set_color();
	}
	pub fn on_deselect(this: &mut CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) {
		let original = this.original;
		let kind = this.menu_kind.clone();
		if let Some(game_color) = GameColor::get() {
			if let Some(content) = this.menu_item_content.as_mut() {
				match kind {
					UnitInventorySubMenuItem => { this.on_deselect_base(); }
					_ => {
						if original { content.name_text.set_color(game_color.yellow_text); }
						else { content.name_text.set_color(Color{r: 1.0, g: 1.0, b: 1.0, a: 1.0}); }
					}
				}
			}
		}
	}
	fn set_color(&mut self) {
		let is_decided = self.decided;
		let menu_kind = self.menu_kind.clone();
		let mut idx = menu_kind.to_index();
		if let Some(content) = self.menu_item_content.as_ref(){
			let icon = menu_kind.get_icon(self);
			if icon == CustomMenuIcon::Color {
				let mut rgb: Option<(u8, u8, u8)> = None;
				let preview = UnitAssetMenuData::get_preview();
				content.kind_icon_image.set_no_sprite();
				if idx >= 1140 {
					let kind = (idx - 1140) % 16;
					if preview.preview_data.colors[kind as usize].has_color() { idx = 100 + kind; }
					else { idx = 30 + kind; }
				}
				if idx >= 30 && idx < 46 {	// Default Color
					let k = (idx - 30) as usize;
					rgb = Some((preview.original_color[4 * k] , preview.original_color[4 * k + 1], preview.original_color[4 * k + 2]));
				}
				else if idx >= 100 && idx < 116 {	// Preview Color / Set Color
					let k = (idx - 100) as usize;
					rgb = Some((preview.color_preview[4 * k], preview.color_preview[4 * k + 1], preview.color_preview[4 * k + 2]));
				}
				else if idx >= 80 && idx < 96 {	// Color Preset
					rgb = Some(((self.hash & 255) as u8, ((self.hash >> 8) & 255) as u8, ((self.hash >> 16) & 255) as u8));
				}
				if let Some((r, g, b)) = rgb.filter(|(r, g, b)| *r > 0 || *g > 0 || *b > 0) {
					content.kind_icon.set_active(true);
					content.kind_icon_image.set_color2(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0,1.0);
				}
				else { content.kind_icon.set_active(false); }
			}
			else {
				content.kind_icon_image.set_color2(1.0, 1.0, 1.0,1.0);
				if let Some(icon) = icon.get_icon(){
					content.kind_icon.set_active(true);
					content.kind_icon_image.set_sprite2(icon);
				}
				content.fixed_cursor_object.set_active2(is_decided);
			}
		}
	}
	pub fn a_call(this: &mut CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) -> BasicMenuResult {
		let s = this.menu_kind.clone();
		s.a_call(this)
	}
	pub fn on_build_menu_item_content(this: &mut CustomAssetMenuItem, _optional_method: unity2::OptionalMethod) {
		let idx = this.hash;
		let kind = this.menu_kind.clone();
		let kind_idx = kind.to_index();
		if kind_idx == 0 { return; }	// UnitInventorySubMenuItem
		else if kind_idx == -4 {	// FaceThumb
			let name = this.name.to_string().trim_end_matches(".png").to_string();
			if let Some(content) = this.menu_item_content.as_mut() {
				content.name_text.set_text(name.into(), true);
				content.name_text.set_color(GameColor::get().as_ref().unwrap().default_color);
				if let Some(sprite) = FaceThumbnail::get_item(format!("LOAD_{}", idx)) {
					content.kind_icon.set_active(true);
					if let Some(rect) = content.kind_icon.get_component_by_type::<RectTransform>() {
						rect.set_anchored_position_injected(&Vector2::new(90.0, 0.0));
						rect.set_size_delta(Vector2::new(127.0, 50.0));
					}
					if let Some(rect) = content.name_object.get_component_by_type::<RectTransform>() {
						rect.set_anchored_position_injected(&Vector2::new(160.0, -40.0));
					}
					content.kind_icon_image.set_sprite2(sprite);
				}
			}
			return;
		}
		else {
			let game_color = GameColor::get().as_ref().unwrap();
			let name = kind.get_name(this);
			let icon = kind.get_icon(this);
			let decided = this.decided;
			let original = this.original;
			let disable = this.attribute & 2 != 0;
			if original {
				this.cursor_color = game_color.yellow_text;
				this.inactive_text = game_color.yellow_text;
				this.active_text = game_color.yellow_text;
			} else {
				this.active_text = game_color.default_color;
				this.inactive_text = game_color.second_color;
			}
			if let Some(content) = this.menu_item_content.as_mut() {
				content.name_text.set_text(name, true);
				content.fixed_cursor_object.set_active(decided);
				if let Some(rect) = content.name_object.get_component_by_type::<RectTransform>() {
					rect.set_anchored_position_injected(&Vector2::new(104.0, -40.0));
				}
				if original { content.name_text.set_color(game_color.yellow_text); }
				else if disable { content.name_text.set_color(game_color.disable_character); }
				else { content.name_text.set_color(game_color.default_character); }

				if let Some(icon) = icon.get_icon() {
					content.kind_icon.set_active(true);
					if let Some(rect) = content.kind_icon.get_component_by_type::<RectTransform>() {
						rect.set_anchored_position_injected(&Vector2::new(70.0, 0.0));
						rect.set_size_delta(Vector2::new(48.0, 48.0));
					}
					content.kind_icon_image.set_sprite2(icon);
				}
			}
			this.set_color();
		}
	}
}
 */
pub fn accessory_menu_item_content_build_text(this: engage_il2cpp::app::AccessoryMenuItemContent, _: unity2::OptionalMethod) {
	IAccessoryMenuItemContentMethods::build_text(this);
	if !UnitAssetMenuData::get().is_preview { return; }
	let custom_item = this.get_menu_item();
	if !custom_item.is_null() {
		let custom_item = unsafe { custom_item.cast::<CustomAssetMenuItem3>() };
		let kind = custom_item.menu_item_kind();
		this.m_name_object().set_active(true);
		let name_text = this.m_name_text();
		name_text.set_text_2(kind.get_name(custom_item), true);
		custom_item.set_icon();
		let decided = custom_item.get_m_decided();
		this.m_fixed_cursor_object().set_active(decided);
		if custom_item.is_original() {
			let yellow = engage_il2cpp::unity_engine::Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
			custom_item.set_cursor_color(yellow);
			custom_item.set_text_color(yellow, false);
			custom_item.set_text_color(yellow, true);
		}
		else {
			let original = engage_il2cpp::unity_engine::Color { r: 0.60, g: 0.95, b: 1.0, a: 1.0 };
			custom_item.set_cursor_color(original);
			custom_item.set_text_color(original , false);
			custom_item.set_text_color(engage_il2cpp::unity_engine::Color::get_white(), true);
		}
	}
	return;
}