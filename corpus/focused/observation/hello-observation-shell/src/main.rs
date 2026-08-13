//! Deterministic, headless corpus proof for Observation Shell Slices 1 through 5.

use hello_runtime_observation::{
    CommandAuthority, CommandDisposition, CommandRequest, ObservationEnvelope, ObservationLimits,
    PlaybackCommand, PlaybackCommandResult, RuntimeCommand, RuntimeInspectionAdapter,
};
use observation_shell::{
    ApplicationCommandDescription, ApplicationCommandInvocation, ApplicationCommandResult,
    ApplicationMutationReceipt, ApplicationQueryField, ApplicationQueryResult,
    DiagnosticObservation, DiagnosticsObservation, EntityObservation, ObservationShell,
    ObservationSource, RelationshipEdgeObservation, RelationshipObservation, TypeObservation,
    WorldObservation,
};
use tokimu_core::{Diagnostics, EntityId, World};

#[derive(Debug)]
struct Follows;

fn main() {
    let mut world = World::default();
    let observer = world.spawn();
    let target = world.spawn();
    world.add_relationship::<Follows>(observer, target);

    let mut diagnostics = Diagnostics::default();
    diagnostics.record("hello-observation-shell fixture initialized");
    let source = ObservationSource::from_world_and_diagnostics(&world, &diagnostics);
    let script = [
        "help",
        "inspect world",
        "list entities",
        "inspect entity 0",
        "list relationships 0",
        "observe diagnostics",
        "select entity 0",
        "context",
        "back",
        "format json",
        "inspect world",
        "application fixture summary current",
        "application fixture reset",
        "watch world 2",
        "watch diagnostics 3",
        "list watches",
    ];

    let mut shell = ObservationShell::default();
    shell
        .register_application_command(ApplicationCommandDescription::query(
            "fixture",
            "summary",
            "Discover a corpus-local query without attaching execution authority.",
            vec!["scope".to_owned()],
        ))
        .expect("fixture query registration must be valid");
    shell
        .register_application_command(ApplicationCommandDescription::query(
            "runtime",
            "list-animations",
            "List the scenario-owned hole-punch playback catalog.",
            Vec::new(),
        ))
        .expect("runtime animation query registration must be valid");
    shell
        .register_application_command(ApplicationCommandDescription::mutation(
            "fixture",
            "reset",
            "Discover a corpus-local mutation without attaching execution authority.",
            Vec::new(),
        ))
        .expect("fixture mutation registration must be valid");
    for command in script {
        let record = shell.execute(&source, command);
        println!("> {command}\n{}\n", record.projection);
    }

    // The application supplies a logical observation sequence. The shell owns
    // neither a clock nor an asynchronous result queue.
    for sequence in [0_u64, 1, 2, 3, 9] {
        for refresh in shell.refresh_watches(&source, sequence) {
            println!(
                "watch: id={} target={:?} sequence={} unchanged={} truncated={} summary={:?}",
                refresh.watch_id,
                refresh.summary.target,
                refresh.sequence,
                refresh.unchanged,
                refresh.truncated,
                refresh.summary,
            );
        }
    }

    let record = shell.execute(&source, "unwatch 1");
    println!("> unwatch 1\n{}\n", record.projection);

    // Slice 5: the shell only parses and routes this owner-qualified command.
    // The runtime corpus owns validation, queueing, the explicit apply phase,
    // and the resulting world revision.
    let mut runtime = RuntimeInspectionAdapter::new(4)
        .expect("the hole-punch runtime inspection adapter must load its catalog");
    let runtime_source =
        source_from_runtime(runtime.observe(0, None, ObservationLimits::default()));
    shell
        .register_application_command(ApplicationCommandDescription::mutation(
            "runtime",
            "set-enabled",
            "Queue a scenario-owned enabled-state change for the selected arm.",
            vec!["entity".to_owned(), "enabled".to_owned()],
        ))
        .expect("runtime mutation registration must be valid");
    shell
        .register_application_command(ApplicationCommandDescription::query(
            "runtime",
            "playback",
            "Inspect scenario-owned hole-punch playback state.",
            Vec::new(),
        ))
        .expect("runtime playback query registration must be valid");
    shell
        .register_application_command(ApplicationCommandDescription::mutation(
            "runtime",
            "play",
            "Start one scenario-owned animation clip by catalog index.",
            vec!["clip".to_owned()],
        ))
        .expect("runtime playback mutation registration must be valid");
    shell
        .register_application_command(ApplicationCommandDescription::mutation(
            "runtime",
            "advance",
            "Advance scenario-owned playback by one fixed step.",
            Vec::new(),
        ))
        .expect("runtime playback advance registration must be valid");
    for (command, help, arguments) in [
        (
            "pause",
            "Pause the currently playing scenario-owned animation clip.",
            Vec::new(),
        ),
        (
            "resume",
            "Resume the currently paused scenario-owned animation clip.",
            Vec::new(),
        ),
        (
            "stop",
            "Stop scenario-owned playback and return its local time to zero.",
            Vec::new(),
        ),
        (
            "seek",
            "Seek the selected scenario-owned animation clip in seconds.",
            vec!["seconds".to_owned()],
        ),
        (
            "reset",
            "Restore scenario-owned playback to its initial selection and state.",
            Vec::new(),
        ),
    ] {
        shell
            .register_application_command(ApplicationCommandDescription::mutation(
                "runtime", command, help, arguments,
            ))
            .expect("runtime playback lifecycle registration must be valid");
    }
    shell
        .register_application_command(ApplicationCommandDescription::query(
            "runtime",
            "presentation",
            "Inspect the scenario-owned resolved presentation targets.",
            Vec::new(),
        ))
        .expect("runtime presentation query registration must be valid");
    shell
        .register_application_command(ApplicationCommandDescription::mutation(
            "runtime",
            "select-arm",
            "Select the scenario-owned arm presentation target.",
            Vec::new(),
        ))
        .expect("runtime presentation mutation registration must be valid");
    for (command, help) in [
        (
            "set-arm-hotspot",
            "Mark the scenario-owned arm presentation target as a hotspot.",
        ),
        (
            "clear-arm-selection",
            "Clear the scenario-owned arm selection presentation override.",
        ),
        (
            "clear-arm-hotspot",
            "Clear the scenario-owned arm hotspot presentation override.",
        ),
        (
            "select-missing-target",
            "Exercise the scenario provider's explicit unknown-target rejection.",
        ),
    ] {
        shell
            .register_application_command(ApplicationCommandDescription::mutation(
                "runtime",
                command,
                help,
                Vec::new(),
            ))
            .expect("runtime presentation lifecycle registration must be valid");
    }
    let record = shell.execute_with_application_handler(
        &runtime_source,
        "application runtime list-animations",
        |invocation| {
            assert_eq!(invocation.owner, "runtime");
            assert_eq!(invocation.command, "list-animations");
            let clips = runtime.animation_catalog();
            ApplicationCommandResult::Query {
                result: ApplicationQueryResult {
                    summary: format!("{} scenario-owned animation clips", clips.len()),
                    fields: clips
                        .iter()
                        .map(|clip| {
                            ApplicationQueryField::visible(
                                format!("clip {}", clip.id),
                                format!(
                                    "{}; {:.3}s; {} translation channel(s); nodes {:?}",
                                    clip.name,
                                    clip.duration_seconds,
                                    clip.translation_channels,
                                    clip.animated_nodes
                                ),
                            )
                        })
                        .collect(),
                },
            }
        },
    );
    println!(
        "> application runtime list-animations\n{}\n",
        record.projection
    );

    let record = shell.execute_with_application_handler(
        &runtime_source,
        "application runtime play 1",
        |invocation| {
            let Some(clip) = invocation
                .arguments
                .first()
                .and_then(|value| value.parse::<usize>().ok())
            else {
                return ApplicationCommandResult::Mutation {
                    receipt: ApplicationMutationReceipt {
                        accepted: false,
                        applied_tick: None,
                        resulting_revision: None,
                        message: "runtime requires: play <clip>".to_owned(),
                    },
                };
            };
            ApplicationCommandResult::Mutation {
                receipt: playback_receipt(
                    runtime.apply_playback_command(PlaybackCommand::Play { clip }),
                ),
            }
        },
    );
    println!("> application runtime play 1\n{}\n", record.projection);

    let record = shell.execute_with_application_handler(
        &runtime_source,
        "application runtime play 99",
        |invocation| {
            let clip = invocation
                .arguments
                .first()
                .and_then(|value| value.parse::<usize>().ok())
                .expect("the fixed corpus command must contain one numeric clip index");
            ApplicationCommandResult::Mutation {
                receipt: playback_receipt(
                    runtime.apply_playback_command(PlaybackCommand::Play { clip }),
                ),
            }
        },
    );
    println!("> application runtime play 99\n{}\n", record.projection);

    let record = shell.execute_with_application_handler(
        &runtime_source,
        "application runtime advance",
        |_| {
            runtime.advance_animation_fixed_step();
            ApplicationCommandResult::Mutation {
                receipt: ApplicationMutationReceipt {
                    accepted: true,
                    applied_tick: Some(runtime.tick()),
                    resulting_revision: Some(runtime.revision()),
                    message: "advanced playback by one fixed scenario step".to_owned(),
                },
            }
        },
    );
    println!("> application runtime advance\n{}\n", record.projection);

    // The lifecycle commands are intentionally routed through the same
    // application-owned adapter. The rejected pause after stop is evidence
    // that the runtime, rather than the shell, retains transition validity.
    for command in [
        "application runtime pause",
        "application runtime resume",
        "application runtime seek 0.750",
        "application runtime stop",
        "application runtime pause",
        "application runtime reset",
    ] {
        let record =
            shell.execute_with_application_handler(&runtime_source, command, |invocation| {
                match playback_command_from_invocation(invocation) {
                    Ok(command) => ApplicationCommandResult::Mutation {
                        receipt: playback_receipt(runtime.apply_playback_command(command)),
                    },
                    Err(message) => ApplicationCommandResult::Mutation {
                        receipt: ApplicationMutationReceipt {
                            accepted: false,
                            applied_tick: None,
                            resulting_revision: None,
                            message,
                        },
                    },
                }
            });
        println!("> {command}\n{}\n", record.projection);
    }

    let record = shell.execute_with_application_handler(
        &runtime_source,
        "application runtime playback",
        |_| ApplicationCommandResult::Query {
            result: playback_query(&runtime),
        },
    );
    println!("> application runtime playback\n{}\n", record.projection);

    let record = shell.execute_with_application_handler(
        &runtime_source,
        "application runtime select-arm",
        |_| ApplicationCommandResult::Mutation {
            receipt: presentation_receipt(runtime.select_arm_presentation()),
        },
    );
    println!("> application runtime select-arm\n{}\n", record.projection);

    // Presentation identity remains scenario-owned. These commands operate on
    // the adapter's explicit arm mapping rather than exposing renderer handles
    // or making the shell reconstruct presentation target identifiers.
    for command in [
        "application runtime set-arm-hotspot",
        "application runtime clear-arm-selection",
        "application runtime clear-arm-hotspot",
        "application runtime select-missing-target",
    ] {
        let record =
            shell.execute_with_application_handler(&runtime_source, command, |invocation| {
                let result = match invocation.command.as_str() {
                    "set-arm-hotspot" => runtime.set_arm_hotspot_presentation(),
                    "clear-arm-selection" => runtime.clear_arm_selection_presentation(),
                    "clear-arm-hotspot" => runtime.clear_arm_hotspot_presentation(),
                    "select-missing-target" => runtime.select_missing_presentation(),
                    other => {
                        return ApplicationCommandResult::Mutation {
                            receipt: ApplicationMutationReceipt {
                                accepted: false,
                                applied_tick: None,
                                resulting_revision: None,
                                message: format!(
                                    "runtime does not expose presentation command: {other}"
                                ),
                            },
                        };
                    }
                };
                ApplicationCommandResult::Mutation {
                    receipt: presentation_receipt(result),
                }
            });
        println!("> {command}\n{}\n", record.projection);
    }

    let record = shell.execute_with_application_handler(
        &runtime_source,
        "application runtime presentation",
        |_| ApplicationCommandResult::Query {
            result: presentation_query(&runtime),
        },
    );
    println!(
        "> application runtime presentation\n{}\n",
        record.projection
    );

    let target = runtime.arm_id().0;
    let record = shell.execute_with_mutation_handler(
        &runtime_source,
        &format!("application runtime set-enabled {target} false"),
        |invocation| {
            let parsed_target = invocation
                .arguments
                .first()
                .and_then(|value| value.parse::<u64>().ok());
            let enabled = invocation
                .arguments
                .get(1)
                .and_then(|value| match value.as_str() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                });
            let (Some(target), Some(enabled)) = (parsed_target, enabled) else {
                return ApplicationMutationReceipt {
                    accepted: false,
                    applied_tick: None,
                    resulting_revision: None,
                    message: "runtime requires: set-enabled <entity> <true|false>".to_owned(),
                };
            };

            let queued = runtime.enqueue(CommandRequest {
                id: 1,
                target,
                authority: CommandAuthority::Operator,
                expected_revision: Some(runtime.revision()),
                command: RuntimeCommand::SetEnabled { enabled },
            });
            if queued.disposition != CommandDisposition::Queued {
                return ApplicationMutationReceipt {
                    accepted: false,
                    applied_tick: queued.applied_tick,
                    resulting_revision: queued.resulting_revision,
                    message: format!("runtime rejected queued command: {:?}", queued.disposition),
                };
            }

            let next_tick = runtime.tick() + 1;
            let trace = runtime.apply_pending_at_tick(next_tick);
            let result = trace
                .results
                .into_iter()
                .next()
                .expect("applied runtime queue must return its command result");
            ApplicationMutationReceipt {
                accepted: result.disposition == CommandDisposition::Accepted,
                applied_tick: result.applied_tick,
                resulting_revision: result.resulting_revision,
                message: format!("runtime command result: {:?}", result.disposition),
            }
        },
    );
    println!(
        "> application runtime set-enabled {target} false\n{}\n",
        record.projection
    );

    let refreshed_runtime_source = source_from_runtime(runtime.observe(
        1,
        Some(EntityId(target)),
        ObservationLimits::default(),
    ));
    let record = shell.execute(&refreshed_runtime_source, "inspect world");
    println!(
        "> inspect world (after runtime apply)\n{}\n",
        record.projection
    );

    println!(
        "hello-observation-shell: records={}, final_format={:?}",
        shell.history().len(),
        shell.format()
    );
}

