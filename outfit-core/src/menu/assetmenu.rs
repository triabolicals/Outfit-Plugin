use std::sync::OnceLock;
use super::*;
use engage::{
    unit::Unit,
    gamesound::GameSound,
    titlebar::TitleBar,
    keyhelp::*,
    pad::Pad,
    unitinfo::*,
    manager::BackgroundManager,
    proc::{Bindable, ProcInst, ProcInstFields},
    gamedata::{assettable::AssetTableResult, WeaponMask},
    menu::{
        menu_item::accessory::*,
        content::{BasicMenuContent, AccessoryShopChangeMenuContent, AccessoryEquipmentInfo, AccessoryDetailInfoWindow},
        BasicMenuResult, BasicMenuSelect, menus::accessory::change::*
    },
    unityengine::GameObject,
    sortie::{SortieUtil, SortieSequenceUnitSelect},
    mess::Mess,
    pad::NpadButton,
    combat::Kaneko,
    titlebar::KeyHelpButton,
    tmpro::TextMeshProUGUI,
};
use unity::{system::List, il2cpp::object::Array, engine::Vector2, };
use crate::menu::items::{CustomAssetMenuKind, MainShop};

pub static CUSTOM_ASSET_MENU: OnceLock<&'static mut Il2CppClass> = OnceLock::new();

#[repr(C)]
pub struct CustomAssetMenu {
    pub klass: &'static Il2CppClass,
    pub monitor: u64,
    pub proc: ProcInstFields,
    pub menu_content: &'static mut AccessoryShopChangeMenuContent,
    pub menu_item_list: &'static mut List<CustomAssetMenuItem>,
    pub full_menu_item_list: &'static mut List<CustomAssetMenuItem>,
    pub status_field: &'static mut WeaponMask,
    pub result: i32,
    scroll_preceed_input_a: bool,
    pub disable: bool,
    pub is_shop: bool,
    pub is_photo: bool,
    pub row_num: i32,
    pub show_row_num: i32,
    pub select_index: i32,
    pub select_index_old: i32,
    pub scroll_index: i32,
    pub scroll_index_old: i32,
    pub reserved_select_index: i32,
    pub reserved_scroll_index: i32,
    pub reserved_show_row_num: i32,
    pub memory_display_index: i32,
    pub suspend: i32,
    pub kind: i32,
    pub unit: &'static mut Unit,
    pub selects: &'static mut Array<&'static mut BasicMenuSelect>,
    pub accessory_select: u64,
    pub accessory_decide: u64,
    pub request_close: Option<&'static mut AccessoryShopChangeMenuRequestCloseHandler>,
    pub change_kind: u64,
    pub menu_kind: CustomAssetMenuKind,
}
impl Il2CppClassData for CustomAssetMenu {
    const NAMESPACE: &'static str = "App";
    const CLASS: &'static str = "AccessoryShopChangeMenu";

    fn class() -> &'static Il2CppClass { CUSTOM_ASSET_MENU.get_or_init(||Self::create_class()) }
    fn class_mut() -> &'static mut Il2CppClass { Self::class().clone() }
}

impl BasicMenuMethods for CustomAssetMenu {}
impl Bindable for CustomAssetMenu {}

