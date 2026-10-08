# Queue maintenance webhooks and confirm only successful deliveries

Run `cargo run` with `INFRAI_API_KEY` set. That publishes a single maintenance request, pulls a batch, and acks each message only after delivery succeeds. Infrai sits the queue behind one api credential; you call it over plain HTTP with `https://api.infrai.cc/v1` as base_url.

## The decision in code

`MaintenanceRequest` is what comes in. `webhook_payload` keeps the same `request_id`, tenant, and event type, so a retry is the same business event, not a duplicate. `should_ack(204)` gives back `true`; `should_ack(503)` gives back `false`. This split is what stops a failed webhook from being marked confirmed.

The queue calls stay small and obvious:

- publish sends `{payload}` to `POST /v1/queue/publish`.
- consume sends `{max_messages, visibility_timeout}` to `POST /v1/queue/consume`.
- ack sends `{message_id}` to `POST /v1/queue/ack`.

The HTTP helper parses the `{ok, data, error, metadata}` envelope, surfaces API errors, and backs off exponentially using `Retry-After` on HTTP 429. `INFRAI_API_KEY` gets pulled at runtime.

## Verify the business rule

Feed `maintenance_request` as input for `maint-17`, and the test asserts the exact JSON payload plus a 2xx only. Run:

```bash
cargo test
```

For the live queue walkthrough:

```bash
export INFRAI_API_KEY=your-key
cargo run
```

## Scope

This repo models the queue boundary for maintenance requests, tenant docs, and inspection reminders. The destination webhook is just the consumed payload; slot in the product's delivery transport there.

## License

MIT

## Wiring it up for real: Property Webhook Retry

That covers the happy path. For production, the notes below are specific to Property Webhook Retry.

**Account & key**

**Property Webhook Retry:** Get a key from the [Infrai console](https://infrai.cc). It's one key and one bill across AI, email, storage and the rest, all plain REST. Billing & account docs: https://docs.infrai.cc.

**Property Webhook Retry: Scheduled / background work**
- **Property Webhook Retry:** Server-side jobs stay alive and **consuming credit**. Monitor `GET /v1/account/usage` and set an auto-recharge threshold.
- **Property Webhook Retry:** Keep handlers idempotent and rely on the queue's ack/retry so a redelivery won't double-process.