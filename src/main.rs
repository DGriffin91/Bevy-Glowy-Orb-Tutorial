use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    core_pipeline::{
        fxaa::Fxaa,
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass, NormalPrepass},
    },
    pbr::{DefaultOpaqueRendererMethod, OpaqueRendererMethod},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

mod bevy_basic_camera;

use bevy_basic_camera::{CameraController, CameraControllerPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(DefaultOpaqueRendererMethod(OpaqueRendererMethod::Deferred))
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_secs_f32(0.1)),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window { ..default() }),
                    ..default()
                }),
        )
        .add_plugin(CameraControllerPlugin)
        .add_plugin(MaterialPlugin::<GlowyMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, switch_mode)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut glowmaterials: ResMut<Assets<GlowyMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load Texture
    let env_texture = asset_server.load("textures/stone_alley_02_1k.hdr");

    let material = glowmaterials.add(GlowyMaterial {
        env_texture: Some(env_texture),
    });

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 100.0,
            ..default()
        })),
        material: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
        ..default()
    });

    // orb locations
    let locations = [
        Vec3::new(-0.15, 1.0, -2.0),
        Vec3::new(1.7, 1.07, -0.61),
        Vec3::new(0.21, 1.05, 1.99),
        Vec3::new(-2.16, 1.0, 0.01),
        Vec3::new(-2.2, 1.0, 2.13),
        Vec3::new(-1.06, 2.04, 1.02),
        Vec3::new(1.94, 1.02, 1.16),
        Vec3::new(0.91, 2.47, 0.83),
        Vec3::new(0.46, 2.48, -0.81),
        Vec3::new(-2.05, 0.93, -1.92),
        Vec3::new(-1.38, 2.46, -0.91),
        Vec3::new(-0.22, 3.48, 0.18),
    ];

    for location in locations {
        // spawn orbs
        commands
            .spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 1.0,
                    ..default()
                })),
                transform: Transform::from_translation(location),
                material: material.clone(),
                ..default()
            })
            .with_children(|parent| {
                // child light
                parent.spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 10000.0,
                        radius: 1.0,
                        color: Color::rgb(0.5, 0.1, 0.0),
                        ..default()
                    },
                    ..default()
                });
            });
    }

    // camera
    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: Transform::from_xyz(8.0, 5.0, 8.0)
                    .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
                ..default()
            },
            DepthPrepass,
            MotionVectorPrepass,
            DeferredPrepass,
            Fxaa::default(),
        ))
        .insert(CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        });
}

impl Material for GlowyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/glowy.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/glowy.wgsl".into()
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "717f64fe-6844-4822-8926-e0ed374294c8"]
pub struct GlowyMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub env_texture: Option<Handle<Image>>,
}

fn switch_mode(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut default_opaque_renderer_method: ResMut<DefaultOpaqueRendererMethod>,
    mut gmaterials: ResMut<Assets<GlowyMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<Entity, With<Camera>>,
) {
    if input.just_pressed(KeyCode::Key1) {
        default_opaque_renderer_method.0 = OpaqueRendererMethod::Deferred;
        println!("DefaultOpaqueRendererMethod: Deferred");
        for _ in gmaterials.iter_mut() {}
        for _ in materials.iter_mut() {}
        for camera in &cameras {
            commands.entity(camera).remove::<NormalPrepass>();
            commands.entity(camera).insert(DepthPrepass);
            commands.entity(camera).insert(MotionVectorPrepass);
            commands.entity(camera).insert(DeferredPrepass);
        }
    }
    if input.just_pressed(KeyCode::Key2) {
        default_opaque_renderer_method.0 = OpaqueRendererMethod::Forward;
        println!("DefaultOpaqueRendererMethod: Forward");
        for _ in gmaterials.iter_mut() {}
        for _ in materials.iter_mut() {}
        for camera in &cameras {
            commands.entity(camera).remove::<NormalPrepass>();
            commands.entity(camera).remove::<DepthPrepass>();
            commands.entity(camera).remove::<MotionVectorPrepass>();
            commands.entity(camera).remove::<DeferredPrepass>();
        }
    }
}
