use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MaintenanceRequest { pub request_id: String, pub tenant_id: String, pub kind: String }

pub fn webhook_payload(request: &MaintenanceRequest) -> String {
    serde_json::json!({"event_id": request.request_id, "tenant_id": request.tenant_id, "event": request.kind}).to_string()
}

pub fn should_ack(http_status: u16) -> bool { (200..300).contains(&http_status) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn successful_webhook_is_confirmed_but_server_error_stays_visible() {
        let input = MaintenanceRequest { request_id: "maint-17".into(), tenant_id: "tenant-4".into(), kind: "inspection_reminder".into() };
        assert_eq!(webhook_payload(&input), r#"{"event":"inspection_reminder","event_id":"maint-17","tenant_id":"tenant-4"}"#);
        assert!(should_ack(204));
        assert!(!should_ack(503));
    }
}

