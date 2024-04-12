use std::io::Write;
use async_recursion::async_recursion;

#[async_recursion]
async fn download_html(url: &str) -> String {
    let response = reqwest::get(url).await;
    match response {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return String::new();
        }
    }
    let response = response.unwrap();
    // if the request is successful (status code 200), continue, otherwise return an empty string
    if !response.status().is_success() {
        return String::new();
    }
    // if the response is 429 (too many requests), sleep try again
    if response.status().as_u16() == 429 {
        sleep();
        return download_html(url).await;
    }
    response.text().await.unwrap()
}

fn parse_html(html: &str) -> Vec<String> {
    let href = "href";
    let mut found_href = Vec::new();
    let mut new_url = Vec::new();
    for line in html.lines() {
        if line.contains(href) {
            if let Some(subline) = line.split("href=\"").nth(1) {
                if let Some(href_value) = subline.split("\"").nth(0) {
                    found_href.push(href_value.to_string());
                }
            }
        }
    }
    for link in found_href.iter_mut() {
        if link.contains("http") || link.contains("https") {
            new_url.push(link.to_string());
        }
    }
    new_url
}

fn sleep() {
    std::thread::sleep(std::time::Duration::from_secs(5));
}

fn save_state(new_urls: Vec<String>, searched_urls: Vec<String>) {
    let mut file = std::fs::File::create("state.json").unwrap();
    // Serialize the data to a JSON string with new urls being "urls" and searched urls being "searched"
    let state = serde_json::json!({ "urls": new_urls, "searched": searched_urls });
    let state_str = serde_json::to_string(&state).unwrap();
    file.write_all(state_str.as_bytes()).unwrap();
}

fn recover_state() -> (Vec<String>, Vec<String>) {
    let file = std::fs::File::open("state.json").unwrap();
    let state: serde_json::Value = serde_json::from_reader(file).unwrap();
    let new_urls = state["urls"].as_array().unwrap();
    let searched_urls = state["searched"].as_array().unwrap();
    let mut new_urls_vec = Vec::new();
    let mut searched_urls_vec = Vec::new();
    for url in new_urls.iter() {
        new_urls_vec.push(url.as_str().unwrap().to_string());
    }
    for url in searched_urls.iter() {
        searched_urls_vec.push(url.as_str().unwrap().to_string());
    }
    (new_urls_vec, searched_urls_vec)
}

fn clean_urls(urls: Vec<String>) -> Vec<String> {
    let mut cleaned_urls = Vec::new();
    // remove any url that doesn't start with http or https
    for url in urls.iter() {
        if url.contains("http") || url.contains("https") {
            cleaned_urls.push(url.to_string());
        }
    }
    // remove all query parameters from the url
    for url in cleaned_urls.iter_mut() {
        if let Some(index) = url.find("?") {
            *url = url.split_at(index).0.to_string();
        }
    }
    // remove all fragments from the url (anything after #)
    for url in cleaned_urls.iter_mut() {
        if let Some(index) = url.find("#") {
            *url = url.split_at(index).0.to_string();
        }
    }
    // remove all duplicate urls
    cleaned_urls.sort();
    cleaned_urls
}

#[tokio::main]
async fn main() {
    let mut new_urls = Vec::new();
    let mut searched_urls = Vec::new();
    (new_urls, searched_urls) = recover_state();
    if searched_urls.len() > 0 {
        let start_url = "https://www.rust-lang.org";
        new_urls.push(start_url.to_string());
        searched_urls.push(start_url.to_string());
    }
    let mut searched_total = 0;
    while let Some(url) = new_urls.pop() {
        let html = download_html(&url).await;
        searched_total += 1;
        let new_links = parse_html(&html);
        for link in new_links.iter() {
            if !searched_urls.contains(link) {
                new_urls.insert(0, link.to_string()); // Insert at the beginning to process sequentially
                searched_urls.push(link.to_string());
            }
        }
        println!(
            "Total URL's Searched: {}\nTotal URL's to Search: {}\nNew URL's: {}",
            searched_total,
            new_urls.len(),
            new_links.len()
        );
        if searched_total % 25 == 0 {
            save_state(new_urls.clone(), searched_urls.clone());
        }
        new_urls = clean_urls(new_urls.clone());
    }
}
