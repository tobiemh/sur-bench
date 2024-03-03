use std::process::Command;

use tracing::{error, info};

pub(crate) struct DockerContainer {
    id: String,
    running: bool,
}

impl DockerContainer {
    pub fn start(image: &str, args: Arguments) -> Self {
        info!("Start Docker image {image}");
        let mut arguments =
            Arguments::new(["run"]);
        arguments.add(args.0);
        arguments.add(["-d", image]);
        let id = Self::docker(arguments);
        Self {
            id,
            running: true,
        }
    }

    pub fn stop(&mut self) {
        if self.running {
            info!("Stopping Docker container {}", self.id);
            Self::docker(Arguments::new(["stop", &self.id]));
            self.running = false;
        }
    }
    fn docker(args: Arguments) -> String {
        let mut command = Command::new("docker");

        let output = command.args(args.0).output().unwrap();
        let std_out = String::from_utf8(output.stdout).unwrap().trim().to_string();
        if !output.stderr.is_empty() {
            error!("{}", String::from_utf8(output.stderr).unwrap());
        }
        assert_eq!(output.status.code(), Some(0), "Docker command failure: {:?}", command);
        std_out
    }
}

impl Drop for DockerContainer {
    fn drop(&mut self) {
        // Be sure the container is stopped
        self.stop();
        // Delete the container
        info!("Delete Docker container {}", self.id);
        Self::docker(Arguments::new(["rm", &self.id]));
    }
}

pub(crate) struct Arguments(Vec<String>);

impl Arguments {
    pub(crate) fn new<I, S>(args: I) -> Self
        where
            I: IntoIterator<Item=S>,
            S: Into<String>,
    {
        let mut a = Self(vec![]);
        a.add(args);
        a
    }

    pub(crate) fn add<I, S>(&mut self, args: I)
        where
            I: IntoIterator<Item=S>,
            S: Into<String>,
    {
        for arg in args {
            self.0.push(arg.into());
        }
    }
}
