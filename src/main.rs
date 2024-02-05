use bevy::{animation, prelude::*};
use bevy_editor_pls::prelude::*;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EditorPlugin::default())
    .add_systems(Startup, spawn_cam)
    .add_systems(Startup, spawn_player)
    .add_systems(Update,animate_sprite)
    .add_systems(Update, move_player)
    .add_systems(Update, change_player_animation)
    .register_type::<TextureAtlasSprite>()
    .run()
}

fn spawn_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}


#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands, mut texture_atlas: ResMut<Assets<TextureAtlas>>,
asset_server: Res<AssetServer>,) {
let atlas = TextureAtlas::from_grid(asset_server.load("Main Characters/Mask Dude/Idle (32x32).png"),
Vec2::splat(32.), 11, 1, None, None);
commands.spawn((SpriteSheetBundle {
    texture_atlas: texture_atlas.add(atlas),
    sprite: TextureAtlasSprite {index:0, ..Default::default()},
    ..Default::default()
}, Player, 
SpriteAnimation {
    len: 11,
    frame_time: 1./10.
},
FrameTime(0.0)));
}

#[derive(Component)]
struct SpriteAnimation {
    len: usize,
    frame_time: f32,
}

#[derive(Component)]
struct FrameTime(f32);

fn animate_sprite(
    mut animations: Query<(&mut TextureAtlasSprite, &SpriteAnimation, &mut FrameTime)>, time:Res<Time>,
) {
    for (mut sprite, animation, mut frame_time) in animations.iter_mut() {
        frame_time.0 += time.delta_seconds();
        if frame_time.0 > animation.frame_time {
            let frames = (frame_time.0 / animation.frame_time) as usize;
            sprite.index += frames;
            if sprite.index >=animation.len {sprite.index %= animation.len;}
            frame_time.0 -= animation.frame_time * frames as f32;
        }
    }
}

const MOVE_SPEED: f32 = 100.;

fn move_player(mut player: Query<&mut Transform, With<Player>>, time: Res<Time>, input: Res<Input<KeyCode>>) {
    let mut player = player.single_mut();
    if input.any_pressed([KeyCode::A, KeyCode::Left]) {
        player.translation.x -= MOVE_SPEED * time.delta_seconds();
    } else if input.any_pressed([KeyCode::D, KeyCode::Right]) {
        player.translation.x += MOVE_SPEED * time.delta_seconds();
    }
}

fn change_player_animation(
    mut player: Query<(&mut Handle<TextureAtlas>, &mut SpriteAnimation, &mut TextureAtlasSprite), With<Player>>, 
    input: Res<Input<KeyCode>>,
    mut texture_atlas: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let (mut atlas, mut animation, mut sprite) = player.single_mut();

    if input.any_just_pressed([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right]) {
        let new_atlas = TextureAtlas::from_grid(
            asset_server.load("Main Characters/Mask Dude/Run (32x32).png"),
            Vec2::splat(32.),
            12, 1, None, None);
        *atlas = texture_atlas.add(new_atlas);
        animation.len = 12;
        sprite.index = 0;
    }

    if input.any_just_released([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right])
    && !input.any_pressed([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right]) {
        let new_atlas = TextureAtlas::from_grid(
            asset_server.load("Main Characters/Mask Dude/Idle (32x32).png"),
            Vec2::splat(32.),
            11, 1, None, None);
        *atlas = texture_atlas.add(new_atlas);
        animation.len = 11;
        sprite.index = 0;
    }

    if input.any_just_pressed([KeyCode::A, KeyCode::Left]) {
        sprite.flip_x = true;
    } else if input.any_just_pressed([KeyCode::D, KeyCode::Right])
    && !input.any_pressed([KeyCode::A, KeyCode::Left]) {
        sprite.flip_x = false;
    } else if input.any_just_released([KeyCode::A, KeyCode::Left])
    && !input.any_pressed([KeyCode::A, KeyCode::Left])
    && input.any_pressed([KeyCode::D, KeyCode::Right]) {
        sprite.flip_x = false;
    }
}