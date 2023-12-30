use reqwest::{header::USER_AGENT, Client, Response, Url};

use crate::{
    config::SETTINGS,
    score::{Results, Score},
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
) -> Result<Vec<Score>, Box<dyn std::error::Error>> {
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
        Err(err) => return Err(format!("Url Parse Error: {:?}", err).into()),
    };

    let resp = raw_request(&client, url)
        .await?
        .json::<Results<Score>>()
        .await?;

    if resp.error.is_some() {
        let (code, error) = (resp.code.unwrap(), resp.error.unwrap());
        return Err(format!("Parse Error: [{}] {}", code, error).into());
    }

    Ok(resp.results.unwrap())
}

pub async fn raw_request(
    client: &Client,
    url: Url,
) -> Result<Response, Box<dyn std::error::Error>> {
    match client
        .get(url)
        .header(APPLICATION_ID_HEADER, &SETTINGS.read().unwrap().parse.appid)
        .header(USER_AGENT, get_user_agent())
        .send()
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => return Err(format!("Request Error: {:?}", err.status()).into()),
    }
}
