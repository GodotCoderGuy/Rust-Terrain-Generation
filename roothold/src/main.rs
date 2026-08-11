//---Plugins---//
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use bevy::asset::RenderAssetUsages;
use std::io;
//-------------//


//---Global Values---//
const WORLD_SIZES: [u32; 4] = [256, 512, 1024, 1536];

const MINERAL_SEED: u32 = 0x12345678;
const TEMP_SEED: u32 = 0xCE1C_FA5E;
const MOISTURE_SEED: u32 = 0xABCDEF_12;
//-------------------//


//---Structs---//
#[derive(Clone)]
struct Chunk {
    x: u32,
    y: u32,
    tiles: [[u32; 16]; 16]
}
#[derive(Resource)]
struct World {
    size: u32,
    chunks: Vec<Vec<Chunk>>
}
#[derive(Resource)]
struct WorldSettings {
    mode: u8,
    scale: u32,
    civ_size: u8,
    seed: u32
}
#[derive(Resource)]
struct CameraTimer {
    time: f32
}
#[derive(Resource)]
struct OtherNoises {
    mineral_noise: Perlin,
    temp_noise: Perlin,
    moisture_noise: Perlin
}
#[derive(Component)]
struct CivImg;
//-------------//


//---Implememnted---//
impl Chunk {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            tiles: [[0; 16]; 16]
        }
    }
}
impl World {
    fn new(settings: &Res<WorldSettings>) -> Self {
        let size = *WORLD_SIZES.get(settings.mode as usize).unwrap_or(&1024)/16;
        Self {
            size: size,
            chunks: create_chunk_grid(size)
        }
    }
}
impl WorldSettings {
    fn new() -> Self {
        Self {
            mode: 5,
            scale: 50,
            civ_size: 0,
            seed: 0
        }
    }
}
impl OtherNoises {
    fn new(settings: Res<WorldSettings>) -> Self {
        let seed = settings.seed;
        Self {
            mineral_noise: Perlin::new(seed ^ MINERAL_SEED),
            temp_noise: Perlin::new(seed ^ TEMP_SEED),
            moisture_noise: Perlin::new(seed ^ MOISTURE_SEED)
        }
    }
}
//------------------//


//---Helper Functions---//
fn mouse_pos(windows: Query<&Window>, camera: Query<(&Camera, &GlobalTransform)>) -> Vec2 {
    let window = windows.single().unwrap();
    let (camera, camera_transform) = camera.single().unwrap();

    let mut world_pos = Vec2::new(0.0, 0.0);
    if let Some(cursor) = window.cursor_position() {
        world_pos = camera.viewport_to_world_2d(camera_transform, cursor).ok().unwrap();
    }
    world_pos
}


fn fractal_2(perlin: &Perlin, x: f64, y: f64) -> f64 {
    let mut value = 0.0;

    let mut freq = 1.0;
    let mut amp = 1.0;

    for _ in 0..5 {
        value += perlin.get([
            x*freq,
            y*freq
        ])*amp;
        freq *= 2.0;
        amp /= 2.0;
    }
    value
}


fn create_chunk_grid(size: u32) -> Vec<Vec<Chunk>> {
    let mut chunks = Vec::new();

    for _ in 0..size as usize {
        let mut row = Vec::new();
        for _ in 0..size as usize {
            let chunk = Chunk::new();
            row.push(chunk);
        }
        chunks.push(row)
    }
    chunks

}


fn generate_chunk(chunk_num_x: u32, chunk_num_y: u32, perlin: Perlin, settings: &Res<WorldSettings>) -> Chunk {
    let mut chunk = Chunk::new();
    chunk.x = chunk_num_x;
    chunk.y = chunk_num_y;

    for y in 0..16 {
        for x in 0..16 {
            let value = fractal_2(&perlin, (chunk_num_x*16+x) as f64 / settings.scale as f64, (chunk_num_y*16+y) as f64 / settings.scale as f64);

            chunk.tiles[y as usize][x as usize] = (((value+1.0)*0.5)*50.0) as u32
        }
    }

    chunk
}

fn height_to_color(height: u32) -> Color {
    match height {
        0..=15 => Color::srgb(0.0, 0.2, 0.8), // deep water
        16..=20 => Color::srgb(0.2, 0.6, 1.0), // shallow water
        21..=22 => Color::srgb(0.85, 0.82, 0.2), // sand
        23..=30 => Color::srgb(0.2, 0.8, 0.2), // grass
        31..=38 => Color::srgb(0.5, 0.5, 0.2), // hills
        39..=43 => Color::srgb(0.361, 0.424, 0.137), // mix
        44..=47 => Color::srgb(0.0, 0.32, 0.0), // mountains
        _ => Color::WHITE // snowy peaks
    }
}


