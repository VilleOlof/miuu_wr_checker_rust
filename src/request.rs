//! Makes all parse requests
//!
//! Used to automatically send some headers alongside the actual request

use anyhow::{anyhow, Result};
use reqwest::{header::USER_AGENT, Client, Response, Url};

use crate::{
    config::SETTINGS,
    miu::score::{Results, Score},
};

fn get_user_agent() -> String {
    format!(
        "MIUWRParser/{} VilleOlof/Discord",
        env!("CARGO_PKG_VERSION")
    )
}

const APPLICATION_ID_HEADER: &str = "X-Parse-Application-Id";

/// Makes a request to the given `DOMAIN` using a parse backend
///
/// Manual "low" level function that requires all seperate params in tuple form to be given
///
/// Can give a path but will otherwise default to `/parse/classes/`
///
/// The class can also be give but will default to whatever the `CLASSNAME` Env is.
///
/// #### Env variables required:  
/// * `DOMAIN` - The domain/site to request to  
/// * `CLASSNAME` - The sub path for the request  
/// * `APPID` - The app Id to send alongside in a header  
///
/// # Example
///
/// ```
/// let results = make_request(
///     &client,
///     vec![("limit", "1"), ("where", "{\"username\":\"VilleOlof\"}")],
///     None,
///     None
/// )
/// .await?;
/// ```
pub async fn make_request(
    client: &Client,
    params: Vec<(&str, &str)>,
    path: Option<&str>,
    class: Option<String>,
) -> Result<Vec<Score>> {
    let unwrapped_path = path.unwrap_or("/parse/classes/");

    let class_name = class.unwrap_or(SETTINGS.read().unwrap().parse.class_name.clone());

    let url = match Url::parse_with_params(
        &format!(
            "https://{}{}{}",
            &SETTINGS.read().unwrap().parse.domain,
            unwrapped_path,
            class_name
        ),
        params,
    ) {
        Ok(url) => url,
        Err(err) => return Err(anyhow!("Url Parse Error: {:?}", err)),
    };

    let resp = raw_request(&client, url)
        .await?
        .json::<Results<Score>>()
        .await?;

    if resp.error.is_some() {
        let (code, error) = (resp.code.unwrap(), resp.error.unwrap());
        return Err(anyhow!("Parse Error: [{}] {}", code, error));
    }

    Ok(resp.results.unwrap())
}

/// Sends a "raw" request to the specified url with some headers
///
/// Sends a `USER AGENT` header with the programs identifier
///
/// Also sends the appid from settings in a parse header
pub async fn raw_request(client: &Client, url: Url) -> Result<Response> {
    match client
        .get(url)
        .header(APPLICATION_ID_HEADER, &SETTINGS.read().unwrap().parse.appid)
        .header(USER_AGENT, get_user_agent())
        .send()
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => return Err(anyhow!("Request Error: {:?}", err.status())),
    }
}
