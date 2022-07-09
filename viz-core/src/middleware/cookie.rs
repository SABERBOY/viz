//! Cookie Middleware

use crate::{
    async_trait,
    handler::Transform,
    header::{HeaderValue, COOKIE, SET_COOKIE},
    types, Body, Handler, IntoResponse, Request, Response, Result,
};

pub struct Config {
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    key: std::sync::Arc<types::CookieKey>,
}

#[cfg(not(any(feature = "cookie-signed", feature = "cookie-private")))]
impl Config {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
impl Config {
    pub fn new(key: types::CookieKey) -> Self {
        Self {
            key: std::sync::Arc::new(key),
        }
    }
}

impl<H> Transform<H> for Config
where
    H: Clone,
{
    type Output = CookieMiddleware<H>;

    fn transform(&self, h: H) -> Self::Output {
        CookieMiddleware {
            h,
            #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
            key: self.key.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CookieMiddleware<H> {
    h: H,
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    key: std::sync::Arc<types::CookieKey>,
}

#[async_trait]
impl<H, O> Handler<Request<Body>> for CookieMiddleware<H>
where
    O: IntoResponse,
    H: Handler<Request<Body>, Output = Result<O>> + Clone,
{
    type Output = Result<Response<Body>>;

    async fn call(&self, mut req: Request<Body>) -> Self::Output {
        let jar = req
            .headers()
            .get_all(COOKIE)
            .iter()
            .filter_map(|c| HeaderValue::to_str(c).ok())
            .fold(types::CookieJar::new(), add_cookie);

        let cookies = types::Cookies::new(jar);
        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        let cookies = cookies.with_key(self.key.clone());

        req.extensions_mut()
            .insert::<types::Cookies>(cookies.clone());

        self.h
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .map(|mut res| {
                match cookies.jar().lock() {
                    Ok(c) => {
                        c.delta()
                            .into_iter()
                            .filter_map(|cookie| {
                                HeaderValue::from_str(&cookie.encoded().to_string()).ok()
                            })
                            .fold(res.headers_mut(), |headers, cookie| {
                                headers.append(SET_COOKIE, cookie);
                                headers
                            });
                    }
                    Err(_) => {
                        // TODO: trace error
                    }
                }
                res
            })
    }
}

#[inline]
fn add_cookie(mut jar: types::CookieJar, value: &str) -> types::CookieJar {
    value
        .split(types::Cookies::SPLITER)
        .map(str::trim)
        .filter_map(|v| types::Cookie::parse_encoded(v).ok())
        .for_each(|cookie| jar.add_original(cookie.into_owned()));
    jar
}
