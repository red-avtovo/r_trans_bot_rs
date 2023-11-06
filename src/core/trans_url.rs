use url::Url;

#[derive(Debug, Clone)]
pub struct TransUrl(String);

impl TransUrl {
    pub fn from_web_url(url: &String) -> Option<Self> {
        let lowercased_url = url.clone().to_lowercase();
        let base_url = lowercased_url.split("/transmission/web").into_iter().next();
        base_url.map(|url| TransUrl(url.to_owned()))
    }
    pub fn to_rpc_url(&self) -> Url {
        (self.0.clone() + "/transmission/rpc").parse().unwrap()
    }

    pub(crate) fn get_base_url(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for TransUrl {
    fn from(url: String) -> Self {
        TransUrl(url)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_trans_url_rpc_generation() {
        let url = TransUrl("http://localhost".to_owned());
        assert_eq!("http://localhost/transmission/rpc".parse::<Url>().unwrap(), url.to_rpc_url())
    }

    #[test]
    fn test_trans_url_web_parsing() {
        let full_url = "http://localhost:9091/transmission/web/#confirm".to_owned();
        let t = TransUrl::from_web_url(&full_url).unwrap();
        assert_eq!("http://localhost:9091", t.get_base_url());
    }
}
