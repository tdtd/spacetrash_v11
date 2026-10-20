use bevy::prelude::*;

use crate::components::{AiBehaviorState, BehaviorDirective, Blackbox, Drone};
use crate::resources::scenario::{ActiveScenario, ScenarioCatalog};
use crate::resources::{
    DiegeticView, FocusedEntity, SimulationSpeed, TerminalInputBuffer, TerminalOutputLog,
};

/// Parsed terminal action for the first playable terminal slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalCommand {
    FocusBlackbox,
    FocusDrone(u8),
    Status,
    Guard,
    Patrol,
    Scavenge,
    Aggressive,
    Flee,
    Passive,
    ViewNone,
    Help,
    Speed(Option<String>),
    ScenarioList,
    ScenarioLoad(String),
    Unknown(String),
}

/// Pure, deterministic parser for the current minimal command set.
pub fn parse_terminal_command(input: &str) -> TerminalCommand {
    let mut tokens = input
        .trim()
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase());

    let Some(command) = tokens.next() else {
        return TerminalCommand::Unknown("empty command".to_string());
    };

    let args: Vec<String> = tokens.collect();

    match command.as_str() {
        "focus" | "f" => match args.first().map(String::as_str) {
            Some("blackbox") | Some("0") => TerminalCommand::FocusBlackbox,
            Some(arg) => match arg.parse::<u8>() {
                Ok(index) if (1..=9).contains(&index) => TerminalCommand::FocusDrone(index),
                _ => TerminalCommand::Unknown(format!("invalid focus target: {arg}")),
            },
            None => TerminalCommand::Unknown("focus requires a target".to_string()),
        },
        "status" | "st" => TerminalCommand::Status,
        "guard" => TerminalCommand::Guard,
        "patrol" => TerminalCommand::Patrol,
        "scavenge" => TerminalCommand::Scavenge,
        "aggressive" => TerminalCommand::Aggressive,
        "flee" => TerminalCommand::Flee,
        "passive" => TerminalCommand::Passive,
        "view" => match args.first().map(String::as_str) {
            Some("none") => TerminalCommand::ViewNone,
            Some(other) => TerminalCommand::Unknown(format!("view mode not implemented: {other}")),
            None => TerminalCommand::Unknown("view requires a mode".to_string()),
        },
        "speed" => TerminalCommand::Speed(args.first().cloned()),
        "scenario" => match args.first().map(String::as_str) {
            Some("list") => TerminalCommand::ScenarioList,
            Some("load") => match args.get(1).cloned() {
                Some(id) => TerminalCommand::ScenarioLoad(id),
                None => TerminalCommand::Unknown("scenario load requires an id".to_string()),
            },
            Some(other) => TerminalCommand::Unknown(format!("unknown scenario subcommand: {other}")),
            None => TerminalCommand::Unknown("scenario requires a subcommand".to_string()),
        },
        "help" | "?" => TerminalCommand::Help,
        other => TerminalCommand::Unknown(format!("unknown command: {other}")),
    }
}

/// Small helper to convert simple keyboard codes into command-line characters.
fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some('-'),
        _ => None,
    }
}

