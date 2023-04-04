use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::{Window, WindowMode},
};

use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Resource)]
struct MyRng(StdRng);

fn main() {
    let rng = MyRng(StdRng::from_entropy());
    App::new()
        .insert_resource(rng)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(move_aruco)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("aruco.png"),
        transform: Transform::IDENTITY.with_scale(Vec3::new(0.3, 0.3, 1.)),
        ..default()
    });
}

fn move_aruco(
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut query: Query<(&mut Transform, &Handle<Image>), With<Sprite>>,
    image: Res<Assets<Image>>,
    mut rng: ResMut<MyRng>,
    windows: Query<&Window>,
) {
    let (mut transform, texture) = query.single_mut();

    if let (Some(img), Ok(focused_window)) = (image.get(texture), windows.get_single()) {
        let window_height = focused_window.height();
        let window_width = focused_window.width();
        let aruco_width = (img.texture_descriptor.size.width as f32) * transform.scale.x;
        let aruco_height = (img.texture_descriptor.size.height as f32) * transform.scale.x;
        let left_bound = -window_width / 2. + aruco_width / 2.;
        let right_bound = window_width / 2. - aruco_width / 2.;
        let top_bound = window_height / 2. - aruco_height / 2.;
        let bot_bound = -window_height / 2. + aruco_height / 2.;

        for e in mousebtn_evr.iter() {
            if let MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            } = e
            {
                let next_width = rng.0.gen_range(0f32..1f32);
                let next_height = rng.0.gen_range(0f32..1f32);
                let new_x = right_bound + (left_bound - right_bound) * next_width;
                let new_y = bot_bound + (top_bound - bot_bound) * next_height;
                println!(
                    "window_height: {}, window_width: {}",
                    window_height, window_width
                );
                println!("x: {}, y: {}", new_x, new_y);
                transform.translation = Vec3::new(new_x, new_y, 0.);
            }
        }
    }
}
