use pdk::hl::HeadersHandler;

pub const POST_METHOD: &'static str = "POST";
pub const GET_METHOD: &'static str = "GET";
pub const CONTENT_TYPE_HEADER: &'static str = "content-type";
pub const APPLICATION_JSON: &'static str = "application/json";
pub const CONTENT_LENGTH_HEADER: &'static str = "content-length";
pub const ACCEPT_HEADER: &'static str = "accept";
pub const METHOD_HEADER: &'static str = ":method";
pub const PATH_HEADER: &'static str = ":method";
pub const TIMEOUT_HEADER: &'static str = "x-envoy-upstream-rq-timeout-ms";

pub fn with_no_timeout(header_handler: &dyn HeadersHandler) {
    header_handler.set_header(TIMEOUT_HEADER, "0");
}