impl CustomAssetMenu {
    #[unity::class_method(2,AccessoryShopChangeMenu)]
    pub fn base_ctor(
        &self, list: &List<CustomAssetMenuItem>,
        content: &AccessoryShopChangeMenuContent,
        unit: Option<&Unit>,
        selec: Option<&AccessoryMenuItemSelectHandler>,
        decide: Option<&AccessoryMenuItemDecideHandler>,
        request: Option<&AccessoryShopChangeMenuRequestCloseHandler>,
        change: Option<&AccessoryShopChangeMenuChangeKindHandler>
    );
    pub fn get_menu_content(only_menu_content: bool) -> Option<&'static mut AccessoryShopChangeRoot> {
        AccessoryShopChangeRoot::load_prefab_async();
        if AccessoryShopChangeRoot::is_loading_prefab() { return None; }
        BasicMenuContent::get_canvas()
            .and_then(|canvas| ResourceManager::instantiate("UI/Hub/Shop/Prefabs/ShopAccChangeRoot", Some(canvas.get_transform())))
            .and_then(|x|{
                if only_menu_content {
                    for x in ["CharacterName", "EquipmentAcc", "WdwAccHelp", "KeyHelpAll"]{
                        if let Some(obj) = GameObject::find(x) { obj.destroy(); }
                    }
                }
            x.get_component_by_type::<AccessoryShopChangeRoot>()
        })
    }
    pub fn set_unit_name(name: &Il2CppString){
        if let Some(accessory_change) = GameObject::find("CharacterName").and_then(|v| v.get_component_in_parent::<AccessoryShopChangeRoot>(false)) {
            accessory_change.unit_name.set_text(name, true)
        }
    }
    pub fn photo_graph_bind<B: Bindable>(proc: &B) {
        let menu_data = UnitAssetMenuData::get();
        UnitAssetMenuData::init_photo_profiles();
        if let Some(content1) = Self::get_menu_content(true){
            if let Some(content) = content1.menu_object.get_component_by_type::<AccessoryShopChangeMenuContent>() {
                menu_data.mode = MenuMode::PhotoGraph;
                menu_data.is_preview = true;
                menu_data.is_shop_combat = false;
                keyhelp::start_key_help(OutfitMenuKind::Photo);
                let list = List::<CustomAssetMenuItem>::with_capacity(0).unwrap();
                let menu = CUSTOM_ASSET_MENU.get_or_init(|| Self::create_class()).instantiate_as::<CustomAssetMenu>().unwrap();
                menu.base_ctor(list, content, None, None, None, None, None);
                let count = 50;
                let klass = Il2CppClass::from_name("App", "BasicMenuSelect").expect("Unable to find BasicMenuSelect Class");
                menu.selects = Il2CppArray::new_from_element_class(klass, count).expect("Failed to create BasicMenuSelect[]");
                for i in 0..count {
                    menu.selects[i] = BasicMenuSelect::instantiate().unwrap();
                    menu.selects[i].index = 0;
                    menu.selects[i].scroll = 0;
                }
                menu.disable = false;
                menu.is_shop = false;
                menu.full_menu_item_list.clear();
                menu.kind = 0;
                menu.menu_kind = MainShop;
                MainShop.create_menu_items(menu);
                let descs = menu.create_default_desc();
                menu_data.control.initialize(MenuMode::PhotoGraph);
                menu.create_bind(proc, descs, "OutfitUnitMenu");
            }
        }
    }
    pub fn create_unit_info_bind<B: Bindable>(proc: &B, unit: &Unit) {
        let menu_data = UnitAssetMenuData::get();
        if let Some(content1) = Self::get_menu_content(false){
            adjust_menu_size(content1);
            if let Some(content) = content1.menu_object.get_component_by_type::<AccessoryShopChangeMenuContent>() {
                let content_trans = content.get_transform().get_position();
                let x_max = content_trans.x - 280.0;
                let mut x_min = 400.0;
                menu_data.mode = MenuMode::UnitInfo;
                menu_data.is_preview = true;
                menu_data.is_shop_combat = GameUserData::get_sequence() == 3;
                UnitAssetMenuData::set_unit(unit);
                content1.unit_name.set_text(unit.get_name(), true);
                keyhelp::start_key_help(OutfitMenuKind::UnitInfo);
                let list = List::<CustomAssetMenuItem>::with_capacity(0).unwrap();
                let menu = CUSTOM_ASSET_MENU.get_or_init(|| Self::create_class()).instantiate_as::<CustomAssetMenu>().unwrap();
                menu.base_ctor(list, content, Some(unit), None, None, None, None);
                let count = 50;
                let klass = Il2CppClass::from_name("App", "BasicMenuSelect").expect("Unable to find BasicMenuSelect Class");
                menu.selects = Il2CppArray::new_from_element_class(klass, count).expect("Failed to create BasicMenuSelect[]");
                for i in 0..count {
                    menu.selects[i] = BasicMenuSelect::instantiate().unwrap();
                    menu.selects[i].index = 0;
                    menu.selects[i].scroll = 0;
                }
                menu.full_menu_item_list.clear();
                menu.kind = 0;
                menu.menu_kind = MainShop;
                menu.disable = false;
                MainShop.create_menu_items(menu);
                BackgroundManager::bind();
                let descs = menu.create_default_desc();
                menu.create_bind(proc, descs, "OutfitUnitMenu");
                if let Some(go) = GameObject::find("EquipmentAcc"){
                    let trans = go.get_transform().get_position();
                    x_min = trans.x + 400.0;
                    if let Some(equipment) = go.get_component_by_type::<AccessoryEquipmentInfo>() {
                        equipment.build(unit);
                    }
                }
                let rows = if menu.full_menu_item_list.len() >= 12 { 12 } else { menu.full_menu_item_list.len() as i32 };
                menu.show_row_num = rows;
                menu.reserved_show_row_num = rows;

                UnitInfo::chara_only_on(false);
                if GameUserData::get_sequence() != 3 {
                    engage::unitinfo::UnitStatus::close();
                    if let Some(sortie) = SortieSequenceUnitSelect::get_instance() { sortie.window.unit_list_root.set_active(false); }
                }
                else { UnitInfo::chara_only_on(false); }
                let mask1 = UnitInfo::get_instance().unwrap().windows[0].unit_info_window_chara_model.render_texture;

                UnitAssetMenuData::get().menu_adj = 0.0;
                UnitInfoCharaImageMaskOffset::get_instance().iter_mut().for_each(|mask|{
                    if mask.texture.equal(mask1) && mask.is_visible() {
                        let mut pos = mask.rect.get_position();
                        if UnitAssetMenuData::get().menu_adj == 0.0 { UnitAssetMenuData::get().menu_adj = pos.x; }
                        unit_info_char_mask_setup(mask, false);
                        pos.x = (x_max + x_min) * 0.5;
                        mask.rect.set_position(pos);
                    }
                });
                menu_data.control.initialize(MenuMode::UnitInfo);
                EquipmentBoxMode::update(EquipmentBoxMode::CurrentProfile);
                Self::set_unit_name(unit.get_name());
            }
        }
    }
    pub fn create_class() -> &'static mut Il2CppClass {
        let klass = Il2CppClass::from_name("App", "AccessoryShopChangeMenu").unwrap().clone();
        let vtable = klass.get_vtable_mut();
        vtable[10].method_ptr = Self::on_dispose as _;
        vtable[24].method_ptr = Self::on_build as _;
        vtable[39].method_ptr = Self::tick_input as _;
        vtable[42].method_ptr = Self::key_left as _;
        vtable[43].method_ptr = Self::key_right as _;
        vtable[51].method_ptr = Self::b_call as _;
        vtable[56].method_ptr = Self::plus_call as _;
        klass
    }
    pub fn init1(this: &mut AccessoryShopChangeMenu, first: bool) {
        let custom_menu = unsafe { std::mem::transmute::<&mut AccessoryShopChangeMenu, &mut CustomAssetMenu>(this) };
        let count = 50;
        if first || custom_menu.selects.len() < 50 {
            custom_menu.klass = *CUSTOM_ASSET_MENU.get_or_init(|| Self::create_class());
            custom_menu.selects = Il2CppArray::new_from_element_class(BasicMenuSelect::class(), count).unwrap();
            for i in 0..count {
                custom_menu.selects[i] = BasicMenuSelect::instantiate().unwrap();
                custom_menu.selects[i].index = 0;
                custom_menu.selects[i].scroll = 0;
            }
        }
        else { custom_menu.save_current_select(); }
        for x in 1..count {
            custom_menu.selects[x] = BasicMenuSelect::instantiate().unwrap();
            custom_menu.selects[x].index = 0;
            custom_menu.selects[x].scroll = 0;
        }
        custom_menu.full_menu_item_list.clear();
        custom_menu.kind = 0;
        custom_menu.menu_kind = MainShop;
        custom_menu.is_shop = true;
        custom_menu.disable = false;
        custom_menu.is_photo = false;
        MainShop.create_menu_items(custom_menu);
        if !first { custom_menu.rebuild_menu(); }
    }
    pub fn init(this: &mut AccessoryShopChangeMenu, first: bool) {
        Self::init1(this, first);
        UnitAssetMenuData::get().mode = MenuMode::Shop;
        let custom_menu = unsafe { std::mem::transmute::<&mut AccessoryShopChangeMenu, &mut CustomAssetMenu>(this) };
        if let Some(request_close) = custom_menu.request_close.as_mut() {
            request_close.method_ptr = crate::shop::change_root::accessory_menu_on_close_menu as _;
        }
    }
    pub fn on_build(_this: &CustomAssetMenu, _optional_method: OptionalMethod) {
        if let Some(cat) = GameObject::find("Category").filter(|s| !s.is_null()) {
            cat.set_active(false);
            cat.destroy();
        }
    }
    pub fn plus_call(_this: &mut CustomAssetMenu, _optional_method: OptionalMethod) -> BasicMenuResult {
        if UnitAssetMenuData::is_unit_info() {
            if let Some(obj) = GameObject::find("CharacterName").and_then(|go| go.get_component_by_type::<Animator>()){
                let closed = obj.get_bool("isClosed");
                if closed { obj.play("Open"); }
                else {  obj.play("Close"); }
                if let Some(equip) = GameObject::find("EquipmentAcc").and_then(|go| go.get_component_by_type::<AccessoryEquipmentInfo>()) {
                    if closed { equip.open(); }
                    else { equip.close(); }
                }
                if let Some(detail_box) = GameObject::find("WdwAccHelp")
                    .and_then(|go| go.get_component_by_type::<AccessoryDetailInfoWindow>())
                    .and_then(|go| go.get_component::<Animator>() )
                {
                    if closed { detail_box.play("Open"); }
                    else {  detail_box.play("Close"); }
                }
                return BasicMenuResult::se_cursor();
            }
        }
        BasicMenuResult::new()
    }
    pub fn on_dispose(this: &mut CustomAssetMenu, _optional_method: OptionalMethod) {
        let menu = UnitAssetMenuData::get();
        TitleBar::close_header();
        menu.control.reset_all();
        match menu.mode {
            MenuMode::UnitInfo => {
                menu.is_preview = false;
                let mask1 = UnitInfo::get_instance().unwrap().windows[0].unit_info_window_chara_model.render_texture;
                UnitInfoCharaImageMaskOffset::get_instance().iter_mut().for_each(|mask|{
                    if mask.texture.equal(mask1) && mask.is_visible() {
                        let mut pos = mask.rect.get_position();
                        unit_info_char_mask_setup(mask, true);
                        pos.x = UnitAssetMenuData::get().menu_adj;
                        mask.rect.set_position(pos);
                    }
                });
                for x in ["CharacterName", "EquipmentAcc", "WdwAccHelp"]{ if let Some(obj) = GameObject::find(x) { obj.destroy(); } }
                let parent_proc = this.proc.parent.as_ref();
                if let Some(parent) = parent_proc {
                    if let Some(method) = parent.klass.get_virtual_method("OpenAnimeAll") {
                        let on_open_anime = unsafe { std::mem::transmute::<_, fn(&ProcInst, &MethodInfo)>(method.method_ptr)};
                        on_open_anime(parent, method.method_info);
                    }
                }
                AccessoryShopChangeRoot::unload_prefab();
                UnitInfo::set_unit(UnitInfoSide::Left, None, false, false, false, None);
                UnitInfo::chara_only_off();
                if GameUserData::get_sequence() == 3 {
                    UnitInfo::set_unit(UnitInfoSide::Left, engage::map::mind::MapMind::get_unit().as_deref(), false, false, false, None);
                }
                if let Some(sortie) = SortieSequenceUnitSelect::get_instance(){
                    sortie.disp_all();
                    sortie.window.unit_list_root.set_active(true);
                    let unit = SortieSelectionUnitManager::get_unit();
                    let current_select = sortie.select_menu.get_select_index();

                    sortie.select_menu.set_select_index_from_unit(unit);
                    let new_select = sortie.select_menu.get_select_index();
                    if new_select != current_select {
                        if let Some(item) = sortie.select_menu.get_item(current_select) { item.on_deselect(); }
                    }
                    sortie.select_menu.adjust_scroll_index();
                    sortie.select_menu.scroll_instant();
                    UnitInfo::set_unit(UnitInfoSide::Left, Some(unit), false, false, false, None);
                }
                BackgroundManager::unbind();
            }
            MenuMode::PhotoGraph => {
                let parent_proc = this.proc.parent.as_ref();
                if let Some(parent) = parent_proc {
                    if let Some(method) = parent.klass.get_virtual_method("OpenAnimeAll") {
                        let on_open_anime = unsafe { std::mem::transmute::<_, fn(&ProcInst, &MethodInfo)>(method.method_ptr) };
                        on_open_anime(parent, method.method_info);
                    }
                }
                KeyHelp::set_visible(true);
                menu.is_preview = false;
            }
            _ => {}
        }
    }
    pub fn rebuild_menu(&mut self) {
        let rows = if self.full_menu_item_list.len() >= 12 { 12 } else { self.full_menu_item_list.len() as i32 };
        self.show_row_num = rows;
        self.reserved_show_row_num = rows;
        let select = BasicMenuSelect::instantiate().unwrap();
        let (index, scroll) = self.menu_kind.get_save_select_index().map(|v|(self.selects[v].index, self.selects[v].scroll)).unwrap_or((0, 0));
        select.index = index;
        select.scroll = scroll;
        self.rebuild_instant2(select);
        self.after_build();
        self.restore_select(select);
        if self.menu_kind == MainShop { self.kind = 0; } else { self.kind = 1; }
    }
    pub fn b_call(this: &mut CustomAssetMenu, _method_info: OptionalMethod) -> BasicMenuResult {
        this.menu_kind.b_call();
        if let Some(previous) = this.menu_kind.get_previous() {
            this.save_current_select();
            this.full_menu_item_list.clear();
            previous.create_menu_items(this);
            this.menu_kind = previous;
            this.kind = if previous == MainShop { 0 } else { 1 };
            this.rebuild_menu();
            BasicMenuResult::new().with_se_cancel(true)
        }
        else {
            UnitAssetMenuData::commit();
            if !UnitAssetMenuData::is_unit_info() {
                if let Some(request_close) = this.request_close.as_ref() {
                    request_close.invoke();
                }
            }
            BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
        }
    }
    pub fn save_current_select(&mut self){
        if let Some(i) = self.menu_kind.get_save_select_index() {
            let scroll = self.scroll_index;
            let select = self.select_index;
            self.selects[i].index = select;
            self.selects[i].scroll = scroll;
        }
    }
    pub fn minus_call(this: &mut CustomAssetMenu, _method_info: OptionalMethod) -> BasicMenuResult {
        let select_index = this.select_index as usize;
        this.full_menu_item_list.get_mut(select_index)
            .map(|v|  CustomAssetMenuItem::minus_call(v, None)).unwrap_or(BasicMenuResult::new() )
    }
    pub fn key_right(this: &mut CustomAssetMenu, trigger: bool, _method_info: OptionalMethod) {
        let pad = get_instance::<Pad>();
        if !UnitAssetMenuData::is_shop() && (pad.npad_state.buttons.stick_l_right() || pad.npad_state.buttons.stick_r_right()) { return; }
        Self::key_base(this, trigger, true);
    }
    pub fn key_left(this: &mut CustomAssetMenu, trigger: bool, _method_info: OptionalMethod) {
        let pad = get_instance::<Pad>();
        if !UnitAssetMenuData::is_shop() && (pad.npad_state.buttons.stick_l_left() || pad.npad_state.buttons.stick_r_left()) { return; }
        Self::key_base(this, trigger, false);
    }
    fn key_base(this: &mut CustomAssetMenu, trigger: bool, right_key: bool) {
        if trigger {
            let new_menu = if right_key { this.menu_kind.get_right() } else { this.menu_kind.get_left() };
            if let Some(new_menu) = new_menu {
                this.save_current_select();
                this.full_menu_item_list.clear();
                new_menu.create_menu_items(this);
                this.menu_kind = new_menu;
                this.rebuild_menu();
                GameSound::post_event("Category_Change", None);
            }
        }
    }
    fn lr_base(this: &mut CustomAssetMenu, right: bool) {
        if let Some(select) = SortieSelectionUnitManager::get_instance() {
            let unit = SortieSelectionUnitManager::get_unit();
            let next = if right { SortieUtil::get_next_unit_loop(unit) } else { SortieUtil::get_prev_unit_loop(unit) };
            UnitAssetMenuData::commit();
            UnitInfo::set_unit(UnitInfoSide::Left, Some(next), false, false, false, None);
            SortieSelectionUnitManager::set_unit(select, next);
            UnitAssetMenuData::set_unit(next);
            this.rebuild_menu();
            let sequence = GameUserData::get_sequence();
            let result = if (1 << sequence) & 76 != 0 { AssetTableResult::get_for_unit_info(next) }
            else { AssetTableResult::get_for_accessory(unit) };
            result.left_hand = "null".into();
            result.right_hand = "null".into();
            result.body_anim = result.hub_anims;
            hub_room_set_by_result(Some(result), ReloadType::All);
            EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets).update();
            if let Some(char_name) = GameObject::find("CharacterName").and_then(|v| v.get_component_in_children::<TextMeshProUGUI>(true)){
                char_name.set_text(next.get_name(), true);
            }
            GameSound::post_event("Chara_Change", None);
        }
    }
    fn can_facial(&self) -> bool {
        if self.menu_kind.can_facial() {
            self.full_menu_item_list.get(self.select_index as usize).is_some_and(|v| v.menu_kind.can_facial())
        }
        else { false }
    }
    fn tick_input(this: &mut CustomAssetMenu, optional_method: OptionalMethod) -> bool {
        let left = Pad::is_trigger(NpadButton::new().with_left(true));
        let right = Pad::is_trigger(NpadButton::new().with_right(true));
        if (left || right) && left != right {
            if this.can_facial() || this.disable {
                hub_room_set_by_result(None, ReloadType::Facial(right));
                GameSound::post_event("Category_Change", None);
            }
        }
        if !this.is_shop {
            let stick = model_camera_control();
            let trigger = Pad::is_trigger(NpadButton::new().with_plus(true).with_b(true));
            if this.disable {
                if trigger {
                    if let Some(obj) = this.menu_content.get_game_object() { obj.set_active2(true); }
                    Self::plus_call(this, optional_method);
                    this.disable = false;
                    TitleBar::show_header();
                    add_key_help(KeyHelpButton::Plus, "Hide");
                }
                else if Pad::is_trigger(NpadButton::new().with_x(true)) {
                    let title = TitleBar::get_instance();
                    if title.is_show_header { TitleBar::hide_header(); }
                    else { TitleBar::show_header(); }
                }
                return true;
            }
            else if Pad::is_trigger(NpadButton::new().with_plus(true)){
                Self::plus_call(this, optional_method);
                if let Some(obj) = this.menu_content.get_game_object() { obj.set_active2(false); }
                this.disable = true;
                add_key_help(KeyHelpButton::Plus, Mess::get("MID_KEYHELP_MENU_UI_HIDE").to_string().as_str());
                return true;
            }
            if stick { return true; }
            if this.menu_kind == MainShop && UnitAssetMenuData::is_unit_info() {
                let l = Pad::is_trigger(NpadButton::new().with_l(true));
                let r = Pad::is_trigger(NpadButton::new().with_r(true));
                if (l || r) && l != r { Self::lr_base(this, r); }
            }
        }
        this.tick_input_base()
    }
}
fn model_camera_control() -> bool {
    let menu_data = UnitAssetMenuData::get();
    let pad = get_instance::<Pad>();
    let fast = pad.npad_state.buttons.y();
    let mut translation_change: [i32; 3] = [0; 3];
    let r_stick = Pad::is_trigger(NpadButton::new().with_stick_r(true));

    let rotation_y =
        if pad.npad_state.buttons.stick_r_left() { if fast { -7.5 } else { -2.5 } }
        else if pad.npad_state.buttons.stick_r_right() { if fast { 7.5 } else { 2.5 } }
        else { 0.0 };

    if pad.npad_state.buttons.stick_r_up() { translation_change[2] = -1; }
    else if pad.npad_state.buttons.stick_r_down() { translation_change[2] = 1; }

    if pad.npad_state.buttons.stick_l_left() { translation_change[0] = 1; }
    else if pad.npad_state.buttons.stick_l_right() { translation_change[0] = -1; }

    if pad.npad_state.buttons.stick_l_down() { translation_change[1] = -1; }
    else if pad.npad_state.buttons.stick_l_up() { translation_change[1] = 1; }

    if fast { for x in 0..3 { translation_change[x] *= 3; } }

    if r_stick {
        menu_data.control.reset_character_position();
        menu_data.control.reset_character_rotation();
    }
    if rotation_y != 0.0 { menu_data.control.character_rotation(0.0, rotation_y, 0.0); }
    let rl_stick = translation_change.iter().any(|&x| x != 0) || rotation_y != 0.0;
    match menu_data.mode {
        MenuMode::UnitInfo => { menu_data.control.translate_character(translation_change); }
        MenuMode::PhotoGraph => {
            let mut rot_x = 0.0;
            let mut rot_z = 0.0;
            if pad.npad_state.buttons.zl() { rot_x = -1.25; }
            else if pad.npad_state.buttons.zr() { rot_x = 1.25; }

            if pad.npad_state.buttons.l() { rot_z = -1.25; }
            else if pad.npad_state.buttons.r() { rot_z = 1.25; }

            if rot_x != 0.0 || rot_z != 0.0 { menu_data.control.camera_rotation(rot_x, 0.0, rot_z); }

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
fn adjust_menu_size(content: &AccessoryShopChangeRoot) {
    let transform = content.get_transform();
    if let Some(t) = Kaneko::find_in_children(transform, "WdwAccChange".into()){
        if let Some(rect) = Kaneko::find_in_children(t, "Content".into()) {
            for x in 0..13 {
                if let Some(acc) = Kaneko::find_in_children(rect, format!("Acc{}", x).into()) {
                    let acc_rect = acc.to_rect_transform();
                    if let Some(icon) = acc_rect.get_child(1) {
                        icon.translate_local(-10.0, 0.0, 0.0);
                    }
                    if let Some(name) = acc_rect.get_child(2) {
                        name.translate_local(-10.0, 0.0, 0.0);
                        name.change_size(10.0, 0.0);
                    }
                    acc.get_components_in_children_gen::<TextMeshProUGUI>(true).iter_mut().for_each(|t|{
                        t.m_min_font_size = 22.0;
                        t.m_max_font_size = 24.0;
                        t.m_font_size_max = 24.0;
                        t.m_font_size_min = 22.0;
                    });
                }
            }
        }
    }
    if let Some(t) = Kaneko::find_in_children(transform, "EquipmentAcc".into()) {
        let rect = t.to_rect_transform();
        rect.change_size(60.0, 0.0);
        for x in 0..6 {
            if let Some(acc) = Kaneko::find_in_children(transform, format!("Acc{}", x).into()) {
                let rect = acc.to_rect_transform();
                rect.change_size(60.0, 0.0);

                if let Some(name) = rect.get_child(2) { name.change_size(60.0, 0.0); }
                acc.get_components_in_children_gen::<TextMeshProUGUI>(true).iter_mut().for_each(|t|{
                    t.m_min_font_size = 22.0;
                    t.m_max_font_size = 24.0;
                    t.m_font_size_max = 24.0;
                    t.m_font_size_min = 22.0;
                });
            }
        }
    }
}

fn unit_info_char_mask_setup(mask: &mut UnitInfoCharaImageMaskOffset, revert: bool) {
    mask.texture = UnitInfo::get_render_texture(UnitInfoSide::Left);
    if revert {
        mask.rect.set_size_delta(Vector2{ x: 640.0, y: 1080.0 });
        if mask.texture.get_width() != 640 {
            mask.texture.release();
            mask.texture.set_width(640);
            mask.texture.create();
            mask.update_camera(UnitInfoSide::Left);
            mask.text_old = mask.material.get_texture("_MaskTex".into());
        }
    }
    else {
        mask.rect.set_size_delta(Vector2{ x: 1500.0, y: 1080.0 });
        if mask.texture.get_width() != 1500 {
            mask.texture.release();
            mask.texture.set_width(1500);
            mask.texture.create();
            mask.update_camera(UnitInfoSide::Left);
            mask.text_old = mask.material.get_texture("_MaskTex".into());
        }
    }
}