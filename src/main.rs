use bevy::{animation, prelude::*, utils::HashMap};
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
    .init_resource::<PlayerAnimations>()
    .run()
}

fn spawn_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}


#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands, animations: Res<PlayerAnimations>,) {
    let Some((texture_atlas, animation)) = animations.get(Animation::Idle) else {error!("Failed to find animation: Idle"); return;};
commands.spawn((SpriteSheetBundle {
    texture_atlas,
    sprite: TextureAtlasSprite {index:0, ..Default::default()},
    ..Default::default()
}, Player, 
animation,
FrameTime(0.0)));
}

#[derive(Component, Clone, Copy)]
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
    animations: Res<PlayerAnimations>,
) {
    let (mut atlas, mut animation, mut sprite) = player.single_mut();

    if input.any_just_pressed([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right]) {
        let Some((new_atlas, new_animation)) = animations.get(Animation::Run) else {error!("No Animation Run Loaded"); return;};
        *atlas = new_atlas;
        *animation = new_animation;
        sprite.index = 0;
    }

    if input.any_just_released([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right])
    && !input.any_pressed([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right]) {
        let Some((new_atlas, new_animation)) = animations.get(Animation::Idle) else {error!("No Animation Idle Loaded"); return;};
        *atlas = new_atlas;
        *animation = new_animation;
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

#[derive(Resource)]
struct PlayerAnimations {
    map: HashMap<Animation, (Handle<TextureAtlas>, SpriteAnimation)>
}

impl FromWorld for PlayerAnimations {
    fn from_world(world: &mut World) -> Self {
        let mut map = PlayerAnimations {map: HashMap::new()};
        let asset_server = world.resource::<AssetServer>();
        let idle_atlas = TextureAtlas::from_grid(
            asset_server.load("Main Characters/Mask Dude/Idle (32x32).png"),
            Vec2::splat(32.),
            11, 1, None, None);
        let run_atlas = TextureAtlas::from_grid(
            asset_server.load("Main Characters/Mask Dude/Run (32x32).png"),
            Vec2::splat(32.),
            12, 1, None, None);
        let mut texture_atlas = world.resource_mut::<Assets<TextureAtlas>>();
        map.add(Animation::Idle, texture_atlas.add(idle_atlas),
        SpriteAnimation {len: 11, frame_time: 1./10.});
        map.add(Animation::Run, texture_atlas.add(run_atlas),
        SpriteAnimation {len: 12, frame_time: 1./10.});

        map
    }
}

impl PlayerAnimations {
    fn add(&mut self, id: Animation, handle: Handle<TextureAtlas>, animation: SpriteAnimation) {
        self.map.insert(id, (handle, animation));
    }
    fn get(&self, id: Animation) -> Option<(Handle<TextureAtlas>, SpriteAnimation)> {
        self.map.get(&id).cloned()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Animation {
    Run,
    Idle,
}