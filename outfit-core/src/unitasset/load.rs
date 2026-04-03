use std::path::PathBuf;
use crate::EquipmentBoxPage;
use super::*;

pub struct UnitAssetLoadData {
    pub data: PlayerOutfitData,
    pub path: PathBuf,
}
impl UnitAssetLoadData {
    pub fn new(data: PlayerOutfitData, path: PathBuf) -> Self { Self { data, path } }
    pub fn get_filename(&self) -> String { self.path.file_name().unwrap().to_str().unwrap().to_string() }
}

pub struct UnitAssetLoader {
    pub loaded_data: Vec<UnitAssetLoadData>,
    pub selected_index: Option<i32>,
    pub profile: i32,
    pub equipment_box_state: EquipmentBoxPage,
}
impl UnitAssetLoader {
    pub const fn new() -> Self {
        Self { loaded_data: vec![], profile: 0, selected_index: None, equipment_box_state: EquipmentBoxPage::Flags, }
    }
    pub fn get_selected_data(&self) -> Option<&UnitAssetLoadData> {
        self.loaded_data.get(self.selected_index? as usize)
    }
    pub fn set_profile(&mut self, profile: i32) { self.profile = profile; }
    pub fn load_files(&mut self, gender_restrict: Gender) -> LoadResult {
        self.loaded_data.clear();
        self.selected_index = None;
        let path = std::path::Path::new(crate::INPUT_DIR);
        if let Ok(dir) = read_dir(path) {
            dir.filter_map(|f| f.ok().filter(|f| f.path().is_file() && read_to_string(f.path()).is_ok())) //.is_ok_and(|f| f.starts_with("#Outfit Plugin"))))
            .for_each(|file| {
                if let Some(load_data) = PlayerOutfitData::try_load_from_file(&file, Some(gender_restrict)){
                    self.loaded_data.push(UnitAssetLoadData::new(load_data, file.path().to_path_buf()));
                }
            });
            if self.loaded_data.len() == 0 { LoadResult::NoFiles } else { LoadResult::Success }
        }
        else { LoadResult::MissingDirectory }
    }
}