/// Translates the runtime corpus' public observation DTO into the shell's
/// provider-neutral snapshot. Neither library receives the other's runtime
/// state or mutation API.
fn source_from_runtime(envelope: ObservationEnvelope) -> ObservationSource {
    let payload = envelope.payload;
    ObservationSource {
        world: WorldObservation {
            revision: envelope.revision,
            entities: payload
                .entities
                .into_iter()
                .map(|id| EntityObservation { id })
                .collect(),
            component_types: payload
                .component_types
                .into_iter()
                .map(|entry| TypeObservation {
                    type_name: entry.type_name,
                    count: entry.count,
                })
                .collect(),
            resource_types: payload
                .resource_types
                .into_iter()
                .map(|entry| TypeObservation {
                    type_name: entry.type_name,
                    count: entry.count,
                })
                .collect(),
            relationship_types: payload
                .relationship_types
                .into_iter()
                .map(|relation| RelationshipObservation {
                    type_name: relation.type_name,
                    edges: relation
                        .edges
                        .into_iter()
                        .map(|edge| RelationshipEdgeObservation {
                            source: edge.source,
                            targets: edge.targets,
                        })
                        .collect(),
                })
                .collect(),
        },
        diagnostics: DiagnosticsObservation {
            dropped_records: 0,
            records: payload
                .diagnostics
                .into_iter()
                .enumerate()
                .map(|(sequence, diagnostic)| DiagnosticObservation {
                    sequence: sequence as u64,
                    severity: "information".to_owned(),
                    kind: diagnostic.code.to_owned(),
                    source: diagnostic.owner.to_owned(),
                    message: diagnostic.message,
                    performance: None,
                })
                .collect(),
        },
    }
}

