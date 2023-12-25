use bevy::{
  prelude::*,
  sprite::collide_aabb::{collide, Collision},
  sprite::MaterialMesh2dBundle, window::close_on_esc,
};

use bevy::input::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const PADDLE_SIZE: Vec3 = Vec3::new(120., 20., 0.);
const GAP_BETWEEN_PADDLE_AND_FLOOR : f32 = 60.;
const PADDLE_SPEED: f32 = 500.;
const PADDLE_PADDING: f32 = 10.0;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0., -50., 1.);
const BALL_SIZE: Vec3 = Vec3::new(30., 30., 0.);
const BALL_SPEED: f32 = 400.;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THIKNESS: f32 = 10.;

const LEFT_WALL: f32 = -450.;
const RIFHT_WALL: f32 = 450.;

const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);

const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.;
const GAP_BETWEEN_BRICKS: f32 = 5.;

const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.;

const SCOREBOARD_FONT_SIZE: f32 = 40.;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);


#[derive(Resource)]
struct Scoreboard {
  score: usize
}

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Bundle)]
struct WallBundle {
  sprite_bundle: SpriteBundle,
  collider: Collider
}

enum WallLocation {
  Left,
  Right,
  Bottom,
  Top
}

impl WallLocation {
  fn position(&self) -> Vec2 {
    match self {
        WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
        WallLocation::Right => Vec2::new(RIFHT_WALL, 0.),
        WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
        WallLocation::Top => Vec2::new(0., TOP_WALL)
    }
  }

  fn size(&self) -> Vec2 {
    let arena_height = TOP_WALL - BOTTOM_WALL;
    let arena_width  = RIFHT_WALL - LEFT_WALL;
    assert!(arena_height > 0.0);
    assert!(arena_width > 0.0);

    match  self {
        WallLocation::Left | WallLocation::Right => {
          Vec2::new(WALL_THIKNESS, arena_height + WALL_THIKNESS)
        }
        WallLocation::Bottom | WallLocation::Top => {
          Vec2::new(arena_width + WALL_THIKNESS, WALL_THIKNESS)
        }
    }
  }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
      WallBundle {
        sprite_bundle: SpriteBundle {
          transform: Transform {
            translation: location.position().extend(0.),
            scale: location.size().extend(0.),
            ..default()
          },
          sprite: Sprite {
            color: WALL_COLOR,
            ..default()
          },
          ..default()
        },
        collider: Collider
      }
    }
}

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Brick;

#[derive(Event, Default)]
struct CollisionEvent;