/// Reads player terminal input, parses commands, and applies them to the
/// focused entity / world state.
///
/// This system runs only while `DiegeticView::CommandLine` is active. It is
/// deliberately deterministic and reads from shared terminal resources so
/// headless automation can drive the same interface later.
pub fn terminal_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<DiegeticView>,
    mut input: ResMut<TerminalInputBuffer>,
    mut output: ResMut<TerminalOutputLog>,
    mut commands: Commands,
    mut focused_entity: ResMut<FocusedEntity>,
    blackbox_query: Query<Entity, With<Blackbox>>,
    drone_query: Query<(Entity, &Drone)>,
    ai_query: Query<Option<&AiBehaviorState>>,
    directive_query: Query<Option<&BehaviorDirective>>,
    mut simulation_speed: ResMut<SimulationSpeed>,
    scenario_catalog: Res<ScenarioCatalog>,
    mut active_scenario: ResMut<ActiveScenario>,
) {
    if *view != DiegeticView::CommandLine {
        return;
    }

    for key in keyboard.get_just_pressed() {
        match *key {
            KeyCode::Enter => {
                let raw = input.current.trim().to_string();
                input.current.clear();

                if raw.is_empty() {
                    output.push_line(">");
                    continue;
                }

                output.push_line(format!("> {raw}"));
                execute_terminal_command(
                    parse_terminal_command(&raw),
                    &mut output,
                    &mut commands,
                    &mut focused_entity,
                    &mut view,
                    &blackbox_query,
                    &drone_query,
                    &ai_query,
                    &directive_query,
                    &mut simulation_speed,
                    &scenario_catalog,
                    &mut active_scenario,
                );
            }

            KeyCode::Backspace => {
                input.current.pop();
            }

            other => {
                if input.current.is_empty()
                    && matches!(
                        other,
                        KeyCode::Digit0
                            | KeyCode::Digit1
                            | KeyCode::Digit2
                            | KeyCode::Digit3
                            | KeyCode::Digit4
                            | KeyCode::Digit5
                            | KeyCode::Digit6
                            | KeyCode::Digit7
                            | KeyCode::Digit8
                            | KeyCode::Digit9
                    )
                {
                    continue;
                }

                if let Some(c) = key_to_char(other) {
                    input.current.push(c);
                }
            }
        }
    }
}

fn execute_terminal_command(
    command: TerminalCommand,
    output: &mut TerminalOutputLog,
    commands: &mut Commands,
    focused_entity: &mut FocusedEntity,
    view: &mut DiegeticView,
    blackbox_query: &Query<Entity, With<Blackbox>>,
    drone_query: &Query<(Entity, &Drone)>,
    ai_query: &Query<Option<&AiBehaviorState>>,
    directive_query: &Query<Option<&BehaviorDirective>>,
    simulation_speed: &mut SimulationSpeed,
    scenario_catalog: &ScenarioCatalog,
    active_scenario: &mut ActiveScenario,
) {
    match command {
        TerminalCommand::FocusBlackbox => {
            if let Ok(entity) = blackbox_query.get_single() {
                focused_entity.entity = Some(entity);
                *view = DiegeticView::FirstPerson;
                output.push_line("Focused: Blackbox");
            } else {
                output.push_line("Blackbox not found");
            }
        }

        TerminalCommand::FocusDrone(index) => {
            let mut found = None;
            for (entity, drone) in drone_query.iter() {
                if drone.index == index {
                    found = Some(entity);
                    break;
                }
            }

            if let Some(entity) = found {
                focused_entity.entity = Some(entity);
                *view = DiegeticView::FirstPerson;
                output.push_line(format!("Focused: Drone {index}"));
            } else {
                output.push_line(format!("Drone {index} not found"));
            }
        }

        TerminalCommand::Status => {
            let Some(entity) = focused_entity.entity else {
                output.push_line("No focused unit");
                return;
            };

            let name = focused_name(entity, blackbox_query, drone_query);
            let behavior = ai_query
                .get(entity)
                .ok()
                .and_then(|opt| opt)
                .copied()
                .unwrap_or_default();
            let directive = directive_query
                .get(entity)
                .ok()
                .and_then(|opt| opt)
                .copied()
                .unwrap_or_default();

            output.push_line(name);
            output.push_line(format!("  ai:        {behavior:?}"));
            output.push_line(format!("  directive: {directive:?}"));
        }

        TerminalCommand::Guard => {
            set_behavior_directive(commands, focused_entity, BehaviorDirective::Guard, output);
        }
        TerminalCommand::Patrol => {
            set_behavior_directive(commands, focused_entity, BehaviorDirective::Patrol, output);
        }
        TerminalCommand::Scavenge => {
            set_behavior_directive(commands, focused_entity, BehaviorDirective::Scavenge, output);
        }
        TerminalCommand::Aggressive => {
            set_behavior_directive(commands, focused_entity, BehaviorDirective::Aggressive, output);
        }
        TerminalCommand::Flee => {
            set_behavior_directive(commands, focused_entity, BehaviorDirective::Flee, output);
        }
        TerminalCommand::Passive => {
            set_behavior_directive(commands, focused_entity, BehaviorDirective::Passive, output);
        }

        TerminalCommand::ViewNone => {
            output.push_line("Ray view disabled; no live visual feed");
        }

        TerminalCommand::Speed(arg) => {
            match arg.as_deref() {
                None => {
                    output.push_line(format!("SIM SPEED: {:.1}x", simulation_speed.scale));
                }
                Some("normal") => {
                    simulation_speed.scale = 1.0;
                    output.push_line("SIM SPEED: 1.0x");
                }
                Some("pause") => {
                    simulation_speed.scale = 0.0;
                    output.push_line("SIM SPEED: paused");
                }
                Some(value) => match value.parse::<f32>() {
                    Ok(scale) if scale >= 0.0 => {
                        simulation_speed.scale = scale;
                        output.push_line(format!("SIM SPEED: {scale:.1}x"));
                    }
                    _ => output.push_line("Invalid speed; use 0.5, 1, 2, 4, normal, pause"),
                },
            }
        }

        TerminalCommand::ScenarioList => {
            if scenario_catalog.scenarios.is_empty() {
                output.push_line("No scenarios available");
            } else {
                output.push_line("Available scenarios:");
                for (id, scenario) in &scenario_catalog.scenarios {
                    output.push_line(format!("  {id} - {}", scenario.name));
                }
            }
        }

        TerminalCommand::ScenarioLoad(id) => {
            if let Some(scenario) = scenario_catalog.scenarios.get(&id) {
                active_scenario.scenario = Some(scenario.clone());
                output.push_line(format!("Scenario set: {id} ({}). Restart or return to menu to load.", scenario.name));
            } else {
                output.push_line(format!("Unknown scenario: {id}"));
            }
        }

        TerminalCommand::Help => {
            output.push_line("Available commands:");
            output.push_line("  focus <1-9|blackbox|0>  Focus a drone or Blackbox");
            output.push_line("  status                 Show focused unit status");
            output.push_line("  view <none>            Disable ray view");
            output.push_line("  guard                  Guard behavior");
            output.push_line("  patrol                 Patrol behavior");
            output.push_line("  scavenge               Scavenge behavior");
            output.push_line("  aggressive             Aggressive behavior");
            output.push_line("  flee                   Flee behavior");
            output.push_line("  passive                Passive behavior");
            output.push_line("  speed                  Show current simulation speed");
            output.push_line("  speed <x|normal|pause> Set simulation speed");
            output.push_line("  scenario list          List available scenarios");
            output.push_line("  scenario load <id>     Select scenario (load on next entry)");
            output.push_line("  help                   Show this help");
        }

        TerminalCommand::Unknown(message) => {
            output.push_line(message);
        }
    }
}

