use log;
use redis;
use rouille::{router, Request, Response, ResponseBody};

pub fn routes(request: &Request, db: &redis::Connection) -> Response {
    router!(request,

            (GET) (/) => {
                log::info!("GET /");

                Response::text("Kyle's Peripheral Abstraction Layer (KPAL)")
            },

            (GET) (/api/v0/libraries) => {
                log::info!("GET /api/v0/libraries");

                let keys: Vec<String> = match redis::cmd("KEYS")
                    .arg("libraries:*")
                    .query(db) {
                        Ok(result) => result,
                        Err(e) => {
                            log::error!("{}", e);
                            return Response::empty_404()
                        },
                    };

                let result: String = match redis::cmd("JSON.MGET")
                    .arg(keys)
                    .arg(".")
                    .query::<Vec<String>>(db) {
                        Ok(result) => "[".to_owned() + &result.join(",") + "]",
                        Err(e) => {
                            log::error!("{}", e);
                            return Response::empty_404()
                        },
                    };

                Response {
                    status_code: 200,
                    headers: vec![("Content-Type".into(), "application/json; charset=utf-8".into())],
                    data: ResponseBody::from_data(result),
                    upgrade: None,
                }

            },

            (GET) (/api/v0/libraries/{id: usize}) => {
                log::info!("GET /api/v0/libraries/{}", id);

                let result: String = match redis::cmd("JSON.GET")
                    .arg(format!("libraries:{}", &id))
                    .arg(".")
                    .query(db) {
                        Ok(result) => result,
                        Err(_) => return Response::empty_404(),
                    };

                Response {
                    status_code: 200,
                    headers: vec![("Content-Type".into(), "application/json; charset=utf-8".into())],
                    data: ResponseBody::from_data(result),
                    upgrade: None,
                }
            },
    /*
            // GET /peripherals
            (GET) (/peripherals) => {
                // Returns a list of all peripherals currently registered with the daemon.
                //
                // peripherals are devices or processes that may be controlled by KPAL.
                Response::empty_404()
            },

            // GET /peripherals/{id}
            (GET) (/peripherals/{id: usize}) => {
                // Returns a single peripheral.
                Response::empty_404()
            },

            // PATCH /peripherals/{id}
            (PATCH) (/peripherals/{id: usize}) => {
                // Updates the state of a given peripheral.
                Response::empty_404()
            },
    */
            _ => Response::empty_404()
        )
}