fn main() {
    App::new()
      .add_plugins(DefaultPlugins)
      .insert_resource(Scoreboard{score: 0})
      .insert_resource(ClearColor(BACKGROUND_COLOR))
      .add_event::<CollisionEvent>()
      .add_systems(Startup, setup)
      .add_systems(FixedUpdate, (apply_velocity, move_paddle, check_for_collisions, play_collision_sound).chain())
      .add_systems(Update, (update_scoreboard,bevy::window::close_on_esc))
      .run();
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  asset_server: Res<AssetServer>
){
  commands.spawn(Camera2dBundle::default());

  let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
  commands.insert_resource(CollisionSound(ball_collision_sound));

  let paddly_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

  commands.spawn((
    SpriteBundle {
      transform: Transform {
        translation: Vec3::new(0., paddly_y, 0.),
        scale: PADDLE_SIZE,
        ..default()
      },
      sprite: Sprite {
        color: PADDLE_COLOR,
        ..default()
      },
      ..default()
    },
    Paddle,
    Collider
  ));

  commands.spawn((
    MaterialMesh2dBundle {
      mesh: meshes.add(shape::Circle::default().into()).into(),
      material: materials.add(ColorMaterial::from(BALL_COLOR)),
      transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
      ..default()
    },
    Ball,
    Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED)
  ));

  commands.spawn(
    TextBundle::from_sections([
      TextSection::new("Score: ", TextStyle {font_size: SCOREBOARD_FONT_SIZE, color: TEXT_COLOR, ..default()}),
      TextSection::from_style(TextStyle {
        font_size: SCOREBOARD_FONT_SIZE,
        color: SCORE_COLOR,
        ..default()
      }),
    ]).with_style(Style {
      position_type: PositionType::Absolute,
      top: SCOREBOARD_TEXT_PADDING,
      left: SCOREBOARD_TEXT_PADDING,
      ..default()
    })
  );

  commands.spawn(WallBundle::new(WallLocation::Left));
  commands.spawn(WallBundle::new(WallLocation::Right));
  commands.spawn(WallBundle::new(WallLocation::Bottom));
  commands.spawn(WallBundle::new(WallLocation::Top));

  let total_width_of_bricks = (RIFHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
  let bottom_edge_of_bricks = paddly_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
  let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;
  
  assert!(total_width_of_bricks > 0.0);
  assert!(total_height_of_bricks > 0.);

  let n_columns =(total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
  let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
  let n_vertical_gaps = n_columns - 1;

  let center_of_bricks = (LEFT_WALL + RIFHT_WALL) / 2.0;
  let left_edge_of_bricks: f32 = center_of_bricks - (n_columns as f32 / 2. * BRICK_SIZE.x) - n_vertical_gaps as f32 / 2. * GAP_BETWEEN_BRICKS;

  let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
  let offse_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

  for row in 0..n_rows {
    for column in 0..n_columns {
      let brick_position = Vec2::new(
        offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
        offse_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)
      );

      commands.spawn((
        SpriteBundle {
          sprite: Sprite {
            color: BRICK_COLOR,
            ..default()
          },
          transform: Transform {
            translation: brick_position.extend(0.),
            scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
            ..default()
          },
          ..default()
        },
        Brick,
        Collider
      ));
    }
  }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
  for (mut transform, velocity) in &mut query {
    transform.translation.x += velocity.x * time.delta_seconds();
    transform.translation.y += velocity.y * time.delta_seconds();
  }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
  let mut text = query.single_mut();
  text.sections[1].value = scoreboard.score.to_string();
}

fn check_for_collisions(
  mut commands: Commands,
  mut scoreboard: ResMut<Scoreboard>,
  mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
  collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
  mut collision_event: EventWriter<CollisionEvent>,
) {
  let (mut ball_velocity, ball_transform) = ball_query.single_mut();
  let ball_size = ball_transform.scale.truncate();

  for (collider_entity, transform, maybe_brick) in &collider_query {
    let collision = collide(ball_transform.translation, ball_size, transform.translation, transform.scale.truncate());
    if let Some(collisions) = collision {
      collision_event.send_default();

      if maybe_brick.is_some() {
        scoreboard.score += 1;
        commands.entity(collider_entity).despawn();
      }

      let mut reflect_x = false;
      let mut reflect_y = false;

      match collisions {
          Collision::Left => reflect_x = ball_velocity.x > 0.,
          Collision::Right => reflect_x = ball_velocity.x < 0.,
          Collision::Top => reflect_y = ball_velocity.y < 0.,
          Collision::Bottom => reflect_y = ball_velocity.y > 0.,
          Collision::Inside => {}
      }

      if reflect_x {
        ball_velocity.x = -ball_velocity.x;
      }

      if reflect_y {
        ball_velocity.y = -ball_velocity.y;
      }
    }
  }
}

fn play_collision_sound(
  mut commands: Commands,
  mut collision_event: EventReader<CollisionEvent>,
  sound: Res<CollisionSound>
){
  if !collision_event.is_empty() {
    collision_event.clear();
    commands.spawn(AudioBundle{
      source: sound.0.clone(),
      settings: PlaybackSettings::DESPAWN
    });
  }
}

fn move_paddle(
  keyboard_input: Res<Input<KeyCode>>,
  mut query: Query<&mut Transform, With<Paddle>>,
  time: Res<Time>
) {
  let mut paddle_transform = query.single_mut();
  let mut direction = 0.;

  if keyboard_input.pressed(KeyCode::Left) {
    direction -= 1.;
  }

  if keyboard_input.pressed(KeyCode::Right) {
    direction += 1.;
  }

  let new_paddle_position = paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_seconds();

  let left_bound = LEFT_WALL + WALL_THIKNESS / 2. + PADDLE_SIZE.x / 2. + PADDLE_PADDING;
  let right_bound = RIFHT_WALL - WALL_THIKNESS /2. - PADDLE_SIZE.x / 2. - PADDLE_PADDING;

  paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound)

}