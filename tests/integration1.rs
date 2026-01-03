#[cfg(test)]
mod tests {
    use std::{process::Command};
    #[test]
    fn test_integration1() {
        let configfile = "./tests/simple_inventory.yaml";

        // compose server commandline
        // cargo run --bin server -- -c tests/simple_inventory.yaml serve
        let mut command = Command::new("cargo");
        command.args(format!("run --bin server -- -c {configfile} serve").split(" "));
        let mut server_process = command.spawn().unwrap();

        // Make sure the process has exited before we exit

        // start the server in a subprocess, make stdout, stderr available for checking

        // compose client commandline
        let mut command = Command::new("cargo");
        command.args("run --bin client -- --name pool1 while  -- ls -al".split(" "));
        command.env("RP_SERVER", "http://localhost:3000");
        let mut client_process = command.spawn().unwrap();

        let result = client_process.wait();
        dbg!(&result);
        assert!(
            result.expect("command did not run").success(),
            "Command did not ocmplete sucessuflly"
        );

        // TODO: fix lint error
        server_process.kill().unwrap();
        // set env var
        // start the client in a subprocess
        // expect shell ok
    }
}
