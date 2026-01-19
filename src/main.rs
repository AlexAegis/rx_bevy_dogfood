use std::time::Duration;

use bevy::prelude::*;
use rx_bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RxPlugin,
            RxSchedulerPlugin::<FixedUpdate, Fixed>::default(),
            RxSchedulerPlugin::<Update, Virtual>::default(),
            RxSchedulerPlugin::<Update, Real>::default(),
        ))
        .init_resource::<ExampleSubscriptions>()
        .add_systems(Startup, setup)
        .run()
}

#[derive(Resource, Default, Deref, DerefMut)]
struct ExampleSubscriptions {
    subscriptions: SharedSubscription,
}

fn setup(
    mut commands: Commands,
    rx_schedule_update_virtual: RxSchedule<Update, Virtual>,
    mut example_subscriptions: ResMut<ExampleSubscriptions>,
) {
    println!("Press 1 - 5 to change the virtual time speed!");
    println!("Press 6 - 9 to change the interval duration!");

    let _virtual_time_setting_subscription = commands
        .with_observable(
            KeyboardObservable::new(
                KeyboardObservableOptions {
                    emit: KeyboardObservableEmit::JustPressed,
                },
                rx_schedule_update_virtual.handle(),
            ),
            rx_schedule_update_virtual.handle(),
        )
        .filter(|key_code, _| {
            matches!(
                key_code,
                KeyCode::Digit1
                    | KeyCode::Digit2
                    | KeyCode::Digit3
                    | KeyCode::Digit4
                    | KeyCode::Digit5
            )
        })
        .subscribe(ResourceDestination::new(
            |mut virtual_time: Mut<'_, Time<Virtual>>, signal| {
                let speed = match signal {
                    ObserverNotification::Next(key_code) => match key_code {
                        KeyCode::Digit1 => 0.5,
                        KeyCode::Digit2 => 1.0,
                        KeyCode::Digit3 => 1.5,
                        KeyCode::Digit4 => 2.5,
                        KeyCode::Digit5 => 5.0,
                        _ => 1.0,
                    },
                    _ => 1.0,
                };

                println!("Setting the virtual clocks relative speed to {speed}!");

                virtual_time.set_relative_speed(speed);
            },
            rx_schedule_update_virtual.handle(),
        ));

    let scheduler = rx_schedule_update_virtual.handle();

    let subscription = KeyboardObservable::new(default(), rx_schedule_update_virtual.handle())
        .filter(|key_code, _| {
            matches!(
                key_code,
                KeyCode::Digit6 | KeyCode::Digit7 | KeyCode::Digit8 | KeyCode::Digit9
            )
        })
        .start_with(KeyCode::Digit8)
        .switch_map(
            move |key_code| {
                let duration = match key_code {
                    KeyCode::Digit6 => Duration::from_millis(5),
                    KeyCode::Digit7 => Duration::from_millis(100),
                    KeyCode::Digit8 => Duration::from_millis(500),
                    KeyCode::Digit9 => Duration::from_millis(2000),
                    _ => unreachable!("Change the filter and start_with too!"),
                };
                println!("Switching to a new inner observable with duration: {duration:?}");
                IntervalObservable::new(
                    IntervalObservableOptions {
                        duration,
                        start_on_subscribe: false,
                        max_emissions_per_tick: 4,
                    },
                    scheduler.clone(),
                )
            },
            Never::map_into(),
        )
        .scan(|acc, _next| acc + 1, 0_usize)
        .subscribe(PrintObserver::new("woof"));

    example_subscriptions.add(subscription);
}
