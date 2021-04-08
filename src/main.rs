use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  login: String,
}

fn main() {
  let args = Cli::from_args();

  println!("{:?}", args.login);
}