/// Projects caller-owned playback state without exposing importer internals to
/// the shell. The scenario remains the authority for state transitions.
fn playback_query(runtime: &RuntimeInspectionAdapter) -> ApplicationQueryResult {
    let state = runtime.playback();
    let clip = runtime.animation_catalog().get(state.selected_clip);
    ApplicationQueryResult {
        summary: "scenario-owned playback state".to_owned(),
        fields: vec![
            ApplicationQueryField::visible(
                "selected clip",
                clip.map(|clip| format!("{} ({})", clip.id, clip.name))
                    .unwrap_or_else(|| format!("{} (missing)", state.selected_clip)),
            ),
            ApplicationQueryField::visible("mode", format!("{:?}", state.mode)),
            ApplicationQueryField::visible(
                "local time",
                format!("{:.3}s", state.local_time_seconds),
            ),
            ApplicationQueryField::visible("speed", format!("{:.3}x", state.speed)),
            ApplicationQueryField::visible("looping", state.looping.to_string()),
        ],
    }
}

fn playback_receipt(result: PlaybackCommandResult) -> ApplicationMutationReceipt {
    ApplicationMutationReceipt {
        accepted: matches!(
            result.disposition,
            hello_runtime_observation::PlaybackDisposition::Accepted
        ),
        applied_tick: None,
        resulting_revision: None,
        message: playback_receipt_message(&result),
    }
}

