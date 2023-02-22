use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use async_trait::async_trait;
use lazy_static::lazy_static;
use serde_json::Value;
use tokio_stream::Stream;
use crate::connection::Api;
use crate::json::ValueExt;
use crate::request::Request;
use crate::response::{ParseResponse, ParseResponseError, Response, TypedResponse, WrongFieldsError};

lazy_static! {
    static ref MARKER_KEY: String = "marker".to_string();
}

pub trait PaginatorExtractor: ParseResponse + Unpin {
    // TODO: Rename the methods.
    fn list_obj(result: &Value) -> Result<&Value, WrongFieldsError>;
    fn list_part(result: &Value) -> Result<&Vec<Value>, WrongFieldsError> {
        Ok(Self::list_obj(result)?.as_array_valid()?)
    }
}

pub struct Paginator<'a, A: Api, T: PaginatorExtractor> where A::Error: From<ParseResponseError> {
    api: &'a A,
    request: Request<'a>,
    list: VecDeque<T>, // more efficient than `Vec`
    marker: Option<Value>,
}

impl<'a, A: Api, T: PaginatorExtractor> Paginator<'a, A, T>
    where A::Error: From<ParseResponseError> + From<WrongFieldsError>
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
        // TODO: Duplicate code:
        let list = T::list_part(&response.result).map_err(|_| WrongFieldsError::new())?.into_iter().map(|e| T::from_json(e))
            .collect::<Result<Vec<T>, ParseResponseError>>()?.into();
        Ok((response, Self::new(api, request, list)))
    }
}

impl<'a, A: Api, T: PaginatorExtractor> Stream for Paginator<'a, A, T>
    where A::Error: From<ParseResponseError> + From<WrongFieldsError>
{
    type Item = Result<TypedResponse<T>, A::Error>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut forwarded: bool = false;
        let mut load: bool = false;
        if let Some(front) = this.list.pop_front() {
            Poll::Ready(Some(Ok(TypedResponse {
                result: front.into(),
                load: load && this.list.is_empty(), // for the last item in the downloaded list
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
                        // TODO: Duplicate code:
                        this.list = T::list_part(&response.result)?.iter().map(|e| T::from_json(e))
                            .collect::<Result<Vec<T>, ParseResponseError>>()?.into();
                        this.marker = response.result.get(&*MARKER_KEY).map(|v| v.clone());
                        if let Some(front) = this.list.pop_front() {
                            Poll::Ready(Some(Ok(
                                TypedResponse {
                                    result: front.into(),
                                    load: load && this.list.is_empty(), // for the last item in the downloaded list
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
                if let Some(mut params) = request.params.as_object() {
                    params.insert(MARKER_KEY.clone(), marker);
                }
                loader(&request)
            } else {
                Poll::Ready(None)
            }
        }
    }
}

// TODO: Is it needed?
#[async_trait]
trait Paginated<Api, Element> {
    fn get_raw_list_one_page(result: &Value) -> &Vec<Value>;
    fn parse_element(e: &Value) -> Element;
    fn get_list_one_page(result: &Value) -> Vec<Element> {
        Self::get_raw_list_one_page(result).into_iter().map(|e| Self::parse_element(e)).collect()
    }
    // FIXME
    // async fn get_full_list(api: &Api, request: Request<'async_trait>) {
    //     let result = api.call(request).await?;
    //     yield from get_list_one_page(&result);
    // }
}
