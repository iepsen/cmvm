use clap::{arg, App, AppSettings};

mod cache;
mod constants;
mod commands;
mod versions;
mod releases;
mod http;

fn main() {
  let matches = App::new("cmvm")
    .about("Cmake version manager")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .setting(AppSettings::AllowExternalSubcommands)
    .setting(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
    .subcommand(
      App::new("install")
        .about("Install a new cmake version")
        .arg(arg!(<VERSION> "The cmake version to install"))
        .setting(AppSettings::ArgRequiredElseHelp),
    )
    .subcommand(
      App::new("use")
        .about("Set a cmake version to use")
        .arg(arg!(<VERSION> "The cmake version to use"))
        .setting(AppSettings::ArgRequiredElseHelp),
    )
    .subcommand(
      App::new("list")
        .about("List installed cmake versions")
    )
    .subcommand(
      App::new("list-remote")
        .about("List remove cmake versions available")
    )
    .subcommand(App::new("version").about("cmvm version")).get_matches();

    match matches.subcommand() {
      Some(("install", sub_matches)) => {
        commands::install_version(sub_matches.value_of("VERSION").expect("required"));
      }
      Some(("use", sub_matches)) => {
        commands::use_version(sub_matches.value_of("VERSION").expect("required"));
      }
      Some(("list", _)) => {
        commands::list_versions();
      }
      Some(("list-remote", _)) => {
        commands::list_remote_versions();
      }
      Some(("version", _)) => {
        println!(
          "cmvm version: {}",
          env!("CARGO_PKG_VERSION")
        );
      }
      _ => unreachable!(),
    }
}
