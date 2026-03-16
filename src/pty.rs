use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::grid::TerminalGrid;

pub struct PtySession {
    pub id: String,
    pub grid: Arc<TerminalGrid>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PtySession {
    pub fn spawn(id: &str, command: &str) -> anyhow::Result<Self> {
        let pty_system = NativePtySystem::default();
        
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Support standard POSIX shell spawning
        // TODO: Expand for Windows
        let mut cmd = CommandBuilder::new("/bin/sh");
        cmd.arg("-c");
        cmd.arg(command);

        let mut child = pair.slave.spawn_command(cmd)?;
        
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let grid = Arc::new(TerminalGrid::new());
        let grid_clone = Arc::clone(&grid);

        // Background thread to read PTY bytes and ingest them into the ghostty grid
        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        grid_clone.ingest(&buf[..n]);
                    }
                    Err(_) => break, // Likely PTY closed
                }
            }
            // Ensure the child is cleaned up when done
            let _ = child.wait();
        });

        Ok(Self {
            id: id.to_string(),
            grid,
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    pub fn write_stdin(&self, input: &str) -> anyhow::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(input.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub fn read_history(&self, agent_id: &str, start_line: Option<usize>, max_lines: Option<usize>) -> String {
        self.grid.read_history(agent_id, start_line, max_lines)
    }
}
