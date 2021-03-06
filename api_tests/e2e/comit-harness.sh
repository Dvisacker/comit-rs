#!/usr/bin/env bash

set -e;
export PROJECT_ROOT=$(git rev-parse --show-toplevel)
source "$PROJECT_ROOT/api_tests/harness-lib.sh"

TEST_PATH="$1";
export TEST_PATH=$(cd ${TEST_PATH} && pwd); # Convert to absolute path
export LOG_DIR="$TEST_PATH/log"

if [[ -z "${TEST_PATH}" ]] || [[ ! -d "${TEST_PATH}" ]]
then
    log "Path to test directory needs to be passed.";
    exit 1;
fi

DIR=${TEST_PATH%*/} # Removes trailing slash
DIR=${DIR##*/} # Extract child dir

ALPHA=${DIR%_*}
ALPHA_CHAIN=${ALPHA%-*}

BETA=${DIR#*_}
BETA_CHAIN=${BETA%-*}

CHAINS="${ALPHA_CHAIN} ${BETA_CHAIN}" # Extract chain name before and after underscore

END(){
    set +e;
    for pid in "$BOB_COMIT_NODE_PID" "$ALICE_COMIT_NODE_PID" "$LQS_PID" "$BTC_BLOCKLOOP_PID"; do
        if test "$pid" && ps "$pid" >/dev/null; then
            echo "KILLING $pid";
            kill "$pid" 2>/dev/null;
            # Here if one of the above is a job is doesn't print out an annoying "Terminated" line to stderr
            wait "$pid" 2>/dev/null;
        fi
    done
    log "KILLING docker containers";
    (
        cd "$PROJECT_ROOT/api_tests/regtest";
        docker-compose rm -sfv;
    );
}

trap 'END' EXIT;

function start() {
    log "INFO: Removing all previous logs from $LOG_DIR"
    rm -rf "$LOG_DIR"
    mkdir -p "$LOG_DIR"

    #### Env variable to run all services
    set -a;
    source "$PROJECT_ROOT/api_tests/regtest/regtest.env"
    set +a;

    export BITCOIN_RPC_URL="http://$BITCOIN_RPC_HOST:$BITCOIN_RPC_PORT";
    #### Start all services
    (
        cd "$PROJECT_ROOT/api_tests/regtest";

        log "Starting up docker containers";
        docker-compose up -d ${CHAINS};
        docker-compose logs --tail=all >"$LOG_DIR/docker-compose.log";
    );

    sleep 10;

    export BOB_CONFIG_FILE="$PROJECT_ROOT/api_tests/regtest/bob/default.toml";
    export BOB_COMIT_NODE_HOST=127.0.0.1;
    BOB_COMIT_NODE_PID=$(
        export RUST_BACKTRACE=1 \
               COMIT_NODE_CONFIG_PATH="$PROJECT_ROOT/api_tests/regtest/bob";
        start_target "comit_node" "Bob";
    );

    export ALICE_COMIT_NODE_HOST=127.0.0.1;
    export ALICE_CONFIG_FILE="$PROJECT_ROOT/api_tests/regtest/alice/default.toml";
    ALICE_COMIT_NODE_PID=$(
        export COMIT_NODE_CONFIG_PATH="$PROJECT_ROOT/api_tests/regtest/alice";
        start_target "comit_node" "Alice";
    );

    LQS_PID=$(
        export LEDGER_QUERY_SERVICE_CONFIG_PATH="$PROJECT_ROOT/api_tests/regtest/ledger_query_service";
        export ETHEREUM_POLLING_TIME_SEC=1;
        export RUST_LOG=warn,ledger_query_service=debug,warp=info;

        start_target "ledger_query_service" "LQS";
    );
}

test "$*" || { log "ERROR: The harness requires to test to run!"; exit 1; }

start;

debug "Bitcoin RPC url: $BITCOIN_RPC_URL";
debug "Ethereum node url: $ETHEREUM_NODE_ENDPOINT";

sleep 2;

run_test "${TEST_PATH}/test.js";