fn draw_terrain(world: Res<World>) -> Image {
    let mut pixels: Vec<u8> = Vec::new();
    let size = &world.size;
    for world_y in 0..*size as usize {
        for index in 0..16 {
            for chunk in &world.chunks[world_y] {
                for height in chunk.tiles[index] {
                    let color: Color = height_to_color(height);

                    let srgba = color.to_srgba();

                    pixels.push((srgba.red*255.0) as u8);
                    pixels.push((srgba.green*255.0) as u8);
                    pixels.push((srgba.blue*255.0) as u8);
                    pixels.push((srgba.alpha*255.0)as u8);
                }
            }
        }
    }
    let image = Image::new(
        Extent3d {
            width: size * 16 as u32,
            height: size * 16 as u32,
            depth_or_array_layers: 1  
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default()
    );
    image
}


fn make_civ_img(settings: Res<WorldSettings>) -> Image {
    let mut pixels: Vec<u8> = Vec::new();
    for _ in 0..settings.civ_size*4 {
        for _ in 0..settings.civ_size*4 {
            for a in [57, 218, 218, 240] {
                pixels.push(a)
            }
        }
    }
    let image = Image::new(
        Extent3d {
            width: (settings.civ_size as u32)*4,
            height: (settings.civ_size as u32)*4,
            depth_or_array_layers: 1  
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default()
    );
    image
}
//----------------------//


//---Startup Functions---//
fn get_mode(mut commands: Commands) {
    let mut settings = WorldSettings::new();
    println!("Getting Mode.");
    println!("1: Tiny (256 by 256)");
    println!("2: Small (512 by 512)");
    println!("3: Normal (1024 by 1024");
    println!("4: Large (1536 by 1536)");
    println!("Input: ");
    while settings.mode > WORLD_SIZES.len() as u8 {
        println!("Enter a number (1-4):");
        settings.mode = loop {
            let mut input: String = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input.");
            match input.trim().parse::<u8>() {
                Ok(num) => break num-1,
                Err(_) => println!("Enter a number!")
            }
        };
    }
    println!("Scaling.");
    println!("20: Lots of hills");
    println!("50: Medium Continents");
    println!("120: Landmasses");
    println!("200: Huge Landmasses");
    settings.scale = loop {
        let mut scale_str = String::new();

        io::stdin().read_line(&mut scale_str).expect("Failed to read input");

        match scale_str.trim().parse::<u32>() {
            Ok(n) =>  break n,
            Err(_) => println!("Enter a positive number.")
        }
    };
    settings.seed = loop {
        println!("Choose your own seed [y/N]: ");
        let mut seed: String = String::new();
        io::stdin().read_line(&mut seed).expect("Failed to read input.");
        let seed = seed.trim();
        if seed.is_empty() {
            break rand::random::<u32>();
        }
        match seed {
            "y" => {
                println!("What is the chosen seed: ");
                let seed: u32 = loop {
                    let mut input: String = String::new();

                    io::stdin().read_line(&mut input).expect("Failed to read line.");
                    match input.trim().parse::<u32>() {
                        Ok(n) => break n,
                        Err(_) => println!("Enter a positive number.")
                    }
                };
                break seed;
            }
            _ => break rand::random::<u32>()
        }
    };
    println!("Your civilization will be a square of tiles, choose a side length.");
    println!("Maximum is 8 by 8, minimum is 4 by 4.");
    println!("How big should your civiliation be: ");
    while !((4..=8).contains(&settings.civ_size)) {
        let civ_size: u8 = loop {
            let mut input: String = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input.");
            match input.trim().parse::<u8>() {
                Ok(n) => break n,
                Err(_) => println!("Enter a number inbetween 4 and 8 inclusive.")
            }
        };
        settings.civ_size = civ_size;
    }


    commands.insert_resource(settings);
}


fn setup_image(
    mut commands: Commands, 
    mut images: ResMut<Assets<Image>>, 
    world: Res<World>, 
    settings: Res<WorldSettings>
) {
    let image = draw_terrain(world);
    let handle = images.add(image);

    let cursor_rect = make_civ_img(settings);
    let handle2 = images.add(cursor_rect);

    commands.spawn((
        Sprite::from_image(handle2),
        Transform::default(),
        CivImg
    ));

    commands.spawn((
        Sprite::from_image(handle),
        Transform::default()
    ));

    commands.spawn(Camera2d);
}

fn make_world(mut commands: Commands, settings: Res<WorldSettings>) {
    let seed = settings.seed;
    let perlin = Perlin::new(seed);

    let mut world = World::new(&settings);    commands.insert_resource(CameraTimer{ time: 0.0 });


    let size: u32 = *WORLD_SIZES.get(settings.mode as usize).unwrap();
    // This way we do rows of chunks, not columns of chunks
    for y in 0..((size/16) as usize) {
        for x in 0..((size/16) as usize) {
            world.chunks[y as usize][x as usize] = generate_chunk(x as u32, y as u32, perlin, &settings)
        }
    }

    commands.insert_resource(OtherNoises::new(settings));

    commands.insert_resource(world);
}


fn setup(mut commands: Commands) {
    commands.insert_resource(CameraTimer{ time: 0.0 });
    println!("Setting up!")
}


fn setup_camera(mut cameras: Query<&mut Projection, With<Camera2d>>) {
    let mut projection = cameras.single_mut().unwrap();
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = 0.3;
    }

}


fn text_box(mut commands: Commands) {
    commands.spawn((
        Text::new("Hello, world!"),
        TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));
}



fn camera_movement_check(
    mut cameras: Query<&mut Transform, With<Camera2d>>, 
    keyboard: Res<ButtonInput<KeyCode>>, 
    mut delay: ResMut<CameraTimer>,
    time: Res<Time>,
    world: Res<World>
) {
    let size = world.size;
    let delta = time.delta_secs();
    let mut transform = cameras.single_mut().unwrap();
    if delay.time <= 0.0 {
        if keyboard.pressed(KeyCode::KeyW) {
            transform.translation.y += 20.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            transform.translation.x -= 20.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            transform.translation.y -= 20.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            transform.translation.x += 20.0;
        }
        transform.translation.x = transform.translation.x.clamp(-((size*8) as f32), (size*8) as f32);
        transform.translation.y = transform.translation.y.clamp(-((size*8) as f32), (size*8) as f32);
        delay.time = 0.25
    } else {
        delay.time -= delta
    }
}


fn cursor_rect(
    mut sprite: Query<(&Sprite, &mut Transform, &CivImg)>, 
    windows: Query<&Window>, 
    camera: Query<(&Camera, &GlobalTransform)>, 
    mut display_stat: Query<(&mut Text, &Node)>,
    settings: Res<WorldSettings>,
    resources: Res<OtherNoises>,
    world: Res<World>
) {
    let (_rect, mut pos, _civ) = sprite.single_mut().unwrap();
    let prev_pos = [*&pos.translation.x as i32, *&pos.translation.y as i32];
    let mouse_position = mouse_pos(windows, camera);
    pos.translation.x = mouse_position.x as i32 as f32 + (settings.civ_size*2) as f32;
    pos.translation.y = mouse_position.y as i32 as f32 - (settings.civ_size*2) as f32;
    let curr_pos = [*&pos.translation.x as i32, *&pos.translation.y as i32];

    pos.translation.x = pos.translation.x.clamp(-((world.size*8)as f32), (world.size*8) as f32);
    pos.translation.y = pos.translation.y.clamp(-((world.size*8) as f32), (world.size*8) as f32);



    if prev_pos == curr_pos {
        return;
    }


    let mut max_resc = 0.0;
    let mut max_temp = 0.0;
    let mut max_humid: f64 = 0.0;
    let mut min_resc: f64 = 2.0;
    let mut min_temp: f64 = 2.0;
    let mut min_humid: f64 = 2.0;

    let scale = settings.scale;

    let pos_x = pos.translation.x;
    let pos_y = pos.translation.y;

    let resc = resources.mineral_noise;
    let temp = resources.temp_noise;
    let humid = resources.moisture_noise;


    for y in 0..(settings.civ_size*4) as usize {
        for x in 0..(settings.civ_size*4) as usize {
            let resc_val = resc.get([(pos_x+x as f32) as f64 / (scale as f64), (pos_y+y as f32) as f64 / (scale as f64)])+1.0;

            if resc_val < min_resc {
                min_resc = resc_val;
            }else if resc_val > max_resc {
                max_resc = resc_val;
            }
            
            let temp_val = temp.get([(pos_x+x as f32) as f64 / (scale as f64), (pos_y+y as f32) as f64 / (scale as f64)])+1.0;
            if temp_val < min_temp {
                min_temp = temp_val;
            } else if temp_val > max_temp {
                max_temp = temp_val;
            }

            let humid_val = humid.get([(pos_x+x as f32) as f64 / (scale as f64), (pos_y+y as f32) as f64 / (scale as f64)])+1.0;
            if humid_val < min_humid {
                min_humid = humid_val;
            }else if humid_val > max_humid {
                max_humid = humid_val;
            }
        }
    }

    let (mut text, _node) = display_stat.single_mut().unwrap();

    text.0 = format!("Resources\n-----------\nMin: {}\nMax: {}\nTemperature\n-------------\nMin: {}\nMax: {}\nHumidity\n----------\nMin: {}\nMax: {}\n", (min_resc*50.0) as u32, (max_resc*50.0) as u32, (min_temp*50.0) as u32, (max_temp*50.0) as u32, (min_humid*50.0) as u32, (max_humid*50.0) as u32);
}


 fn main() {
    println!("Started!");
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, (setup, get_mode, make_world, setup_image, text_box, setup_camera).chain())
    .add_systems(Update, (camera_movement_check, cursor_rect))
    .run();
    println!("Finished!");
}
//-----------------------//