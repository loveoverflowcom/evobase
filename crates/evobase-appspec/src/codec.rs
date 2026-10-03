use crate::{CheckedAppSpec, Error, MAX_BYTES, MAX_DEPTH, MAX_NODES, RawAppSpec};

pub(crate) fn json_error(error: serde_json::Error) -> Error {
    Error::InvalidJson {
        message: error.to_string(),
    }
}

pub(crate) fn bounded_encode<T: serde::Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, Error> {
    struct Buffer(Vec<u8>);
    impl std::io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.0.len().saturating_add(bytes.len()) > MAX_BYTES {
                return Err(std::io::Error::other("AppSpec byte limit"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer(Vec::new());
    serde_json::to_writer(&mut buffer, value).map_err(|error| {
        if error.is_io() {
            Error::LimitExceeded {
                resource: "bytes",
                limit: MAX_BYTES,
            }
        } else {
            json_error(error)
        }
    })?;
    preflight(&buffer.0)?;
    Ok(buffer.0)
}

/// Lexical preflight bounds bytes, containers and token starts before serde allocates a DTO.
/// It is deliberately not a JSON parser: serde remains the syntax/duplicate-key authority.
pub(crate) fn preflight(bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() > MAX_BYTES {
        return Err(Error::LimitExceeded {
            resource: "bytes",
            limit: MAX_BYTES,
        });
    }
    let mut depth: usize = 0;
    let mut nodes = 0;
    let mut string = false;
    let mut escape = false;
    let mut atom = false;
    for &byte in bytes {
        if string {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == b'"' {
                string = false;
            }
            continue;
        }
        match byte {
            b'"' => {
                string = true;
                atom = false;
                nodes += 1;
            }
            b'{' | b'[' => {
                depth += 1;
                nodes += 1;
                atom = false;
                if depth > MAX_DEPTH {
                    return Err(Error::LimitExceeded {
                        resource: "depth",
                        limit: MAX_DEPTH,
                    });
                }
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
                atom = false;
            }
            b':' | b',' | b' ' | b'\n' | b'\r' | b'\t' => {
                atom = false;
            }
            _ => {
                if !atom {
                    nodes += 1;
                    atom = true;
                }
            }
        }
        if nodes > MAX_NODES {
            return Err(Error::LimitExceeded {
                resource: "nodes",
                limit: MAX_NODES,
            });
        }
    }
    Ok(())
}

impl CheckedAppSpec {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        preflight(bytes)?;
        let raw: RawAppSpec = serde_json::from_slice(bytes).map_err(json_error)?;
        Self::compile(raw)
    }
    /// Canonical v1 UTF-8 JSON: fixed struct keys and stable identity ordering, no whitespace.
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        bounded_encode(self.definition())
    }
}
