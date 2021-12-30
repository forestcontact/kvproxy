extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use js_sys::{Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Request, Response, ResponseInit};

use wasm_bindgen_futures::JsFuture;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub async fn handle(kv: WorkersKvJs, req: JsValue) -> Result<Response, JsValue> {
    let magic_token = String::from("N2rBwhuRyscJg5nqkuagiQy2ecmvt6Xxw") + env!("OFFSET");
    let req: Request = req.dyn_into()?;
    let url = web_sys::Url::new(&req.url())?;
    let headers = req.headers();
    let pathname = url.pathname();
    let query_params = url.search_params();
    let kv = WorkersKv { kv };
    let authorized = match headers.get("X-AUTH")? {
        Some(maybe_authorized) => {
            let kvalauth = kv.get_text(&maybe_authorized).await?.unwrap_or_default();
            kvalauth == magic_token
        }
        None => false,
    };
    let mut init = ResponseInit::new();
    match req.method().as_str() {
        "GET" => {
            let mut resp_value = match kv.get_text(&pathname).await {
                Ok(val) => val.unwrap_or_default(),
                Err(_) => String::from("ERROR"),
            };
            resp_value = match resp_value.is_empty() {
                true => String::from("EMPTY"),
                false => resp_value,
            };
            init.status(200);
            Response::new_with_opt_str_and_init(Some(&resp_value), &init)
        }
        "POST" => {
            let maybe_param_val = query_params.get("value").unwrap_or_default();
            let value: String = match maybe_param_val.is_empty() {
                true => {
                    let jsblob: JsValue = JsFuture::from(req.text()?).await?;
                    jsblob.as_string().unwrap_or_default()
                }
                false => maybe_param_val.clone(),
            };
            // default ttl 10min
            let ttl = query_params
                .get("ttl")
                .unwrap_or_else(|| String::from("600"));
            let response = if authorized {
                kv.put_text(&pathname, &value, ttl.parse::<u64>().unwrap())
                    .await?;
                init.status(200);
                Some("OK")
            } else {
                init.status(403);
                Some("AUTH FAILED")
            };
            Response::new_with_opt_str_and_init(response, &init)
        }
        _ => {
            init.status(400);
            Response::new_with_opt_str_and_init(None, &init)
        }
    }
}

#[wasm_bindgen]
extern "C" {
    pub type WorkersKvJs;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn put(
        this: &WorkersKvJs,
        k: JsValue,
        v: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn get(
        this: &WorkersKvJs,
        key: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;
}

struct WorkersKv {
    kv: WorkersKvJs,
}

impl WorkersKv {
    async fn put_text(&self, key: &str, value: &str, ttl: u64) -> Result<(), JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(ttl as f64).into())?;
        self.kv
            .put(JsValue::from_str(key), value.into(), options.into())
            .await?;
        Ok(())
    }

    async fn get_text(&self, key: &str) -> Result<Option<String>, JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"type".into(), &"text".into())?;
        Ok(self
            .kv
            .get(JsValue::from_str(key), options.into())
            .await?
            .as_string())
    }
}
