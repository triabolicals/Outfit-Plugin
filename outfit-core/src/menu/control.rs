use engage::{
    app::{ISingletonProcInst_1Methods, IUnitInfo, IUnitInfoWindowCharaModel, IUnitInfo_Window},
    unity_engine::{IComponentMethods, ITransformMethods},
    app::{IPhotographCameraController, IPhotographDisposInfo, IPhotographDisposManager, IPhotographSequence},
    combat::ICharacterJointMethods,
    unity_engine::{ICameraMethods, IGameObjectMethods, IRendererMethods, Renderer}
};
use unity::Cast;
use crate::{clamp_value, MenuMode};

pub struct PositionRotation {
    pub pos: engage::unity_engine::Vector3,
    pub rotation: engage::unity_engine::Quaternion,
}
impl PositionRotation {
    pub const fn default() -> Self {
        Self {
            pos: engage::unity_engine::Vector3 {x: 0.0, y: 0.0, z: 0.0 },
            rotation: engage::unity_engine::Quaternion{x: 0.0, y: 0.0, z: 0.0, w: 1.0},
        }
    }
    pub fn new2(pos: engage::unity_engine::Vector3, rotation: engage::unity_engine::Quaternion) -> Self {
        Self { pos, rotation }
    }
    pub fn from_transform(transform: engage::unity_engine::Transform, local_rotation: bool) -> Self {
        let pos = transform.get_position();
        let rot = if local_rotation { transform.get_local_rotation() } else { transform.get_rotation() };
        Self::new2(pos, rot)
    }
}
pub struct PhotoCameraControl {
    pub current_character: PositionRotation,
    pub current_camera: PositionRotation,
    pub init_camera: PositionRotation,
    reset_camera: PositionRotation,
    reset_character: PositionRotation,
    pub rotation_change: engage::unity_engine::Vector3,
    character_basis: [engage::unity_engine::Vector3; 3],
    camera_fov: f32,
    cam_translation: [i32; 3],
    cam_bounds: [i32; 3],
    y_min: f32,
    z_init: i32,
    mode: MenuMode,
}
impl PhotoCameraControl {
    pub const fn default() -> Self {
        Self {
            current_character: PositionRotation::default(),
            current_camera: PositionRotation::default(),
            init_camera: PositionRotation::default(),
            reset_camera: PositionRotation::default(),
            reset_character: PositionRotation::default(),
            rotation_change: engage::unity_engine::Vector3{x: 0.0, y: 0.0, z: 0.0},
            character_basis: [
                engage::unity_engine::Vector3{x: 0.0, y: 0.0, z: 0.0},
                engage::unity_engine::Vector3{x: 0.0, y: 0.0, z: 0.0},
                engage::unity_engine::Vector3{x: 0.0, y: 0.0, z: 0.0},
            ],
            camera_fov: 60.0,
            cam_translation: [0; 3],
            cam_bounds: [0; 3],
            y_min: 0.0,
            z_init: 0,
            mode: MenuMode::Inactive,
        }
    }
    pub fn setup(&mut self, position: bool, bounds: bool) {
        if self.mode == MenuMode::PhotoGraph {
            if !position && !bounds { return; }
            if let Some(p) = crate::photo::get_photosequence(){
                if bounds {
                    let mut size: [f32; 3] = [0.0; 3];
                    let char = p.m_dispos_manager().m_current_dispos_info().m_locator().get_component_in_children_3::<engage::combat::Character>();
                    if !char.is_null() {
                        let trans = char.get_transform();
                        if position {
                            self.reset_character = PositionRotation::from_transform(trans, true);
                            self.current_character = PositionRotation::from_transform(trans, true);
                            self.character_basis = [trans.get_right(), trans.get_up(), trans.get_forward()]
                        }
                        let char_trans = trans.get_position();
                        if bounds {
                            self.y_min = char_trans.y;
                            char.get_components_in_children::<Renderer>(true).iter().for_each(|p|{
                                let bounds = p.get_bounds();
                                if size[0] < bounds.m_extents.x { size[0] = bounds.m_extents.x; }
                                if size[1] < bounds.m_extents.y { size[1] = bounds.m_extents.y; }
                                if size[2] < bounds.m_extents.z { size[2] = bounds.m_extents.z; }
                            });
                            self.cam_bounds[0] = (( 1.5 * size[0]) / 0.015 ) as i32;
                            self.cam_bounds[1] = (( 1.5 * size[1]) / 0.015 ) as i32;
                            self.cam_bounds[2] = (( 3.0 * size[2]) / 0.015 ) as i32;
                        }
                    }
                }
                let camera = engage::unity_engine::Camera::get_main();
                if !camera.is_null(){
                    let camera_trans = camera.get_transform();
                    let aspect = camera.get_aspect();
                    let fov_rad = (self.camera_fov.to_radians() * 0.5).tan();
                    p.m_camera_controller().set_m_is_enable(false);
                    let p_trans = p.m_camera_controller().m_current_parameter().get_transform();
                    if !position {
                        self.reset_character_rotation();
                        self.reset_camera_rotation();
                    }
                    let joint =  p.m_dispos_manager().m_current_dispos_info().m_locator().get_component_in_children_3::<engage::combat::CharacterJoint>();
                    if !joint.is_null() {
                        let trans = joint.get_c_head_loc();
                        let head_pos = trans.get_position();
                        let mut cam_pos = trans.get_position();
                        let distance = (head_pos.y - self.y_min + 0.5) * fov_rad;
                        let distance_x = aspect * distance * 250.0 / 960.0;
                        cam_pos.x += distance * self.character_basis[2].x;
                        cam_pos.y += distance * self.character_basis[2].y;
                        cam_pos.z += distance * self.character_basis[2].z;
                        self.z_init = (distance / 0.015) as i32;
                        if self.z_init < 0 { self.z_init *= -1; }
                        camera_trans.set_position(cam_pos);
                        p_trans.set_position(cam_pos);
                        p_trans.look_at_2(trans);
                        camera_trans.look_at_2(trans);
                        let right = camera_trans.get_right();
                        cam_pos.x += 0.5*distance_x * right.x;
                        cam_pos.y += 0.5*distance_x * right.y;
                        cam_pos.z += 0.5*distance_x * right.z;
                        self.cam_translation[0] = 0;
                        self.cam_translation[1] = 0;
                        self.cam_translation[2] = self.z_init;
                        camera_trans.set_position(cam_pos);
                        p_trans.set_position(cam_pos);
                        let v = camera_trans.get_local_rotation();
                        if position { self.init_camera = PositionRotation::new2(cam_pos, v); }
                        self.current_camera = PositionRotation::new2(cam_pos, v);
                    }
                }
            }
        }
    }
    pub fn initialize(&mut self, mode: MenuMode) {
        self.mode = mode;
        self.character_basis[0] = engage::unity_engine::Vector3{x: 0.0, y: 0.0, z: 0.0};
        self.character_basis[1] = engage::unity_engine::Vector3{x: 0.0, y: 0.0, z: 0.0};
        self.character_basis[2] = engage::unity_engine::Vector3{x: 0.0, y: 0.0, z: 0.0};
        self.cam_translation[0] = 0;
        self.cam_translation[1] = 0;
        self.cam_translation[2] = 0;
        match self.mode {
            MenuMode::PhotoGraph => {
                let camera = engage::unity_engine::Camera::get_main();
                if !camera.is_null() {
                    let camera_trans = camera.get_transform();
                    let fov = camera.get_field_of_view();
                    self.camera_fov = fov;
                    self.reset_camera = PositionRotation::new2(camera_trans.get_position(), camera_trans.get_rotation());
                    self.setup(true, true);
                }
            }
            MenuMode::UnitInfo => {
                let transform = engage::app::UnitInfo::get_instance().m_windows().get(0).m_unit_info_window_chara_model().m_chara().get_transform();
                self.reset_character = PositionRotation::from_transform(transform, true);
                self.current_character = PositionRotation::from_transform(transform, true);
            }
            _ => {}
        }
    }
    pub fn reset_camera_rotation(&mut self) {
        if let Some(para) = self.get_camera_parameter_transform() {
            let rot = self.rotation_change;
            let x = -1.0*rot.x;
            let y = -1.0*rot.y;
            let z = -1.0*rot.z;
            para.rotate_3(x, y,z, engage::unity_engine::Space::self_());
            let camera = engage::unity_engine::Camera::get_main();
            if !camera.is_null() {
                let transform = camera.get_transform();
                transform.rotate_3(x, y, z, engage::unity_engine::Space::self_());
                self.rotation_change.x = 0.0;
                self.rotation_change.y = 0.0;
                self.rotation_change.z = 0.0;

            }
        }
    }
    /// Rotates Camera with angles in degrees between -30 to 30 degrees
    pub fn camera_rotation(&mut self, x: f32, y: f32, z: f32) {
        if let Some(para) = self.get_camera_parameter_transform() {
            if self.camera_rotation_check(x, y, z){
                para.rotate_3(x, y, z, engage::unity_engine::Space::self_());
                let camera = engage::unity_engine::Camera::get_main();
                if !camera.is_null() {
                    let transform = camera.get_transform();
                    transform.rotate_3(x, y, z, engage::unity_engine::Space::self_());
                }
                self.rotation_change.x = wrap_angle(self.rotation_change.x + x);
                self.rotation_change.y = wrap_angle(self.rotation_change.y + y);
                self.rotation_change.z = wrap_angle(self.rotation_change.z + z);
            }
        }
    }
    pub fn reset_all(&mut self) {
        if let Some(para) = self.get_camera_parameter_transform() {
            para.set_local_rotation(self.reset_camera.rotation);
            para.set_position(self.reset_camera.pos);
        }
        let camera = engage::unity_engine::Camera::get_main();
        if !camera.is_null() {
            let trans = camera.get_transform();
            trans.set_local_rotation(self.reset_camera.rotation);
            trans.set_position(self.reset_camera.pos);
        }
        if let Some(char) = self.get_character_transform() {
            char.set_local_rotation(self.reset_character.rotation);
            char.set_position(self.reset_character.pos);
        }
        self.rotation_change.x = 0.0;
        self.rotation_change.y = 0.0;
        self.rotation_change.z = 0.0;
    }
    /// Rotates Character with angles in degrees
    pub fn character_rotation(&mut self, x: f32, y: f32, z: f32) {
        if let Some(transform) = self.get_character_transform() {
            transform.rotate_3(x, y, z, engage::unity_engine::Space::self_());
            let q = transform.get_local_rotation();
            self.current_character.rotation.x = q.x;
            self.current_character.rotation.y = q.y;
            self.current_character.rotation.z = q.z;
            self.current_character.rotation.w = q.w;
        }
    }
    pub fn reset_character_rotation(&mut self) {
        if let Some(transform) = self.get_character_transform() {
            transform.set_local_rotation(self.reset_character.rotation);
            self.current_character.rotation = transform.get_local_rotation();
        }
    }
    pub fn reset_character_position(&mut self) {
        if let Some(transform) = self.get_character_transform() {
            transform.set_position(self.reset_character.pos);
            self.current_character.pos = self.reset_character.pos;
        }
    }
    pub fn reset_camera_position(&mut self) {
        if let Some(para) = self.get_camera_parameter_transform() {
            para.set_position(self.init_camera.pos);
            let camera = engage::unity_engine::Camera::get_main();
            if !camera.is_null() {
                let t = camera.get_transform();
                t.set_position(self.init_camera.pos)
            }
            self.current_camera.pos = self.init_camera.pos;
            self.cam_translation[0] = 0;
            self.cam_translation[1] = 0;
            self.cam_translation[2] = self.z_init;
        }
    }
    pub fn update_character_pos_rot(&self) {
        if let Some(transform) = self.get_character_transform() {
            transform.set_position(self.current_character.pos);
            transform.set_local_rotation(self.current_character.rotation);
        }
    }
    pub fn get_character_transform(&self) -> Option<engage::unity_engine::Transform> {
        match self.mode {
            MenuMode::PhotoGraph => {
                crate::photo::get_photosequence().map(|p| p.m_dispos_manager().m_current_dispos_info().m_character_cmp().get_transform())
            }
            MenuMode::UnitInfo => {
                Some(engage::app::UnitInfo::get_instance().m_windows().get(0).m_unit_info_window_chara_model().m_chara().get_transform())
            }
            _ => None,
        }
    }
    pub fn get_main_camera() -> Option<engage::unity_engine::Camera> {
        let camera = engage::unity_engine::Camera::get_main();
        if !camera.is_null() { Some(camera) } else { None }
    }
    pub fn get_camera_parameter_transform(&self) -> Option<engage::unity_engine::Transform> {
        crate::photo::get_photosequence().map(|p| p.m_camera_controller().m_current_parameter().get_transform())
    }
    pub fn translate_camera(&mut self, r: [i32; 3]) {
        if r[0] == 0 && r[1] == 0 && r[2] == 0 { return; }
        if let Some((p_trans, cam_trans)) = self.get_camera_parameter_transform().zip(Self::get_main_camera().map(|v| v.get_transform())){
            let rxyz = [r[0] as f32 * 0.015, r[1] as f32 * 0.015, r[2] as f32 * 0.015];
            let pos = self.current_camera.pos;
            let mut displacement = [pos.x, pos.y, pos.z];
            for x in 0..3 {
                let change = [rxyz[x] * self.character_basis[x].x, rxyz[x] * self.character_basis[x].y, rxyz[x] * self.character_basis[x].z];
                if self.camera_bound_check(&change, &r, x){
                    for y in 0..3 { displacement[y] += change[y]; }
                    self.cam_translation[x] += r[x];
                }
            }
            self.current_camera.pos.x = displacement[0];
            self.current_camera.pos.y = displacement[1];
            self.current_camera.pos.z = displacement[2];
            cam_trans.set_position(self.current_camera.pos);
            p_trans.set_position(self.current_camera.pos);
        }
    }
    pub fn translate_character(&mut self, r: [i32; 3]) {
        if r[0] == 0 && r[1] == 0 && r[2] == 0 { return; }
        if self.mode == MenuMode::UnitInfo {
            if let Some(transform) = self.get_character_transform() {
                let rxyz = [r[0] as f32 * 0.015, r[1] as f32 * 0.015, r[2] as f32 * 0.015];
                self.current_character.pos.x = clamp_value(self.current_character.pos.x + rxyz[0], -1.25, 1.0);
                self.current_character.pos.y = clamp_value(self.current_character.pos.y + rxyz[1], -1.50, 1.75);
                self.current_character.pos.z = clamp_value(self.current_character.pos.z + rxyz[2], -2.75, 2.25);
                let pos = engage::unity_engine::Vector3{
                    x: self.current_character.pos.x,
                    y: self.current_character.pos.y,
                    z: self.current_character.pos.z,
                };
                transform.set_position(pos);
            }
        }
    }
    pub fn camera_rotation_check(&self, x: f32, y: f32, z: f32) -> bool {
        let new = [wrap_angle(self.rotation_change.x + x), wrap_angle(self.rotation_change.y + y), wrap_angle(self.rotation_change.z + z)];
        for i in 0..3 { if new[i] >= 30.0 || new[i] < -30.0 { return false; } }
        true
    }
    pub fn camera_bound_check(&self, new_position: &[f32; 3], count: &[i32; 3], i: usize) -> bool {
        if (self.current_camera.pos.y + new_position[1]) < self.y_min { return false;}
        if i == 2 {
            let new_count = self.cam_translation[2] + count[2];
            if (self.cam_translation[2] > 2 && new_count < 0) || (self.cam_translation[2] < -2 && new_count > 0) {
                return false;
            }
        }
        let new_count = count[i] + self.cam_translation[i];
        new_count < self.cam_bounds[i] && new_count > -self.cam_bounds[i]
    }
}

fn wrap_angle(v: f32) -> f32 {
    if v >= 360.0 { v - 360.0 }
    else if v <= -360.0 { v + 360.0 }
    else { v }
}
