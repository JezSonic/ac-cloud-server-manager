use crate::core::error::ServerManagerError;

use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use tokio::sync::mpsc;
use tokio::task;

pub struct SshClient {
    pub session: Session,
}

impl SshClient {
    /// Establishes an SSH connection synchronously
    pub fn connect(
        host: &str,
        username: &str,
        key_path: &str,
        port: u16,
    ) -> Result<Self, ServerManagerError> {
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .map_err(|e| ServerManagerError::SetupError(e.to_string()))?;

        let mut sess = Session::new().map_err(|e| ServerManagerError::SetupError(e.to_string()))?;

        sess.set_tcp_stream(tcp);
        sess.handshake()
            .map_err(|e| ServerManagerError::SetupError(e.to_string()))?;

        // Use SSH key
        let key_path_expanded = shellexpand::tilde(key_path).to_string();
        sess.userauth_pubkey_file(username, None, Path::new(&key_path_expanded), None)
            .map_err(|e| ServerManagerError::SetupError(format!("Key auth failed: {}", e)))?;

        if !sess.authenticated() {
            return Err(ServerManagerError::SetupError(
                "Authentication failed".to_string(),
            ));
        }

        Ok(Self { session: sess })
    }

    /// Executes a simple command and returns (stdout, exit_status)
    pub fn execute_command(&self, command: &str) -> Result<(String, i32), ServerManagerError> {
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;
        let mut result = String::new();
        channel.read_to_string(&mut result)?;
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        Ok((result, exit_status))
    }

    /// Executes a command and streams output line by line (or chunk by chunk) via a callback.
    pub fn execute_command_stream<F>(
        &self,
        command: &str,
        mut callback: F,
    ) -> Result<i32, ServerManagerError>
    where
        F: FnMut(&str),
    {
        let mut channel = self.session.channel_session()?;
        // Request a pseudo-terminal to ensure we get interactive output and prevent sudo from hanging
        let _ = channel.request_pty("vt100", None, None);

        // Wrap command to merge stderr into stdout, so we don't lose error messages from fake/failed commands
        // We use stdbuf to disable buffering if available
        let wrapped_cmd = format!(
            "stdbuf -oL -eL bash -c '{} 2>&1' || bash -c '{} 2>&1'",
            command.replace("'", "'\\''"),
            command.replace("'", "'\\''")
        );
        channel.exec(&wrapped_cmd)?;

        let mut buf = [0; 4096];
        loop {
            let bytes_read = channel.read(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            let s = String::from_utf8_lossy(&buf[..bytes_read]);
            callback(&s);
        }
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        Ok(exit_status)
    }

    /// A reusable controller for fetching logs in real-time.
    /// Returns a receiver that will yield new log chunks.
    pub fn spawn_log_streamer(
        host: String,
        username: String,
        key_path: String,
        port: u16,
        command: String,
    ) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let client_res =
                task::spawn_blocking(move || SshClient::connect(&host, &username, &key_path, port))
                    .await;

            if let Ok(Ok(client)) = client_res {
                let _ = tokio::task::spawn_blocking(move || {
                    let _ = client.execute_command_stream(&command, |chunk| {
                        let _ = tx.blocking_send(chunk.to_string());
                    });
                })
                .await;
            }
        });

        rx
    }
}
