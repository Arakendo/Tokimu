use std::process::ExitCode;

use presentation_geometry_corpus::{
    all_cases, bless_case, compare_case, find_case, run_case, run_generated_case,
    write_glyph_artifacts, CaseReport, CorpusCase,
};

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().is_some_and(|argument| argument == "list") {
        for case in all_cases() {
            println!("{}", case.id());
        }
        return ExitCode::SUCCESS;
    }

    if arguments.len() == 1 && (arguments[0] == "compare-all" || arguments[0] == "bless-all") {
        let bless = arguments[0] == "bless-all";
        let mut failed = 0;
        for case in all_cases() {
            let result = if bless {
                bless_case(*case).map(|path| format!("blessed {}", path.display()))
            } else {
                compare_case(*case).map(|()| format!("golden match {}", case.id()))
            };
            match result {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("{error}");
                    failed += 1;
                }
            }
        }
        return if failed == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }

    if arguments.len() == 2 {
        let command = &arguments[0];
        let id = &arguments[1];
        if command == "bless" || command == "compare" {
            let Some(case) = find_case(id) else {
                eprintln!("unknown corpus case: {id}");
                return ExitCode::from(2);
            };
            let result = if command == "bless" {
                bless_case(case).map(|path| format!("blessed {}", path.display()))
            } else {
                compare_case(case).map(|()| format!("golden match {id}"))
            };
            match result {
                Ok(message) => {
                    println!("{message}");
                    return ExitCode::SUCCESS;
                }
                Err(error) => {
                    eprintln!("{error}");
                    return ExitCode::from(1);
                }
            }
        }
    }

    if arguments.len() == 3 && arguments[0] == "generate" {
        let Ok(seed) = arguments[1].parse::<u64>() else {
            eprintln!("invalid generation seed: {}", arguments[1]);
            return ExitCode::from(2);
        };
        let Ok(count) = arguments[2].parse::<usize>() else {
            eprintln!("invalid generation count: {}", arguments[2]);
            return ExitCode::from(2);
        };
        if count > 1000 {
            eprintln!("generation count must be at most 1000");
            return ExitCode::from(2);
        }
        let mut failed = 0;
        for index in 0..count {
            let report = run_generated_case(seed, index);
            print_report(&report);
            failed += usize::from(!report.passed());
        }
        println!("generated summary: count={count} failed={failed} seed={seed}");
        return if failed == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }

    let cases = match arguments.as_slice() {
        [] => all_cases().to_vec(),
        [command, id] if command == "run" => match find_case(id) {
            Some(case) => vec![case],
            None => {
                eprintln!("unknown corpus case: {id}");
                return ExitCode::from(2);
            }
        },
        [id] => match find_case(id) {
            Some(case) => vec![case],
            None => {
                eprintln!("unknown corpus case: {id}");
                return ExitCode::from(2);
            }
        },
        _ => {
            eprintln!(
                "usage: presentation-geometry-corpus [list|run <case-id>|compare <case-id>|bless <case-id>|<case-id>]\n       presentation-geometry-corpus [compare-all|bless-all]\n       presentation-geometry-corpus generate <seed> <count>"
            );
            return ExitCode::from(2);
        }
    };

    let total = cases.len();
    let mut passed = 0;
    for case in cases {
        let report = run_case(case);
        print_report(&report);
        if report.passed() {
            if let CorpusCase::Glyph(glyph) = case {
                match write_glyph_artifacts(glyph) {
                    Ok(path) => println!("  artifacts: {}", path.display()),
                    Err(error) => {
                        eprintln!("  artifact error: {error}");
                        return ExitCode::from(1);
                    }
                }
            }
            if let CorpusCase::W3cSvg(w3c) = case {
                match presentation_geometry_corpus::write_w3c_artifacts(w3c) {
                    Ok(path) => println!("  artifacts: {}", path.display()),
                    Err(error) => {
                        eprintln!("  artifact error: {error}");
                        return ExitCode::from(1);
                    }
                }
            }
        }
        passed += usize::from(report.passed());
    }

    println!("summary: passed={passed} failed={}", total - passed);
    if passed == total {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn print_report(report: &CaseReport) {
    println!("case {} producer={}", report.id, report.producer);
    println!(
        "  selected stages: {}",
        report
            .selected_stages
            .iter()
            .map(|stage| stage.name())
            .collect::<Vec<_>>()
            .join(" -> ")
    );
    for stage in &report.stages {
        println!(
            "  {}: {} {}",
            stage.stage.name(),
            stage.status.name(),
            stage.summary
        );
    }
    for diagnostic in &report.diagnostics {
        println!("  diagnostic: {diagnostic}");
    }
}
