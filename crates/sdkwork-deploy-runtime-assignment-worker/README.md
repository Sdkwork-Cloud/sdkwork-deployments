# SDKWork Deploy Runtime Assignment Worker

Claims durable runtime assignments with expiring database leases and publishes them through the
generated Web Internal SDK adapter. Multiple replicas are safe: lease ownership fences stale
workers, retries are bounded, and older generations are superseded transactionally.
