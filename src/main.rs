use std::sync::{Arc, Mutex};

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::{Window, WindowMode},
};

use opencv::{
    aruco::{detect_markers, DetectorParameters, Dictionary, DICT_4X4_100},
    core::{add_weighted, no_array, subtract, Point2f, Size as cvSize, Vector, BORDER_DEFAULT},
    imgproc::{cvt_color, gaussian_blur, COLOR_BGR2GRAY},
    prelude::{DetectorParametersTrait, Mat},
    videoio::{
        VideoCapture, VideoCaptureTrait, CAP_PROP_FPS, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH,
    },
};
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Resource)]
struct MyCam(Arc<Mutex<VideoCapture>>);

#[derive(Resource)]
struct MyRng(StdRng);

fn main() {
    let mut vc = VideoCapture::new_default(0).unwrap();
    vc.set(CAP_PROP_FRAME_WIDTH, 1920.0).unwrap();
    vc.set(CAP_PROP_FRAME_HEIGHT as i32, 1080.0).unwrap();
    vc.set(CAP_PROP_FPS as i32, 30.0).unwrap();
    let cam = MyCam(Arc::new(Mutex::new(vc)));

    let rng = MyRng(StdRng::from_entropy());

    App::new()
        .insert_resource(cam)
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
    mut cam: ResMut<MyCam>,
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

fn get_aruco(cam: &ResMut<MyCam>) -> [(f32, f32);4] {
    let mut a = cam.0.lock().unwrap();
    let mut frame = Mat::default();
    let dict = Dictionary::get(DICT_4X4_100).unwrap();
    let mut corners = Vector::<Vector<Point2f>>::new();
    let mut ids = Vector::<i32>::new();
    let mut parameters = DetectorParameters::create().unwrap();
    parameters.set_adaptive_thresh_win_size_max(31);
    parameters.set_adaptive_thresh_win_size_min(10);
    parameters.set_adaptive_thresh_win_size_step(3);
    let mut rejected_img_points = no_array();
    let mut res = [(0.,0.);4];
    while let Ok(_) = a.read(&mut frame) {
        let mut gray_mat = Mat::default();
        // let mut bin_mat = Mat::default();
        cvt_color(&frame, &mut gray_mat, COLOR_BGR2GRAY, 0).unwrap();

        let mut blur_output = Mat::default();
        gaussian_blur(
            &gray_mat,
            &mut blur_output,
            cvSize::new(0, 0),
            5.,
            0.,
            BORDER_DEFAULT,
        )
        .unwrap();

        let mut subtract_output = Mat::default();
        subtract(
            &gray_mat,
            &blur_output,
            &mut subtract_output,
            &no_array(),
            -1,
        )
        .unwrap();

        let mut sharp_output = Mat::default();
        add_weighted(
            &gray_mat,
            1.,
            &subtract_output,
            1.5,
            0.,
            &mut sharp_output,
            -1,
        )
        .unwrap();
        // adaptive_threshold(
        //     &mut sharp_output,
        //     &mut bin_mat,
        //     255.0,
        //     ADAPTIVE_THRESH_MEAN_C,
        //     THRESH_BINARY_INV,
        //     41,
        //     7.0,
        // )
        // .unwrap();
        detect_markers(
            &sharp_output,
            &dict,
            &mut corners,
            &mut ids,
            &parameters,
            &mut rejected_img_points,
        )
        .unwrap();
        if corners.len() != 1 {
            // draw_detected_markers(
            //     &mut frame,
            //     &mut corners,
            //     &mut ids,
            //     Scalar::new(0.0, 255.0, 0.0, 0.0),
            // )
            // .unwrap();
            // estimate_pose_single_markers(
            //     &corners,
            //     0.03,
            //     &camera_matrix,
            //     &dist_coeffs,
            //     &mut rvecs,
            //     &mut tvecs,
            // )
            // .unwrap();
            // println!("{:?}", rvecs);
            // println!("{:?}", tvecs);
            let corner = corners.iter().next().unwrap();
            for (i,p) in corner.iter().enumerate() {
                res[i] = (p.x, p.y);
            }
            break;
        }
    }
    res
}
