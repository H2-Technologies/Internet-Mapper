async fn download_html(url: &str) -> String {
    let response = reqwest::get(url).await.unwrap();
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    body
}

fn parse_html(html: String) -> Vec<String> {
    let href: String = String::from("href");
    let mut found_href: Vec<String> = Vec::new();
    let mut new_url: Vec<String> = Vec::new();
    for line in html.lines() {
        if line.contains(&href) {
            let line = line.to_string();
            let mut line = line.split("href=\"");
            let line = line.nth(1).unwrap();
            let mut line = line.split("\"");
            let line = line.nth(0).unwrap();
            found_href.push(line.to_string());
        }
    }
    for link in found_href.iter_mut() {
        if link.contains("http") || link.contains("https") {
            new_url.push(link.to_string());
        }
    }
    new_url
}

#[tokio::main]
async fn main() {
    let mut new_url: Vec<String> = Vec::new();
    let mut searched_url: Vec<String> = Vec::new();
    let url: String = String::from("https://www.rust-lang.org");
    let html: String = download_html(&url).await;
    new_url = parse_html(html);
    searched_url.push(url.clone());
    println!("url: {}", url);
    println!("new_url: {:?}", new_url);
}