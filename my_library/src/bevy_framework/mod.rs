use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

mod game_menus;

mod bevy_animation;
pub use bevy_animation::*;

mod bevy_physics;
pub use bevy_physics::*;

mod bevy_collision;
pub use bevy_collision::*;

pub struct GameStatePlugin<T> {
    menu_state: T,
    game_start_state: T,
    game_end_state: T,
}

impl<T> GameStatePlugin<T>
where
    T: States + FromWorld + FreelyMutableState,
{
    #[allow(clippy::new_without_default)]
    pub fn new(menu_state: T, game_start_state: T, game_end_state: T) -> Self {
        Self {
            menu_state,
            game_start_state,
            game_end_state,
        }
    }
}

impl<T> Plugin for GameStatePlugin<T>
where
    T: States + Copy + FromWorld + FreelyMutableState + Default,
{
    fn build(&self, app: &mut App) {
        app.init_state::<T>();
        app.add_plugins(bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: false,
        });
        let start = MenuResource {
            menu_state: self.menu_state,
            game_start_state: self.game_start_state,
            game_end_state: self.game_end_state,
        };
        app.insert_resource(start);

        app.add_systems(OnEnter(self.menu_state), game_menus::setup::<T>);
        app.add_systems(
            Update,
            game_menus::run::<T>.run_if(in_state(self.menu_state)),
        );
        app.add_systems(OnExit(self.menu_state), cleanup::<game_menus::MenuElement>);

        app.add_systems(OnEnter(self.game_end_state), game_menus::setup::<T>);
        app.add_systems(
            Update,
            game_menus::run::<T>.run_if(in_state(self.game_end_state)),
        );
        app.add_systems(
            OnExit(self.game_end_state),
            cleanup::<game_menus::MenuElement>,
        );

        app.add_systems(OnEnter(T::default()), crate::bevy_assets::setup);
        app.add_systems(
            Update,
            crate::bevy_assets::run::<T>.run_if(in_state(T::default())),
        );
        app.add_systems(OnExit(T::default()), crate::bevy_assets::exit);

        app.add_event::<PhysicsTick>();
        app.add_event::<Impulse>();
    }
}

#[derive(Resource)]
pub(crate) struct MenuResource<T> {
    pub(crate) menu_state: T,
    pub(crate) game_start_state: T,
    pub(crate) game_end_state: T,
}

pub fn cleanup<T>(query: Query<Entity, With<T>>, mut commands: Commands)
where
    T: Component,
{
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn())
}

#[macro_export]
macro_rules! add_phase {
    (
        $app:expr, $type:ty, $phase:expr,
        start => [ $($start:expr),* ],
        run => [ $($run:expr),* ],
        exit => [ $($exit:expr),* ]
    ) => {
        $($app.add_systems(
            bevy::prelude::OnEnter::<$type>($phase),
            $start
        );)*
        $($app.add_systems(
            bevy::prelude::Update, $run.run_if(in_state($phase))
        );)*
        $($app.add_systems(
            bevy::prelude::OnExit::<$type>($phase),
            $exit
        );)*
    };
}
