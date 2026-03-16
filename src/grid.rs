use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TerminalGrid {
    buffer: Arc<Mutex<Vec<String>>>,
    max_history: usize,
    /// Tracks the last read line index for different agent connections
    cursors: Arc<Mutex<HashMap<String, usize>>>,
}

impl TerminalGrid {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            max_history: 100_000, 
            cursors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn ingest(&self, bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes);
        let mut buf = self.buffer.lock().unwrap();
        
        let mut lines = text.split('\n').collect::<Vec<_>>();
        
        if let Some(last_buf_line) = buf.last_mut() {
            if !lines.is_empty() {
                last_buf_line.push_str(&strip_ansi_escapes(lines[0]));
                lines.remove(0);
            }
        }
        
        for line in lines {
            let clean_line = strip_ansi_escapes(line);
            buf.push(clean_line);
        }
        
        if buf.len() > self.max_history {
            let overflow = buf.len() - self.max_history;
            buf.drain(0..overflow);
            
            // Adjust agent cursors to account for the drain
            let mut cursors = self.cursors.lock().unwrap();
            for cursor in cursors.values_mut() {
                *cursor = cursor.saturating_sub(overflow);
            }
        }
    }

    pub fn read_history(
        &self, 
        agent_id: &str, 
        start_line: Option<usize>, 
        max_lines: Option<usize>
    ) -> String {
        let buf = self.buffer.lock().unwrap();
        let total_lines = buf.len();
        
        if total_lines == 0 {
            return String::from("Session is empty.");
        }

        let mut cursors = self.cursors.lock().unwrap();
        let limit = max_lines.unwrap_or(100);

        // If start_line is omitted, fetch from this agent's specific cursor
        // If the agent is new, default to tailing the last `limit` lines
        let start = start_line.unwrap_or_else(|| {
            *cursors.entry(agent_id.to_string()).or_insert_with(|| total_lines.saturating_sub(limit))
        });
        
        let safe_start = start.min(total_lines);
        let end = (safe_start + limit).min(total_lines);

        // Update the cursor so the next read starts exactly where we left off
        cursors.insert(agent_id.to_string(), end);

        let output = buf[safe_start..end].join("\n");
        format!("Showing lines {}-{} of {} (Agent: {})\n---\n{}", safe_start, end, total_lines, agent_id, output)
    }
}

fn strip_ansi_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c != '\r' {
            result.push(c);
        }
    }
    result
}
