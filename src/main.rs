use bevy::{math::*, prelude::*, sprite::collide_aabb::*};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<AppState>()
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9))) // background
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            Update,
            check_game_over.run_if(in_state(AppState::InGamePlay)),
        )
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::InGamePlay), setup_ingame)
        .add_systems(OnExit(AppState::InGamePlay), clear_ingame)
        .add_systems(
            FixedUpdate,
            (
                move_paddle,
                move_ball,
                check_ball_collision.after(move_ball),
                check_despawn_ball.after(check_ball_collision),
                apply_despawn.after(check_despawn_ball),
            )
                .run_if(in_state(AppState::InGamePlay)),
        )
        .add_systems(OnEnter(AppState::InGameOver), setup_gameover)
        .add_systems(OnExit(AppState::InGameOver), cleanup_menu)
        .add_systems(Update, menu.run_if(in_state(AppState::InGameOver)))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    InGamePlay,
    #[default]
    InGameOver,
}

// paddle
const PADDLE_START_Y: f32 = -100.0;
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const PADDLE_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const PADDLE_SPEED: f32 = 500.0;

// ball
const BALL_START_POS: Vec2 = Vec2::new(0.0, -50.0);
const BALL_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BALL_SPEED: f32 = 500.0;
const BALL_INIT_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

// boundary
const LOWER_BOUND: Vec2 = Vec2::new(-400.0, -300.0);
const HIGHER_BOUND: Vec2 = Vec2::new(400.0, 300.0);
const WALL_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
const WALL_THICKNESS: f32 = 10.0;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball {
    size: Vec2,
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider {
    size: Vec2,
}

#[derive(Component)]
struct Despawn(bool);

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    marker: OnGameScreen,
}

#[derive(Bundle)]
struct InGameObjectBundle<T: Component> {
    sprite_bundle: SpriteBundle,
    component: T,
    collider: Collider,
    marker: OnGameScreen,
}

#[derive(Bundle)]
struct DespawnableInGameObjectBundle<T: Component> {
    sprite_bundle: SpriteBundle,
    component: T,
    collider: Collider,
    marker: OnGameScreen,
    despawn: Despawn,
}

//bricks
const BRICK_SIZE: Vec2 = Vec2::new(40., 10.);
const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
// const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: Vec2 = vec2(50.0, 10.0);
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

#[derive(PartialEq, Eq)]
enum BrickType {
    Normal,
    AddTripleBall,
}
#[derive(Component)]
struct Brick {
    health: i32,
    brick_type: BrickType,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
struct OnGameScreen;

fn clear_ingame(
    mut commands: Commands,
    mut query: Query<Entity, With<OnGameScreen>>,
) {
    for entity in &mut query {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_ingame(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    // clear score
    scoreboard.score = 0;

    commands.spawn(InGameObjectBundle {
        sprite_bundle: SpriteBundle {
            transform: Transform {
                translation: vec3(0., PADDLE_START_Y, 0.),
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                custom_size: Some(PADDLE_SIZE),
                ..default()
            },
            ..default()
        },
        component: Paddle,
        collider: Collider { size: PADDLE_SIZE },
        marker: OnGameScreen,
    });

    spawn_ball(
        &mut commands,
        &asset_server,
        BALL_START_POS,
        BALL_INIT_DIRECTION
    );

    // verticle walls
    let vertical_wall_size = vec2(
        WALL_THICKNESS,
        HIGHER_BOUND.y - LOWER_BOUND.y + WALL_THICKNESS,
    );
    for i in [LOWER_BOUND.x, HIGHER_BOUND.x] {
        commands.spawn(WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: vec3(i, 0., 0.),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(Vec2::new(vertical_wall_size.x, vertical_wall_size.y)),
                    ..default()
                },
                ..default()
            },
            collider: Collider {
                size: Vec2::new(vertical_wall_size.x, vertical_wall_size.y),
            },
            marker: OnGameScreen,
        });
    }

