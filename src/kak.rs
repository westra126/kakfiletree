use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct KakClient {
    pub session: String,
    pub client: String,
}

impl KakClient {
    pub fn open_file(&self, path: &Path) {
        let cmd = format!(
            "evaluate-commands -client {} edit '{}'",
            self.client,
            path.display()
        );
        self.send(&cmd);
    }

    pub fn send(&self, cmd: &str) {
        if let Ok(mut child) = Command::new("kak")
            .args(["-p", &self.session])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(cmd.as_bytes());
            }
        }
    }
}
