use spectreq::{Client, Method, Profile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a profile (Chrome 143 on Windows)
    let profile = Profile::chrome_143_windows();
    println!("Using profile: {:?}", profile);

    // 2. Create the client
    let client = Client::new(profile).await?;

    // 3. Make a GET request
    let url = "https://httpbin.org/get";
    println!("\nMaking GET request to {}", url);

    let response = client.request(Method::GET, url, None).await?;

    println!("Status: {}", response.status);
    println!("Headers: {:?}", response.headers);

    if let Ok(body_str) = String::from_utf8(response.body) {
        println!("Body: {}", body_str);
    }

    Ok(())
}