/// This scenario adapter owns lifecycle argument parsing. The shell has
/// already bounded and owner-qualified the command envelope, but it must not
/// acquire animation-command grammar or state-transition rules.
fn playback_command_from_invocation(
    invocation: &ApplicationCommandInvocation,
) -> Result<PlaybackCommand, String> {
    let no_arguments = |name: &str, command| {
        if invocation.arguments.is_empty() {
            Ok(command)
        } else {
            Err(format!("runtime requires: {name}"))
        }
    };

    match invocation.command.as_str() {
        "pause" => no_arguments("pause", PlaybackCommand::Pause),
        "resume" => no_arguments("resume", PlaybackCommand::Resume),
        "stop" => no_arguments("stop", PlaybackCommand::Stop),
        "reset" => no_arguments("reset", PlaybackCommand::Reset),
        "seek" => match invocation.arguments.as_slice() {
            [seconds] => seconds
                .parse::<f32>()
                .map(|seconds| PlaybackCommand::Seek { seconds })
                .map_err(|_| "runtime requires: seek <seconds>".to_owned()),
            _ => Err("runtime requires: seek <seconds>".to_owned()),
        },
        other => Err(format!(
            "runtime does not expose lifecycle command: {other}"
        )),
    }
}

fn playback_receipt_message(result: &PlaybackCommandResult) -> String {
    let diagnostic = result
        .diagnostic
        .as_ref()
        .map(|diagnostic| format!("; diagnostic={}: {}", diagnostic.code, diagnostic.message))
        .unwrap_or_default();
    format!(
        "playback result: {:?}; clip={}; mode={:?}; local_time={:.3}s{}",
        result.disposition,
        result.state.selected_clip,
        result.state.mode,
        result.state.local_time_seconds,
        diagnostic,
    )
}

