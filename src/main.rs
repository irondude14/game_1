use bevy::{prelude::*, utils::HashMap};
use bevy_editor_pls::prelude::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EditorPlugin::default())
        .add_systems(Startup, spawn_cam)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_map)
        .add_systems(Update, ground_detection) // Ensure ground detection runs early
        .add_systems(Update, animate_sprite)
        .add_systems(Update, move_player)
        .add_systems(Update, player_jump)
        .add_systems(Update, player_fall)
        .add_systems(Update, change_player_animation) // Ensure animation change runs last
        .init_resource::<PlayerAnimations>()
        .init_resource::<TerrainSprites>()
        .register_type::<TextureAtlasSprite>()
        .run()
}

fn spawn_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_map(mut commands: Commands) {

    // let ground_texture = asset_server.load("Items/Terrain/Terrain.pn");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::NEG_Y * 16.),
            sprite: Sprite { custom_size: Some(Vec2::new(200., 5.)),
                color: Color::WHITE,
                // texture: Some(ground_texture),
                ..Default::default()
            },
            ..Default::default()
        },
        HitBox(Vec2::new(200., 5.)),
    ));
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(100., 25., 0.)),
            sprite: Sprite { custom_size: Some(Vec2::new(32., 32.)),
                color: Color::WHITE,
                ..Default::default()
            },
            ..Default::default()
        },
        HitBox(Vec2::new(32., 32.)),
    ));
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
FrameTime(0.0),
Grounded(true),
HitBox(Vec2::splat(32.)),
));
}

#[derive(Component, Clone, Copy)]
struct SpriteAnimation {
    len: usize,
    frame_time: f32,
}

#[derive(Component)]
struct FrameTime(f32);
fn animate_sprite(
    mut animations: Query<(&mut TextureAtlasSprite, &SpriteAnimation, &mut FrameTime)>,
    time: Res<Time>,
) {
    for (mut sprite, animation, mut frame_time) in animations.iter_mut() {
        frame_time.0 += time.delta_seconds();
        if frame_time.0 > animation.frame_time {
            let frames = (frame_time.0 / animation.frame_time) as usize;
            sprite.index += frames;
            if sprite.index >= animation.len {
                sprite.index %= animation.len;
            }
            frame_time.0 -= animation.frame_time;
        }
    }
}

const MOVE_SPEED: f32 = 100.;

fn move_player(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &Grounded, &HitBox), With<Player>>,
    hitboxs: Query<(&HitBox, &Transform), Without<Player>>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
) {
    let (entity, mut p_offset, grounded, &p_hitbox) = player.single_mut();
    let delay = if input.any_just_pressed([KeyCode::W, KeyCode::Up, KeyCode::Space]) && grounded.0 {
        commands.entity(entity).insert(Jump(100.));
        return;
    } else if input.any_pressed([KeyCode::A, KeyCode::Left]) {
        -MOVE_SPEED * time.delta_seconds() * (0.5 + (grounded.0 as u16) as f32)
    } else if input.any_pressed([KeyCode::D, KeyCode::Right]) {
        MOVE_SPEED * time.delta_seconds() * (0.5 + (grounded.0 as u16) as f32)
    } else {
        return;
    };
    let new_pos = p_offset.translation + Vec3::X * delay;
    for (&hitbox, offset) in &hitboxs {
        if check_hit(p_hitbox, new_pos, hitbox, offset.translation) {return;}
    }
    p_offset.translation = new_pos;
}

fn change_player_animation(
    mut player: Query<(&mut Handle<TextureAtlas>, &mut SpriteAnimation, &mut TextureAtlasSprite), With<Player>>,
    player_jump: Query<(Option<&Jump>, &Grounded), With<Player>>,
    input: Res<Input<KeyCode>>,
    animations: Res<PlayerAnimations>,
) {
    let (mut atlas, mut animation, mut sprite) = player.single_mut();
    let (jump, grounded) = player_jump.single();
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
    
    let set = 
    //Jumping if jump
    if jump.is_some() {
        Animation::Jump
    //Falling if no on ground
    } else if !grounded.0 {
        Animation::Fall
    // if any move keys pressed set run sprite
    } else if input.any_pressed([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right]) {
        Animation::Run
    } else {
        Animation::Idle
    };
    let Some((new_atlas, new_animation)) = animations.get(set) else {error!("No Animation Jump Loaded"); return;};
    *atlas = new_atlas;
    sprite.index %= new_animation.len;
    *animation = new_animation;
}


