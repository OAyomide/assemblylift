use clap::ArgMatches;

pub type CommandFn = fn(Option<&ArgMatches>);

pub mod init;