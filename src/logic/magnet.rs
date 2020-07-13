use serde::Serialize;
use url::Url;
use url::form_urlencoded;
use crate::errors::MagnetMappingError;
use percent_encoding::percent_decode_str;
use log::*;

#[derive(Debug, Clone, Serialize)]
pub struct MagnetLink {
    xt: String,
    tr: Vec<String>,
    dn: String,
}

impl MagnetLink {
    pub fn from(string: &String) -> Result<Self, MagnetMappingError> {
        debug!("Parsing magnet: {}", string);
        let url: Url = Url::parse(string.as_ref()).expect("Invalid magnet");
        let parameters = url.query_pairs();

        let xts: Vec<String> = parameters.filter(|pair| pair.0 == "xt")
        .map(|pair| pair.1.to_string())
        .collect();
        let xt = match xts.len() {
            0 => Err(MagnetMappingError::new("No XT parameter found")),
            1 => Ok(xts.first().unwrap().to_owned()),
            _ => Err(MagnetMappingError::new("There were found more then 1 xt parameter")),
        }?;

        let tr: Vec<String> = parameters.filter(|pair| pair.0 == "tr")
        .map(|pair| pair.1.to_string())
        .collect();

        let dns: Vec<String> = parameters.filter(|pair| pair.0 == "dn" )
        .map(|pair| pair.1.to_string())
        .map(|encoded_name| percent_decode_str(&encoded_name).decode_utf8().unwrap().to_string())
        .collect();

        let dn = match dns.len() {
            0 => Ok(hash_from_xt(&xt)),
            1 => Ok(dns.first().unwrap().to_owned()),
            _ => Err(MagnetMappingError::new("There were found more then 1 dn parameter")),
        }?;

        Ok(MagnetLink{ xt, tr, dn })
    }

    pub fn find(string: &String) -> Option<Self> {
        string.split("\n")
            .map(|line| line.split(" "))
            .flatten()
            .find(|part| part.starts_with("magnet:?"))
            .map(|it| match MagnetLink::from(&it.to_string()){
                Ok(res) =>  Some(res),
                Err(_) => None
            })
            .flatten()
    }

    pub fn hash(self) -> String {
        hash_from_xt(&self.xt)
    }

    pub fn dn(self) -> String {
        self.dn
    }

    pub fn short_link(self) -> String {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        self.tr.iter().for_each(|tr| {
            &serializer.append_pair("tr", tr);
        });
        let tracker_params = serializer.finish();
        format!("magnet:?xt={}&{}", &self.xt, tracker_params)
    }

    pub fn full_link(self) -> String {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        self.tr.iter().for_each(|tr| {
            &serializer.append_pair("tr", tr);
        });
        let tracker_params = serializer.finish();
        format!("magnet:?dn={}&xt={}&{}", &self.dn, &self.xt, tracker_params)
    }
}

fn hash_from_xt(xt: &String) -> String {
    let parts: Vec<String> = xt.split(":")
    .map(|part| String::from(part))
    .collect();
    parts[2].clone()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_short_magnet_generation_from_magnet_string() {
        let magnet = String::from("magnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=ubuntu-mate-16.10-desktop-amd64.iso&tr=http%3A%2F%2Fsometracker.com%2Fannounce2");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let short = MagnetLink::from(&magnet).unwrap();
        assert_eq!(short.clone().xt, urn);
        assert_eq!(short.clone().tr, trackers);
    }

    #[test]
    pub fn test_short_magnet_generation_from_magnet_string_in_message() {
        let magnet = &String::from("some info magnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=ubuntu-mate-16.10-desktop-amd64.iso&tr=http%3A%2F%2Fsometracker.com%2Fannounce2 and some comment after");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let short = MagnetLink::find(magnet);
        assert!(short.clone().is_some());
        assert_eq!(short.clone().unwrap().xt, urn);
        assert_eq!(short.clone().unwrap().tr, trackers);
    }

    #[test]
    pub fn test_short_magnet_generation_from_magnet_string_in_message_on_new_line() {
        let magnet = &String::from("some info\nmagnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=ubuntu-mate-16.10-desktop-amd64.iso&tr=http%3A%2F%2Fsometracker.com%2Fannounce2\nand some comment after");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let link = MagnetLink::find(magnet);
        assert!(link.clone().is_some());
        assert_eq!(link.clone().unwrap().xt, urn);
        assert_eq!(link.clone().unwrap().tr, trackers);
    }

    #[test]
    pub fn test_short_magnet_string_generation() {
        let magnet = String::from("magnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&tr=http%3A%2F%2Fsometracker.com%2Fannounce2");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let link = MagnetLink { xt: urn, tr: trackers, dn: "test".to_owned() };
        let actual:String = link.short_link();
        assert_eq!(actual, magnet);
    }

    #[test]
    pub fn test_short_magnet_hash() {
        let hash = String::from("e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let short = MagnetLink { xt: urn, tr: trackers, dn: "test".to_owned()};
        let actual:String = short.hash();
        assert_eq!(actual, hash);
    }

    #[test]
    pub fn test_dn() {
        let magnet = &String::from("some info\nmagnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=ubuntu-mate-16.10-desktop-amd64.iso&tr=http%3A%2F%2Fsometracker.com%2Fannounce2\nand some comment after");
        let link = MagnetLink::find(magnet);
        assert_eq!(link.unwrap().dn(), "ubuntu-mate-16.10-desktop-amd64.iso")
    }

    #[test]
    pub fn test_dn_cirillic() {
        let magnet = &String::from("some info\nmagnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=%D1%82%D0%B5%D1%81%D1%82&tr=http%3A%2F%2Fsometracker.com%2Fannounce2\nand some comment after");
        let link = MagnetLink::find(magnet);
        assert_eq!(link.unwrap().dn(), "тест")
    }

    #[test]
    pub fn test_dn_with_no_dn() {
        let magnet = &String::from("some info\nmagnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&tr=http%3A%2F%2Fsometracker.com%2Fannounce2\nand some comment after");
        let hash = String::from("e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let link = MagnetLink::find(magnet);
        assert_eq!(link.unwrap().dn(), hash)
    }
}