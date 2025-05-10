use pdk::hl::HeadersHandler;

pub const POST_METHOD: &str = "POST";
pub const GET_METHOD: &str = "GET";
pub const CONTENT_TYPE_HEADER: &str = "content-type";
pub const APPLICATION_JSON: &str = "application/json";
pub const CONTENT_LENGTH_HEADER: &str = "content-length";
pub const ACCEPT_HEADER: &str = "accept";
pub const METHOD_HEADER: &str = ":method";
pub const PATH_HEADER: &str = ":method";
pub const TIMEOUT_HEADER: &str = "x-envoy-upstream-rq-timeout-ms";

pub fn with_no_timeout(header_handler: &dyn HeadersHandler) {
    header_handler.set_header(TIMEOUT_HEADER, "0");
}
