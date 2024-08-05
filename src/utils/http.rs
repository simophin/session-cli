pub trait RequestExt {
    fn to_http_1_1(&self) -> Vec<u8>;
}

impl<T> RequestExt for http::Request<T>
where
    T: AsRef<[u8]>,
{
    fn to_http_1_1(&self) -> Vec<u8> {
        use std::io::Write;

        let mut buf = Vec::new();
        // Write the request line
        let _ = write!(
            buf,
            "{} {} HTTP/1.1\r\n",
            self.method(),
            self.uri().path_and_query().unwrap()
        );

        // Write the headers
        for (name, value) in self.headers() {
            buf.extend_from_slice(name.as_str().as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // Write the body
        buf.extend_from_slice(self.body().as_ref());
        buf.extend_from_slice(b"\r\n");

        buf
    }
}
