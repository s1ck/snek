use std::{ops::{Deref, DerefMut}, time::Duration};

use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::rc::Rc;

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

struct SnekHead {
    direction: Direction,
    next_direction: Option<Direction>,
}

struct SnekSegment;

#[derive(Default)]
struct SnekSegments(Vec<Entity>);

struct SnekMoveTimer(Timer);
impl Deref for SnekMoveTimer {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SnekMoveTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}
#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
        }
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Bevy requires a specific ordering to the params when registering systems.
    // Commands → Resources → Components/Queries.
    // If you get a mysterious compile-time error after messing with a system, check your order.
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.4, 0.2, 0.0).into()),
        segment_material: materials.add(Color::rgb(0.0, 0.2, 0.4).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
    });
}

fn game_setup(mut commands: Commands, materials: Res<Materials>, mut segments: ResMut<SnekSegments>) {
    let first_segment = spawn_segment(
        &mut commands,
        &materials.segment_material,
        Position { x: 3, y: 2 },
    );
    segments.0 = vec![first_segment];

    commands
        .spawn(SpriteComponents {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(SnekHead {
            direction: Direction::Up,
            next_direction: None,
        })
        .with(Position { x: 3, y: 3 })
        .with(Size::square(0.8));
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    for (size, mut sprite) in q.iter_mut() {
        let window = windows.get_primary().unwrap();
        sprite.size = Vec2::new(
            size.width as f32 / ARENA_WIDTH as f32 * window.width() as f32,
            size.height as f32 / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(p: f32, bound_window: f32, bound_game: f32) -> f32 {
        p / bound_game * bound_window - (bound_window / 2.) + (bound_window / bound_game / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn snek_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snek_timer: ResMut<SnekMoveTimer>,
    mut heads: Query<(&mut SnekHead, &mut Position)>,
    mut segment: Query<(&mut SnekSegment, &mut Position)>,
) {
    let dir: Option<Direction> = keyboard_input
        .get_pressed()
        .filter_map(|input| match input {
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            _ => None,
        })
        .next();

    for (mut head, mut pos) in heads.iter_mut() {
        let current_direction = head.direction;
        if let Some(dir) = dir {
            if dir != current_direction && dir != current_direction.opposite() {
                head.next_direction = Some(dir);
            } else {
                head.next_direction = None
            }
        }

        let mut last_pos = *pos;
        if snek_timer.finished {
            let dir = head.next_direction.take().unwrap_or(head.direction);
            head.direction = dir;
            match dir {
                Direction::Left => pos.x -= 1,
                Direction::Right => pos.x += 1,
                Direction::Up => pos.y += 1,
                Direction::Down => pos.y -= 1,
            }
            for (mut _segment, mut pos) in segment.iter_mut() {
                let tmp = *pos;
                *pos = last_pos;
                last_pos = tmp;
            }
        }

    }
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .with(Size::square(0.8));
    }
}

fn snek_timer(time: Res<Time>, mut snek_timer: ResMut<SnekMoveTimer>) {
    snek_timer.0.tick(time.delta_seconds);
}

fn spawn_segment(
    commands: &mut Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteComponents {
            material: material.clone(),
            .. SpriteComponents::default()
        })
        .with(SnekSegment)
        .with(position)
        .with(Size::square(0.65));
    commands.current_entity().unwrap()
}

fn main() {

    App::build()
        .add_resource(WindowDescriptor {
            title: "Snek!".to_string(),
            width: 1000,
            height: 1000,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(SnekMoveTimer(Timer::new(
            Duration::from_millis(500. as u64),
            true,
        )))
        .add_resource(SnekSegments::default())
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(snek_movement.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_system(food_spawner.system())
        .add_system(snek_timer.system())
        .add_plugins(DefaultPlugins)
        .run();
}
