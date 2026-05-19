CREATE TABLE deliveries
(
    id                  UUID                     PRIMARY KEY,

    -- references to external systems
    order_id            VARCHAR(255)             NOT NULL,
    courier_id          UUID,                              -- nullable: courier is not assigned yet

    recipient_name      VARCHAR(255)             NOT NULL,
    recipient_phone     VARCHAR(20)              NOT NULL,

    city                VARCHAR(100)             NOT NULL,
    street              VARCHAR(255)             NOT NULL,
    building            VARCHAR(20)              NOT NULL,
    apartment           VARCHAR(20),
    postal_code         VARCHAR(20)              NOT NULL,

    status              VARCHAR(30)              NOT NULL DEFAULT 'PENDING',

    scheduled_date      DATE                     NOT NULL,
    delivered_at        TIMESTAMP WITH TIME ZONE,
    cancelled_at        TIMESTAMP WITH TIME ZONE,
    cancellation_reason TEXT,

    created_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    CONSTRAINT deliveries_status_check CHECK (
        status IN ('PENDING', 'ASSIGNED', 'PICKED_UP', 'IN_TRANSIT', 'DELIVERED', 'FAILED', 'CANCELLED')
    ),

    CONSTRAINT deliveries_delivered_at_check CHECK (
        (status = 'DELIVERED') = (delivered_at IS NOT NULL)
    ),
    CONSTRAINT deliveries_cancelled_at_check CHECK (
        (status = 'CANCELLED') = (cancelled_at IS NOT NULL)
    )
);

CREATE INDEX deliveries_order_id_idx ON deliveries (order_id);

CREATE INDEX deliveries_courier_scheduled_idx ON deliveries (courier_id, scheduled_date)
    WHERE status NOT IN ('DELIVERED', 'CANCELLED');
-- monitoring
CREATE INDEX deliveries_status_idx ON deliveries (status)
    WHERE status NOT IN ('DELIVERED', 'CANCELLED');


-- delivery_items
CREATE TABLE delivery_items
(
    id           UUID         PRIMARY KEY,
    delivery_id  UUID         NOT NULL REFERENCES deliveries (id) ON DELETE CASCADE,
    sku          VARCHAR(100) NOT NULL,
    name         VARCHAR(255) NOT NULL,
    quantity     INT          NOT NULL,
    weight_grams INT,

    CONSTRAINT delivery_items_quantity_check CHECK (quantity > 0),
    CONSTRAINT delivery_items_weight_check   CHECK (weight_grams IS NULL OR weight_grams > 0)
);

CREATE INDEX delivery_items_delivery_id_idx ON delivery_items (delivery_id);


-- delivery_status_history (audit log)
CREATE TABLE delivery_status_history
(
    id          UUID                     PRIMARY KEY DEFAULT gen_random_uuid(),
    delivery_id UUID                     NOT NULL REFERENCES deliveries (id) ON DELETE CASCADE,
    status      VARCHAR(30)              NOT NULL,
    reason      TEXT,
    changed_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    CONSTRAINT delivery_status_history_status_check CHECK (
        status IN ('PENDING', 'ASSIGNED', 'PICKED_UP', 'IN_TRANSIT', 'DELIVERED', 'FAILED', 'CANCELLED')
    )
);

CREATE INDEX delivery_status_history_delivery_id_idx ON delivery_status_history (delivery_id);
CREATE INDEX delivery_status_history_delivery_time_idx ON delivery_status_history (delivery_id, changed_at);


-- outbox_message (transactional outbox pattern)
CREATE TABLE outbox_message
(
    id                  UUID                     NOT NULL,
    aggregate_type      VARCHAR(50)              NOT NULL,
    topic               VARCHAR(255)             NOT NULL,
    key                 VARCHAR(255)             NOT NULL,
    payload             JSONB                    NOT NULL,
    status              VARCHAR(20)              NOT NULL DEFAULT 'NEW',
    processing_attempts SMALLINT                 NOT NULL DEFAULT 0,
    next_retry_at       TIMESTAMP WITH TIME ZONE,          -- for exponential backoff
    last_error          TEXT,
    processed_at        TIMESTAMP WITH TIME ZONE,
    created_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    CONSTRAINT outbox_message_pk PRIMARY KEY (id),
    CONSTRAINT outbox_message_aggregate_type_check CHECK (aggregate_type IN ('DELIVERY')),
    CONSTRAINT outbox_message_status_check         CHECK (status IN ('NEW', 'PROCESSED', 'WAITING_RETRY', 'FAILED')),
    CONSTRAINT outbox_message_attempts_check       CHECK (processing_attempts >= 0)
);

CREATE INDEX outbox_message_pending_idx ON outbox_message (created_at)
    WHERE status IN ('NEW', 'WAITING_RETRY');
CREATE INDEX outbox_message_retry_idx ON outbox_message (next_retry_at)
    WHERE status = 'WAITING_RETRY';
CREATE INDEX outbox_message_cleanup_idx ON outbox_message (processed_at)
    WHERE status = 'PROCESSED';
CREATE INDEX outbox_message_failed_idx ON outbox_message (created_at)
    WHERE status = 'FAILED';
