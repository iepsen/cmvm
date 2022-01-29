use clap::{arg, App, AppSettings};

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
    .subcommand(
      App::new("version")
        .about("cmvm version")
    )
    .get_matches();

    match matches.subcommand() {
      Some(("install", sub_matches)) => {
        println!(
          "TODO: Install cmake version {}",
          sub_matches.value_of("VERSION").expect("required")
        );
      }
      Some(("use", sub_matches)) => {
        println!(
          "TODO: Use cmake version {}",
          sub_matches.value_of("VERSION").expect("required")
        );
      }
      Some(("list", _)) => {
        println!(
          "TODO: List cmake versions",
        );
      }
      Some(("list-remote", _)) => {
        println!(
          "TODO: List remote cmake versions",
        );
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