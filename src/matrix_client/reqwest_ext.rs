use reqwest;
use ruma_client_api as api;

pub trait RqwClientExt {
    fn matrix_request<E>(&self,
                         path_params: E::PathParams,
                         body_params: E::BodyParams)
                         -> reqwest::Result<MatrixResponse<E>>
        where E: api::Endpoint;

    // fn matrix_request<E>(&self, body_params: E::BodyParams) -> reqwest::Result<MatrixResponse<E>>
    //     where E: api::Endpoint<PathParams = ()>
    // {
    //     self.matrix_request((), body_params)
    // }
}

impl RqwClientExt for reqwest::Client {
    fn matrix_request<E>(&self,
                         path_params: E::PathParams,
                         body_params: E::BodyParams)
                         -> reqwest::Result<MatrixResponse<E>>
        where E: api::Endpoint
    {
        self.request(to_reqwest_method(E::method()),
                     &E::request_path(path_params))
            .json(&body_params)
            .send()
            .and_then(MatrixResponse::from_rqw_response)
    }
}

pub struct MatrixResponse<E: api::Endpoint> {
    rqw_response: reqwest::Response,
    body: E::Response,
}

impl<E: api::Endpoint> MatrixResponse<E> {
    fn from_rqw_response(mut rqw_response: reqwest::Response)
                         -> reqwest::Result<MatrixResponse<E>> {
        rqw_response.json::<E::Response>().map(|body| {
            MatrixResponse {
                rqw_response: rqw_response,
                body: body,
            }
        })
    }

    pub fn status(&self) -> &reqwest::StatusCode {
        self.rqw_response.status()
    }

    pub fn headers(&self) -> &reqwest::header::Headers {
        self.rqw_response.headers()
    }

    pub fn version(&self) -> &reqwest::HttpVersion {
        self.rqw_response.version()
    }

    pub fn body(&mut self) -> &E::Response {
        &self.body
    }
}

fn to_reqwest_method(api_method: api::Method) -> reqwest::Method {
    match api_method {
        api::Method::Delete => reqwest::Method::Delete,
        api::Method::Get => reqwest::Method::Get,
        api::Method::Post => reqwest::Method::Post,
        api::Method::Put => reqwest::Method::Put,
    }
}