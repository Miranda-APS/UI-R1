#!/bin/bash
# Deploy UI-R1 su server remoto 194.116.73.38 (utente miranda)
# Uso: bash deploy.sh
# Prerequisiti: ssh key configurata per miranda@194.116.73.38

SERVER="miranda@194.116.73.38"
REMOTE_DIR="~/prometeo_standalone"
LOCAL_DIR="c:/Users/Fra/Desktop/Prometeo/prometeo_standalone"

echo "=== DEPLOY UI-R1 ==="

echo "[1/3] Upload sorgenti..."
scp -r "$LOCAL_DIR/src" "$LOCAL_DIR/Cargo.toml" "$LOCAL_DIR/Cargo.lock" "$SERVER:$REMOTE_DIR/"

echo "[2/3] Build sul server..."
ssh "$SERVER" "cd $REMOTE_DIR && cargo build --release --bin prometeo-web --features web 2>&1 | tail -5"

echo "[3/3] Riavvio servizio..."
ssh "$SERVER" "sudo systemctl restart prometeo.service && sleep 2 && systemctl status prometeo.service --no-pager | head -10"

echo "=== DEPLOY COMPLETATO ==="
echo "Endpoint: http://194.116.73.38:3000/biennale"
