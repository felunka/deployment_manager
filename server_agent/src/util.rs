use std::process::Command;

pub fn command_output(command_str: &str, args: Option<Vec<&str>>, current_dir: Option<&str>) -> String {
    let mut command = Command::new(command_str);
    if args.is_some() {
        for arg in args.unwrap() {
            command.arg(arg);
        }
    }
    if current_dir.is_some() {
        command.current_dir(current_dir.unwrap());
    }
    let output = command.output().expect("Failed to execute command");
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    println!("{}", command_str);
    println!("{}", stdout_str);
    println!("{}", stderr_str);
    
    stdout_str
}
