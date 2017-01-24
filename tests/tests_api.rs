extern crate main;
extern crate hyper;

#[cfg(test)]
mod tests_api {
    use main::core::*;
    use main::rori_utils::data::RoriData;
    use std::thread;
    use hyper::client::Client;
    use std::io::Read;

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

        let client = Client::new();
        let mut s = String::new();

        // Find endpoint
        let _ = client.get("http://localhost:3000/client/AmarOk/text")
            .send()
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();

        assert!(s != String::from("[]"));

        // Find endpoint
        s = String::new();
        let _ = client.get("http://localhost:3000/client/*/text")
            .send()
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();

        assert!(s != String::from("[]"));

        // Don't find any endpoint for user
        s = String::new();
        let _ = client.get("http://localhost:3000/client/NONE/text")
            .send()
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();

        assert_eq!(s, String::from("[]"));

        // Find endpoint
        s = String::new();
        let _ = client.get("http://localhost:3000/client/*/NONE")
            .send()
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();

        assert_eq!(s, String::from("[]"));

        // Remove endpoint
        s = String::new();
        let _ = client.get("http://localhost:3000/rm/0")
            .send()
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        s = String::new();
        let _ = client.get("http://localhost:3000/client/*/text")
            .send()
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();

        assert_eq!(s, String::from("[]"));
    }
}
