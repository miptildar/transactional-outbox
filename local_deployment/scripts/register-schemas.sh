#!/bin/sh

echo '=== Starting schema initialization ==='

apk add --no-cache curl jq
if [ $? -ne 0 ]; then
    echo 'Failed to install dependencies'
    exit 1
fi

echo '=== Checking schemas directory ==='
if [ ! -d /schemas ]; then
    echo 'ERROR: /schemas directory does not exist'
    exit 1
fi

ls -la /schemas
schema_count=$(find /schemas -type f -name '*.avsc' | wc -l)
echo "Found $schema_count .avsc files"

if [ $schema_count -eq 0 ]; then
    echo 'No .avsc files found'
    exit 0
fi

echo '=== Waiting for Schema Registry ==='
max_attempts=30
attempt=0

while [ $attempt -lt $max_attempts ]; do
    if curl -sf http://schema-registry:8087/subjects > /dev/null 2>&1; then
        echo 'Schema Registry is available!'
        break
    fi
    attempt=$((attempt + 1))
    echo "Attempt $attempt/$max_attempts: Waiting for Schema Registry..."
    sleep 5
done

if [ $attempt -eq $max_attempts ]; then
    echo 'ERROR: Schema Registry not available after 150 seconds'
    exit 1
fi

# Register schemas
echo '=== Registering Avro schemas ==='
find /schemas -type f -name '*.avsc' | while read f; do
    echo "Processing file: $f"

    subject=$(basename "$f" .avsc)
    echo "Subject name: $subject"

    schema=$(jq -Rs . < "$f")
    if [ $? -ne 0 ]; then
        echo "ERROR: Failed to process file $f"
        continue
    fi

    response=$(curl -s -w '%{http_code}' -X POST \
        http://schema-registry:8087/subjects/$subject/versions \
        -H 'Content-Type: application/vnd.schemaregistry.v1+json' \
        -d "{\"schema\": $schema}")

    http_code=${response: -3}
    body=${response%???}

    case $http_code in
        200)
            echo "✓ SUCCESS: Schema $subject registered"
            ;;
        409)
            echo "✓ INFO: Schema $subject already exists"
            ;;
        *)
            echo "✗ ERROR: Failed to register $subject (HTTP: $http_code)"
            echo "Response: $body"
            ;;
    esac
    echo ""
done

echo '=== Schema registration completed ==='
echo 'Keeping container running for 60 seconds...'
sleep 60