    // horizontal walls
    let horizontal_wall_size = vec2(
        HIGHER_BOUND.x - LOWER_BOUND.x + WALL_THICKNESS,
        WALL_THICKNESS,
    );
    for i in [HIGHER_BOUND.y] {
        commands.spawn(WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: vec3(0., i, 0.),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(Vec2::new(horizontal_wall_size.x, horizontal_wall_size.y)),
                    ..default()
                },
                ..default()
            },
            collider: Collider {
                size: Vec2::new(
                    HIGHER_BOUND.x - LOWER_BOUND.x + WALL_THICKNESS,
                    WALL_THICKNESS,
                ),
            },
            marker: OnGameScreen,
        });
    }

    // brick
    setup_bricks(commands, 5);
}

fn setup_bricks(mut commands: Commands, num_rows: i32) {
    let mut rng = rand::thread_rng();

    let left_bound = LOWER_BOUND.x + GAP_BETWEEN_BRICKS_AND_SIDES + BRICK_SIZE.x / 2.;
    let right_bound = HIGHER_BOUND.x - GAP_BETWEEN_BRICKS_AND_SIDES - BRICK_SIZE.x / 2.;
    let top_bound = HIGHER_BOUND.y - GAP_BETWEEN_BRICKS_AND_CEILING - BRICK_SIZE.y / 2.;

    // let bottom_bound = top_bound - (num_rows as f32) * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS);

    for row in 0..num_rows {
        let mut nearest_brick_left_x = left_bound;
        while nearest_brick_left_x + GAP_BETWEEN_BRICKS.x + BRICK_SIZE.x / 2.0 < right_bound {
            nearest_brick_left_x += GAP_BETWEEN_BRICKS.x + BRICK_SIZE.x / 2.0;

            let brick_type: BrickType;
            let random_number = rng.gen_range(0..100);
            if random_number < 10 {
                brick_type = BrickType::AddTripleBall;
            } else {
                brick_type = BrickType::Normal;
            }

            let color = match brick_type {
                BrickType::AddTripleBall => Color::rgb(1.0, 0.0, 0.0),
                _ => BRICK_COLOR,
            };

            commands
                .spawn(DespawnableInGameObjectBundle {
                        sprite_bundle: SpriteBundle {
                            transform: Transform {
                                translation: vec3(
                                    nearest_brick_left_x,
                                    top_bound - row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS.y),
                                    0.,
                                ),
                                ..default()
                            },
                            sprite: Sprite {
                                color: color,
                                custom_size: Some(BRICK_SIZE),
                                ..default()
                            },
                            ..default()
                        },
                        component: Brick {
                            health: 1,
                            brick_type: brick_type,
                        },
                        collider: Collider { size: BRICK_SIZE },
                        despawn: Despawn(false),
                        marker: OnGameScreen,
                    },
                );
        }
    }
}

fn move_ball(time_step: Res<FixedTime>, mut query: Query<(&mut Transform, &Velocity), With<Ball>>) {
    let delta_time = time_step.period.as_secs_f32();
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * delta_time;
        transform.translation.y += velocity.y * delta_time;
    }
}

fn move_paddle(
    input: Res<Input<KeyCode>>,
    time_step: Res<FixedTime>,
    mut query: Query<&mut Transform, With<Paddle>>,
) {
    let mut paddle_transform = query.single_mut();

    let mut direction = 0.;
    if input.pressed(KeyCode::Left) || input.pressed(KeyCode::A) {
        direction -= 1.;
    }
    if input.pressed(KeyCode::Right) || input.pressed(KeyCode::D) {
        direction += 1.;
    }

    let new_x =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time_step.period.as_secs_f32();
    paddle_transform.translation.x = new_x
        .min(HIGHER_BOUND.x - PADDLE_SIZE.x / 2.)
        .max(LOWER_BOUND.x + PADDLE_SIZE.x / 2.);
}

