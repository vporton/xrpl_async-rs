use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use lazy_static::lazy_static;
use serde::{de, Deserialize};
use serde_json::Value;
use tokio_stream::Stream;
use crate::connection::{Api, XrplError};
use crate::request::Request;
use crate::response::{Response, TypedResponse, Warning};

lazy_static! {
    static ref MARKER_KEY: String = "marker".to_string();
}

pub trait PaginatorExtractor<'de>: Deserialize<'de> + Unpin {
    fn list_obj(result: &Value) -> Result<&Value, XrplError>;
    fn list(result: &Value) -> Result<&Vec<Value>, XrplError> {
        Self::list_obj(result)?.as_array().ok_or::<XrplError>(de::Error::custom("expected array"))
    }
}

pub struct Paginator<'a, A: Api, T: PaginatorExtractor<'a>> where A::Error: From<XrplError> {
    api: &'a A,
    request: Request<'a>,
    list: VecDeque<T>, // more efficient than `Vec`
    marker: Option<Value>,
}

impl<'a, A: Api, T: PaginatorExtractor<'a>> Paginator<'a, A, T>
    where A::Error: From<XrplError>
{
    fn new(api: &'a A, request: Request<'a>, first_page_list: VecDeque<T>) -> Self {
        Self {
            api,
            request,
            list: first_page_list,
            marker: None,
        }
    }
    pub async fn start(api: &'a A, request: Request<'a>) -> Result<(Response, Paginator<'a, A, T>), A::Error> {
        let response = api.call(request.clone()).await?;
        let list = T::list(&response.result)
            .map_err(de::Error::custom)?
            .iter()
            .map(|e| T::deserialize(e.clone()).map_err(de::Error::custom))
            .collect::<Result<VecDeque<T>, XrplError>>().map_err::<XrplError, _>(de::Error::custom)?;
        Ok((response, Self::new(api, request, list)))
    }
    pub async fn first_page(api: &'a A, request: Request<'a>) -> Result<(Response, Vec<T>), A::Error> {
        let response = api.call(request.clone()).await?;
        let list: Vec<T> = T::list(&response.result)
            .map_err(de::Error::custom)?
            .iter()
            .map(|e| T::deserialize(e.clone()).map_err(de::Error::custom))
            .collect::<Result<Vec<T>, XrplError>>().map_err::<XrplError, _>(de::Error::custom)?;
        Ok((response, list))
    }
}

impl<'a, A: Api, T: PaginatorExtractor<'a>> Stream for Paginator<'a, A, T>
    where A::Error: From<XrplError>
{
    type Item = Result<TypedResponse<T>, A::Error>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut forwarded: bool = false;
        let mut load: bool = false;
        let mut warnings: Option<Vec<Warning>> = None;
        if let Some(front) = this.list.pop_front() {
            Poll::Ready(Some(Ok(TypedResponse {
                result: front,
                load: load && this.list.is_empty(), // for the last item in the downloaded list
                warnings,
                forwarded,
            })))
        } else {
            let marker = this.marker.clone();
            let mut loader = |request: &Request| {
                match this.api.call(request.clone()).as_mut().poll(cx) { // Think, if clone can be removed here.
                    Poll::Ready(response) => {
                        let response = response?;
                        load = response.load;
                        forwarded = response.forwarded;
                        warnings = response.warnings;
                        // Duplicate code:
                        // Can do without `clone`?
                        this.list = T::list(&response.result)
                            .map_err(de::Error::custom)?
                            .iter()
                            .map(|e| T::deserialize(e.clone()).map_err(de::Error::custom))
                            .collect::<Result<VecDeque<T>, XrplError>>()?;
                        this.marker = response.result.get(&*MARKER_KEY).cloned();
                        if let Some(front) = this.list.pop_front() {
                            #[allow(unused_assignments)]
                            Poll::Ready(Some(Ok(
                                TypedResponse {
                                    result: front,
                                    load: load && this.list.is_empty(), // for the last item in the downloaded list
                                    warnings: warnings.clone(), // TODO: `clone` is against "value captured by `warnings` is never read" in Clippy (apparently, a compiler bug)
                                    forwarded,
                                }
                            )))
                        } else {
                            Poll::Ready(None)
                        }
                    },
                    Poll::Pending => Poll::Pending,
                }
            };
            if let Some(marker) = marker {
                let mut request = this.request.clone();
                if let Value::Object(obj) = request.params {
                    let mut m = obj;
                    m.insert(MARKER_KEY.clone(), marker);
                    request.params = Value::Object(m);
                }
                loader(&request)
            } else {
                Poll::Ready(None)
            }
        }
    }
}