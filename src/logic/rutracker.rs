use scraper::{Selector, Html};
use reqwest::{Url};

pub async fn get_magnet(url: String) -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::Client::new();
    client.get(Url::parse(&url).unwrap()).send().await?
        .text().await.map(|body| find_magnet(body))
}

pub fn find_magnet(html: String) -> Option<String> {
    let document = Html::parse_document(&html);
    let selector = Selector::parse("a.magnet-link").unwrap();
    document.select(&selector).next()
        .map(|e| e.value().attr("href"))
        .flatten()
        .map(|s| s.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    pub async fn test_url() {
        let s = "https://rutracker.org/forum/viewtopic.php?t=5956127".to_string();
        let result = get_magnet(s).await;
        assert!(result.is_ok());
        let link = result.unwrap();
        assert!(link.is_some());
        let res = link.unwrap();
        println!("Res={}", res);
        assert!(res.starts_with("magnet:"));
    }
}