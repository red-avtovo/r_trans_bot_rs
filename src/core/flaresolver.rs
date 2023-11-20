
pub struct Flaresolver {
    url: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FlaresolverRequest {
    cmd: String,
    url: String,
    max_timeout: u32,
}

impl FlaresolverRequest {
    fn get(url: String) -> Self {
        Self {
            cmd: "request.get".to_string(),
            url,
            max_timeout: 60000,
        }
    }
}

#[derive(serde::Deserialize)]
struct FlaresolverResponse {
    status: String,
    solution: FlaresolverSolution,
}

#[derive(serde::Deserialize)]
struct FlaresolverSolution {
    response: String,
}

impl Flaresolver {
    pub(crate) fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn get_page_html(&self, url: String) -> Result<Option<String>, reqwest::Error> {
        let client = reqwest::Client::new();
        let request = FlaresolverRequest::get(url);
        let result = client
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .json::<FlaresolverResponse>()
            .await?;
        if result.status == "ok" {
            Ok(Some(result.solution.response))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::flaresolver::Flaresolver;

    #[tokio::test]
    async fn get_rutracker() {
        let s = "https://rutracker.org/forum/viewtopic.php?t=5956127".to_string();
        let res = Flaresolver::new("http://10.43.129.203/v1".to_string())
            .get_page_html(s)
            .await
            .unwrap();

        assert!(res.is_some());
        assert!(res.unwrap().contains("magnet:?"));
    }
}