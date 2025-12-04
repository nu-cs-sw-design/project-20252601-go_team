use actix_web::{test, web, App};
use orderbook::controller::controller::{
    get_open_orders, load_event_stream, place_limit_order, run_simulation_to_end,
    LoadEventStreamResponse, OrderDto, RunSimulationToEndResponse,
};
use serde_json::json;
use std::fs;
use std::io::Write;
use uuid::Uuid;

fn write_event_file(events: serde_json::Value) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("simulation-events-{}.json", Uuid::new_v4()));
    let mut file = fs::File::create(&path).expect("unable to create event file");
    let payload = serde_json::to_string_pretty(&events).expect("serialize events");
    file.write_all(payload.as_bytes())
        .expect("write events to disk");
    path
}

#[actix_web::test]
async fn run_simulation_replays_events_after_mutation() {
    let app = test::init_service(
        App::new()
            .route("/load_event_stream", web::post().to(load_event_stream))
            .route(
                "/run_simulation_to_end",
                web::post().to(run_simulation_to_end),
            )
            .route("/get_open_orders", web::get().to(get_open_orders))
            .route("/place_limit_order", web::post().to(place_limit_order)),
    )
    .await;

    let symbol = format!("SIM-{}", Uuid::new_v4().simple());
    let events = json!([
        {
            "type": "limit",
            "order_id": "sim-limit-1",
            "side": "BUY",
            "price": "101.00",
            "quantity": "5",
            "tif": "GTC"
        },
        {
            "type": "limit",
            "order_id": "sim-limit-2",
            "side": "SELL",
            "price": "105.00",
            "quantity": "7",
            "tif": "GTC"
        }
    ]);

    let path = write_event_file(events);

    let load_req = test::TestRequest::post()
        .uri("/load_event_stream")
        .set_json(json!({
            "symbol": symbol,
            "path": path.to_string_lossy()
        }))
        .to_request();

    let load_resp = test::call_service(&app, load_req).await;
    assert!(load_resp.status().is_success());
    let payload: LoadEventStreamResponse = test::read_body_json(load_resp).await;
    assert_eq!(payload.applied_events, 2);
    assert!(payload.errors.is_empty());
    assert!(!payload.simulation_id.is_empty());
    let sim_id = payload.simulation_id.clone();
    let target_symbol = payload.symbol.clone();

    let extra_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": target_symbol.clone(),
            "side": "SELL",
            "price": "110.00",
            "quantity": "3",
            "tif": "GTC"
        }))
        .to_request();
    let extra_resp = test::call_service(&app, extra_req).await;
    assert!(extra_resp.status().is_success());

    let open_before_req = test::TestRequest::get()
        .uri(&format!(
            "/get_open_orders?symbol={}&side=SELL",
            target_symbol
        ))
        .to_request();
    let open_before_resp = test::call_service(&app, open_before_req).await;
    assert!(open_before_resp.status().is_success());
    let open_before: Vec<OrderDto> = test::read_body_json(open_before_resp).await;
    assert_eq!(open_before.len(), 2);

    let run_req = test::TestRequest::post()
        .uri("/run_simulation_to_end")
        .set_json(json!({
            "simId": sim_id
        }))
        .to_request();
    let run_resp = test::call_service(&app, run_req).await;
    assert!(run_resp.status().is_success());
    let run_payload: RunSimulationToEndResponse = test::read_body_json(run_resp).await;
    assert_eq!(run_payload.applied_events, 2);
    assert_eq!(run_payload.rejected_events, 0);
    assert!(run_payload.errors.is_empty());

    let open_after_req = test::TestRequest::get()
        .uri(&format!(
            "/get_open_orders?symbol={}&side=SELL",
            run_payload.symbol
        ))
        .to_request();
    let open_after_resp = test::call_service(&app, open_after_req).await;
    assert!(open_after_resp.status().is_success());
    let mut open_after: Vec<OrderDto> = test::read_body_json(open_after_resp).await;
    open_after.sort_by(|a, b| a.order_id.cmp(&b.order_id));
    let ids: Vec<String> = open_after.into_iter().map(|o| o.order_id).collect();
    assert_eq!(ids, vec!["sim-limit-2".to_string()]);

    let buy_after_req = test::TestRequest::get()
        .uri(&format!(
            "/get_open_orders?symbol={}&side=BUY",
            run_payload.symbol
        ))
        .to_request();
    let buy_after_resp = test::call_service(&app, buy_after_req).await;
    assert!(buy_after_resp.status().is_success());
    let mut buy_after: Vec<OrderDto> = test::read_body_json(buy_after_resp).await;
    buy_after.sort_by(|a, b| a.order_id.cmp(&b.order_id));
    let buy_ids: Vec<String> = buy_after.into_iter().map(|o| o.order_id).collect();
    assert_eq!(buy_ids, vec!["sim-limit-1".to_string()]);

    fs::remove_file(path).ok();
}

#[actix_web::test]
async fn run_simulation_returns_not_found_for_unknown_id() {
    let app = test::init_service(App::new().route(
        "/run_simulation_to_end",
        web::post().to(run_simulation_to_end),
    ))
    .await;

    let run_req = test::TestRequest::post()
        .uri("/run_simulation_to_end")
        .set_json(json!({
            "simId": "missing-sim"
        }))
        .to_request();

    let run_resp = test::call_service(&app, run_req).await;
    assert_eq!(run_resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}
