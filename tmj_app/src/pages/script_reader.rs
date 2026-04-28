use std::fs::File;
use std::io::{self, Read, Seek};
use std::path::Path;

#[derive(Debug)]
pub struct SectionReadResult {
    pub content: String,
    pub is_eof: bool,
}

pub struct StreamSectionReader {
    file: File,
    /// 我们的缓冲区，包含所有已读取但未处理的数据
    buffer: Vec<u8>,
    /// 读取块的大小
    chunk_size: usize,
    /// 是否已到达文件结尾
    eof_reached: bool,
}

impl StreamSectionReader {
    pub fn new<P: AsRef<Path>>(path: P, chunk_size: usize) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            file,
            buffer: Vec::with_capacity(chunk_size * 2),
            chunk_size,
            eof_reached: false,
        })
    }

    pub fn reset(&mut self) -> io::Result<()> {
        self.buffer.clear();
        self.eof_reached = false;
        self.file.seek(io::SeekFrom::Start(0))?;
        Ok(())
    }

    pub fn read_section(&mut self, n: u64) -> io::Result<SectionReadResult> {
        const MAX_MARKER_LOOKBACK: usize = 20;
        let start_marker = format!("#{}", n).into_bytes();

        let mut start_found = false;
        let mut section_data_offset = 0;

        loop {
            // 如果已到达 EOF 且缓冲区中没有足够数据
            if self.eof_reached && self.buffer.is_empty() {
                if !start_found {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        format!("Marker #{} not found until EOF", n),
                    ));
                } else {
                    // 找到了起始标记但没找到结束标记
                    return Ok(SectionReadResult {
                        content: String::new(),
                        is_eof: true,
                    });
                }
            }

            // 继续读取：
            // - 未找到起始标记时，按原策略补足缓冲区；
            // - 找到起始标记后，即使当前缓冲区不小，也要持续读入直到出现结束标记或 EOF。
            let should_read_more = if self.eof_reached {
                false
            } else if start_found {
                true
            } else {
                // 未找到起始 marker 时也必须持续推进读取；否则在某些长度条件下会原地循环。
                true
            };
            if should_read_more {
                let mut chunk = vec![0u8; self.chunk_size];
                let bytes_read = self.file.read(&mut chunk)?;
                if bytes_read == 0 {
                    self.eof_reached = true;
                    // 继续处理缓冲区中剩余的数据
                    if self.buffer.is_empty() {
                        if !start_found {
                            return Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                format!("Marker #{} not found until EOF", n),
                            ));
                        } else {
                            let content_bytes = &self.buffer[section_data_offset..];
                            let content = String::from_utf8(content_bytes.to_vec())
                                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                            self.buffer.clear();
                            return Ok(SectionReadResult { content, is_eof: true });
                        }
                    }
                } else {
                    chunk.truncate(bytes_read);
                    self.buffer.extend(chunk);
                }
            }

            // 查找起始标记
            if !start_found {
                match Self::find_marker_boundary(&self.buffer, &start_marker) {
                    Some(pos) => {
                        start_found = true;
                        section_data_offset = pos;
                        // 丢弃标记之前的数据
                        self.buffer.drain(0..section_data_offset);
                        section_data_offset = 0;
                    }
                    None => {
                        // 没找到起始标记
                        if self.buffer.len() > self.chunk_size + MAX_MARKER_LOOKBACK {
                            let keep_from = self.buffer.len() - MAX_MARKER_LOOKBACK;
                            self.buffer.drain(0..keep_from);
                        }
                        if self.eof_reached {
                            return Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                format!("Marker #{} not found until EOF", n),
                            ));
                        }
                        continue;
                    }
                }
            }

            // 查找结束标记
            if start_found {
                // 结束条件：遇到下一个 section marker（不限于 n+1）或 EOF
                // 从当前 start_marker 之后开始找，避免把起始 marker 本身当成结束 marker。
                match Self::find_next_section_boundary(&self.buffer, start_marker.len()) {
                    Some(pos) => {
                        // 找到结束标记
                        let content_bytes = &self.buffer[0..pos];
                        let content = String::from_utf8(content_bytes.to_vec())
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                        // 关键：保留结束标记及之后的数据
                        let remaining = self.buffer.split_off(pos);
                        self.buffer = remaining;

                        return Ok(SectionReadResult { content, is_eof: false });
                    }
                    None => {
                        // 还没找到结束标记
                        if self.eof_reached {
                            // 文件结束了，返回已累积的内容
                            let content_bytes = &self.buffer[section_data_offset..];
                            let content = String::from_utf8(content_bytes.to_vec())
                                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                            self.buffer.clear();
                            return Ok(SectionReadResult { content, is_eof: true });
                        }
                        continue;
                    }
                }
            }
        }
    }

    fn find_marker_boundary(buffer: &[u8], marker: &[u8]) -> Option<usize> {
        let mut search_start = 0;
        loop {
            if let Some(relative_pos) = Self::find_bytes(buffer, search_start, marker) {
                let absolute_pos = relative_pos;
                let next_char_index = absolute_pos + marker.len();
                if next_char_index < buffer.len() {
                    let next_byte = buffer[next_char_index];
                    if next_byte >= b'0' && next_byte <= b'9' {
                        search_start = absolute_pos + 1;
                        continue;
                    }
                }
                return Some(absolute_pos);
            } else {
                return None;
            }
        }
    }

    fn find_next_section_boundary(buffer: &[u8], min_pos: usize) -> Option<usize> {
        if buffer.len() <= min_pos {
            return None;
        }
        for pos in min_pos..buffer.len() {
            if buffer[pos] != b'#' {
                continue;
            }
            // section marker 必须出现在行首：文件开头或前一个字节是换行
            if pos > 0 && buffer[pos - 1] != b'\n' && buffer[pos - 1] != b'\r' {
                continue;
            }
            let digit_pos = pos + 1;
            if digit_pos >= buffer.len() || !buffer[digit_pos].is_ascii_digit() {
                continue;
            }

            let mut end = digit_pos + 1;
            while end < buffer.len() && buffer[end].is_ascii_digit() {
                end += 1;
            }
            // marker 边界：后续不是数字（或到达缓冲区末尾）
            if end == buffer.len() || !buffer[end].is_ascii_digit() {
                return Some(pos);
            }
        }
        None
    }

    fn find_bytes(haystack: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || start >= haystack.len() {
            return None;
        }
        let search_slice = &haystack[start..];
        for (i, window) in search_slice.windows(needle.len()).enumerate() {
            if window == needle {
                return Some(start + i);
            }
        }
        None
    }
}
// ================= 使用示例 =================
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_reader() -> io::Result<()> {
        // 创建一个测试文件
        let test_path = "test_sections.txt";
        let content = r#"
Header info...
#1
This is section 1 content.
Line 2 of section 1.
#2
This is section 2 content.
It has multiple lines.
#3
Section 3 is the last one.
No end marker here.
"#;
        std::fs::write(test_path, content)?;

        // 初始化阅读器
        let mut reader = StreamSectionReader::new(test_path, 1024)?;

        // 读取第 1 节 (#1 到 #2)
        println!("--- Reading Section 1 ---");
        match reader.read_section(1) {
            Ok(result) => {
                println!("Content:\n{}", result.content);
                println!("Is EOF: {}", result.is_eof);
            }
            Err(e) => println!("Error: {}", e),
        }

        // 读取第 2 节 (#2 到 #3)
        println!("\n--- Reading Section 2 ---");
        match reader.read_section(2) {
            Ok(result) => {
                println!("Content:\n{}", result.content);
                println!("Is EOF: {}", result.is_eof);
            }
            Err(e) => println!("Error: {}", e),
        }

        // 读取第 3 节 (#3 到 #4，但文件里没有 #4，应触发 EOF)
        println!("\n--- Reading Section 3 ---");
        match reader.read_section(3) {
            Ok(result) => {
                println!("Content:\n{}", result.content);
                println!("Is EOF: {}", result.is_eof);
            }
            Err(e) => println!("Error: {}", e),
        }

        // 尝试读取不存在的第 5 节
        println!("\n--- Reading Section 5 (Not Exist) ---");
        match reader.read_section(5) {
            Ok(result) => {
                println!("Content:\n{}", result.content);
                println!("Is EOF: {}", result.is_eof);
            }
            Err(e) => println!("Error: {}", e),
        }

        // 清理测试文件
        std::fs::remove_file(test_path)?;

        Ok(())
    }

    #[test]
    fn test_long_section_crosses_chunk_boundary() -> io::Result<()> {
        let test_path = "test_sections_long.txt";
        let long_body = "A".repeat(4096);
        let content = format!("#1\n{}\n#2\nend", long_body);
        std::fs::write(test_path, content)?;

        // 刻意使用很小的 chunk，确保 #2 不在首个 chunk 内
        let mut reader = StreamSectionReader::new(test_path, 64)?;
        let result = reader.read_section(1)?;
        assert!(result.content.contains(&long_body));
        assert!(!result.is_eof);

        std::fs::remove_file(test_path)?;
        Ok(())
    }
}
