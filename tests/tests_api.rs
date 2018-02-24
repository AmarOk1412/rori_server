extern crate main;
extern crate reqwest;

#[cfg(test)]
mod tests_api {
    use main::core::*;
    use main::rori_utils::data::RoriData;
    use std::thread;
    use reqwest;

    #[test]
    fn test_users() {
        thread::spawn(move || {
            let mut api = API::new("config_server.json");
            api.start();
        });
        let data_to_process = RoriData::from_json(String::from("{
  \"author\":\"AmarOk\",
  \"content\":\"127.0.0.1:4000|text\",
  \"client\":\"rori_desktop_client\",
  \"datatype\":\"register\",
  \"secret\":\"2BB80D537B1DA3E38BD30361AA855686BDE0EACD7162FEF6A25FE97BF527A25B\"
}"));

        ENDPOINTMANAGER.lock().unwrap().register_endpoint(data_to_process);

        // Find endpoint
        let body = reqwest::get("http://localhost:3000/client/AmarOk/text").unwrap()
                   .text().unwrap();

        assert!(body != String::from("[]"));

        // Find endpoint
        let body = reqwest::get("http://localhost:3000/client/*/text").unwrap()
                   .text().unwrap();
        assert!(body != String::from("[]"));

        // Don't find any endpoint for user
        let body = reqwest::get("http://localhost:3000/client/NONE/text").unwrap()
                   .text().unwrap();

        assert_eq!(body, String::from("[]"));

        // Find endpoint
        let body = reqwest::get("http://localhost:3000/client/*/NONE").unwrap()
                   .text().unwrap();

        assert_eq!(body, String::from("[]"));

        // Remove endpoint
        let _ = reqwest::get("http://localhost:3000/rm/0").unwrap()
                   .text().unwrap();
        let body = reqwest::get("http://localhost:3000/client/*/text").unwrap()
                   .text().unwrap();

        assert_eq!(body, String::from("[]"));
    }
}
