#[cfg(test)]
mod tests {
    use std::process::{Child, Command};
    struct AutoKilledChild(Child);

    impl Drop for AutoKilledChild {
        fn drop(&mut self) {
            println!("Killing child");
            self.0.kill().expect("failed to kill child")
        }
    }
    impl From<Child> for AutoKilledChild {
        fn from(child: Child) -> AutoKilledChild {
            AutoKilledChild(child)
        }
    }
    impl AsRef<Child> for AutoKilledChild {
        fn as_ref(self: &AutoKilledChild) -> &Child {
            &self.0
        }
    }
    impl AutoKilledChild {
        fn new(child: Child) -> AutoKilledChild {
            AutoKilledChild(child)
        }
        fn kill(&mut self) -> Result<(), std::io::Error> {
            self.0.kill()
        }
    }

    #[test]
    fn test_integration1() -> Result<(), Box<dyn std::error::Error>> {
        let configfile = "./tests/simple_inventory.yaml";

        // compose server commandline
        // cargo run --bin server -- -c tests/simple_inventory.yaml serve
        let mut command = Command::new("cargo");
        command.args(format!("run --bin server -- -c {configfile} serve").split(" "));
        dbg!(&command);
        let mut server_process: AutoKilledChild = AutoKilledChild::new(command.spawn()?);

        // Make sure the process has exited before we exit

        // start the server in a subprocess, make stdout, stderr available for checking

        // compose client commandline
        let mut command = Command::new("cargo");
        command.args("run --bin client -- --name pool1 while -- /usr/bin/ls -l".split(" "));
        command.env("RP_SERVER", "http://localhost:3000");
        dbg!(&command);
        let mut client_process = command.spawn()?;

        let result = client_process.wait();
        dbg!(&result);
        assert!(
            result.expect("command did not run").success(),
            "Command did not ocmplete sucessuflly"
        );

        server_process.kill()?;
        // set env var
        // start the client in a subprocess
        // expect shell ok
        Ok(())
    }
}
