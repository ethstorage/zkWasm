// #![deny(warnings)]

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use app_builder::app;
use command::Subcommands;
use delphinus_host::StandardHostEnvBuilder;
use delphinus_zkwasm::runtime::host::default_env::DefaultHostEnvBuilder;

use args::HostMode;

mod app_builder;
mod args;
mod command;
mod config;
mod names;

const TRIVIAL_WASM: &'static str = r#"
(module
    (func (export "zkmain"))
)
"#;

#[derive(Debug)]
struct ZkWasmCli {
    name: String,
    params_dir: PathBuf,
    subcommand: Subcommands,
}

/// Simple program to greet a person
fn main() -> Result<()> {
    {
        env_logger::init();
    }

    let app = app();

    let cli: ZkWasmCli = app.get_matches().into();

    println!("{:?}", cli);

    match cli.subcommand {
        Subcommands::Setup(arg) => {
            let wasm_binary = arg.wasm_image.as_ref().map_or(
                wabt::wat2wasm(&TRIVIAL_WASM).map_err(|err| anyhow::anyhow!(err)),
                |file| fs::read(file).map_err(|err| anyhow::anyhow!(err)),
            )?;

            match arg.host_mode {
                HostMode::DEFAULT => {
                    arg.setup::<DefaultHostEnvBuilder>(&cli.name, &cli.params_dir, wasm_binary)?;
                }
                HostMode::STANDARD => {
                    arg.setup::<StandardHostEnvBuilder>(&cli.name, &cli.params_dir, wasm_binary)?;
                }
            }
        }
        Subcommands::DryRun(_) => todo!(),
        Subcommands::Prove(_) => todo!(),
        Subcommands::Verify(_) => todo!(),
    }

    /*
        match cli.subcommand {
            Subcommands::Setup(arg) => {
                let wasm_binary = fs::read(&arg.wasm_image.value)?;

                match arg.host_mode.value {
                    HostMode::DEFAULT => {
                        Config::setup::<DefaultHostEnvBuilder>(
                            &cli.name,
                            arg.circuit_size.value,
                            wasm_binary,
                            arg.phantom_functions.value,
                            arg.host_mode.value,
                            &cli.params_dir,
                        )?;
                    }
                    HostMode::STANDARD => {
                        Config::setup::<StandardHostEnvBuilder>(
                            &cli.name,
                            arg.circuit_size.value,
                            wasm_binary,
                            arg.phantom_functions.value,
                            arg.host_mode.value,
                            &cli.params_dir,
                        )?;
                    }
                }
            }

            Subcommands::Prove(arg) => {
                let config = Config::read(&mut fs::File::open(
                    cli.params_dir.join(&name_of_config(&cli.name)),
                )?)?;

                fs::create_dir_all(&arg.output_dir)?;

                let public_inputs = parse_args(&arg.running_arg.public_inputs);
                let private_inputs = parse_args(&arg.running_arg.private_inputs);
                let context_inputs = parse_args(&arg.running_arg.context_inputs);

                match config.host_mode {
                    HostMode::DEFAULT => {
                        config.prove::<DefaultHostEnvBuilder>(
                            &arg.wasm_image.value,
                            &cli.params_dir,
                            &arg.output_dir,
                            ExecutionArg {
                                public_inputs,
                                private_inputs,
                                context_inputs,
                                context_outputs: ContextOutput::default(),
                            },
                            arg.running_arg.context_output,
                            arg.mock_test,
                        )?;
                    }
                    HostMode::STANDARD => {
                        config.prove::<StandardHostEnvBuilder>(
                            &arg.wasm_image.value,
                            &cli.params_dir,
                            &arg.output_dir,
                            delphinus_host::ExecutionArg {
                                public_inputs,
                                private_inputs,
                                context_inputs,
                                context_outputs: ContextOutput::default(),
                                indexed_witness: Rc::new(RefCell::new(HashMap::new())),
                                tree_db: None,
                            },
                            arg.running_arg.context_output,
                            arg.mock_test,
                        )?;
                    }
                }
            }

            Subcommands::DryRun(arg) => {
                let config = Config::read(&mut fs::File::open(
                    cli.params_dir.join(&name_of_config(&cli.name)),
                )?)?;

                fs::create_dir_all(&arg.output_dir)?;

                let public_inputs = parse_args(&arg.running_arg.public_inputs);
                let private_inputs = parse_args(&arg.running_arg.private_inputs);
                let context_inputs = parse_args(&arg.running_arg.context_inputs);

                match config.host_mode {
                    HostMode::DEFAULT => {
                        config.dry_run::<DefaultHostEnvBuilder>(
                            &arg.wasm_image.value,
                            &arg.output_dir,
                            ExecutionArg {
                                public_inputs,
                                private_inputs,
                                context_inputs,
                                context_outputs: ContextOutput::default(),
                            },
                            arg.running_arg.context_output,
                        )?;
                    }
                    HostMode::STANDARD => {
                        config.dry_run::<StandardHostEnvBuilder>(
                            &arg.wasm_image.value,
                            &arg.output_dir,
                            delphinus_host::ExecutionArg {
                                public_inputs,
                                private_inputs,
                                context_inputs,
                                context_outputs: ContextOutput::default(),
                                indexed_witness: Rc::new(RefCell::new(HashMap::new())),
                                tree_db: None,
                            },
                            arg.running_arg.context_output,
                        )?;
                    }
                }
            }

            Subcommands::Verify(arg) => {
                let config = Config::read(&mut fs::File::open(
                    cli.params_dir.join(&name_of_config(&cli.name)),
                )?)?;

                config.verify(&cli.params_dir, &arg.output_dir)?;
            }
        };
    */
    Ok(())
}
