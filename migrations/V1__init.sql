CREATE TABLE deliveries (
                            delivery_id UUID PRIMARY KEY,
                            order_id BIGINT NOT NULL,
                            address TEXT NOT NULL,
                            status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
                            created_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE delivery_items (
                                id UUID PRIMARY KEY,
                                delivery_id BIGINT NOT NULL REFERENCES deliveries(delivery_id) ON DELETE CASCADE,
                                sku VARCHAR(50) NOT NULL,
                                quantity INT NOT NULL
);

-- Transactional outbox pattern table:
CREATE TABLE public.outbox_message
(
    id                  UUID                      NOT NULL,
    topic               VARCHAR(255)              NOT NULL,
    key                 VARCHAR(255)              NOT NULL,
    type                VARCHAR(50)               NOT NULL, -- type is for different message types, for now only `ISSUE` is supported
    payload             JSONB                     NOT NULL, -- JSON serialized payload
    status              VARCHAR(50) DEFAULT 'NEW' NOT NULL,
    processing_attempts INT         DEFAULT 0     NOT NULL,
    last_error          TEXT,
    processed_at        TIMESTAMP WITH TIME ZONE,
    created_at          TIMESTAMP WITH TIME ZONE  NOT NULL,
    updated_at          TIMESTAMP WITH TIME ZONE  NOT NULL,
    CONSTRAINT inbox_messagePK PRIMARY KEY (id),
    CONSTRAINT type_check CHECK (type IN ('ISSUE')),
    CONSTRAINT status_check CHECK (status IN ('NEW', 'PROCESSED', 'WAITING_RETRY', 'FAILED'))
);

CREATE INDEX outbox_message_process_status_idx
    ON public.outbox_message (type, status, created_at)
    WHERE status = 'NEW' OR status = 'WAITING_RETRY';

CREATE INDEX outbox_message_cleanup_idx
    ON public.outbox_message (processed_at)
    WHERE status = 'PROCESSED';

CREATE INDEX outbox_message_failed_idx
    ON public.outbox_message (id)
    WHERE status = 'FAILED';