fn focused_name(
    entity: Entity,
    blackbox_query: &Query<Entity, With<Blackbox>>,
    drone_query: &Query<(Entity, &Drone)>,
) -> String {
    if blackbox_query.get(entity).is_ok() {
        return "Blackbox".to_string();
    }

    if let Ok((_, drone)) = drone_query.get(entity) {
        return format!("Drone {}", drone.index);
    }

    "Unknown".to_string()
}

fn set_behavior_directive(
    commands: &mut Commands,
    focused_entity: &FocusedEntity,
    directive: BehaviorDirective,
    output: &mut TerminalOutputLog,
) {
    let Some(entity) = focused_entity.entity else {
        output.push_line("No focused unit");
        return;
    };

    commands.entity(entity).insert(directive);
    output.push_line(format!("Behavior set to {directive:?}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_help_command() {
        assert_eq!(parse_terminal_command("help"), TerminalCommand::Help);
    }

    #[test]
    fn parse_help_question_mark_alias() {
        assert_eq!(parse_terminal_command("?"), TerminalCommand::Help);
    }

    #[test]
    fn command_cannot_begin_with_number() {
        assert_eq!(
            parse_terminal_command("1"),
            TerminalCommand::Unknown("unknown command: 1".to_string())
        );
        assert_eq!(
            parse_terminal_command("2foo"),
            TerminalCommand::Unknown("unknown command: 2foo".to_string())
        );
    }
}
