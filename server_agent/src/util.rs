use std::process::Command;

pub fn command_output(command: &str, args: Option<Vec<&str>>, current_dir: Option<&str>) -> String {
    let mut command = Command::new(command);
    if args.is_some() {
        for arg in args.unwrap() {
            command.arg(arg);
        }
    }
    if current_dir.is_some() {
        command.current_dir(current_dir.unwrap());
    }
    let output = command.output().expect("Failed to execute command");
    String::from_utf8_lossy(&output.stdout).to_string()
}
