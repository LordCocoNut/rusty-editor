use crate::{
    do_command,
    inspector::{handlers::node::base::handle_base_property_changed, SenderHelper},
    scene::commands::camera::*,
};
use rg3d::{
    core::{futures::executor::block_on, pool::Handle},
    gui::inspector::{FieldKind, PropertyChanged},
    resource::texture::{Texture, TextureWrapMode},
    scene::{
        camera::{Camera, ColorGradingLut, Exposure, SkyBox, SkyBoxBuilder},
        node::Node,
    },
};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
enum SkyBoxFace {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

fn modify_skybox(
    camera: &Camera,
    texture: Option<Texture>,
    face: SkyBoxFace,
) -> Option<Box<SkyBox>> {
    if let Some(skybox) = camera.skybox_ref() {
        if let Some(texture) = texture.clone() {
            block_on(texture).ok()?;
        }

        let skybox = SkyBoxBuilder {
            front: if face == SkyBoxFace::Front {
                texture.clone()
            } else {
                skybox.front()
            },
            back: if face == SkyBoxFace::Back {
                texture.clone()
            } else {
                skybox.back()
            },
            left: if face == SkyBoxFace::Left {
                texture.clone()
            } else {
                skybox.left()
            },
            right: if face == SkyBoxFace::Right {
                texture.clone()
            } else {
                skybox.right()
            },
            top: if face == SkyBoxFace::Top {
                texture.clone()
            } else {
                skybox.top()
            },
            bottom: if face == SkyBoxFace::Bottom {
                texture
            } else {
                skybox.bottom()
            },
        }
        .build()
        .unwrap();

        // Set S and T coordinate wrap mode, ClampToEdge will remove any possible seams on edges
        // of the skybox.
        if let Some(cubemap) = skybox.cubemap() {
            let mut data = cubemap.data_ref();
            data.set_s_wrap_mode(TextureWrapMode::ClampToEdge);
            data.set_t_wrap_mode(TextureWrapMode::ClampToEdge);
        }

        Some(Box::new(skybox))
    } else {
        None
    }
}

pub fn handle_camera_property_changed(
    args: &PropertyChanged,
    handle: Handle<Node>,
    node: &Node,
    helper: &SenderHelper,
) -> Option<()> {
    if let Node::Camera(camera) = node {
        match args.value {
            FieldKind::Object(ref value) => match args.name.as_ref() {
                Camera::EXPOSURE => helper.do_scene_command(SetExposureCommand::new(
                    handle,
                    *value.cast_value::<Exposure>()?,
                )),
                Camera::Z_NEAR => {
                    do_command!(helper, SetZNearCommand, handle, value)
                }
                Camera::Z_FAR => {
                    do_command!(helper, SetZFarCommand, handle, value)
                }
                Camera::FOV => {
                    do_command!(helper, SetFovCommand, handle, value)
                }
                Camera::VIEWPORT => {
                    do_command!(helper, SetViewportCommand, handle, value)
                }
                Camera::ENABLED => {
                    do_command!(helper, SetCameraPreviewCommand, handle, value)
                }
                Camera::SKY_BOX => {
                    do_command!(helper, SetSkyBoxCommand, handle, value)
                }
                Camera::ENVIRONMENT => {
                    do_command!(helper, SetEnvironmentMap, handle, value)
                }
                Camera::COLOR_GRADING_LUT => {
                    do_command!(helper, SetColorGradingLutCommand, handle, value)
                }
                Camera::COLOR_GRADING_ENABLED => {
                    do_command!(helper, SetColorGradingEnabledCommand, handle, value)
                }
                _ => None,
            },
            FieldKind::Inspectable(ref inner) => match args.name.as_ref() {
                Camera::EXPOSURE => {
                    if let FieldKind::Object(ref value) = inner.value {
                        match inner.name.as_ref() {
                            Exposure::AUTO_KEY_VALUE => {
                                let mut current_auto_exposure = camera.exposure();
                                if let Exposure::Auto {
                                    ref mut key_value, ..
                                } = current_auto_exposure
                                {
                                    *key_value = *value.cast_value::<f32>()?;
                                }

                                helper.do_scene_command(SetExposureCommand::new(
                                    handle,
                                    current_auto_exposure,
                                ))
                            }
                            Exposure::AUTO_MIN_LUMINANCE => {
                                let mut current_auto_exposure = camera.exposure();
                                if let Exposure::Auto {
                                    ref mut min_luminance,
                                    ..
                                } = current_auto_exposure
                                {
                                    *min_luminance = *value.cast_value::<f32>()?;
                                }

                                helper.do_scene_command(SetExposureCommand::new(
                                    handle,
                                    current_auto_exposure,
                                ))
                            }
                            Exposure::AUTO_MAX_LUMINANCE => {
                                let mut current_auto_exposure = camera.exposure();
                                if let Exposure::Auto {
                                    ref mut max_luminance,
                                    ..
                                } = current_auto_exposure
                                {
                                    *max_luminance = *value.cast_value::<f32>()?;
                                }

                                helper.do_scene_command(SetExposureCommand::new(
                                    handle,
                                    current_auto_exposure,
                                ))
                            }
                            Exposure::MANUAL_F_0 => {
                                helper.do_scene_command(SetExposureCommand::new(
                                    handle,
                                    Exposure::Manual(value.cast_value::<f32>().cloned()?),
                                ))
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                Camera::COLOR_GRADING_LUT => {
                    if let FieldKind::Object(ref value) = inner.value {
                        match inner.name.as_ref() {
                            ColorGradingLut::LUT => {
                                if let Some(texture) =
                                    value.cast_value::<Option<Texture>>().cloned()?
                                {
                                    if let Ok(lut) = block_on(ColorGradingLut::new(texture)) {
                                        helper.do_scene_command(SetColorGradingLutCommand::new(
                                            handle,
                                            Some(lut),
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                Camera::SKY_BOX => {
                    if let FieldKind::Object(ref value) = inner.value {
                        let texture = value.cast_value::<Option<Texture>>().cloned()?;
                        let face = match inner.name.as_ref() {
                            SkyBox::FRONT => Some(SkyBoxFace::Front),
                            SkyBox::BACK => Some(SkyBoxFace::Back),
                            SkyBox::LEFT => Some(SkyBoxFace::Left),
                            SkyBox::RIGHT => Some(SkyBoxFace::Right),
                            SkyBox::BOTTOM => Some(SkyBoxFace::Bottom),
                            SkyBox::TOP => Some(SkyBoxFace::Top),
                            _ => None,
                        };

                        if let Some(face) = face {
                            helper.do_scene_command(SetSkyBoxCommand::new(
                                handle,
                                modify_skybox(camera, texture, face),
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Camera::BASE => handle_base_property_changed(inner, handle, node, helper),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    }
}
