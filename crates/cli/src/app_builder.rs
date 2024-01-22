use std::path::PathBuf;

use clap::arg;
use clap::command;
use clap::value_parser;
use clap::App;
use clap::ArgMatches;
use clap::Command;

use crate::args::HostMode;
use crate::command::DryRunArg;
use crate::command::ProveArg;
use crate::command::SetupArg;
use crate::command::Subcommands;
use crate::command::VerifyArg;
use crate::ZkWasmCli;

fn setup_command() -> Command<'static> {
    let command = Command::new("setup")
        .about("Setup a new zkWasm circuit for provided Wasm image")
        .arg(
            arg!(-k <K> "Size of the circuit.")
                .default_value(if cfg!(feature = "continuation") {
                    "22"
                } else {
                    "18"
                })
                .value_parser(value_parser!(u32).range(18..))
                .required(false),
        )
        .arg(
            arg!(
                --host <HOST_MODE> "Specify execution host envionment for the runtime"
            )
            .default_value("default")
            .value_parser(value_parser!(HostMode))
            .required(false),
        )
        .arg(
            arg!(
                --phantom <PHANTOM_FUNCTIONS> "Specify phantom functions whose body will be ignored in the circuit"
            ).takes_value(true)
            .value_delimiter(',')   
            .required(false)
        );

    let command = if cfg!(not(feature = "uniform-circuit")) {
        command .arg(
            arg!(
                --wasm <WASM> "Path to the Wasm image"
            ).value_parser(value_parser!(PathBuf))
        )
    } else {
        command
    };

    command
       
}

fn dry_run_command() -> Command<'static> {
    Command::new("dry-run").about("Execute the Wasm image without generating a proof")
}

fn prove_command() -> Command<'static> {
    Command::new("prove").about("Execute the Wasm image and generate a proof")
}

fn verify_command() -> Command<'static> {
    Command::new("verify").about("Verify the proof")
}

pub(crate) fn app() -> App<'static> {
    command!()
        .author("delphinus-lab")
        .arg(arg!(<NAME> "Name of the configuration."))
        .arg(
            arg!(
                --params <PARAMS> "Directory to setup params and configuration."
            )
            .value_parser(value_parser!(PathBuf)),
        )
        .subcommand(setup_command())
        .subcommand(dry_run_command())
        .subcommand(prove_command())
        .subcommand(verify_command())
        .subcommand_required(true)
}

impl Into<SetupArg> for &ArgMatches {
    fn into(self) -> SetupArg {
        SetupArg {
            k: *self.get_one::<u32>("K").unwrap(),
            host_mode: *self.get_one::<HostMode>("host").unwrap(),
            phantom_functions: self
                .get_many::<String>("phantom")
                 .unwrap_or_default()
                .map(|v| v.to_string())
                .collect::<Vec<_>>(),
            wasm_image: self.get_one::<PathBuf>("wasm").cloned()
        }
    }
}

impl Into<DryRunArg> for &ArgMatches {
    fn into(self) -> DryRunArg {
        DryRunArg {
            wasm_image: todo!(),
            output_dir: todo!(),
            running_arg: todo!(),
        }
    }
}

impl Into<ProveArg> for &ArgMatches {
    fn into(self) -> ProveArg {
        ProveArg {
            wasm_image: todo!(),
            output_dir: todo!(),
            running_arg: todo!(),
            mock_test: todo!(),
        }
    }
}

impl Into<VerifyArg> for &ArgMatches {
    fn into(self, ) -> VerifyArg {
        VerifyArg {
            output_dir: todo!(),
        }
    }
}

impl Into<ZkWasmCli> for ArgMatches {
    fn into(self) -> ZkWasmCli {
        let subcommand = match self.subcommand() {
            Some(("setup", sub_matches)) => Subcommands::Setup(sub_matches.into()),
            Some(("dry-run", sub_matches)) => Subcommands::DryRun(sub_matches.into()),
            Some(("prove", sub_matches)) => Subcommands::Prove(sub_matches.into()),
            Some(("verify", sub_matches)) => Subcommands::Verify(sub_matches.into()),
            _ => unreachable!("unknown subcommand"),
        };

        ZkWasmCli {
            name: self.get_one::<String>("NAME").unwrap().to_owned(),
            params_dir: self.get_one::<PathBuf>("params").unwrap().to_owned(),
            subcommand,
        }
    }
}
