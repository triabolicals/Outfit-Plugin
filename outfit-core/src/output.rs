use engage::gamedata::assettable::{AssetTableResult, AssetTableStaticFields};
use crate::{OUTPUT_ASSET_TABLE_DIR};
use super::*;
use std::io::Write;
use std::path::Path;

pub fn get_next_filename(dir: &str, stem: &String, ext: &str) -> String {
	let mut file_path = format!("{}{}.{}", dir, stem, ext);
	if Path::new(file_path.as_str()).exists() {
		let mut c = 1;
		loop {
			file_path = format!("{}{}-{}.{}", dir, stem, c, ext);
			if Path::new(file_path.as_str()).exists() { c += 1; }
			else { break; }
		}
	}
	file_path
}
pub fn output_unit_result(preview: bool) -> (String, String, bool) {
	let person_hash = UnitAssetMenuData::get_preview().person;
	let emblem = UnitAssetMenuData::get().god_mode;
	let name = if emblem {
		GodData::try_get_hash(person_hash)
			.map(|v| format!("{}", Mess::get(v.mid)))
			.unwrap_or(format!("Emblem_{}", person_hash))
	}
	else {
		UnitAssetMenuData::get_shop_unit()
			.map(|v| format!("{}", v.get_name()))
			.unwrap_or(format!("Unit_{}", person_hash))
	};
	let filename1 = get_next_filename(INPUT_DIR, &name, "txt");
	if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(filename1.as_str()){
		writeln!(file, "Outfit Plugin\nVersion: {}", UnitAssetData::version()).unwrap();
		if preview { writeln!(file, "Preview").unwrap(); }
		writeln!(file, "{}", UnitAssetMenuData::get_preview().preview_data.out(person_hash, preview)).unwrap();
	}
	let filename = get_next_filename(OUTPUT_ASSET_TABLE_DIR, &name, "txt");
	if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(filename.as_str()){
		let entry = result_to_string(UnitAssetMenuData::get_result(), 2);
		writeln!(&mut file, "Mode: {}", 2).unwrap();
		writeln!(&mut file, "{}\n", entry).unwrap();
		return (filename, filename1, true);
	}
	(filename, filename1, false)
}
pub fn result_to_string(result: &AssetTableResult, mode: i32) -> String {
	let bits = &AssetTableStaticFields::get().condition_flags;
	let bit_size = bits.bits.bits.len() * 8;
	let mut conditions = String::new();
	for xx in 0..bit_size {
		if bits.bits.get(xx as i32) {
			if let Some(con) = AssetTableStaticFields::get().condition_indexes.entries.iter().find(|x| x.value == xx as i32) {
				conditions.push_str(&con.key.unwrap().to_string());
				conditions.push_str(";");
			}
		}
	}
	let mut out = format!("			<Param Out=\"\" PresetName=\"\" Mode=\"{}\" Conditions=\"{}\" BodyModel=", mode, conditions);
	if result.body_model.is_null() { out.push_str("\"\" DressModel="); } else { out.push_str(format!("\"{}\" DressModel=", result.body_model).as_str()) };
	if result.dress_model.is_null() { out.push_str("\"\" "); } else { out.push_str(format!("\"{}\" ", result.dress_model).as_str()) };
	for x in 0..4 {
		let r = if result.unity_colors[4+x].r >= 1.0 { 255 } else { ( result.unity_colors[4+x].r * 255.0 ) as u8 };
		let g = if result.unity_colors[4+x].g >= 1.0 { 255 } else { ( result.unity_colors[4+x].g * 255.0 ) as u8 };
		let b = if result.unity_colors[4+x].b >= 1.0 { 255 } else { ( result.unity_colors[4+x].b * 255.0 ) as u8 };
		let color_str = format!("{}R=\"{}\" {}G=\"{}\" {}B=\"{}\" ", COLOR_MASK[x], r, COLOR_MASK[x], g, COLOR_MASK[x], b);
		out.push_str(&color_str);
	}
	if result.head_model.is_null() { out.push_str("HeadModel=\"\" HairModel="); } else { out.push_str(format!("HeadModel=\"{}\" HairModel=", result.head_model).as_str()) };
	if result.hair_model.is_null() { out.push_str("\"\" "); } else { out.push_str(format!("\"{}\" ", result.hair_model).as_str()); }
	for x in 0..4 {
		let r = if result.unity_colors[x].r >= 1.0 { 255 } else { ( result.unity_colors[x].r * 255.0 ) as u8 };
		let g = if result.unity_colors[x].g >= 1.0 { 255 } else { ( result.unity_colors[x].g * 255.0 ) as u8 };
		let b = if result.unity_colors[x].b >= 1.0 { 255 } else { ( result.unity_colors[x].b * 255.0 ) as u8 };
		let color_str = format!("{}R=\"{}\" {}G=\"{}\" {}B=\"{}\" ", COLOR_MASK[4+x], r, COLOR_MASK[4+x], g, COLOR_MASK[4+x], b);
		out.push_str(&color_str);
	}
	if result.ride_model.is_none()  { out.push_str("RideModel=\"\" RideDressModel="); } else { out.push_str(format!("RideModel=\"{}\" RideDressModel=", result.ride_model.unwrap()).as_str()) };
	if result.ride_dress_model.is_none() { out.push_str("\"\" LeftHand="); } else { out.push_str(format!("\"{}\" LeftHand=", result.ride_dress_model.unwrap()).as_str()) };
	if result.left_hand.is_null() { out.push_str("\"\" RightHand="); } else { out.push_str(format!("\"{}\" RightHand=", result.left_hand).as_str()) };
	if result.right_hand.is_null() { out.push_str("\"\" Trail="); } else { out.push_str(format!("\"{}\" Trail=", result.right_hand).as_str()) };
	if result.trail.is_null() { out.push_str("\"\" Magic="); } else { out.push_str(format!("\"{}\" Magic=", result.trail).as_str()) };
	if result.magic.is_null() { out.push_str("\"\" "); } else { out.push_str(format!("\"{}\" ", result.magic).as_str()) };
	let mut count = 0;
	let dic_count = result.accessory_dictionary.get_count();
	if dic_count > 0 {
		result.accessory_list.list.iter().flat_map(|s| s.locator.zip(s.model) ).for_each(|(loc, model)|{
			if count < 8 {
				count += 1;
				out.push_str(format!("Acc{}.Locator=\"{}\" Acc{}.Model=\"{}\" ", count, loc, count, model).as_str());
			}
		});
	}

	if count < 8 { for x in count..8 { out.push_str(format!("Acc{}.Locator=\"\" Acc{}.Model=\"\" ", x+1, x+1).as_str()); } }
	out.push_str(format!("BodyAnim=\"{}\" ", result.body_anim.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	out.push_str(format!("InfoAnim=\"{}\" ", result.info_anims.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	out.push_str(format!("TalkAnim=\"{}\" ", result.talk_anims.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	out.push_str(format!("DemoAnim=\"{}\" ", result.demo_anims.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	out.push_str(format!("HubAnim=\"{}\" ", result.hub_anims.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	for x in 0..9 { out.push_str(format!("{}=\"{:.2}\" ", SCALE[x], result.scale_stuff[x]).as_str()); }
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[9], result.scale_stuff[14]).as_str());   // VolumeArm -> VolumeBaseArms
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[10], result.scale_stuff[15]).as_str());
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[11], result.scale_stuff[9]).as_str());
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[12], result.scale_stuff[10]).as_str());
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[13], result.scale_stuff[11]).as_str());
	for x in 14..19 {
		out.push_str(format!("{}=\"{:.2}\" ", SCALE[x], result.scale_stuff[x]).as_str());
	}
	out.push_str(format!("Voice=\"{}\" ", result.sound.voice.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	out.push_str(format!("FootStep=\"{}\" ", result.sound.footstep.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	out.push_str(format!("Material=\"{}\" ", result.sound.material.map(|x| x.to_string()).unwrap_or("".to_string())).as_str());
	out.push_str("Comment=\"Generated by the Unit Asset/Outfit Plugin\" />\n");
	out.push_str("Body Anim List\n");
	result.body_anims.iter().for_each(|anim| {out.push_str(format!("\t{}\n", anim).as_str())});
	out
}