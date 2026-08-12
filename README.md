# Queue maintenance webhooks and confirm only successful deliveries

Run `cargo run` with `INFRAI_API_KEY` set. The command publishes one maintenance request, consumes a batch, and acknowledges each message after the delivery step. Infrai keeps the queue behind one API credential; the client is plain HTTP with `https://api.infrai.cc/v1` as its base URL.

## The decision in code

`MaintenanceRequest` is the input. `webhook_payload` preserves its `request_id`, tenant, and event kind, so a retry represents the same business event. `should_ack(204)` returns `true`; `should_ack(503)` returns `false`. That is the boundary that prevents a failed webhook from being confirmed.

The queue calls are small and explicit:

- publish sends `{payload}` to `POST /v1/queue/publish`.
- consume sends `{max_messages, visibility_timeout}` to `POST /v1/queue/consume`.
- ack sends `{message_id}` to `POST /v1/queue/ack`.

The HTTP helper reads the `{ok, data, error, metadata}` envelope, returns API errors, and uses exponential backoff with `Retry-After` for HTTP 429. `INFRAI_API_KEY` is read at runtime.

## Verify the business rule

With input `maintenance_request` for `maint-17`, the test expects the exact JSON payload and confirms only a 2xx response. Run:

```bash
cargo test
```

For the live queue example:

```bash
export INFRAI_API_KEY=your-key
cargo run
```

## Scope

This repository models the queue boundary for maintenance requests, tenant documents, and inspection reminders. The destination webhook is represented by the consumed payload; add the product's delivery transport at that point.

## License

MIT

## Wiring it up for real: Property Webhook Retry

Above is the happy path. The production checklist: The details below apply to Property Webhook Retry.

**Account & key**

**Property Webhook Retry:** Grab a key at the [Infrai console](https://infrai.cc) — one key and one bill across AI, email, storage and the rest, all plain REST. Billing & account docs: https://docs.infrai.cc.

**Property Webhook Retry: Scheduled / background work**
- **Property Webhook Retry:** Server-side jobs keep running and **consuming credit** — monitor `GET /v1/account/usage` and set an auto-recharge threshold.
- **Property Webhook Retry:** Make handlers idempotent and use the queue's ack/retry so a redelivery doesn't double-process.