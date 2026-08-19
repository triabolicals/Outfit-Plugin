
use engage::{
	app::ISpriteAtlasManager_2,
	unity_engine::{
		IGameObjectMethods, ui::{IGraphicMethods, IImageMethods},
		recttransform::*
	},
	tm_pro::ITMP_TextMethods,
	system::collections::generic::IDictionary_2Methods,
};
use unity::{Cast, FromIlInstance, Il2CppString, IlNull};
use super::{
	*,
	CustomAssetMenu,
	items::{CustomMenuItem, *}
};
use crate::{
	AssetItem, AssetLabelTable, AssetType, OtherAssetItem, UnitAssetMenuData,
	menu::icons::CustomMenuIcon
};

#[unity::inject(
	namespace = "App",
	name = "CustomAssetMenuItem",
	parent= AccessoryMenuItem
)]
pub struct CustomAssetMenuItem3 {
	pub menu_item_v: i32,
	pub value: i32,
	pub value2: i32,
	pub is_original: bool,
	pub female: bool,
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
		item.set_m_inactive_text_color(engage::unity_engine::Color::get_white());
		item.set_m_active_text_color(engage::unity_engine::Color{r: 0.75, g: 0.95, b: 1.0, a: 1.0});
		item.set_vtable();
		item
	}
	pub fn new(menu_item_type: CustomAssetMenuItemKind) -> Self {
		let item = Self::new_internal();
		item.set_menu_item_kind(menu_item_type);
		item
	}
	pub fn new_menu(menu_type: CustomAssetMenuKind, name: Il2CppString) -> Self {
		let item = Self::new_internal();
		if !name.is_null() { IBasicMenuItemMethods::set_name(item, name); }
		item.set_menu_item_kind(Menu(menu_type));
		item
	}
	pub fn new_asset(kind: AssetType, hash: i32, name: Il2CppString, decided: bool, original: bool) -> Self {
		let item = Self::new_internal();
		IBasicMenuItemMethods::set_name(item, name);
		item.set_m_decided(decided);
		item.set_is_original(original);
		item.set_menu_item_kind(Asset(kind));
		item.set_value(hash);
		item
	}
	pub fn new_asset2(asset: &AssetItem, label: &str, female: bool) -> Self {
		let item = Self::new_internal();
		let kind = asset.kind;
		item.set_m_decided(UnitAssetMenuData::get_current_unit_hash(asset.kind, female) == asset.hash);
		item.set_value(asset.hash);
		item.set_m_name(asset.get_name(label));
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
		item.set_female(female);
		if is_original {
			let yellow = engage::unity_engine::Color{ r: 1.0, g: 1.0, b: 0.0, a: 1.0};
			item.set_cursor_color(yellow);
			item.set_m_inactive_text_color(yellow);
			item.set_m_active_text_color(yellow);
		}
		item
	}
	pub fn new_asset3(other: &OtherAssetItem, labels: &AssetLabelTable, is_body: bool, female: bool) -> Self {
		let item = Self::new_asset2(&other.asset, other.label.as_str(), female);
		if !other.is_mess { item.set_m_name(other.get_name(labels, is_body)); }
		item
	}
	pub fn as_basic_menu_item(self) -> BasicMenuItem { unsafe { self.cast() } }
	pub fn get_asset_menu(self) -> CustomAssetMenu { unsafe { IBasicMenuItemMethods::get_menu(self).cast() } }
	pub fn get_color(self) -> Option<engage::unity_engine::Color> {
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
					else { Some(engage::unity_engine::Color{r, g, b, a: 1.0}) }
				}
			}
			_ => {
				if self.is_original() { Some(engage::unity_engine::Color{r: 1.0, g: 1.0, b: 0.0, a: 1.0}) }
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
			content.m_name_text().set_text_2(menu_item_kind.get_name(self), true);
		}
		self.set_icon();
	}
	pub fn set_icon(self) {
		let is_decided = self.get_m_decided();
		let menu_kind = self.menu_item_kind();
		let mut idx = menu_kind.to_index();
		let menu_item_content = self.get_menu_item_content();
		if menu_item_content.is_null() { return; }
		if idx == -4 { // FaceThumb
			let name = self.m_name().to_rust_string();
			let name_trimmed = name.trim_end_matches(".png").to_string();
			if let Some(content) = menu_item_content.try_cast::<AccessoryMenuItemContent>() {
				content.m_name_text().set_text_2(name_trimmed.as_str(), true);
				let index = self.get_index();
				let key = format!("LOAD_{}", index);
				let (found, sprite) = engage::app::FaceThumbnail::s_face_thumb().m_cache_table().try_get_value(key.into());
				if found && !sprite.is_null() {
					content.m_kind_icon_object().set_active(true);
					let rec = content.m_kind_icon_object().get_component::<RectTransform>();
					if !rec.is_null() {
						rec.set_size_delta(engage::unity_engine::Vector2{x: 128.0, y: 50.0});
						rec.set_anchored_position(engage::unity_engine::Vector2{x: 90.0, y: 0.0});
					}
					let rec_name = content.m_name_object().get_component::<RectTransform>();
					if !rec_name.is_null() { rec_name.set_anchored_position(engage::unity_engine::Vector2{x: 174.0, y: -40.0}); }
					content.m_kind_icon_image().set_sprite(sprite);
					return;
				}
			}
		}
		if let Some(content) = self.get_item_content() {
			content.m_fixed_cursor_object().set_active(is_decided);
			let icon = menu_kind.get_icon(self);
			let rect = content.m_name_object().get_component::<RectTransform>();
			if !rect.is_null() { rect.set_anchored_position(engage::unity_engine::Vector2{x: 104.0, y: -40.0}); }
			let rect = content.m_kind_icon_object().get_component::<RectTransform>();
			if !rect.is_null() {
				rect.set_anchored_position(engage::unity_engine::Vector2{x: 70.0, y: 0.0});
				rect.set_size_delta(engage::unity_engine::Vector2{x: 48.0, y: 48.0});
			}
			if icon == CustomMenuIcon::Color {
				let mut rgb: Option<(u8, u8, u8)> = None;
				let preview = UnitAssetMenuData::get_preview();
				content.m_kind_icon_image().set_sprite(engage::unity_engine::Sprite::null());
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
					content.m_kind_icon_image().set_color(engage::unity_engine::Color{r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0});
				}
				else { content.m_kind_icon_object().set_active(false); }
			}
			else {
				content.m_kind_icon_image().set_color(engage::unity_engine::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
				if let Some(icon2) = icon.get_icon() {
					content.m_kind_icon_image().set_sprite(icon2);
					content.m_kind_icon_object().set_active(true);
				}
			}
		}
	}
}
#[unity::injected_methods]
impl CustomAssetMenuItem3 {
	#[override_virtual(name ="GetName")]
	pub fn get_menu_item_name(self) -> Il2CppString {
		let name = self.menu_item_kind().get_name(self);
		if !name.is_null() { name }
		else {
			if !self.m_name().is_null() { self.m_name() }
			else { "CustomMenuItem".into() }
		}
	}
	#[override_virtual(name = "BuildAttribute")]
	pub fn build_attribute(self) -> BasicMenuItem_Attribute { self.menu_item_kind().build_attribute() }
	#[override_virtual(name = "OnSelect")]
	pub fn on_select(self) {
		self.menu_item_kind().on_select(self);
		if let Some(c) = self.get_color() { self.set_cursor_color(c); }
		let content = self.get_menu_item_content();
		if !content.is_null() {
			content.set_m_text_base_color(self.m_active_text_color());
			content.update_text_color();
		}
	}
	#[override_virtual(name = "OnDeselect")]
	pub fn on_deselect(self) {
		let content = self.get_menu_item_content();
		if !content.is_null() {
			content.set_m_text_base_color(self.m_inactive_text_color());
			content.update_text_color();
		}
		if let Some(c) = self.get_color() { self.set_cursor_color(c); }
	}
	#[override_virtual(name = "ACall")] pub fn a_call(self) -> BasicMenu_Result { self.menu_item_kind().a_call(self) }
	#[override_virtual(name = "XCall")] pub fn x_call(self) -> BasicMenu_Result { self.menu_item_kind().x_call(self) }
	#[override_virtual(name = "MinusCall")] pub fn minus_call(self) -> BasicMenu_Result { self.menu_item_kind().minus_call(self) }
	#[override_virtual(name = "CustomCall")] pub fn custom_call(self) -> BasicMenu_Result { self.menu_item_kind().custom_call(self) }

