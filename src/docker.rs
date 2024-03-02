use std::process::Command;

use tracing::{error, info};

pub(crate) struct DockerContainer {
    id: String,
    running: bool,
}

impl DockerContainer {
    pub fn start(image: &str) -> Self {
        info!("Start Docker image {image}");
        let args =
            Arguments::new(["run", image]);
        let id = Self::docker(args);
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

struct Arguments(Vec<String>);

impl Arguments {
    fn new<I, S>(args: I) -> Self
        where
            I: IntoIterator<Item=S>,
            S: Into<String>,
    {
        let mut a = Self(vec![]);
        a.add(args);
        a
    }

    fn add<I, S>(&mut self, args: I)
        where
            I: IntoIterator<Item=S>,
            S: Into<String>,
    {
        for arg in args {
            self.0.push(arg.into());
        }
    }
}
