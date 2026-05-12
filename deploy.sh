#!/bin/bash
# Deploy UI-R1 su server remoto 194.116.73.38 (utente miranda)
# Uso: bash deploy.sh
# Prerequisiti: ssh key configurata per miranda@194.116.73.38

SERVER="miranda@194.116.73.38"
# WorkingDirectory del servizio systemd: /home/miranda/UI-r1
REMOTE_DIR="/home/miranda/UI-r1"
LOCAL_DIR="c:/Users/Fra/Desktop/Prometeo/prometeo_standalone"
REMOTE_CARGO="/home/miranda/.cargo/bin/cargo"

echo "=== DEPLOY UI-R1 ==="

echo "[1/5] Upload sorgenti Rust..."
scp -r "$LOCAL_DIR/src" "$LOCAL_DIR/Cargo.toml" "$LOCAL_DIR/Cargo.lock" "$SERVER:$REMOTE_DIR/"

echo "[2/5] Upload UI statica campovasto..."
scp -r "$LOCAL_DIR/campovasto" "$SERVER:$REMOTE_DIR/"

echo "[3/5] Upload KG curato + stato topologia..."
scp "$LOCAL_DIR/prometeo_kg.json" "$SERVER:$REMOTE_DIR/"
scp "$LOCAL_DIR/prometeo_topology_state.bin" "$SERVER:$REMOTE_DIR/"

echo "[4/5] Build sul server..."
ssh "$SERVER" "cd $REMOTE_DIR && $REMOTE_CARGO build --release --bin prometeo-web --features web 2>&1 | tail -5"

echo "[5/5] Riavvio servizio..."
ssh "$SERVER" "sudo systemctl restart prometeo.service && sleep 2 && systemctl status prometeo.service --no-pager | head -10"

echo "=== DEPLOY COMPLETATO ==="
echo "Verifica: curl -s 'http://194.116.73.38:3000/api/understanding?sentence=ciao' | head -c 200"
echo "Campo Vasto: http://194.116.73.38:3000/campovasto/"
echo "Community: http://194.116.73.38:3000/campovasto/community.html"
