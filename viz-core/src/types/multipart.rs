pub use form_data::FormData;

use viz_utils::futures::future::BoxFuture;

use crate::{
    http,
    types::{Payload, PayloadCheck, PayloadError},
    Context, Extract, Result,
};

/// Context Extends
impl Context {
    pub fn multipart(&mut self) -> Result<Multipart, PayloadError> {
        let mut payload = Payload::<Multipart>::new();

        payload.set_limit(self.config().limits.multipart);

        let m = Payload::get_mime(self);
        let l = Payload::get_length(self);

        let m = payload.check_header(m, l)?;

        let boundary = m.get_param(mime::BOUNDARY);

        if boundary.is_none() {
            return Err(PayloadError::Parse);
        }

        let boundary = boundary.unwrap().as_str();

        // let charset = m.get_param(mime::CHARSET).map(|c| c.to_string());

        Ok(Multipart::new(boundary, self.take_body().unwrap()))
    }
}

/// Multipart Extractor
pub type Multipart<T = http::Body> = FormData<T>;

impl PayloadCheck for Multipart {
    fn check_type(m: &mime::Mime) -> bool {
        is_multipart(m)
    }
}

impl Extract for Multipart {
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.multipart() })
    }
}

fn is_multipart(m: &mime::Mime) -> bool {
    m.type_() == mime::MULTIPART && m.subtype() == mime::FORM_DATA
}
