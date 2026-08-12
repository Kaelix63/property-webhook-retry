use property_webhook_retry::{infrai::queue, queue_worker::{webhook_payload, MaintenanceRequest}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = MaintenanceRequest { request_id: "maint-17".into(), tenant_id: "tenant-4".into(), kind: "maintenance_request".into() };
    let payload = webhook_payload(&request);
    let published = queue::publish(&payload)?;
    println!("queued webhook: {:?}", published.message_id);
    let messages = queue::consume(10, 60)?;
    for message in messages { println!("deliver {}: {}", message.message_id, message.payload); queue::ack(&message.message_id)?; }
    Ok(())
}

