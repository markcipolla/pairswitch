pub mod run_command {
  use std::process::{Command, Stdio};
  use std::str;
  use std::io::Write;
  use std::{thread, time};

  #[allow(unused)]
  pub fn run_basic(program:&str) -> String {
    let arguments: &[&str] = &[];
    let std_in_string: &str = "";
    run(program,arguments,std_in_string)
   }

  pub fn run(program: &str, arguments: &[&str], std_in_string: &str) -> String {
    let mut child = Command::new(program)
      .args(arguments)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
      .expect("failed to execute child");

    {
      let stdin = child.stdin.as_mut().expect("Failed to get stdin");
      stdin.write_all(std_in_string.as_bytes()).expect("Failed to write to stdin");
    }

    let check_every = time::Duration::from_millis(10);
    loop {
      match child.try_wait() {
        Ok(Some(_status)) => {break;},  // finished running
        Ok(None) => {}                  // still running
        Err(e) => {panic!("error attempting to wait: {}", e)},
      }
      thread::sleep(check_every);
    }

    let output = child
      .wait_with_output()
      .expect("failed to wait on child");

    let final_output: String = match str::from_utf8(&output.stdout){
      Ok(output) => {output.to_string()},
      Err(e) => {panic!("{}", e);},
    };

    final_output
  }
}
