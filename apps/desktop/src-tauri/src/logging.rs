//! File logging: daily-rotated JSON lines, 7 days kept, with credential
//! redaction applied to every line before it reaches disk.

use std::io::{self, Write};
use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// Wraps a writer and masks credentials in everything written through it.
pub struct RedactingWriter<W: Write> {
    inner: W,
}

impl<W: Write> RedactingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        self.inner
            .write_all(sw_core::redact::redact_secrets(&text).as_bytes())?;
        // Report the original length: the caller's bytes were all consumed.
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Installs the global subscriber. The returned guard flushes pending lines
/// on drop and must live as long as the app.
pub fn init(log_dir: &Path) -> Result<WorkerGuard, String> {
    std::fs::create_dir_all(log_dir).map_err(|e| format!("create log dir: {e}"))?;
    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix("scribewell")
        .filename_suffix("log")
        .max_log_files(7)
        .build(log_dir)
        .map_err(|e| format!("create log file: {e}"))?;
    let (writer, guard) = tracing_appender::non_blocking(RedactingWriter::new(appender));

    let filter =
        EnvFilter::try_from_env("SCRIBEWELL_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let file_layer = fmt::layer()
        .json()
        .with_writer(writer)
        .with_current_span(false);
    let console_layer = cfg!(debug_assertions).then(|| fmt::layer().with_writer(io::stderr));

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| format!("install subscriber: {e}"))?;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(panic = %info, "panic");
        default_hook(info);
    }));
    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacting_writer_masks_keys() {
        let mut out = Vec::new();
        {
            let mut w = RedactingWriter::new(&mut out);
            let line = b"{\"msg\":\"auth failed for sk-ant-api03-SECRETSECRETSECRET\"}\n";
            assert_eq!(w.write(line).unwrap(), line.len());
        }
        let written = String::from_utf8(out).unwrap();
        assert!(!written.contains("SECRETSECRET"));
        assert!(written.contains("sk-[REDACTED]"));
    }
}