/// Projects resolved presentation evidence supplied by the scenario adapter.
/// Source geometry and renderer resources remain below this shell boundary.
fn presentation_query(runtime: &RuntimeInspectionAdapter) -> ApplicationQueryResult {
    let presentation = runtime.presentation();
    ApplicationQueryResult {
        summary: format!(
            "{} scenario-owned resolved presentation target(s)",
            presentation.targets.len()
        ),
        fields: presentation
            .targets
            .into_iter()
            .map(|target| {
                ApplicationQueryField::visible(
                    target.target.to_string(),
                    format!("{:?}", target.resolved),
                )
            })
            .collect(),
    }
}

fn presentation_receipt(
    result: hello_runtime_observation::PresentationCommandResult,
) -> ApplicationMutationReceipt {
    ApplicationMutationReceipt {
        accepted: matches!(
            result.disposition,
            hello_runtime_observation::PresentationCommandDisposition::Accepted
        ),
        applied_tick: None,
        resulting_revision: None,
        message: format!(
            "presentation result: {:?}; target={}{}",
            result.disposition,
            result.target,
            result
                .diagnostic
                .as_ref()
                .map(|diagnostic| format!(
                    "; diagnostic={}: {}",
                    diagnostic.code, diagnostic.message
                ))
                .unwrap_or_default(),
        ),
    }
}
