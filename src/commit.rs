use std::fs;
use clap::ArgMatches;

pub fn commit(sub_matches: &ArgMatches) {
  let path = sub_matches
    .value_of_os("PATH")
    .unwrap();
  println!("path {:?}", path);
  let contents = fs::read_to_string(path);
  println!("contents {:?}", contents);
  let source = sub_matches
    .value_of_os("SOURCE")
    .unwrap();

  println!("source {:?}", source);

  let sha = sub_matches
    .value_of_os("SHA")
    .unwrap();

  println!("sha {:?}", sha);
}
