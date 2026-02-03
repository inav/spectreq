use spectreq::{Client, Method, Profile};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(Profile::chrome_143_windows()).await?;
    let url = "https://httpbin.org/cookies";
    let url_obj = Url::parse(url)?;

    println!("Initial CookieJar size: {}", client.cookie_jar().len());

    // Manually set cookies
    client.cookie_jar().set_cookies(&["session=rust123"], &url_obj);
    println!("Set cookie 'session=rust123'");

    // Verify
    let response = client.request(Method::GET, url, None).await?;
    let body = String::from_utf8(response.body)?;
    println!("Response body:\n{}", body);

    assert!(body.contains("session") && body.contains("rust123"));
    println!("Cookie verified!");

    Ok(())
}