#[derive(Component)]
struct Jump(f32);

const FALL_SPEED: f32 = 98.0;

fn player_jump(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &mut Jump), With<Player>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((player, mut transform, mut jump)) = player.get_single_mut() else { return; };
    let jump_power = (time.delta_seconds() * FALL_SPEED * 2.0).min(jump.0);
    transform.translation.y += jump_power;
    jump.0 -= if input.any_pressed([KeyCode::W, KeyCode::Up, KeyCode::Space]) {jump_power} else {jump_power * 2.};
    if jump.0 <= 0. {
        commands.entity(player).remove::<Jump>();
    }
}

fn player_fall(
    mut player: Query<(&mut Transform, &HitBox, &mut Grounded), (With<Player>, Without<Jump>)>,
    hitboxes: Query<(&HitBox, &Transform), Without<Player>>,
    time: Res<Time>,
) {
    let Ok((mut p_offset, &p_hitbox, mut grounded)) = player.get_single_mut() else { return; };

    // Check if the player is on the ground
    let mut is_on_ground = false;
    let new_pos = p_offset.translation - Vec3::Y * FALL_SPEED * time.delta_seconds();

    for (&hitbox, offset) in &hitboxes {
        if check_hit(p_hitbox, new_pos, hitbox, offset.translation) {
            is_on_ground = true;
            grounded.0 = true;
            p_offset.translation.y = offset.translation.y + (hitbox.0.y + p_hitbox.0.y) / 2.0; // Snap player to the top of the hitbox
            break;
        }
    }

    if !is_on_ground {
        p_offset.translation = new_pos;
        grounded.0 = false;
    }
}

#[derive(Component)]
struct Grounded(bool);

fn ground_detection(
    mut player: Query<(&Transform, &mut Grounded), With<Player>>,
    mut last: Local<Transform>,
) {
    let (pos,mut on_ground) = player.single_mut();
    let current = if pos.translation.y == last.translation.y {
        true
    } else {
        false
    };
    if current != on_ground.0 {
        on_ground.0 = current;
    }

    *last = *pos;
}

#[derive(Debug, Component, Clone, Copy)]
struct HitBox(Vec2);

fn check_hit(hitbox: HitBox, offset: Vec3, other_hitbox: HitBox, other_offset: Vec3) -> bool {
    let h_size = hitbox.0.y /2.;
    let oh_size = other_hitbox.0.y /2.;
    let w_size = hitbox.0.x /2.;
    let ow_size = other_hitbox.0.x /2.;

    offset.x + w_size > other_offset.x - ow_size && offset.x - w_size < other_offset.x + ow_size &&
    offset.y + h_size > other_offset.y - oh_size && offset.y - h_size < other_offset.y + oh_size
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
            let jump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Jump (32x32).png"),
                Vec2::splat(32.),
                1, 1, None, None);
            let fall_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Fall (32x32).png"),
                Vec2::splat(32.),
                1, 1, None, None);
        let mut texture_atlas = world.resource_mut::<Assets<TextureAtlas>>();
        map.add(Animation::Idle, texture_atlas.add(idle_atlas),
        SpriteAnimation {len: 11, frame_time: 1./10.});
        map.add(Animation::Run, texture_atlas.add(run_atlas),
        SpriteAnimation {len: 12, frame_time: 1./10.});
        map.add(Animation::Jump, texture_atlas.add(jump_atlas), SpriteAnimation { len: 1, frame_time: 1. });
        map.add(Animation::Fall, texture_atlas.add(fall_atlas), SpriteAnimation { len: 1, frame_time: 1. });

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
    Jump,
    Fall,
}

#[derive(Resource)]
struct TerrainSprites(Handle<TextureAtlas>);

impl TerrainSprites {
    fn new(handle: Handle<TextureAtlas>) -> TerrainSprites {
        TerrainSprites(handle)
    }

    fn get_atlas(&self) -> Handle<TextureAtlas> {
        self.0.clone()
    }
}

impl FromWorld for TerrainSprites {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_atles = TextureAtlas::from_grid(asset_server.load("Terrain/Terrain (16x16).png"), Vec2::splat(16.), 22, 11, None, None);
        let mut assets = world.resource_mut::<Assets<TextureAtlas>>();
        TerrainSprites::new(assets.add(texture_atles))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TerrainType {
    GoldLeftEnd = 193,
    GoldStraight = 194,
    GoldRightEnd = 195,
}