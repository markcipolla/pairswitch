use clap::ArgMatches;
use std::path::PathBuf;

pub fn commit(sub_matches: &ArgMatches) {
  let paths = sub_matches
    .values_of_os("PATH")
    .unwrap_or_default()
    .map(PathBuf::from)
    .collect::<Vec<_>>();

  println!("path {:?}", paths);

  let source = sub_matches
    .value_of_os("SOURCE")
    .unwrap();

  println!("source {:?}", source);

  let sha = sub_matches
    .value_of_os("SHA")
    .unwrap();

  println!("sha {:?}", sha);
}
