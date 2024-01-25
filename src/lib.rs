use std::collections::HashMap;
use worker::*;

mod utils;

#[event(fetch)]
pub async fn main(mut req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let kv = env.kv("KV_FROM_RUST").unwrap();
    utils::set_panic_hook();
    let magic_token = String::from("N2rBwhuRyscJg5nqkuagiQy2ecmvt6Xxw") + env!("OFFSET");
    let url = req.url().unwrap();
    let headers = req.headers();
    let pathname = url.path();
    let query_params = url
        .query_pairs()
        .into_owned()
        .collect::<HashMap<String, String>>();
    let authorized = match headers.get("Authorization")? {
        Some(maybe_authorized) => {
            let kvalauth = kv.get(&maybe_authorized).text().await?.unwrap();
            kvalauth == magic_token
        }
        None => false,
    };
    let req_text = req.text().await?;
    match req.method() {
        worker::Method::Get => {
            let keyname = pathname.strip_prefix("/GET").unwrap_or(pathname);
            let resp_value = kv.get(keyname).text().await?.unwrap_or("EMPTY".to_string());
            Response::ok(resp_value)
        }
        worker::Method::Post => {
            if !authorized {
                return Response::error("Unauthorized", 400);
            }
            let keyname = pathname.strip_prefix("/SET").unwrap_or(pathname);
            let empty_string = "".to_string();
            let maybe_param_val = query_params.get("value").unwrap_or(&empty_string);
            let value: String = match maybe_param_val.is_empty() {
                true => req_text,
                false => maybe_param_val.clone(),
            };
            // default ttl 10min
            let default_ttl = "600".to_string();
            let ttl = query_params.get("ttl").unwrap_or(&default_ttl);
            kv.put(keyname, value)?
                .expiration_ttl(ttl.parse::<u64>().ok().unwrap())
                .execute()
                .await?;
            Response::ok("OK")
        }
        _ => Response::error("nada", 403),
    }
}