	#[override_virtual(name = "OnBuildMenuItemContent")]
	pub fn on_build_menu_item_content(self) {
		if self.is_original() {
			let yellow = engage::unity_engine::Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
			self.set_cursor_color(yellow);
			self.set_text_color(yellow, false);
			self.set_text_color(yellow, true);
		}
		else {
			let original = engage::unity_engine::Color { r: 0.60, g: 0.95, b: 1.0, a: 1.0 };
			self.set_cursor_color(original);
			self.set_text_color(original , false);
			self.set_text_color(engage::unity_engine::Color::get_white(), true);
		}
		self.set_icon();
	}
}
pub fn accessory_menu_item_content_build_text(this: AccessoryMenuItemContent, method_info:  unity::OptionalMethod) {
	unsafe { build_text(this, method_info) }
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
			let yellow = engage::unity_engine::Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
			custom_item.set_cursor_color(yellow);
			custom_item.set_text_color(yellow, false);
			custom_item.set_text_color(yellow, true);
		}
		else {
			let original = engage::unity_engine::Color { r: 0.60, g: 0.95, b: 1.0, a: 1.0 };
			custom_item.set_cursor_color(original);
			custom_item.set_text_color(original , false);
			custom_item.set_text_color(engage::unity_engine::Color::get_white(), true);
		}
	}
	return;
}
#[skyline::from_offset(0x27b8510)]
fn build_text(this: AccessoryMenuItemContent, optional_method: unity::OptionalMethod);