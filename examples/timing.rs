use spectreq::{Client, Method, Profile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(Profile::chrome_143_windows()).await?;
    
    let url = "https://httpbin.org/delay/1";
    println!("Requesting {} (should take > 1s)...", url);

    let response = client.request(Method::GET, url, None).await?;
    
    println!("Status: {}", response.status);
    println!("Total Time: {:?}", response.timing.total);
    println!("TTFB: {:?}", response.timing.ttfb);

    assert!(response.timing.total.as_secs_f64() > 1.0);
    assert!(response.timing.ttfb.as_secs_f64() > 0.0);

    println!("Timing verified!");
    Ok(())
}
