use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::debug;

pub struct StdioTransport {
    stdin: BufReader<tokio::io::Stdin>,
    stdout: tokio::io::Stdout,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            stdin: BufReader::new(tokio::io::stdin()),
            stdout: tokio::io::stdout(),
        }
    }

    pub async fn read_message(&mut self) -> Result<Option<String>> {
        let mut line = String::new();
        let bytes_read = self
            .stdin
            .read_line(&mut line)
            .await
            .context("Failed to read from stdin")?;

        if bytes_read == 0 {
            debug!("EOF on stdin");
            return Ok(None);
        }

        let line = line.trim();
        if line.is_empty() {
            return Ok(None);
        }

        debug!("Read message: {}...", &line[..line.len().min(80)]);
        Ok(Some(line.to_string()))
    }

    pub async fn write_message(&mut self, message: &str) -> Result<()> {
        debug!("Writing message: {}...", &message[..message.len().min(80)]);

        self.stdout
            .write_all(message.as_bytes())
            .await
            .context("Failed to write to stdout")?;

        self.stdout
            .write_all(b"\n")
            .await
            .context("Failed to write newline to stdout")?;

        self.stdout
            .flush()
            .await
            .context("Failed to flush stdout")?;

        Ok(())
    }
}
