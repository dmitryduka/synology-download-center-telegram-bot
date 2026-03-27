#!/bin/sh
# CGI proxy: routes DSM webapi requests to the syno-bot HTTP API on localhost:8008.
# DSM passes api method as QUERY_STRING parameters. We map method names to API paths.

# Parse the method from QUERY_STRING
METHOD=$(echo "$QUERY_STRING" | sed -n 's/.*method=\([^&]*\).*/\1/p')

case "$METHOD" in
    get_status)
        RESULT=$(curl -s http://localhost:8008/api/status 2>&1)
        ;;
    get_config)
        RESULT=$(curl -s http://localhost:8008/api/config 2>&1)
        ;;
    set_config)
        BODY=$(cat)
        RESULT=$(curl -s -X POST -H "Content-Type: application/json" -d "$BODY" http://localhost:8008/api/config 2>&1)
        ;;
    get_activity)
        RESULT=$(curl -s http://localhost:8008/api/activity 2>&1)
        ;;
    *)
        RESULT='{"success":false,"error":{"code":103}}'
        ;;
esac

echo "Content-Type: application/json"
echo ""
echo "{\"success\":true,\"data\":${RESULT}}"
