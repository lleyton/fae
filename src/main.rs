use async_recursion::async_recursion;
use dashmap::DashMap;
use futures::future::join_all;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fs};
use tokio::process::Command;
use toml;

mod out;

#[derive(Deserialize, Debug)]
struct CommandConfig {
    uses: Option<Vec<String>>,
    run: Option<String>,
    stdout: Option<bool>,
    stderr: Option<bool>,
    cache: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct NPMPackage {
    scripts: Option<HashMap<String, String>>,
}

async fn run_command(
    script: &str,
    command: String,
    args: Vec<String>,
    env_overrides: &HashMap<String, String>,
    stdout: bool,
    stderr: bool,
) {
    let args: Vec<String> = args
        .iter()
        .map(|s| {
            if s.contains(" ") {
                "'".to_string() + s + "'"
            } else {
                s.to_string()
            }
        })
        .collect();

    let mut cmd = Command::new("sh");

    cmd.envs(env_overrides);

    if args.len() > 0 {
        cmd.arg("-c").arg(command.clone() + " " + &args.join(" "));
    } else {
        cmd.arg("-c").arg(&command);
    }

    for arg in args {
        cmd.arg(arg);
    }

    let process = cmd
        .stdout(if stdout {
            std::process::Stdio::inherit()
        } else {
            std::process::Stdio::null()
        })
        .stderr(if stderr {
            std::process::Stdio::inherit()
        } else {
            std::process::Stdio::null()
        })
        .stdin(std::process::Stdio::inherit())
        .spawn();

    let status = process
        .expect(&format!(
            "Failed to create command {} from script {}",
            &command, script
        ))
        .wait()
        .await
        .expect(&format!(
            "Failed to wait for command {} from script {}",
            command, script
        ));
    let code = status.code().expect(&format!("Failed to get exit code fom command {} from script {}, this probably means it was terminated with a signal", command, script));

    if code != 0 {
        panic!(
            "Command {} from script {} failed with exit code {}",
            &command, script, code
        );
    }
}

// TODO: It might be better to build some sort of dependency graph instead of just recusively executing the scripts
// For now, this will serve most use cases perfectly
#[async_recursion]
async fn run_script(
    config: &HashMap<String, CommandConfig>,
    script: &str,
    args: Vec<String>,
    env_overrides: &HashMap<String, String>,
    cache: Arc<DashMap<String, bool>>,
) {
    let value = match config.get(script) {
        Some(value) => value,
        None if which::which_in(
            script,
            Some("node_modules/.bin"),
            env::current_dir()
                .expect("This is awkward, but we couldn't get your working directory"),
        )
        .is_ok() =>
        {
            return run_command(script, script.to_string(), args, env_overrides, true, true).await
        }
        _ => panic!("Could not find script \"{}\" in config file", script),
    };

    if value.cache.unwrap_or(true) {
        let completed = match cache.get(script) {
            Some(value) if value.value() == &true => true,
            _ => {
                cache.insert(script.to_string(), true);
                false
            }
        };

        if completed {
            return;
        }
    }

    if let Some(uses) = &value.uses {
        let dependencies = uses
            .iter()
            .map(|use_script| {
                run_script(config, use_script, Vec::new(), env_overrides, cache.clone())
            })
            .collect::<Vec<_>>();

        join_all(dependencies).await;
    }

    if let Some(run) = &value.run {
        run_command(
            script,
            run.to_string(),
            args,
            env_overrides,
            value.stdout.unwrap_or(true),
            value.stderr.unwrap_or(true),
        )
        .await
    }
}

#[tokio::main]
async fn main() {
    // Before we do anything, we want to change how the panic message will look
    // so they are more user friendly
    out::change_panic_message();

    let args: Vec<String> = env::args().collect();
    let script = &args.get(1);

    if script.is_none() {
        let branch = env!("VERGEN_GIT_BRANCH");
        println!(
            "Fae {} v{}",
            if branch == "main" { "stable" } else { branch },
            env!("VERGEN_BUILD_SEMVER")
        );
        println!("Usage: {} <script>", args[0]);

        return;
    }

    // We have already caught the case where not enough args are passed in, so
    // this is safe
    let script = script.unwrap();

    let mut env_overrides = HashMap::new();
    let mut config = HashMap::new();

    if let Ok(content) = fs::read_to_string("package.json") {
        let package: NPMPackage =
            serde_json::from_str(&content).expect("Failed to parse package.json");
        env_overrides.insert(
            "PATH".to_string(),
            env::current_dir()
                .expect("This is awkward, but we couldn't get your working directory")
                .join("node_modules")
                .join(".bin")
                .into_os_string()
                .into_string()
                .expect("This is awkward, we couldn't convert the OS string into a Rust string")
                + ":"
                + &env::var_os("PATH")
                    .expect("This is awkward, but PATH is not set")
                    .into_string()
                    .expect(
                        "This is awkward, we couldn't convert the OS string into a Rust string",
                    ),
        );

        if let Some(scripts) = &package.scripts {
            for (script, command) in scripts {
                config.insert(
                    script.to_string(),
                    CommandConfig {
                        uses: None,
                        run: Some(command.to_string()),
                        stdout: None,
                        stderr: None,
                        cache: None,
                    },
                );
            }
        }
    }

    if let Ok(content) = fs::read_to_string("fae.toml") {
        let fae: HashMap<String, CommandConfig> = toml::from_str(&content)
            .expect("Could not parse config file, make sure that it is valid TOML");
        config.extend(fae);
    }

    let mut script_args = env::args();
    script_args.nth(1);

    run_script(
        &config,
        script,
        script_args.collect(),
        &env_overrides,
        Arc::new(DashMap::new()),
    )
    .await
}
