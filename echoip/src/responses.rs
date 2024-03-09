pub fn generate_index(content: String) ->String{
  format!("HTTP/1.1 200 OK\r\n\
  Content-Length: {}\r\n\
  Connection: keep-alive\r\n\
  Content-Type: text/plain\r\n\
  \r\n\
  {}", content.len(), content)
}

pub static NOT_FOUND: &[u8] = 
b"HTTP/1.1 404 Not Found\r\n\
Content-Length: 0\r\n\
Connection: keep-alive\r\n\
\r\n\
";

pub static BAD_REQ: &[u8] = 
b"HTTP/1.1 400 Bad Request\r\n\
Content-Length: 0\r\n\
Connection: keep-alive\r\n\
\r\n\
";