fn check_ball_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ball_query: Query<(&mut Velocity, &Transform, &Ball)>,
    mut collider_query: Query<(
        &Transform,
        &Collider,
        Option<&mut Brick>,
        Option<&mut Despawn>,
    )>,
    mut scoreboard: ResMut<Scoreboard>
) {
    for (mut ball_velocity, ball_tranform, ball) in &mut ball_query {
        for (other_transform, other_collider, brick_option, despawn_option) in &mut collider_query {
            let collision = collide(
                ball_tranform.translation,
                ball.size,
                other_transform.translation,
                other_collider.size,
            );

            let mut reflect_x = false;
            let mut reflect_y = false;
            if let Some(collision) = collision {
                match collision {
                    Collision::Left => reflect_x = ball_velocity.x > 0.,
                    Collision::Right => reflect_x = ball_velocity.x < 0.,
                    Collision::Top => reflect_y = ball_velocity.y < 0.,
                    Collision::Bottom => reflect_y = ball_velocity.y > 0.,
                    Collision::Inside => {}
                }

                if reflect_x {
                    ball_velocity.x *= -1.;
                }
                if reflect_y {
                    ball_velocity.y *= -1.;
                }

                if let Some(mut brick) = brick_option {
                    scoreboard.score += 1;

                    brick.health = (brick.health - 1).max(0);
                    if brick.health <= 0 {
                        if brick.brick_type == BrickType::AddTripleBall {
                            // spawn 3 balls
                            spawn_rnd_balls(
                                &mut commands,
                                &asset_server,
                                vec2(ball_tranform.translation.x, ball_tranform.translation.y),
                                3
                            );
                        }
                        // remove brick
                        let mut despawn = despawn_option.unwrap();
                        despawn.0 = true;
                    }
                }
            }
        }
    }
}

fn check_despawn_ball(mut query: Query<(&Transform, &mut Despawn, &Ball)>) {
    for (transform, mut despawn, ball) in &mut query {
        if transform.translation.y < LOWER_BOUND.y - ball.size.y / 2. {
            despawn.0 = true;
        }
    }
}
fn apply_despawn(mut commands: Commands, mut query: Query<(Entity, &Despawn)>) {
    for (entity, despawn) in &mut query {
        if despawn.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_game_over(
    mut next_state: ResMut<NextState<AppState>>,
    ball_query: Query<Entity, With<Ball>>,
    brick_query: Query<Entity, With<Brick>>,
) {
    // count number of balls
    let mut num_balls = 0;
    for _ in &ball_query {
        num_balls += 1;
    }

    let mut num_bricks = 0;
    for _ in &brick_query {
        num_bricks += 1;
    }

    if num_bricks <= 0 {
        println!("You win!");
        next_state.set(AppState::InGameOver);
    }
    if num_balls <= 0 {
        println!("Game over!");
        next_state.set(AppState::InGameOver);
    }
}

fn spawn_ball(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec2,
    direction: Vec2
) {
    commands
        .spawn((
            SpriteBundle {
                transform: Transform {
                    translation: vec3(position.x, position.y, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: BALL_COLOR,
                    custom_size: Some(BALL_SIZE),
                    ..default()
                },
                texture: asset_server.load("textures/circle.png"),
                ..default()
            },
            Ball { size: BALL_SIZE },
            Velocity(direction * BALL_SPEED),
            Despawn(false),
            OnGameScreen,
        ));
}

fn spawn_rnd_balls(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec2,
    num_balls: i32
) {
    let mut rng = rand::thread_rng();

    for _ in 0..num_balls {
        let new_ball_direction =
            vec2(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize();
        spawn_ball(
            commands,
            asset_server,
            position,
            new_ball_direction
        );
    }
}

#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
}

fn setup_gameover(mut commands: Commands, scoreboard: Res<Scoreboard>) {
    let button_entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    "Score: ",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0., 0.0, 0.0),
                        ..default()
                    },
                ),
                TextSection::new(
                    scoreboard.score.to_string(),
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.4, 0.9, 0.0),
                        ..default()
                    },
                ),
            ]));
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .id();
    commands.insert_resource(MenuData { button_entity });
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::rgb(0.1, 0.1, 0.1).into();
                next_state.set(AppState::InGamePlay);
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.1, 0.0, 0.0).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct Scoreboard {
    score: i32,
}
